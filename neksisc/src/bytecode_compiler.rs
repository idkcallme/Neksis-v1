use crate::ast::{
    Program, Statement, Expression, Literal, BinaryOperator, UnaryOperator
};
use crate::vm::BytecodeInstruction;
use crate::error::CompilerError;
use std::collections::HashMap;

pub struct BytecodeCompiler {
    instructions: Vec<BytecodeInstruction>,
    function_definitions: HashMap<String, Vec<BytecodeInstruction>>,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            function_definitions: HashMap::new(),
        }
    }
    
    pub fn compile_program(&mut self, program: &Program) -> Result<Vec<BytecodeInstruction>, CompilerError> {
        let mut has_main_function = false;
        
        // First pass: compile all statements and track if main exists
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                if func_stmt.name == "main" {
                    has_main_function = true;
                }
            }
            self.compile_statement(statement)?;
        }
        
        // If there's a main function, automatically call it
        if has_main_function {
            self.instructions.push(BytecodeInstruction::Call("main".to_string(), 0));
        }
        
        Ok(self.instructions.clone())
    }
    
    fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Let(let_stmt) => {
                self.compile_expression(&let_stmt.value)?;
                self.instructions.push(BytecodeInstruction::Store(let_stmt.name.clone()));
            }
            Statement::AssignmentStatement { name, value } => {
                self.compile_expression(value)?;
                self.instructions.push(BytecodeInstruction::Store(name.clone()));
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                // Pop the result if it's not needed (statement context)
                // If expressions always produce a value, so always pop in statement context
                match expr {
                    Expression::If(_) => {
                        // If expressions always produce a null value, pop it in statement context
                        self.instructions.push(BytecodeInstruction::Pop);
                    }
                    Expression::Block(_) => {
                        // Block expressions leave values, pop them in statement context
                        self.instructions.push(BytecodeInstruction::Pop);
                    }
                    Expression::FunctionCall(_, _) => {
                        // Function calls may or may not leave values, most do
                        self.instructions.push(BytecodeInstruction::Pop);
                    }
                    _ => {
                        // Other expressions leave values, pop them
                        self.instructions.push(BytecodeInstruction::Pop);
                    }
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
                    
                    // Generate parameter storage instructions
                    // Parameters are expected to be on the stack in reverse order
                    // So we store them in reverse order to match the arguments
                    for (_i, param) in func_stmt.parameters.iter().rev().enumerate() {
                        // Store parameter in local variable
                        temp_instructions.push(BytecodeInstruction::Store(param.name.clone()));
                    }
                    
                    for statement in statements {
                        self.compile_statement_for_function(statement, &mut temp_instructions)?;
                    }
                    // Add implicit return if the last instruction is not a Return
                    if temp_instructions.is_empty() || !matches!(temp_instructions.last(), Some(BytecodeInstruction::Return)) {
                        temp_instructions.push(BytecodeInstruction::PushNull);
                        temp_instructions.push(BytecodeInstruction::Return);
                    }
                    
                    // Now adjust all jump addresses to be relative to the main instructions array
                    let function_start_in_main = self.instructions.len();
                    for instruction in &mut temp_instructions {
                        match instruction {
                            BytecodeInstruction::Jump(addr) => {
                                *addr += function_start_in_main;
                            }
                            BytecodeInstruction::JumpIfFalse(addr) => {
                                *addr += function_start_in_main;
                            }
                            BytecodeInstruction::JumpIfTrue(addr) => {
                                *addr += function_start_in_main;
                            }
                            _ => {}
                        }
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
            Statement::AssignmentStatement { name, value } => {
                self.compile_expression_for_function(value, instructions)?;
                instructions.push(BytecodeInstruction::Store(name.clone()));
            }
            Statement::Expression(expr) => {
                if let Expression::Block(statements) = expr {
                    for statement in statements {
                        self.compile_statement_for_function(statement, instructions)?;
                    }
                } else {
                    self.compile_expression_for_function(expr, instructions)?;
                    // If expressions don't leave values on the stack in statement context
                    match expr {
                        Expression::If(_) => {
                            // If expressions in function context now leave a null value, pop it in statement context
                            instructions.push(BytecodeInstruction::Pop);
                        }
                        Expression::FunctionCall(func, _) => {
                            // Check what kind of function call this is
                            if let Expression::Identifier(func_name) = &**func {
                                match func_name.as_str() {
                                    "print" | "println" => {
                                        // Print functions don't leave results on the stack
                                    }
                                    _ => {
                                        // Other function calls may leave values, pop them
                                        instructions.push(BytecodeInstruction::Pop);
                                    }
                                }
                            } else {
                                // Other function calls may leave values, pop them
                                instructions.push(BytecodeInstruction::Pop);
                            }
                        }
                        _ => {
                            // Other expressions leave values, pop them
                            instructions.push(BytecodeInstruction::Pop);
                        }
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
                    Literal::Array(elements) => {
                        // Create array with elements
                        for element in elements {
                            match element {
                                Literal::Int(value) => self.instructions.push(BytecodeInstruction::PushInt(*value)),
                                Literal::Float(value) => self.instructions.push(BytecodeInstruction::PushFloat(*value)),
                                Literal::String(value) => self.instructions.push(BytecodeInstruction::PushString(value.clone())),
                                Literal::Bool(value) => self.instructions.push(BytecodeInstruction::PushBool(*value)),
                                Literal::Char(value) => self.instructions.push(BytecodeInstruction::PushString(value.to_string())),
                                Literal::Null => self.instructions.push(BytecodeInstruction::PushNull),
                                _ => return Err(CompilerError::syntax_error("Unsupported array element type")),
                            }
                        }
                        self.instructions.push(BytecodeInstruction::NewArray);
                    },
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
                    BinaryOperator::Modulo => self.instructions.push(BytecodeInstruction::Mod),
                    BinaryOperator::Equal => self.instructions.push(BytecodeInstruction::Eq),
                    BinaryOperator::NotEqual => self.instructions.push(BytecodeInstruction::Ne),
                    BinaryOperator::LessThan => self.instructions.push(BytecodeInstruction::Lt),
                    BinaryOperator::LessThanOrEqual => self.instructions.push(BytecodeInstruction::Le),
                    BinaryOperator::GreaterThan => self.instructions.push(BytecodeInstruction::Gt),
                    BinaryOperator::GreaterThanOrEqual => self.instructions.push(BytecodeInstruction::Ge),
                    BinaryOperator::And => self.instructions.push(BytecodeInstruction::And),
                    BinaryOperator::Or => self.instructions.push(BytecodeInstruction::Or),
                    _ => return Err(CompilerError::syntax_error("Unsupported binary operator")),
                }
            }
            Expression::UnaryOp(unary_op) => {
                self.compile_expression(&unary_op.operand)?;
                match unary_op.operator {
                    UnaryOperator::Negate => self.instructions.push(BytecodeInstruction::Neg),
                    UnaryOperator::Not => self.instructions.push(BytecodeInstruction::Not),
                    _ => return Err(CompilerError::syntax_error("Unsupported unary operator")),
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
                            "print" => self.instructions.push(BytecodeInstruction::Println),
                            "println" => self.instructions.push(BytecodeInstruction::Println),
                            "read_line" => self.instructions.push(BytecodeInstruction::ReadLine),
                            "read_file" => self.instructions.push(BytecodeInstruction::ReadFile),
                            "write_file" => self.instructions.push(BytecodeInstruction::WriteFile),
                            "append_file" => self.instructions.push(BytecodeInstruction::AppendFile),
                            "file_exists" => self.instructions.push(BytecodeInstruction::FileExists),
                            "abs" => self.instructions.push(BytecodeInstruction::Abs),
                            "sqrt" => self.instructions.push(BytecodeInstruction::Sqrt),
                            "sin" => self.instructions.push(BytecodeInstruction::Sin),
                            "cos" => self.instructions.push(BytecodeInstruction::Cos),
                            "tan" => self.instructions.push(BytecodeInstruction::Tan),
                            "floor" => self.instructions.push(BytecodeInstruction::Floor),
                            "ceil" => self.instructions.push(BytecodeInstruction::Ceil),
                            "round" => self.instructions.push(BytecodeInstruction::Round),
                            "pow" => self.instructions.push(BytecodeInstruction::Pow),
                            "min" => self.instructions.push(BytecodeInstruction::Min),
                            "max" => self.instructions.push(BytecodeInstruction::Max),
                            "len" => self.instructions.push(BytecodeInstruction::StringLen),
                            "substring" => self.instructions.push(BytecodeInstruction::Substring),
                            "concat" => self.instructions.push(BytecodeInstruction::StringConcat),
                            "contains" => self.instructions.push(BytecodeInstruction::StringContains),
                            "starts_with" => self.instructions.push(BytecodeInstruction::StringStartsWith),
                            "ends_with" => self.instructions.push(BytecodeInstruction::StringEndsWith),
                            "to_upper" => self.instructions.push(BytecodeInstruction::StringToUpper),
                            "to_lower" => self.instructions.push(BytecodeInstruction::StringToLower),
                            "trim" => self.instructions.push(BytecodeInstruction::StringTrim),
                            "split" => self.instructions.push(BytecodeInstruction::StringSplit),
                            "join" => self.instructions.push(BytecodeInstruction::StringJoin),
                            "random" => self.instructions.push(BytecodeInstruction::Random),
                            "random_int" => self.instructions.push(BytecodeInstruction::RandomInt),
                            "typeof" => self.instructions.push(BytecodeInstruction::TypeOf),
                            "time" => self.instructions.push(BytecodeInstruction::Time),
                            "sleep" => self.instructions.push(BytecodeInstruction::Sleep),
                            "exit" => self.instructions.push(BytecodeInstruction::Exit),
                            
                            // Advanced Data Structures - HashMap/Dictionary
                            "dict_new" => self.instructions.push(BytecodeInstruction::DictNew),
                            "dict_set" => self.instructions.push(BytecodeInstruction::DictSet),
                            "dict_get" => self.instructions.push(BytecodeInstruction::DictGet),
                            "dict_has" => self.instructions.push(BytecodeInstruction::DictHas),
                            "dict_keys" => self.instructions.push(BytecodeInstruction::DictKeys),
                            "dict_size" => self.instructions.push(BytecodeInstruction::DictSize),
                            "dict_remove" => self.instructions.push(BytecodeInstruction::DictRemove),
                            "dict_clear" => self.instructions.push(BytecodeInstruction::DictClear),
                            
                            // Advanced Array functions
                            "array_push" => self.instructions.push(BytecodeInstruction::ArrayPush),
                            "array_pop" => self.instructions.push(BytecodeInstruction::ArrayPop),
                            "array_reverse" => self.instructions.push(BytecodeInstruction::ArrayReverse),
                            "array_sort" => self.instructions.push(BytecodeInstruction::ArraySort),
                            "array_filter" => self.instructions.push(BytecodeInstruction::ArrayFilter),
                            "array_map" => self.instructions.push(BytecodeInstruction::ArrayMap),
                            "array_reduce" => self.instructions.push(BytecodeInstruction::ArrayReduce),
                            "array_find" => self.instructions.push(BytecodeInstruction::ArrayFind),
                            "array_slice" => self.instructions.push(BytecodeInstruction::ArraySlice),
                            
                            // JSON Support
                            "json_parse" => self.instructions.push(BytecodeInstruction::JsonParse),
                            "json_stringify" => self.instructions.push(BytecodeInstruction::JsonStringify),
                            
                            // Error Handling
                            "try_catch" => self.instructions.push(BytecodeInstruction::TryCatch),
                            "throw_error" => self.instructions.push(BytecodeInstruction::ThrowError),
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
                let mut has_return_value = false;
                
                for (i, statement) in statements.iter().enumerate() {
                    self.compile_statement_for_function(statement, &mut temp_instructions)?;
                    
                    // For block expressions, only the last statement should leave a value on the stack
                    // All other statements should have their values popped
                    if i < statements.len() - 1 {
                        // Not the last statement - check if it's an expression that leaves a value
                        if let Statement::Expression(expr) = statement {
                            if !matches!(expr, Expression::Block(_)) && 
                               !matches!(expr, Expression::FunctionCall(_, _)) &&
                               !matches!(expr, Expression::Assignment(_)) {
                                // This statement left a value, but it's not the last one, so pop it
                                // Actually, compile_statement_for_function already handles popping for expressions
                                // So we don't need to add extra pops here
                            }
                        }
                    } else {
                        // This is the last statement - check if it provides a return value
                        if let Statement::Expression(_) = statement {
                            has_return_value = true;
                        }
                    }
                }
                
                // If the block doesn't have a return value from the last statement, push null
                if !has_return_value {
                    temp_instructions.push(BytecodeInstruction::PushNull);
                }
                
                // Adjust jump addresses to be relative to the main instructions array
                let block_start_in_main = self.instructions.len();
                for instruction in &mut temp_instructions {
                    match instruction {
                        BytecodeInstruction::Jump(addr) => {
                            *addr += block_start_in_main;
                        }
                        BytecodeInstruction::JumpIfFalse(addr) => {
                            *addr += block_start_in_main;
                        }
                        BytecodeInstruction::JumpIfTrue(addr) => {
                            *addr += block_start_in_main;
                        }
                        _ => {}
                    }
                }
                
                self.instructions.extend(temp_instructions);
            }
            Expression::If(if_expr) => {
                // Compile condition
                self.compile_expression(&if_expr.condition)?;
                
                // Add conditional jump instruction (placeholder)
                let jump_if_false_index = self.instructions.len();
                self.instructions.push(BytecodeInstruction::JumpIfFalse(0));
                
                // Compile then branch
                self.compile_expression(&if_expr.then_branch)?;
                
                if let Some(else_expr) = &if_expr.else_branch {
                    // Add jump to skip else branch (placeholder)
                    let jump_index = self.instructions.len();
                    self.instructions.push(BytecodeInstruction::Jump(0));
                    
                    // Update the conditional jump to point to the start of the else branch
                    let else_start = self.instructions.len();
                    self.instructions[jump_if_false_index] = BytecodeInstruction::JumpIfFalse(else_start);
                    
                    // Compile else branch
                    self.compile_expression(else_expr)?;
                    
                    // Update the unconditional jump to point after the else branch
                    let after_if = self.instructions.len();
                    self.instructions[jump_index] = BytecodeInstruction::Jump(after_if);
                    
                    // If expressions with else always produce a null value for consistency
                    self.instructions.push(BytecodeInstruction::PushNull);
                } else {
                    // No else branch - just update the conditional jump to skip the then branch
                    let after_if = self.instructions.len();
                    self.instructions[jump_if_false_index] = BytecodeInstruction::JumpIfFalse(after_if);
                    
                    // If expressions without else also produce a null value for consistency
                    self.instructions.push(BytecodeInstruction::PushNull);
                }
            }
            Expression::While(while_expr) => {
                // Mark the start of the loop
                let loop_start = self.instructions.len();
                
                // Compile condition
                self.compile_expression(&while_expr.condition)?;
                
                // Add conditional jump to exit loop if condition is false
                let jump_if_false_index = self.instructions.len();
                self.instructions.push(BytecodeInstruction::JumpIfFalse(0));
                
                // Compile loop body
                self.compile_expression(&while_expr.body)?;
                
                // Add unconditional jump back to loop start
                self.instructions.push(BytecodeInstruction::Jump(loop_start));
                
                // Update the conditional jump to exit the loop
                let after_loop = self.instructions.len();
                self.instructions[jump_if_false_index] = BytecodeInstruction::JumpIfFalse(after_loop);
                
                // Push null as the while loop's return value
                self.instructions.push(BytecodeInstruction::PushNull);
            }
            Expression::ArrayAccess(array_access) => {
                // Compile array expression
                self.compile_expression(&array_access.array)?;
                // Compile index expression
                self.compile_expression(&array_access.index)?;
                // Get array element
                self.instructions.push(BytecodeInstruction::GetIndex);
            }
            Expression::InterpolatedString(interpolated) => {
                // Handle interpolated strings by concatenating parts
                if interpolated.parts.is_empty() {
                    // Empty interpolated string becomes empty string
                    self.instructions.push(BytecodeInstruction::PushString(String::new()));
                } else {
                    // Process first part
                    match &interpolated.parts[0] {
                        crate::ast::InterpolatedPart::String(s) => {
                            self.instructions.push(BytecodeInstruction::PushString(s.clone()));
                        }
                        crate::ast::InterpolatedPart::Expr(expr) => {
                            self.compile_expression(expr)?;
                            self.instructions.push(BytecodeInstruction::ToString);
                        }
                    }
                    
                    // Concatenate remaining parts
                    for part in &interpolated.parts[1..] {
                        match part {
                            crate::ast::InterpolatedPart::String(s) => {
                                self.instructions.push(BytecodeInstruction::PushString(s.clone()));
                            }
                            crate::ast::InterpolatedPart::Expr(expr) => {
                                self.compile_expression(expr)?;
                                self.instructions.push(BytecodeInstruction::ToString);
                            }
                        }
                        self.instructions.push(BytecodeInstruction::StringConcat);
                    }
                }
            }
            Expression::Assignment(assign_expr) => {
                // Compile the value to assign
                self.compile_expression(&assign_expr.value)?;
                // Duplicate the value on stack (one for storage, one to return)
                self.instructions.push(BytecodeInstruction::Dup);
                // Store the value
                self.instructions.push(BytecodeInstruction::Store(assign_expr.target.clone()));
                // The duplicate value remains on stack as the expression's result
            }
            _ => return Err(CompilerError::syntax_error(&format!("Unsupported expression type: {:?}", expression))),
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
                    Literal::Array(elements) => {
                        // Create array with elements
                        for element in elements {
                            match element {
                                Literal::Int(value) => instructions.push(BytecodeInstruction::PushInt(*value)),
                                Literal::Float(value) => instructions.push(BytecodeInstruction::PushFloat(*value)),
                                Literal::String(value) => instructions.push(BytecodeInstruction::PushString(value.clone())),
                                Literal::Bool(value) => instructions.push(BytecodeInstruction::PushBool(*value)),
                                Literal::Char(value) => instructions.push(BytecodeInstruction::PushString(value.to_string())),
                                Literal::Null => instructions.push(BytecodeInstruction::PushNull),
                                _ => return Err(CompilerError::syntax_error("Unsupported array element type")),
                            }
                        }
                        instructions.push(BytecodeInstruction::NewArray);
                    },
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
                    BinaryOperator::Modulo => instructions.push(BytecodeInstruction::Mod),
                    BinaryOperator::Equal => instructions.push(BytecodeInstruction::Eq),
                    BinaryOperator::NotEqual => instructions.push(BytecodeInstruction::Ne),
                    BinaryOperator::LessThan => instructions.push(BytecodeInstruction::Lt),
                    BinaryOperator::LessThanOrEqual => instructions.push(BytecodeInstruction::Le),
                    BinaryOperator::GreaterThan => instructions.push(BytecodeInstruction::Gt),
                    BinaryOperator::GreaterThanOrEqual => instructions.push(BytecodeInstruction::Ge),
                    BinaryOperator::And => instructions.push(BytecodeInstruction::And),
                    BinaryOperator::Or => instructions.push(BytecodeInstruction::Or),
                    _ => return Err(CompilerError::syntax_error("Unsupported binary operator")),
                }
            }
            Expression::UnaryOp(unary_op) => {
                self.compile_expression_for_function(&unary_op.operand, instructions)?;
                match unary_op.operator {
                    UnaryOperator::Negate => instructions.push(BytecodeInstruction::Neg),
                    UnaryOperator::Not => instructions.push(BytecodeInstruction::Not),
                    _ => return Err(CompilerError::syntax_error("Unsupported unary operator")),
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
                            "print" => instructions.push(BytecodeInstruction::Println),
                            "println" => instructions.push(BytecodeInstruction::Println),
                            "read_line" => instructions.push(BytecodeInstruction::ReadLine),
                            "read_file" => instructions.push(BytecodeInstruction::ReadFile),
                            "write_file" => instructions.push(BytecodeInstruction::WriteFile),
                            "append_file" => instructions.push(BytecodeInstruction::AppendFile),
                            "file_exists" => instructions.push(BytecodeInstruction::FileExists),
                            "abs" => instructions.push(BytecodeInstruction::Abs),
                            "sqrt" => instructions.push(BytecodeInstruction::Sqrt),
                            "sin" => instructions.push(BytecodeInstruction::Sin),
                            "cos" => instructions.push(BytecodeInstruction::Cos),
                            "tan" => instructions.push(BytecodeInstruction::Tan),
                            "floor" => instructions.push(BytecodeInstruction::Floor),
                            "ceil" => instructions.push(BytecodeInstruction::Ceil),
                            "round" => instructions.push(BytecodeInstruction::Round),
                            "pow" => instructions.push(BytecodeInstruction::Pow),
                            "min" => instructions.push(BytecodeInstruction::Min),
                            "max" => instructions.push(BytecodeInstruction::Max),
                            "len" => instructions.push(BytecodeInstruction::StringLen),
                            "substring" => instructions.push(BytecodeInstruction::Substring),
                            "concat" => instructions.push(BytecodeInstruction::StringConcat),
                            "contains" => instructions.push(BytecodeInstruction::StringContains),
                            "starts_with" => instructions.push(BytecodeInstruction::StringStartsWith),
                            "ends_with" => instructions.push(BytecodeInstruction::StringEndsWith),
                            "to_upper" => instructions.push(BytecodeInstruction::StringToUpper),
                            "to_lower" => instructions.push(BytecodeInstruction::StringToLower),
                            "trim" => instructions.push(BytecodeInstruction::StringTrim),
                            "split" => instructions.push(BytecodeInstruction::StringSplit),
                            "join" => instructions.push(BytecodeInstruction::StringJoin),
                            "random" => instructions.push(BytecodeInstruction::Random),
                            "random_int" => instructions.push(BytecodeInstruction::RandomInt),
                            "typeof" => instructions.push(BytecodeInstruction::TypeOf),
                            "time" => instructions.push(BytecodeInstruction::Time),
                            "sleep" => instructions.push(BytecodeInstruction::Sleep),
                            "exit" => instructions.push(BytecodeInstruction::Exit),
                            
                            // Advanced Data Structures - HashMap/Dictionary
                            "dict_new" => instructions.push(BytecodeInstruction::DictNew),
                            "dict_set" => instructions.push(BytecodeInstruction::DictSet),
                            "dict_get" => instructions.push(BytecodeInstruction::DictGet),
                            "dict_has" => instructions.push(BytecodeInstruction::DictHas),
                            "dict_keys" => instructions.push(BytecodeInstruction::DictKeys),
                            "dict_size" => instructions.push(BytecodeInstruction::DictSize),
                            "dict_remove" => instructions.push(BytecodeInstruction::DictRemove),
                            "dict_clear" => instructions.push(BytecodeInstruction::DictClear),
                            
                            // Advanced Array functions
                            "array_push" => instructions.push(BytecodeInstruction::ArrayPush),
                            "array_pop" => instructions.push(BytecodeInstruction::ArrayPop),
                            "array_reverse" => instructions.push(BytecodeInstruction::ArrayReverse),
                            "array_sort" => instructions.push(BytecodeInstruction::ArraySort),
                            "array_filter" => instructions.push(BytecodeInstruction::ArrayFilter),
                            "array_map" => instructions.push(BytecodeInstruction::ArrayMap),
                            "array_reduce" => instructions.push(BytecodeInstruction::ArrayReduce),
                            "array_find" => instructions.push(BytecodeInstruction::ArrayFind),
                            "array_slice" => instructions.push(BytecodeInstruction::ArraySlice),
                            
                            // JSON Support
                            "json_parse" => instructions.push(BytecodeInstruction::JsonParse),
                            "json_stringify" => instructions.push(BytecodeInstruction::JsonStringify),
                            
                            // Error Handling
                            "try_catch" => instructions.push(BytecodeInstruction::TryCatch),
                            "throw_error" => instructions.push(BytecodeInstruction::ThrowError),
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
                let mut has_return_value = false;
                
                for (i, statement) in statements.iter().enumerate() {
                    self.compile_statement_for_function(statement, instructions)?;
                    
                    // For the last statement in a block, check if it provides a return value
                    if i == statements.len() - 1 {
                        if let Statement::Expression(_) = statement {
                            has_return_value = true;
                        }
                    }
                }
                
                // If the block doesn't have a return value from the last statement, push null
                if !has_return_value {
                    instructions.push(BytecodeInstruction::PushNull);
                }
            }
            Expression::If(if_expr) => {
                // Compile condition
                self.compile_expression_for_function(&if_expr.condition, instructions)?;
                
                // Add conditional jump instruction (placeholder)
                let jump_if_false_index = instructions.len();
                instructions.push(BytecodeInstruction::JumpIfFalse(0));
                
                // Compile then branch
                self.compile_expression_for_function(&if_expr.then_branch, instructions)?;
                
                if let Some(else_expr) = &if_expr.else_branch {
                    // Add jump to skip else branch (placeholder)
                    let jump_index = instructions.len();
                    instructions.push(BytecodeInstruction::Jump(0));
                    
                    // Update the conditional jump to point to the start of the else branch
                    let else_start = instructions.len();
                    instructions[jump_if_false_index] = BytecodeInstruction::JumpIfFalse(else_start);
                    
                    // Compile else branch
                    self.compile_expression_for_function(else_expr, instructions)?;
                    
                    // Update the unconditional jump to point to after the else branch
                    let after_else = instructions.len();
                    instructions[jump_index] = BytecodeInstruction::Jump(after_else);
                    
                    // If expressions with else always produce a null value for consistency
                    instructions.push(BytecodeInstruction::PushNull);
                } else {
                    // No else branch - update the conditional jump to point after the then branch
                    let after_then = instructions.len();
                    instructions[jump_if_false_index] = BytecodeInstruction::JumpIfFalse(after_then);
                    
                    // If expressions without else also produce a null value for consistency
                    instructions.push(BytecodeInstruction::PushNull);
                }
            }
            Expression::While(while_expr) => {
                // Mark the start of the loop
                let loop_start = instructions.len();
                
                // Compile condition
                self.compile_expression_for_function(&while_expr.condition, instructions)?;
                
                // Add conditional jump to exit loop if condition is false
                let jump_if_false_index = instructions.len();
                instructions.push(BytecodeInstruction::JumpIfFalse(0));
                
                // Compile loop body
                self.compile_expression_for_function(&while_expr.body, instructions)?;
                
                // Add unconditional jump back to loop start
                instructions.push(BytecodeInstruction::Jump(loop_start));
                
                // Update the conditional jump to exit the loop
                let after_loop = instructions.len();
                instructions[jump_if_false_index] = BytecodeInstruction::JumpIfFalse(after_loop);
                
                // Push null as the while loop's return value
                instructions.push(BytecodeInstruction::PushNull);
            }
            Expression::ArrayAccess(array_access) => {
                // Compile array expression
                self.compile_expression_for_function(&array_access.array, instructions)?;
                // Compile index expression
                self.compile_expression_for_function(&array_access.index, instructions)?;
                // Get array element
                instructions.push(BytecodeInstruction::GetIndex);
            }
            Expression::InterpolatedString(interpolated) => {
                // Handle interpolated strings by concatenating parts
                if interpolated.parts.is_empty() {
                    // Empty interpolated string becomes empty string
                    instructions.push(BytecodeInstruction::PushString(String::new()));
                } else {
                    // Process first part
                    match &interpolated.parts[0] {
                        crate::ast::InterpolatedPart::String(s) => {
                            instructions.push(BytecodeInstruction::PushString(s.clone()));
                        }
                        crate::ast::InterpolatedPart::Expr(expr) => {
                            self.compile_expression_for_function(expr, instructions)?;
                            instructions.push(BytecodeInstruction::ToString);
                        }
                    }
                    
                    // Concatenate remaining parts
                    for part in &interpolated.parts[1..] {
                        match part {
                            crate::ast::InterpolatedPart::String(s) => {
                                instructions.push(BytecodeInstruction::PushString(s.clone()));
                            }
                            crate::ast::InterpolatedPart::Expr(expr) => {
                                self.compile_expression_for_function(expr, instructions)?;
                                instructions.push(BytecodeInstruction::ToString);
                            }
                        }
                        instructions.push(BytecodeInstruction::StringConcat);
                    }
                }
            }
            Expression::Assignment(assign_expr) => {
                // Compile the value to assign
                self.compile_expression_for_function(&assign_expr.value, instructions)?;
                // Duplicate the value on stack (one for storage, one to return)
                instructions.push(BytecodeInstruction::Dup);
                // Store the value
                instructions.push(BytecodeInstruction::Store(assign_expr.target.clone()));
                // The duplicate value remains on stack as the expression's result
            }
            _ => return Err(CompilerError::syntax_error(&format!("Unsupported expression type in function: {:?}", expression))),
        }
        Ok(())
    }
    

    
    pub fn get_function_definitions(&self) -> HashMap<String, Vec<BytecodeInstruction>> {
        self.function_definitions.clone()
    }
} 