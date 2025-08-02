#[allow(dead_code)]
use crate::ast::{
    Expression, BoxExpression,
    RcExpression, ArcExpression, CellExpression, RefCellExpression,
    MallocExpression, FreeExpression, ReallocExpression, LetStatement, AssignmentStatement, ReturnStatement,
    FunctionStatement, Statement, Program, CallArgument, Pattern
};
use crate::error::CompilerError;
use crate::compiler::CompilerOptions;
use crate::stdlib::StandardLibrary;
use std::collections::HashMap;
use std::env;

fn should_emit_asm() -> bool {
    env::var("NEKSIS_EMIT_ASM").is_ok()
}

pub trait CodeGenerator {
    fn generate(&mut self, program: &Program) -> Result<String, CompilerError>;
    fn write_to_file(&self, content: &str, path: &std::path::Path) -> Result<(), CompilerError>;
}

pub struct SimpleCodeGen {
    variables: HashMap<String, String>,
    temp_counter: u32,
    label_counter: u32,
    stdlib: StandardLibrary,
}

impl SimpleCodeGen {
    pub fn new(_options: CompilerOptions) -> Result<Self, CompilerError> {
        Ok(Self {
            variables: HashMap::new(),
            temp_counter: 0,
            label_counter: 0,
            stdlib: StandardLibrary::new(),
        })
    }

    fn next_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("t{}", self.temp_counter)
    }

    fn next_label(&mut self) -> String {
        self.label_counter += 1;
        format!("L{}", self.label_counter)
    }
}

impl CodeGenerator for SimpleCodeGen {
    fn generate(&mut self, program: &Program) -> Result<String, CompilerError> {
        let mut output = String::new();
        output.push_str("; Generated neksis Code\n");
        output.push_str("; ===================\n\n");
        
        for statement in &program.statements {
            match statement {
                Statement::Function(func_stmt) => {
                    self.generate_function(func_stmt)?;
                }
                _ => {
                    // Other statements are handled within functions
                }
            }
        }
        
        Ok(output)
    }

    fn write_to_file(&self, content: &str, path: &std::path::Path) -> Result<(), CompilerError> {
        std::fs::write(path, content)
            .map_err(|e| CompilerError::parse_error("file", &format!("Failed to write file: {}", e)))?;
        Ok(())
    }
}

pub fn generate_simple_code(program: &Program) -> Result<(), CompilerError> {
    let mut codegen = SimpleCodeGen::new(CompilerOptions::default())?;
    
    if should_emit_asm() {
        println!("; Generated neksis Code");
        println!("; ===================");
        println!();
    }
    
    for statement in &program.statements {
        match statement {
            Statement::Function(func_stmt) => {
                codegen.generate_function(func_stmt)?;
            }
            _ => {
                // TODO: Implement other top-level statements
                if should_emit_asm() {
                    println!("; TODO: Implement {:?}", statement);
                }
            }
        }
    }
    
    Ok(())
}

impl SimpleCodeGen {
    fn generate_function(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        if should_emit_asm() {
            println!("; Function: {}", func_stmt.name);
            println!("{}:", func_stmt.name);
        }
        
        // Generate function prologue
        if should_emit_asm() {
            println!("  push rbp");
            println!("  mov rbp, rsp");
        }
        
        // Generate function body
        let _value = self.generate_expression(&func_stmt.body)?;
        
        // Generate function epilogue
        if should_emit_asm() {
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            println!();
        }
        
        Ok(())
    }

    fn generate_statement(&mut self, statement: &Statement) -> Result<String, CompilerError> {
        match statement {
            Statement::Let(let_stmt) => self.generate_let_statement(let_stmt),
            Statement::Return(return_stmt) => self.generate_return_statement(return_stmt),
            Statement::Expression(expr) => {
                let value = self.generate_expression(expr)?;
                Ok(value)
            }
            Statement::Function(_) => Ok("0".to_string()), // Functions are handled separately
            Statement::Module(_) => Ok("0".to_string()), // Modules are handled at a different level
            Statement::Move(_) => Ok("0".to_string()), // TODO: Implement move semantics
            Statement::Drop(_) => Ok("0".to_string()), // TODO: Implement drop semantics
            Statement::Struct(_) | Statement::Enum(_) | Statement::Use(_) => Ok("0".to_string()), // TODO: Implement
            Statement::GenericFunction(_) => Ok("0".to_string()), // TODO: Implement generic functions
            Statement::Trait(_) | Statement::Impl(_) => Ok("0".to_string()), // TODO: Implement traits and impls
            Statement::Class(_) => Ok("0".to_string()),
            Statement::LetStatement { name, value, var_type } => {
                let value_code = self.generate_expression(value)?;
                let type_annotation = var_type.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
                Ok(format!("let {}{} = {};", name, type_annotation, value_code))
            }
            Statement::AssignmentStatement { name, value } => {
                let value_code = self.generate_expression(value)?;
                Ok(format!("{} = {};", name, value_code))
            }
            Statement::FunctionStatement { name, parameters, return_type, body } => {
                let param_list = parameters.iter()
                    .map(|p| format!("{}: {}", p.name, p.type_annotation))
                    .collect::<Vec<_>>()
                    .join(", ");
                let return_annotation = return_type.as_ref().map(|t| format!(" -> {}", t)).unwrap_or_default();
                let body_code = self.generate_expression(body)?;
                Ok(format!("fn {}({}){} {{ {} }}", name, param_list, return_annotation, body_code))
            }
            Statement::ReturnStatement { value } => {
                let value_code = value.as_ref()
                    .map(|v| self.generate_expression(v))
                    .transpose()?
                    .unwrap_or_else(|| "".to_string());
                Ok(format!("return {};", value_code))
            }
            Statement::ExpressionStatement { expression } => {
                let expr_code = self.generate_expression(expression)?;
                Ok(format!("{};", expr_code))
            } // TODO: Implement class codegen
        }
    }

