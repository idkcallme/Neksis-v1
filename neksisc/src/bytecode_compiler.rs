use crate::ast::{
    Program, Statement, Expression, Literal, BinaryOperator
};
use crate::vm::BytecodeInstruction;
use crate::error::CompilerError;
use std::collections::HashMap;

pub struct BytecodeCompiler {
    instructions: Vec<BytecodeInstruction>,
    labels: HashMap<String, usize>,
    label_counter: usize,
    function_definitions: HashMap<String, Vec<BytecodeInstruction>>,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: HashMap::new(),
            label_counter: 0,
            function_definitions: HashMap::new(),
        }
    }
    
    pub fn compile_program(&mut self, program: &Program) -> Result<Vec<BytecodeInstruction>, CompilerError> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        
        Ok(self.instructions.clone())
    }
    
    fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Let(let_stmt) => {
                self.compile_expression(&let_stmt.value)?;
                self.instructions.push(BytecodeInstruction::Store(let_stmt.name.clone()));
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                // Only pop if the expression is not a block or function call
                if !matches!(expr, Expression::Block(_)) && !matches!(expr, Expression::FunctionCall(_, _)) {
                    self.instructions.push(BytecodeInstruction::Pop);
                }
            }
            Statement::Return(return_stmt) => {
                if let Some(expr) = &return_stmt.value {
                    self.compile_expression(expr)?;
                } else {
                    self.instructions.push(BytecodeInstruction::PushNull);
                }
                self.instructions.push(BytecodeInstruction::Return);
            }
            Statement::Function(func_stmt) => {
                // Emit DefineFunction instruction
                self.instructions.push(BytecodeInstruction::DefineFunction(
                    func_stmt.name.clone(),
                    func_stmt.parameters.len()
                ));
                
                // Compile function body
                if let Expression::Block(statements) = &*func_stmt.body {
                    let mut temp_instructions = Vec::new();
                    for statement in statements {
                        self.compile_statement_for_function(statement, &mut temp_instructions)?;
                    }
                    // Add implicit return if the last instruction is not a Return
                    if temp_instructions.is_empty() || !matches!(temp_instructions.last(), Some(BytecodeInstruction::Return)) {
                        temp_instructions.push(BytecodeInstruction::PushNull);
                        temp_instructions.push(BytecodeInstruction::Return);
                    }
                    self.instructions.extend(temp_instructions);
                }
                self.instructions.push(BytecodeInstruction::EndFunction);
            }
            _ => return Err(CompilerError::syntax_error("Unsupported statement type")),
        }
        Ok(())
    }
    
    fn compile_statement_for_function(&mut self, statement: &Statement, instructions: &mut Vec<BytecodeInstruction>) -> Result<(), CompilerError> {
        match statement {
            Statement::Let(let_stmt) => {
                self.compile_expression_for_function(&let_stmt.value, instructions)?;
                instructions.push(BytecodeInstruction::Store(let_stmt.name.clone()));
            }
            Statement::Expression(expr) => {
                if let Expression::Block(statements) = expr {
                    for statement in statements {
                        self.compile_statement_for_function(statement, instructions)?;
                    }
                } else {
                    self.compile_expression_for_function(expr, instructions)?;
                    // Only pop if the expression is not a block or function call
                    if !matches!(expr, Expression::Block(_)) && !matches!(expr, Expression::FunctionCall(_, _)) {
                        instructions.push(BytecodeInstruction::Pop);
                    }
                }
            }
            Statement::Return(return_stmt) => {
                if let Some(expr) = &return_stmt.value {
                    self.compile_expression_for_function(expr, instructions)?;
                } else {
                    instructions.push(BytecodeInstruction::PushNull);
                }
                instructions.push(BytecodeInstruction::Return);
            }
            _ => return Err(CompilerError::syntax_error("Unsupported statement type in function")),
        }
        Ok(())
    }
    
    fn compile_expression(&mut self, expression: &Expression) -> Result<(), CompilerError> {
        match expression {
            Expression::Literal(literal) => {
                match literal {
                    Literal::Int(value) => self.instructions.push(BytecodeInstruction::PushInt(*value)),
                    Literal::Float(value) => self.instructions.push(BytecodeInstruction::PushFloat(*value)),
                    Literal::String(value) => self.instructions.push(BytecodeInstruction::PushString(value.clone())),
                    Literal::Bool(value) => self.instructions.push(BytecodeInstruction::PushBool(*value)),
                    Literal::Char(value) => self.instructions.push(BytecodeInstruction::PushString(value.to_string())),
                    Literal::Array(_) => return Err(CompilerError::syntax_error("Array literals not supported yet")),
                    Literal::Null => self.instructions.push(BytecodeInstruction::PushNull),
                }
            }
            Expression::Identifier(name) => {
                self.instructions.push(BytecodeInstruction::Load(name.clone()));
            }
            Expression::BinaryOp(bin_op) => {
                self.compile_expression(&bin_op.left)?;
                self.compile_expression(&bin_op.right)?;
                match bin_op.operator {
                    BinaryOperator::Add => self.instructions.push(BytecodeInstruction::Add),
                    BinaryOperator::Subtract => self.instructions.push(BytecodeInstruction::Sub),
                    BinaryOperator::Multiply => self.instructions.push(BytecodeInstruction::Mul),
                    BinaryOperator::Divide => self.instructions.push(BytecodeInstruction::Div),
                    _ => return Err(CompilerError::syntax_error("Unsupported binary operator")),
                }
            }
            Expression::FunctionCall(function, args) => {
                // Compile arguments
                for arg in args {
                    self.compile_expression(&arg.value)?;
                }
                
                match &**function {
                    Expression::Identifier(func_name) => {
                        match func_name.as_str() {
                            "print" => self.instructions.push(BytecodeInstruction::Print),
                            "println" => self.instructions.push(BytecodeInstruction::Println),
                            "read_line" => self.instructions.push(BytecodeInstruction::ReadLine),
                            _ => {
                                // User-defined function call
                                self.instructions.push(BytecodeInstruction::Call(func_name.clone(), args.len()));
                            }
                        }
                    }
                    _ => {
                        // Dynamic function call
                        self.compile_expression(function)?;
                        self.instructions.push(BytecodeInstruction::Call("".to_string(), args.len()));
                    }
                }
            }
            Expression::Block(statements) => {
                let mut temp_instructions = Vec::new();
                for statement in statements {
                    self.compile_statement_for_function(statement, &mut temp_instructions)?;
                }
                self.instructions.extend(temp_instructions);
            }
            _ => return Err(CompilerError::syntax_error("Unsupported expression type")),
        }
        Ok(())
    }
    
    fn compile_expression_for_function(&mut self, expression: &Expression, instructions: &mut Vec<BytecodeInstruction>) -> Result<(), CompilerError> {
        match expression {
            Expression::Literal(literal) => {
                match literal {
                    Literal::Int(value) => instructions.push(BytecodeInstruction::PushInt(*value)),
                    Literal::Float(value) => instructions.push(BytecodeInstruction::PushFloat(*value)),
                    Literal::String(value) => instructions.push(BytecodeInstruction::PushString(value.clone())),
                    Literal::Bool(value) => instructions.push(BytecodeInstruction::PushBool(*value)),
                    Literal::Char(value) => instructions.push(BytecodeInstruction::PushString(value.to_string())),
                    Literal::Array(_) => return Err(CompilerError::syntax_error("Array literals not supported yet")),
                    Literal::Null => instructions.push(BytecodeInstruction::PushNull),
                }
            }
            Expression::Identifier(name) => {
                instructions.push(BytecodeInstruction::Load(name.clone()));
            }
            Expression::BinaryOp(bin_op) => {
                self.compile_expression_for_function(&bin_op.left, instructions)?;
                self.compile_expression_for_function(&bin_op.right, instructions)?;
                match bin_op.operator {
                    BinaryOperator::Add => instructions.push(BytecodeInstruction::Add),
                    BinaryOperator::Subtract => instructions.push(BytecodeInstruction::Sub),
                    BinaryOperator::Multiply => instructions.push(BytecodeInstruction::Mul),
                    BinaryOperator::Divide => instructions.push(BytecodeInstruction::Div),
                    _ => return Err(CompilerError::syntax_error("Unsupported binary operator")),
                }
            }
            Expression::FunctionCall(function, args) => {
                // Compile arguments
                for arg in args {
                    self.compile_expression_for_function(&arg.value, instructions)?;
                }
                
                match &**function {
                    Expression::Identifier(func_name) => {
                        match func_name.as_str() {
                            "print" => instructions.push(BytecodeInstruction::Print),
                            "println" => instructions.push(BytecodeInstruction::Println),
                            "read_line" => instructions.push(BytecodeInstruction::ReadLine),
                            _ => {
                                // User-defined function call
                                instructions.push(BytecodeInstruction::Call(func_name.clone(), args.len()));
                            }
                        }
                    }
                    _ => {
                        // Dynamic function call
                        self.compile_expression_for_function(function, instructions)?;
                        instructions.push(BytecodeInstruction::Call("".to_string(), args.len()));
                    }
                }
            }
            Expression::Block(statements) => {
                for statement in statements {
                    self.compile_statement_for_function(statement, instructions)?;
                }
            }
            _ => return Err(CompilerError::syntax_error("Unsupported expression type")),
        }
        Ok(())
    }
    
    fn create_label(&mut self) -> String {
        let label = format!("label_{}", self.label_counter);
        self.label_counter += 1;
        label
    }
    
    pub fn get_function_definitions(&self) -> HashMap<String, Vec<BytecodeInstruction>> {
        self.function_definitions.clone()
    }
} 