    fn generate_let_statement(&mut self, let_stmt: &LetStatement) -> Result<String, CompilerError> {
        let value = self.generate_expression(&let_stmt.value)?;
        
        // Store the value in a variable
        self.variables.insert(let_stmt.name.clone(), value.clone());
        
        if should_emit_asm() {
            println!("  ; let {} = {}", let_stmt.name, value);
            println!("  mov {}, {}", let_stmt.name, value);
        }
        
        Ok(value)
    }

    fn generate_return_statement(&mut self, return_stmt: &ReturnStatement) -> Result<String, CompilerError> {
        if let Some(expr) = &return_stmt.value {
            let value = self.generate_expression(expr)?;
            if should_emit_asm() {
                println!("  ; return {}", value);
                println!("  mov rax, {}", value);
            }
            Ok(value)
        } else {
            if should_emit_asm() {
                println!("  ; return");
            }
            Ok("0".to_string())
        }
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<String, CompilerError> {
        match expr {
            Expression::Literal(literal) => self.generate_literal(literal),
            Expression::Identifier(name) => Ok(name.clone()),
            Expression::BinaryOp(binary_op) => {
                let left = self.generate_expression(&binary_op.left)?;
                let right = self.generate_expression(&binary_op.right)?;
                self.generate_binary_operation(left, &binary_op.operator, right)
            }
            Expression::UnaryOp(unary_op) => {
                let operand = self.generate_expression(&unary_op.operand)?;
                self.generate_unary_operation(&unary_op.operator, operand)
            }
            Expression::FunctionCall(func_call, arguments) => self.generate_function_call(func_call, arguments),
            Expression::If(if_expr) => self.generate_if_expression(if_expr),
            Expression::While(while_expr) => self.generate_while_expression(while_expr),
            Expression::Block(block_expr) => self.generate_block_expression(block_expr),
            Expression::Borrow(borrow_expr) => self.generate_borrow_expression(&borrow_expr.expression),
            Expression::BorrowMut(borrow_mut_expr) => self.generate_borrow_mut_expression(borrow_mut_expr),
            Expression::Clone(clone_expr) => self.generate_clone_expression(&clone_expr.expression),
            Expression::Malloc(malloc_expr) => self.generate_malloc_expression(malloc_expr),
            Expression::Free(free_expr) => self.generate_free_expression(free_expr),
            Expression::Realloc(realloc_expr) => self.generate_realloc_expression(realloc_expr),
            Expression::TryCatch(try_catch_expr) => self.generate_try_catch_expression(try_catch_expr),
            Expression::Move(_move_expr) => {
                // Move expressions are statements, not expressions in the AST
                // For now, just return a placeholder
                let temp = self.next_temp();
                if should_emit_asm() {
                    println!("  ; TODO: Implement move expression generation");
                }
                Ok(temp)
            }
            Expression::Drop(_drop_expr) => {
                // Drop expressions are statements, not expressions in the AST
                // For now, just return a placeholder
                let temp = self.next_temp();
                if should_emit_asm() {
                    println!("  ; TODO: Implement drop expression generation");
                }
                Ok(temp)
            }
            Expression::Match(match_expr) => self.generate_match_expression(match_expr),
            Expression::Spawn(spawn_expr) => self.generate_spawn_expression(spawn_expr),
            Expression::Join(join_expr) => self.generate_join_expression(join_expr),
            Expression::Channel(channel_expr) => self.generate_channel_expression(channel_expr),
            Expression::Try(try_expr) => self.generate_try_expression(try_expr),
            Expression::Pipeline(pipeline_expr) => self.generate_pipeline_expression(pipeline_expr),
            Expression::Loop(loop_expr) => self.generate_loop_expression(loop_expr),
            Expression::StructLiteral(struct_literal) => self.generate_struct_literal(struct_literal),
            Expression::MemberAccess(member_access) => self.generate_member_access(member_access),
            Expression::Box(box_expr) => self.generate_box_expression(box_expr),
            Expression::Rc(rc_expr) => self.generate_rc_expression(rc_expr),
            Expression::Arc(arc_expr) => self.generate_arc_expression(arc_expr),
            Expression::Cell(cell_expr) => self.generate_cell_expression(cell_expr),
            Expression::RefCell(refcell_expr) => self.generate_refcell_expression(refcell_expr),
            Expression::Lifetime(lifetime_expr) => self.generate_lifetime_expression(lifetime_expr),
            Expression::Return(return_expr) => self.generate_return_expression(return_expr),
            Expression::Let(let_stmt) => self.generate_let_expression(let_stmt),
            Expression::Assignment(assignment_stmt) => self.generate_assignment_expression(assignment_stmt),
            Expression::BinaryOperation { left, operator, right } => {
                let left_str = self.generate_expression(left)?;
                let right_str = self.generate_expression(right)?;
                self.generate_binary_operation(left_str, operator, right_str)
            }
            Expression::BuiltinFunction { name, arguments } => self.generate_builtin_function_call(name, arguments),
            Expression::ArrayAccess(array_access) => self.generate_array_access(array_access),
            Expression::Throw(throw_expr) => {
                let value = self.generate_expression(&throw_expr.value)?;
                if should_emit_asm() {
                    println!("  ; THROW: {}", value);
                    println!("  ; TODO: Implement real exception stack unwinding and throw");
                }
                // For now, just return a placeholder and error
                Err(CompilerError::codegen_error("simple", "Throw not fully implemented: stack unwinding required"))
            }
            Expression::Lambda(lambda_expr) => {
                if should_emit_asm() {
                    println!("  ; LAMBDA: params = {:?}", lambda_expr.parameters);
                    println!("  ; TODO: Implement real closure/lambda codegen");
                }
                // For now, just return a placeholder and error
                Err(CompilerError::codegen_error("simple", "Lambda/closure not fully implemented"))
            }
            Expression::InterpolatedString(_interp_expr) => {
                if should_emit_asm() {
                    println!("  ; INTERPOLATED STRING");
                }
                // For now, just return a placeholder and error
                Err(CompilerError::codegen_error("simple", "String interpolation not fully implemented"))
            }
            Expression::ListComprehension(list_comp) => {
                if should_emit_asm() {
                    println!("  ; LIST COMPREHENSION: iterator = {}", list_comp.iterator);
                    println!("  ; TODO: Implement real list comprehension codegen");
                }
                Err(CompilerError::codegen_error("simple", "List comprehension not fully implemented"))
            }
            Expression::Slice(_slice_expr) => {
                if should_emit_asm() {
                    println!("  ; SLICE");
                    println!("  ; TODO: Implement real slicing codegen");
                }
                Err(CompilerError::codegen_error("simple", "Slicing not fully implemented"))
            }
            _ => {
                let temp = self.next_temp();
                if should_emit_asm() {
                    println!("  ; TODO: Implement expression generation for {:?}", expr);
                }
                Ok(temp)
            }
        }
    }

    fn generate_literal(&mut self, literal: &crate::ast::Literal) -> Result<String, CompilerError> {
        match literal {
            crate::ast::Literal::Int(value) => Ok(value.to_string()),
            crate::ast::Literal::Float(value) => Ok(value.to_string()),
            crate::ast::Literal::Bool(value) => Ok(if *value { "1" } else { "0" }.to_string()),
            crate::ast::Literal::String(value) => Ok(format!("\"{}\"", value)),
            crate::ast::Literal::Char(value) => Ok(format!("'{}'", value)),
            crate::ast::Literal::Array(elements) => {
                let temp = self.next_temp();
                if should_emit_asm() {
                    println!("  ; Array literal with {} elements", elements.len());
                    println!("  ; TODO: Implement array allocation and initialization");
                    println!("  mov {}, 0", temp); // Placeholder
                }
                Ok(temp)
            }
            crate::ast::Literal::Null => Ok("0".to_string()),
        }
    }

    fn generate_identifier(&self, name: &str) -> Result<String, CompilerError> {
        // For now, just return the name as a register/variable
        // In a real implementation, this would look up the variable in the symbol table
        Ok(name.to_string())
    }

    fn generate_binary_operation(
        &mut self,
        left: String,
        operator: &crate::ast::BinaryOperator,
        right: String,
    ) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        
        match operator {
            crate::ast::BinaryOperator::Add => {
                if should_emit_asm() {
                    println!("  ; {} = {} + {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  add {}, {}", temp, right);
                }
            }
            crate::ast::BinaryOperator::Subtract => {
                if should_emit_asm() {
                    println!("  ; {} = {} - {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  sub {}, {}", temp, right);
                }
            }
            crate::ast::BinaryOperator::Multiply => {
                if should_emit_asm() {
                    println!("  ; {} = {} * {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  imul {}, {}", temp, right);
                }
            }
            crate::ast::BinaryOperator::Divide => {
                if should_emit_asm() {
                    println!("  ; {} = {} / {}", temp, left, right);
                    println!("  mov rax, {}", left);
                    println!("  mov rbx, {}", right);
                    println!("  cdq");
                    println!("  idiv rbx");
                    println!("  mov {}, rax", temp);
                }
            }
            crate::ast::BinaryOperator::Equal => {
                if should_emit_asm() {
                    println!("  ; {} = {} == {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  cmp {}, {}", temp, right);
                    println!("  sete {}", temp);
                }
            }
            crate::ast::BinaryOperator::NotEqual => {
                if should_emit_asm() {
                    println!("  ; {} = {} != {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  cmp {}, {}", temp, right);
                    println!("  setne {}", temp);
                }
            }
            crate::ast::BinaryOperator::GreaterThan => {
                if should_emit_asm() {
                    println!("  ; {} = {} > {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  cmp {}, {}", temp, right);
                    println!("  setg {}", temp);
                }
            }
            crate::ast::BinaryOperator::GreaterThanOrEqual => {
                if should_emit_asm() {
                    println!("  ; {} = {} >= {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  cmp {}, {}", temp, right);
                    println!("  setge {}", temp);
                }
            }
            crate::ast::BinaryOperator::LessThan => {
                if should_emit_asm() {
                    println!("  ; {} = {} < {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  cmp {}, {}", temp, right);
                    println!("  setl {}", temp);
                }
            }
            crate::ast::BinaryOperator::LessThanOrEqual => {
                if should_emit_asm() {
                    println!("  ; {} = {} <= {}", temp, left, right);
                    println!("  mov {}, {}", temp, left);
                    println!("  cmp {}, {}", temp, right);
                    println!("  setle {}", temp);
                }
            }
            _ => return Err(CompilerError::codegen_error("simple", &format!("Unsupported binary operator: {:?}", operator))),
        }
        
        Ok(temp)
    }

    fn generate_unary_operation(
        &mut self,
        operator: &crate::ast::UnaryOperator,
        operand: String,
    ) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        
        match operator {
            crate::ast::UnaryOperator::Negate => {
                if should_emit_asm() {
                    println!("  ; {} = -{}", temp, operand);
                    println!("  mov {}, {}", temp, operand);
                    println!("  neg {}", temp);
                }
            }
            crate::ast::UnaryOperator::Not => {
                if should_emit_asm() {
                    println!("  ; {} = !{}", temp, operand);
                    println!("  mov {}, {}", temp, operand);
                    println!("  not {}", temp);
                }
            }
            crate::ast::UnaryOperator::Move => {
                // For now, just copy the value
                if should_emit_asm() {
                    println!("  ; {} = move {}", temp, operand);
                    println!("  mov {}, {}", temp, operand);
                }
            }
            crate::ast::UnaryOperator::Drop => {
                // For now, just copy the value
                if should_emit_asm() {
                    println!("  ; {} = drop {}", temp, operand);
                    println!("  mov {}, {}", temp, operand);
                }
            }
            crate::ast::UnaryOperator::Copy | 
            crate::ast::UnaryOperator::Borrow | 
            crate::ast::UnaryOperator::BorrowMut => {
                // For now, just copy the value
                if should_emit_asm() {
                    println!("  ; {} = {} (unary op)", temp, operand);
                    println!("  mov {}, {}", temp, operand);
                }
            }
            crate::ast::UnaryOperator::Dereference |
            crate::ast::UnaryOperator::Reference |
            crate::ast::UnaryOperator::ReferenceMut => {
                // For now, just copy the value
                if should_emit_asm() {
                    println!("  ; {} = {} (unary op)", temp, operand);
                    println!("  mov {}, {}", temp, operand);
                }
            }
            crate::ast::UnaryOperator::Neg => {
                if should_emit_asm() {
                    println!("  ; {} = -{}", temp, operand);
                    println!("  mov {}, {}", temp, operand);
                    println!("  neg {}", temp);
                }
            }
        }
        
        Ok(temp)
    }

    fn generate_function_call(&mut self, func_call: &Expression, arguments: &[CallArgument]) -> Result<String, CompilerError> {
        let func_name = match func_call {
            Expression::Identifier(name) => name.clone(),
            _ => return Err(CompilerError::codegen_error("simple", "Function call must be to an identifier")),
        };

        // Check if this is a builtin function
        if let Some(_builtin) = self.stdlib.get_builtin(&func_name) {
            // TODO: Implement builtin function generation
            if should_emit_asm() {
                println!("  ; builtin function call: {}", func_name);
            }
        } else {
            // Check if this is a module function
            if let Some((module_name, function_name)) = func_name.split_once("::") {
                // For now, just treat module functions as regular builtins
                if let Some(_builtin) = self.stdlib.get_builtin(function_name) {
                    // TODO: Implement module function generation
                    if should_emit_asm() {
                        println!("  ; module function call: {}::{}", module_name, function_name);
                    }
                }
            }
        }

        // Generate argument code
        for (i, arg) in arguments.iter().enumerate() {
            let arg_temp = self.generate_expression(&arg.value)?;
            if should_emit_asm() {
                if let Some(ref name) = arg.name {
                    println!("  ; arg {} ({}): {}", i, name, arg_temp);
                } else {
                    println!("  ; arg {}: {}", i, arg_temp);
                }
            }
        }

        let result_temp = self.next_temp();
        if should_emit_asm() {
            println!("  call {}", func_name);
            println!("  mov {}, rax", result_temp);
        }

        Ok(result_temp)
    }

    // If expression generator - used in generate_expression
    fn generate_if_expression(&mut self, if_expr: &crate::ast::IfExpression) -> Result<String, CompilerError> {
        let condition = self.generate_expression(&if_expr.condition)?;
        let else_label = self.next_label();
        let end_label = self.next_label();
        let temp = self.next_temp();
        
        if should_emit_asm() {
            println!("  ; if {} then ... else ...", condition);
            println!("  cmp {}, 0", condition);
            println!("  je {}", else_label);
        }
        
        // Generate then branch
        let then_value = self.generate_expression(&if_expr.then_branch)?;
        if should_emit_asm() {
            println!("  mov {}, {}", temp, then_value);
            println!("  jmp {}", end_label);
        }
        
        // Generate else branch
        if should_emit_asm() {
            println!("{}:", else_label);
        }
        let else_value = match &if_expr.else_branch {
            Some(else_expr) => self.generate_expression(else_expr)?,
            None => "0".to_string(), // Default value
        };
        if should_emit_asm() {
            println!("  mov {}, {}", temp, else_value);
        }
        
        if should_emit_asm() {
            println!("{}:", end_label);
        }
        
        Ok(temp)
    }

    // While expression generator - used in generate_expression
    fn generate_while_expression(&mut self, while_expr: &crate::ast::WhileExpression) -> Result<String, CompilerError> {
        let condition = self.generate_expression(&while_expr.condition)?;
        let loop_label = self.next_label();
        let end_label = self.next_label();
        let temp = self.next_temp();

        if should_emit_asm() {
            println!("{}:", loop_label);
            println!("  ; while {} do ...", condition);
            println!("  cmp {}, 0", condition);
            println!("  je {}", end_label);
        }

        // Generate body
        let body_value = self.generate_expression(&while_expr.body)?;
        if should_emit_asm() {
            println!("  mov {}, {}", temp, body_value);
            println!("  jmp {}", loop_label);
        }

        if should_emit_asm() {
            println!("{}:", end_label);
        }

        Ok(temp)
    }

    // Loop expression generator - used in generate_expression
    fn generate_loop_expression(&mut self, loop_expr: &crate::ast::LoopExpression) -> Result<String, CompilerError> {
        let loop_label = self.next_label();
        let end_label = self.next_label();
        let temp = self.next_temp();

        if should_emit_asm() {
            println!("{}:", loop_label);
            println!("  ; loop do ...");
        }
        
        // Generate body
        let body_value = self.generate_expression(&loop_expr.body)?;
        if should_emit_asm() {
            println!("  mov {}, {}", temp, body_value);
            println!("  jmp {}", loop_label);
        }

        if should_emit_asm() {
            println!("{}:", end_label);
        }

        Ok(temp)
    }

    fn generate_block_expression(&mut self, statements: &Vec<Statement>) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; block start");
        }
        
        let mut last_value = "0".to_string();
        for statement in statements {
            last_value = self.generate_statement(statement)?;
        }
        
        if should_emit_asm() {
            println!("  mov {}, {}", temp, last_value);
            println!("  ; block end");
        }
        Ok(temp)
    }

    // Move expression generator - currently unused but kept for future use
    #[allow(dead_code)]
    fn generate_move_expression(&mut self, expr: &Expression) -> Result<String, CompilerError> {
        // For now, just generate the expression (move semantics need more complex handling)
        self.generate_expression(expr)
    }

    fn generate_borrow_expression(&mut self, expr: &Expression) -> Result<String, CompilerError> {
        // For now, just generate the expression (borrow semantics need more complex handling)
        self.generate_expression(expr)
    }

    // Drop expression generator - currently unused but kept for future use
    #[allow(dead_code)]
    fn generate_drop_expression(&mut self, expr: &Expression) -> Result<String, CompilerError> {
        // For now, just generate the expression (drop semantics need more complex handling)
        self.generate_expression(expr)
    }

    fn generate_clone_expression(&mut self, expr: &Expression) -> Result<String, CompilerError> {
        // For now, just generate the expression (clone semantics need more complex handling)
        self.generate_expression(expr)
    }

    // Reference generator - currently unused but kept for future use
    #[allow(dead_code)]
    fn generate_reference(&mut self, name: &str, _borrow_type: &crate::ast::BorrowType) -> Result<String, CompilerError> {
        // For now, just return the variable (reference semantics need more complex handling)
        self.generate_identifier(name)
    }

    // Struct literal generator - used in generate_expression
    fn generate_struct_literal(&mut self, struct_literal: &crate::ast::StructLiteralExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; struct literal: {}", struct_literal.struct_name);
        }
        for (field_name, field_value_expr) in &struct_literal.fields {
            let field_value = self.generate_expression(field_value_expr)?;
            if should_emit_asm() {
                println!("  ; field {} = {}", field_name, field_value);
            }
        }
        Ok(temp) // Return the address of the allocated struct
    }

    // Member access generator - used in generate_expression
    fn generate_member_access(&mut self, member_access: &crate::ast::MemberAccessExpression) -> Result<String, CompilerError> {
        let object_value = self.generate_expression(&member_access.object)?;
        let temp = self.next_temp();
        
        if should_emit_asm() {
            println!("  ; member access: {}.{}", object_value, member_access.member);
            println!("  mov {}, {}", temp, object_value); // Placeholder: just return the object for now
        }
        
        Ok(temp)
    }

    // Enum variant access generator - currently unused but kept for future use
    #[allow(dead_code)]
    fn generate_enum_variant_access(&mut self, enum_name: &str, variant_name: &str) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        
        if should_emit_asm() {
            println!("  ; enum variant access: {}::{}", enum_name, variant_name);
            println!("  mov {}, 0", temp); // Placeholder: return 0 for now
        }
        
        Ok(temp)
    }

    // Match expression generator - used in generate_expression
    fn generate_match_expression(&mut self, match_expr: &crate::ast::MatchExpression) -> Result<String, CompilerError> {
        let value = self.generate_expression(&match_expr.expression)?;
        let temp = self.next_temp();
        let end_label = self.next_label();
        
        if should_emit_asm() {
            println!("  ; match expression on {}", value);
        }
        
        // Generate a series of if-else statements for each arm
        for (i, arm) in match_expr.arms.iter().enumerate() {
            let arm_label = self.next_label();
            let next_label = if i < match_expr.arms.len() - 1 { 
                self.next_label() 
            } else { 
                end_label.clone() 
            };
            
            // Generate pattern matching (simplified for now)
            match &arm.pattern {
                Pattern::Literal(literal) => {
                    let pattern_value = self.generate_literal(&literal)?;
                    if should_emit_asm() {
                        println!("  ; match arm {}: literal {}", i, pattern_value);
                        println!("  cmp {}, {}", value, pattern_value);
                        println!("  je {}", arm_label);
                    }
                    if i < match_expr.arms.len() - 1 {
                        if should_emit_asm() {
                            println!("  jmp {}", next_label);
                        }
                    }
                }
                Pattern::Identifier(name) => {
                    if should_emit_asm() {
                        println!("  ; match arm {}: identifier {}", i, name);
                    }
                    // For identifier patterns, just bind the value
                    if should_emit_asm() {
                        println!("  mov {}, {}", name, value);
                    }
                    if should_emit_asm() {
                        println!("  jmp {}", arm_label);
                    }
                }
                Pattern::Wildcard => {
                    if should_emit_asm() {
                        println!("  ; match arm {}: wildcard", i);
                    }
                    if should_emit_asm() {
                        println!("  jmp {}", arm_label);
                    }
                }
                _ => {
                    if should_emit_asm() {
                        println!("  ; match arm {}: complex pattern (not implemented)", i);
                    }
                    if should_emit_asm() {
                        println!("  jmp {}", arm_label);
                    }
                }
            }
            
            // Generate arm body
            if should_emit_asm() {
                println!("{}:", arm_label);
            }
            let arm_value = self.generate_expression(&arm.expression)?;
            if should_emit_asm() {
                println!("  mov {}, {}", temp, arm_value);
            }
            if should_emit_asm() {
                println!("  jmp {}", end_label);
            }
            
            if i < match_expr.arms.len() - 1 {
                if should_emit_asm() {
                    println!("{}:", next_label);
                }
            }
        }
        
        if should_emit_asm() {
            println!("{}:", end_label);
        }
        Ok(temp)
    }
    
    // Generic type call generator - currently unused but kept for future use
    #[allow(dead_code)]
    fn generate_generic_type_call(&mut self, _generic_call: &crate::ast::Expression) -> Result<String, CompilerError> {
        // TODO: Implement generic type call generation
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; generic type call (not implemented)");
        }
        Ok(temp)
    }

    fn generate_builtin_function_call(&mut self, name: &str, arguments: &Vec<Expression>) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        
        // Generate arguments
        for (i, arg) in arguments.iter().enumerate() {
            let arg_val = self.generate_expression(arg)?;
            match i {
                0 => if should_emit_asm() { println!("  mov rdi, {}", arg_val) },
                1 => if should_emit_asm() { println!("  mov rsi, {}", arg_val) },
                2 => if should_emit_asm() { println!("  mov rdx, {}", arg_val) },
                3 => if should_emit_asm() { println!("  mov rcx, {}", arg_val) },
                4 => if should_emit_asm() { println!("  mov r8, {}", arg_val) },
                5 => if should_emit_asm() { println!("  mov r9, {}", arg_val) },
                _ => {
                    // Push additional arguments on stack
                    if should_emit_asm() {
                        println!("  push {}", arg_val);
                    }
                }
            }
        }
        
        // Call the builtin function based on the name
        if should_emit_asm() {
            println!("  ; calling builtin function: {}", name);
        }
        match name {
            "print" => {
                if should_emit_asm() {
                    println!("  call std_print");
                }
            }
            "println" => {
                if should_emit_asm() {
                    println!("  call std_println");
                }
            }
            "read" => {
                if should_emit_asm() {
                    println!("  call std_read");
                }
            }
            "readln" => {
                if should_emit_asm() {
                    println!("  call std_readln");
                }
            }
            "len" => {
                if should_emit_asm() {
                    println!("  call std_len");
                }
            }
            "append" => {
                if should_emit_asm() {
                    println!("  call std_append");
                }
            }
            "remove" => {
                if should_emit_asm() {
                    println!("  call std_remove");
                }
            }
            "contains" => {
                if should_emit_asm() {
                    println!("  call std_contains");
                }
            }
            "index" => {
                if should_emit_asm() {
                    println!("  call std_index");
                }
            }
            "substring" => {
                if should_emit_asm() {
                    println!("  call std_substring");
                }
            }
            "to_string" => {
                if should_emit_asm() {
                    println!("  call std_to_string");
                }
            }
            "to_int" => {
                if should_emit_asm() {
                    println!("  call std_to_int");
                }
            }
            "to_float" => {
                if should_emit_asm() {
                    println!("  call std_to_float");
                }
            }
            "to_bool" => {
                if should_emit_asm() {
                    println!("  call std_to_bool");
                }
            }
            _ => {
                // Default to std_ prefix for unknown functions
                if should_emit_asm() {
                    println!("  call std_{}", name);
                }
            }
        }
        if should_emit_asm() {
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_box_expression(&mut self, box_expr: &BoxExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let value = self.generate_expression(&box_expr.value)?;
        
        if should_emit_asm() {
            println!("  ; Box allocation");
            println!("  mov rdi, {}", value);
            println!("  call box_new");
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_rc_expression(&mut self, rc_expr: &RcExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let value = self.generate_expression(&rc_expr.value)?;
        
        if should_emit_asm() {
            println!("  ; Rc allocation");
            println!("  mov rdi, {}", value);
            println!("  call rc_new");
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_arc_expression(&mut self, arc_expr: &ArcExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let value = self.generate_expression(&arc_expr.value)?;
        
        if should_emit_asm() {
            println!("  ; Arc allocation");
            println!("  mov rdi, {}", value);
            println!("  call arc_new");
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_cell_expression(&mut self, cell_expr: &CellExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let value = self.generate_expression(&cell_expr.value)?;
        
        if should_emit_asm() {
            println!("  ; Cell allocation");
            println!("  mov rdi, {}", value);
            println!("  call cell_new");
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_refcell_expression(&mut self, refcell_expr: &RefCellExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let value = self.generate_expression(&refcell_expr.value)?;
        
        if should_emit_asm() {
            println!("  ; RefCell allocation");
            println!("  mov rdi, {}", value);
            println!("  call refcell_new");
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_malloc_expression(&mut self, malloc_expr: &MallocExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let size = self.generate_expression(&malloc_expr.size)?;
        
        if should_emit_asm() {
            println!("  ; Memory allocation");
            println!("  mov rdi, {}", size);
            println!("  call malloc");
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_free_expression(&mut self, free_expr: &FreeExpression) -> Result<String, CompilerError> {
        let pointer = self.generate_expression(&free_expr.pointer)?;
        
        if should_emit_asm() {
            println!("  ; Memory deallocation");
            println!("  mov rdi, {}", pointer);
            println!("  call free");
        }
        
        Ok("void".to_string())
    }

    fn generate_realloc_expression(&mut self, realloc_expr: &ReallocExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let pointer = self.generate_expression(&realloc_expr.pointer)?;
        let new_size = self.generate_expression(&realloc_expr.new_size)?;
        
        if should_emit_asm() {
            println!("  ; Memory reallocation");
            println!("  mov rdi, {}", pointer);
            println!("  mov rsi, {}", new_size);
            println!("  call realloc");
            println!("  mov {}, rax", temp);
        }
        
        Ok(temp)
    }

    fn generate_return_expression(&mut self, return_expr: &Option<Box<Expression>>) -> Result<String, CompilerError> {
        if let Some(value) = return_expr {
            let value_temp = self.generate_expression(value)?;
            if should_emit_asm() {
                println!("  mov rax, {}", value_temp);
            }
        }
        if should_emit_asm() {
            println!("  ret");
        }
        Ok("rax".to_string())
    }

    fn generate_let_expression(&mut self, let_stmt: &LetStatement) -> Result<String, CompilerError> {
        let value_temp = self.generate_expression(&let_stmt.value)?;
        if should_emit_asm() {
            println!("  ; let {} = {}", let_stmt.name, value_temp);
        }
        Ok(value_temp)
    }

    fn generate_assignment_expression(&mut self, assignment_stmt: &AssignmentStatement) -> Result<String, CompilerError> {
        let value_temp = self.generate_expression(&assignment_stmt.value)?;
        if should_emit_asm() {
            println!("  mov {}, {}", assignment_stmt.target, value_temp);
        }
        Ok(value_temp)
    }
    
    // Missing generate methods
    fn generate_borrow_mut_expression(&mut self, borrow_mut_expr: &crate::ast::BorrowMutExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let expr = self.generate_expression(&borrow_mut_expr.expression)?;
        if should_emit_asm() {
            println!("  ; Borrow mutable: {}", expr);
        }
        Ok(temp)
    }
    
    fn generate_try_catch_expression(&mut self, try_catch_expr: &crate::ast::TryCatchExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; Try-catch expression");
        }
        let _try_result = self.generate_expression(&try_catch_expr.try_block)?;
        if should_emit_asm() {
            println!("  ; Catch block");
        }
        let _catch_result = self.generate_expression(&try_catch_expr.catch_block)?;
        Ok(temp)
    }
    
    // Spawn expression generator - used in generate_expression
    fn generate_spawn_expression(&mut self, spawn_expr: &crate::ast::SpawnExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; Spawn expression");
        }
        let _expr = self.generate_expression(&spawn_expr.expression)?;
        Ok(temp)
    }
    
    // Join expression generator - used in generate_expression
    fn generate_join_expression(&mut self, join_expr: &crate::ast::JoinExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; Join expression");
        }
        let _handle = self.generate_expression(&join_expr.handle)?;
        Ok(temp)
    }
    
    // Channel expression generator - used in generate_expression
    fn generate_channel_expression(&mut self, channel_expr: &crate::ast::ChannelExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; Channel expression");
        }
        if let Some(capacity) = &channel_expr.capacity {
            let _cap = self.generate_expression(capacity)?;
        }
        Ok(temp)
    }
    
    // Try expression generator - used in generate_expression
    fn generate_try_expression(&mut self, try_expr: &crate::ast::TryExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; Try expression");
        }
        let _expr = self.generate_expression(&try_expr.expression)?;
        Ok(temp)
    }
    
    // Pipeline expression generator - used in generate_expression
    fn generate_pipeline_expression(&mut self, pipeline_expr: &crate::ast::PipelineExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; Pipeline expression");
        }
        for stage in &pipeline_expr.stages {
            let _stage_result = self.generate_expression(stage)?;
        }
        Ok(temp)
    }
    
    // Lifetime expression generator - used in generate_expression
    fn generate_lifetime_expression(&mut self, lifetime_expr: &crate::ast::LifetimeExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        if should_emit_asm() {
            println!("  ; Lifetime expression: {}", lifetime_expr.lifetime.name);
        }
        let _expr = self.generate_expression(&lifetime_expr.expression)?;
        Ok(temp)
    }

    fn generate_array_access(&mut self, array_access: &crate::ast::ArrayAccessExpression) -> Result<String, CompilerError> {
        let temp = self.next_temp();
        let array = self.generate_expression(&array_access.array)?;
        let index = self.generate_expression(&array_access.index)?;
        
        if should_emit_asm() {
            println!("  ; Array access: {}[{}]", array, index);
            println!("  ; Calculate address: base + (index * element_size)");
            println!("  mov {}, {}", temp, array);
            println!("  add {}, {}", temp, index);
            println!("  mov {}, [{}]", temp, temp);
        }
        
        Ok(temp)
    }
} 