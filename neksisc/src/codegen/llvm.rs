#[cfg(feature = "llvm-backend")]
use inkwell::context::Context;
#[cfg(feature = "llvm-backend")]
use inkwell::module::Module;
#[cfg(feature = "llvm-backend")]
use inkwell::builder::Builder;
#[cfg(feature = "llvm-backend")]
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, IntValue};
#[cfg(feature = "llvm-backend")]
use inkwell::types::{BasicTypeEnum, IntType};
#[cfg(feature = "llvm-backend")]
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
#[cfg(feature = "llvm-backend")]
use inkwell::OptimizationLevel;
use std::collections::HashMap;

use crate::ast::{
    Program, Statement, Expression, FunctionStatement, LetStatement, ReturnStatement,
    Expression::*, Statement::*, Ownership, BorrowType, Lifetime
};
use crate::error::CompilerError;

#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    pub ownership: Ownership,
    pub lifetime: Option<String>,
    pub is_moved: bool,
    pub borrow_count: usize,
    pub mutable_borrow_count: usize,
}

#[cfg(feature = "llvm-backend")]
pub struct LLVMBackend {
    context: Context,
    module: Module,
    builder: Builder,
    variables: HashMap<String, PointerValue>,
    functions: HashMap<String, FunctionValue>,
    ownership_info: HashMap<String, OwnershipInfo>,
    lifetimes: HashMap<String, usize>, // lifetime name -> scope depth
    current_scope: Vec<String>,
    scope_counter: usize,
}

#[cfg(feature = "llvm-backend")]
impl LLVMBackend {
    pub fn new(module_name: &str) -> Self {
        let context = Context::create();
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            ownership_info: HashMap::new(),
            lifetimes: HashMap::new(),
            current_scope: Vec::new(),
            scope_counter: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<(), CompilerError> {
        // Generate all functions first
        for statement in &program.statements {
            if let Function(func_stmt) = statement {
                self.declare_function(func_stmt)?;
            }
        }

        // Generate function bodies
        for statement in &program.statements {
            if let Function(func_stmt) = statement {
                self.generate_function(func_stmt)?;
            }
        }

        Ok(())
    }

    fn declare_function(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        let fn_type = self.get_function_type(func_stmt)?;
        let function = self.module.add_function(&func_stmt.name, fn_type, None);
        self.functions.insert(func_stmt.name.clone(), function);
        Ok(())
    }

    fn get_function_type(&self, func_stmt: &FunctionStatement) -> Result<inkwell::types::FunctionType, CompilerError> {
        let return_type = if let Some(return_type) = &func_stmt.return_type {
            self.convert_type(return_type)?
        } else {
            self.context.void_type().into()
        };

        let param_types: Vec<BasicTypeEnum> = func_stmt
            .parameters
            .iter()
            .map(|param| self.convert_type(&param.type_annotation))
            .collect::<Result<Vec<_>, CompilerError>>()?;

        Ok(return_type.fn_type(&param_types, false))
    }

    fn convert_type(&self, ast_type: &crate::ast::Type) -> Result<BasicTypeEnum, CompilerError> {
        match ast_type {
            crate::ast::Type::Int => Ok(self.context.i32_type().into()),
            crate::ast::Type::Float => Ok(self.context.f32_type().into()),
            crate::ast::Type::Bool => Ok(self.context.bool_type().into()),
            crate::ast::Type::String => Ok(self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into()),
            crate::ast::Type::Void => Ok(self.context.void_type().into()),
            crate::ast::Type::Reference(inner_type, borrow_type, _lifetime) => {
                // For references, we generate a pointer to the underlying type
                let inner_llvm_type = self.convert_type(inner_type)?;
                match borrow_type {
                    BorrowType::Borrowed => Ok(inner_llvm_type.ptr_type(inkwell::AddressSpace::default()).into()),
                    BorrowType::MutableBorrowed => Ok(inner_llvm_type.ptr_type(inkwell::AddressSpace::default()).into()),
                    BorrowType::Shared => Ok(inner_llvm_type.ptr_type(inkwell::AddressSpace::default()).into()),
                    BorrowType::Weak => Ok(inner_llvm_type.ptr_type(inkwell::AddressSpace::default()).into()),
                    BorrowType::Unique => Ok(inner_llvm_type.ptr_type(inkwell::AddressSpace::default()).into()),
                    BorrowType::Owned => Ok(inner_llvm_type.ptr_type(inkwell::AddressSpace::default()).into()),
                }
            }
            crate::ast::Type::Owned(inner_type) => self.convert_type(inner_type),
            crate::ast::Type::Shared(inner_type) => self.convert_type(inner_type),
            crate::ast::Type::Weak(inner_type) => self.convert_type(inner_type),
            crate::ast::Type::Unique(inner_type) => self.convert_type(inner_type),
            crate::ast::Type::Result(ok_type, err_type) => {
                // Result<T, E> is represented as a tagged union
                let ok_llvm_type = self.convert_type(ok_type)?;
                let err_llvm_type = self.convert_type(err_type)?;
                let tag_type = self.context.i8_type();
                
                // Create a struct with tag and union
                let union_type = self.context.struct_type(&[ok_llvm_type, err_llvm_type], false);
                let result_type = self.context.struct_type(&[tag_type.into(), union_type.into()], false);
                Ok(result_type.into())
            }
            crate::ast::Type::Option(inner_type) => {
                // Option<T> is represented as a tagged union
                let inner_llvm_type = self.convert_type(inner_type)?;
                let tag_type = self.context.i8_type();
                
                // Create a struct with tag and value
                let option_type = self.context.struct_type(&[tag_type.into(), inner_llvm_type], false);
                Ok(option_type.into())
            }
            crate::ast::Type::Array(inner_type, size) => {
                let inner_llvm_type = self.convert_type(inner_type)?;
                Ok(inner_llvm_type.array_type(*size as u32).into())
            }
            crate::ast::Type::Slice(inner_type) => {
                let inner_llvm_type = self.convert_type(inner_type)?;
                let ptr_type = inner_llvm_type.ptr_type(inkwell::AddressSpace::default());
                let len_type = self.context.i64_type();
                
                // Slice is represented as { ptr, len }
                let slice_type = self.context.struct_type(&[ptr_type.into(), len_type.into()], false);
                Ok(slice_type.into())
            }
            crate::ast::Type::Tuple(types) => {
                let llvm_types: Vec<BasicTypeEnum> = types
                    .iter()
                    .map(|t| self.convert_type(t))
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                
                let tuple_type = self.context.struct_type(&llvm_types, false);
                Ok(tuple_type.into())
            }
            crate::ast::Type::Union(types) => {
                let llvm_types: Vec<BasicTypeEnum> = types
                    .iter()
                    .map(|t| self.convert_type(t))
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                
                let union_type = self.context.struct_type(&llvm_types, false);
                Ok(union_type.into())
            }
            crate::ast::Type::Never => {
                // Never type is represented as an empty struct
                let never_type = self.context.struct_type(&[], false);
                Ok(never_type.into())
            }
            _ => Err(CompilerError::codegen_error(&format!("Unsupported type: {:?}", ast_type))),
        }
    }

    fn generate_function(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        let function = self.functions.get(&func_stmt.name)
            .ok_or_else(|| CompilerError::codegen_error(&format!("Function {} not found", func_stmt.name)))?;

        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // Set up function parameters
        for (i, param) in func_stmt.parameters.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32)
                .ok_or_else(|| CompilerError::codegen_error("Parameter not found"))?;
            
            // Allocate space for the parameter
            let param_ptr = self.builder.build_alloca(param_value.get_type(), &param.name);
            self.builder.build_store(param_ptr, param_value);
            
            // Store the parameter
            self.variables.insert(param.name.clone(), param_ptr);
            
            // Initialize ownership info
            self.ownership_info.insert(param.name.clone(), OwnershipInfo {
                ownership: param.ownership.clone(),
                lifetime: param.lifetime.as_ref().map(|lt| lt.name.clone()),
                is_moved: false,
                borrow_count: 0,
                mutable_borrow_count: 0,
            });
        }

        // Generate function body
        for statement in &func_stmt.body {
            self.generate_statement(statement)?;
        }

        // If no return statement, add void return
        if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
            self.builder.build_return(None);
        }

        Ok(())
    }

    fn generate_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Let(let_stmt) => self.generate_let_statement(let_stmt),
            Function(_) => Ok(()), // Functions are handled separately
            Return(return_stmt) => self.generate_return_statement(return_stmt),
            Expression(expr) => {
                self.generate_expression(expr)?;
                Ok(())
            }
            Move(move_stmt) => self.generate_move_statement(move_stmt),
            Drop(drop_stmt) => self.generate_drop_statement(drop_stmt),
            _ => Ok(()), // TODO: Implement other statement types
        }
    }

    fn generate_let_statement(&mut self, let_stmt: &LetStatement) -> Result<(), CompilerError> {
        let value = self.generate_expression(&let_stmt.value)?;
        
        // Allocate space for the variable
        let var_type = value.get_type();
        let var_ptr = self.builder.build_alloca(var_type, &let_stmt.name);
        self.builder.build_store(var_ptr, value);
        
        // Store the variable
        self.variables.insert(let_stmt.name.clone(), var_ptr);
        
        // Initialize ownership info
        self.ownership_info.insert(let_stmt.name.clone(), OwnershipInfo {
            ownership: let_stmt.ownership.clone(),
            lifetime: None, // Will be set based on type annotation
            is_moved: false,
            borrow_count: 0,
            mutable_borrow_count: 0,
        });

        Ok(())
    }

    fn generate_return_statement(&mut self, return_stmt: &ReturnStatement) -> Result<(), CompilerError> {
        if let Some(value) = &return_stmt.value {
            let return_value = self.generate_expression(value)?;
            self.builder.build_return(Some(&return_value));
        } else {
            self.builder.build_return(None);
        }
        Ok(())
    }

    fn generate_move_statement(&mut self, move_stmt: &MoveStatement) -> Result<(), CompilerError> {
        // Get the source variable
        let source_ptr = self.variables.get(&move_stmt.from)
            .ok_or_else(|| CompilerError::codegen_error(&format!("Variable {} not found", move_stmt.from)))?;
        
        // Load the value
        let source_value = self.builder.build_load(source_ptr.get_type().get_element_type(), source_ptr, "source");
        
        // Allocate space for the destination
        let dest_ptr = self.builder.build_alloca(source_value.get_type(), &move_stmt.to);
        self.builder.build_store(dest_ptr, source_value);
        
        // Store the destination variable
        self.variables.insert(move_stmt.to.clone(), dest_ptr);
        
        // Mark source as moved
        if let Some(ownership_info) = self.ownership_info.get_mut(&move_stmt.from) {
            ownership_info.is_moved = true;
        }
        
        // Initialize destination ownership info
        self.ownership_info.insert(move_stmt.to.clone(), OwnershipInfo {
            ownership: Ownership::Owned,
            lifetime: None,
            is_moved: false,
            borrow_count: 0,
            mutable_borrow_count: 0,
        });

        Ok(())
    }

    fn generate_drop_statement(&mut self, drop_stmt: &DropStatement) -> Result<(), CompilerError> {
        // Get the variable to drop
        let var_ptr = self.variables.get(&drop_stmt.variable)
            .ok_or_else(|| CompilerError::codegen_error(&format!("Variable {} not found", drop_stmt.variable)))?;
        
        // For now, we just mark it as moved
        // In a real implementation, we would call the destructor
        if let Some(ownership_info) = self.ownership_info.get_mut(&drop_stmt.variable) {
            ownership_info.is_moved = true;
        }

        Ok(())
    }

    fn generate_expression(&mut self, expression: &Expression) -> Result<BasicValueEnum, CompilerError> {
        match expression {
            Literal(literal) => self.generate_literal(literal),
            Identifier(name) => self.generate_identifier(name),
            BinaryOp(bin_op) => {
                let left = self.generate_expression(&bin_op.left)?;
                let right = self.generate_expression(&bin_op.right)?;
                self.generate_binary_operation(left, &bin_op.operator, right)
            }
            UnaryOp(unary_op) => {
                let operand = self.generate_expression(&unary_op.operand)?;
                self.generate_unary_operation(&unary_op.operator, operand)
            }
            FunctionCall(call) => self.generate_function_call(call),
            If(if_expr) => self.generate_if_expression(if_expr),
            While(while_expr) => self.generate_while_expression(while_expr),
            Loop(loop_expr) => self.generate_loop_expression(loop_expr),
            Block(statements) => self.generate_block_expression(statements),
            StructLiteral(struct_lit) => self.generate_struct_literal(struct_lit),
            MemberAccess(member_access) => self.generate_member_access(member_access),
            EnumVariantAccess { enum_name, variant_name } => {
                self.generate_enum_variant_access(enum_name, variant_name)
            }
            BuiltinFunction { name, arguments } => {
                self.generate_builtin_function(name, arguments)
            }
            // Memory Management
            Box(box_expr) => self.generate_box_expression(box_expr),
            Rc(rc_expr) => self.generate_rc_expression(rc_expr),
            Arc(arc_expr) => self.generate_arc_expression(arc_expr),
            Cell(cell_expr) => self.generate_cell_expression(cell_expr),
            RefCell(refcell_expr) => self.generate_refcell_expression(refcell_expr),
            Malloc(malloc_expr) => self.generate_malloc_expression(malloc_expr),
            Free(free_expr) => self.generate_free_expression(free_expr),
            Realloc(realloc_expr) => self.generate_realloc_expression(realloc_expr),
            // Ownership and Borrowing
            Borrow(borrow_expr) => self.generate_borrow_expression(borrow_expr),
            BorrowMut(borrow_mut_expr) => self.generate_borrow_mut_expression(borrow_mut_expr),
            Move(move_expr) => self.generate_move_expression(move_expr),
            Clone(clone_expr) => self.generate_clone_expression(clone_expr),
            Drop(drop_expr) => self.generate_drop_expression(drop_expr),
            // Lifetime Management
            Lifetime(lifetime_expr) => self.generate_lifetime_expression(lifetime_expr),
            // Pattern Matching
            Match(match_expr) => self.generate_match_expression(match_expr),
            // Concurrency
            Spawn(spawn_expr) => self.generate_spawn_expression(spawn_expr),
            Join(join_expr) => self.generate_join_expression(join_expr),
            Channel(channel_expr) => self.generate_channel_expression(channel_expr),
            // Error Handling
            Try(try_expr) => self.generate_try_expression(try_expr),
            TryCatch(try_catch_expr) => self.generate_try_catch_expression(try_catch_expr),
            // Pipeline
            Pipeline(pipeline_expr) => self.generate_pipeline_expression(pipeline_expr),
        }
    }

    fn generate_literal(&self, literal: &crate::ast::Literal) -> Result<BasicValueEnum, CompilerError> {
        match literal {
            crate::ast::Literal::Integer(value) => {
                Ok(self.context.i32_type().const_int(*value as u64, false).into())
            }
            crate::ast::Literal::Float(value) => {
                Ok(self.context.f32_type().const_float(*value).into())
            }
            crate::ast::Literal::Boolean(value) => {
                Ok(self.context.bool_type().const_int(*value as u64, false).into())
            }
            crate::ast::Literal::String(value) => {
                let string_value = self.builder.build_global_string_ptr(value, "string");
                Ok(string_value.as_pointer_value().into())
            }
            crate::ast::Literal::Character(value) => {
                Ok(self.context.i32_type().const_int(*value as u64, false).into())
            }
            _ => Err(CompilerError::codegen_error(&format!("Unsupported literal: {:?}", literal))),
        }
    }

    fn generate_identifier(&mut self, name: &str) -> Result<BasicValueEnum, CompilerError> {
        let var_ptr = self.variables.get(name)
            .ok_or_else(|| CompilerError::codegen_error(&format!("Variable {} not found", name)))?;
        
        let value = self.builder.build_load(var_ptr.get_type().get_element_type(), var_ptr, name);
        Ok(value)
    }

    fn generate_binary_operation(
        &self,
        left: BasicValueEnum,
        operator: &crate::ast::BinaryOperator,
        right: BasicValueEnum,
    ) -> Result<BasicValueEnum, CompilerError> {
        match operator {
            crate::ast::BinaryOperator::Add => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_add(l, r, "add").into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_add(l, r, "add").into())
                    }
                    _ => Err(CompilerError::codegen_error("Invalid operands for addition")),
                }
            }
            crate::ast::BinaryOperator::Subtract => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_sub(l, r, "sub").into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_sub(l, r, "sub").into())
                    }
                    _ => Err(CompilerError::codegen_error("Invalid operands for subtraction")),
                }
            }
            crate::ast::BinaryOperator::Multiply => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_mul(l, r, "mul").into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_mul(l, r, "mul").into())
                    }
                    _ => Err(CompilerError::codegen_error("Invalid operands for multiplication")),
                }
            }
            crate::ast::BinaryOperator::Divide => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_signed_div(l, r, "div").into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_div(l, r, "div").into())
                    }
                    _ => Err(CompilerError::codegen_error("Invalid operands for division")),
                }
            }
            crate::ast::BinaryOperator::Equal => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eq").into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eq").into())
                    }
                    _ => Err(CompilerError::codegen_error("Invalid operands for equality")),
                }
            }
            _ => Err(CompilerError::codegen_error(&format!("Unsupported binary operator: {:?}", operator))),
        }
    }

    fn generate_unary_operation(
        &self,
        operator: &crate::ast::UnaryOperator,
        operand: BasicValueEnum,
    ) -> Result<BasicValueEnum, CompilerError> {
        match operator {
            crate::ast::UnaryOperator::Negate => {
                match operand {
                    BasicValueEnum::IntValue(value) => {
                        Ok(self.builder.build_int_neg(value, "neg").into())
                    }
                    BasicValueEnum::FloatValue(value) => {
                        Ok(self.builder.build_float_neg(value, "neg").into())
                    }
                    _ => Err(CompilerError::codegen_error("Invalid operand for negation")),
                }
            }
            crate::ast::UnaryOperator::Not => {
                match operand {
                    BasicValueEnum::IntValue(value) => {
                        Ok(self.builder.build_not(value, "not").into())
                    }
                    _ => Err(CompilerError::codegen_error("Invalid operand for logical not")),
                }
            }
            crate::ast::UnaryOperator::Copy => {
                // For copy, we just return the operand as-is
                Ok(operand)
            }
            crate::ast::UnaryOperator::Borrow => {
                // For borrow, we return a pointer to the operand
                let ptr = self.builder.build_alloca(operand.get_type(), "borrow");
                self.builder.build_store(ptr, operand);
                Ok(ptr.into())
            }
            crate::ast::UnaryOperator::BorrowMut => {
                // For mutable borrow, we return a pointer to the operand
                let ptr = self.builder.build_alloca(operand.get_type(), "borrow_mut");
                self.builder.build_store(ptr, operand);
                Ok(ptr.into())
            }
            crate::ast::UnaryOperator::Move => {
                // For move, we just return the operand as-is
                Ok(operand)
            }
            crate::ast::UnaryOperator::Drop => {
                // For drop, we just return void
                Ok(self.context.void_type().into())
            }
        }
    }

    fn generate_function_call(&mut self, call: &crate::ast::FunctionCall) -> Result<BasicValueEnum, CompilerError> {
        let function_name = if let Expression::Identifier(name) = &*call.function {
            name
        } else {
            return Err(CompilerError::codegen_error("Function call must be to an identifier"));
        };

        let function = self.functions.get(function_name)
            .ok_or_else(|| CompilerError::codegen_error(&format!("Function {} not found", function_name)))?;

        let mut args = Vec::new();
        for arg in &call.arguments {
            let arg_value = self.generate_expression(arg)?;
            args.push(arg_value);
        }

        let call_result = self.builder.build_call(function, &args, "call");
        Ok(call_result.try_as_basic_value().left().unwrap_or(self.context.void_type().into()))
    }

    fn generate_if_expression(&mut self, if_expr: &crate::ast::IfExpression) -> Result<BasicValueEnum, CompilerError> {
        let condition = self.generate_expression(&if_expr.condition)?;
        
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let merge_block = self.context.append_basic_block(function, "merge");

        // Build the conditional branch
        if let BasicValueEnum::IntValue(cond) = condition {
            self.builder.build_conditional_branch(cond, then_block, else_block);
        } else {
            return Err(CompilerError::codegen_error("Condition must be an integer"));
        }

        // Generate then branch
        self.builder.position_at_end(then_block);
        let then_value = self.generate_expression(&if_expr.then_branch)?;
        self.builder.build_unconditional_branch(merge_block);

        // Generate else branch
        self.builder.position_at_end(else_block);
        let else_value = if let Some(ref else_branch) = if_expr.else_branch {
            self.generate_expression(else_branch)?
        } else {
            self.context.void_type().into()
        };
        self.builder.build_unconditional_branch(merge_block);

        // Merge the results
        self.builder.position_at_end(merge_block);
        
        // For now, just return the then value
        // In a real implementation, we would need to handle different types
        Ok(then_value)
    }

    fn generate_while_expression(&mut self, while_expr: &crate::ast::WhileExpression) -> Result<BasicValueEnum, CompilerError> {
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let condition_block = self.context.append_basic_block(function, "while_condition");
        let body_block = self.context.append_basic_block(function, "while_body");
        let merge_block = self.context.append_basic_block(function, "while_merge");

        // Jump to condition block
        self.builder.build_unconditional_branch(condition_block);

        // Generate condition
        self.builder.position_at_end(condition_block);
        let condition = self.generate_expression(&while_expr.condition)?;
        
        if let BasicValueEnum::IntValue(cond) = condition {
            self.builder.build_conditional_branch(cond, body_block, merge_block);
        } else {
            return Err(CompilerError::codegen_error("While condition must be an integer"));
        }

        // Generate body
        self.builder.position_at_end(body_block);
        self.generate_expression(&while_expr.body)?;
        self.builder.build_unconditional_branch(condition_block);

        // Set up merge block
        self.builder.position_at_end(merge_block);
        
        Ok(self.context.void_type().into())
    }

    fn generate_loop_expression(&mut self, loop_expr: &crate::ast::LoopExpression) -> Result<BasicValueEnum, CompilerError> {
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let body_block = self.context.append_basic_block(function, "loop_body");
        let merge_block = self.context.append_basic_block(function, "loop_merge");

        // Jump to body block
        self.builder.build_unconditional_branch(body_block);

        // Generate body
        self.builder.position_at_end(body_block);
        self.generate_expression(&loop_expr.body)?;
        self.builder.build_unconditional_branch(body_block);

        // Set up merge block
        self.builder.position_at_end(merge_block);
        
        Ok(self.context.void_type().into())
    }

    fn generate_block_expression(&mut self, statements: &[Statement]) -> Result<BasicValueEnum, CompilerError> {
        let mut last_value = self.context.void_type().into();
        
        for statement in statements {
            match statement {
                Statement::Expression(expr) => {
                    last_value = self.generate_expression(expr)?;
                }
                _ => {
                    self.generate_statement(statement)?;
                }
            }
        }
        
        Ok(last_value)
    }

    fn generate_struct_literal(&mut self, struct_lit: &crate::ast::StructLiteral) -> Result<BasicValueEnum, CompilerError> {
        // For now, just return void
        // In a real implementation, we would create a struct with the given fields
        Ok(self.context.void_type().into())
    }

    fn generate_member_access(&mut self, member_access: &crate::ast::MemberAccess) -> Result<BasicValueEnum, CompilerError> {
        let object = self.generate_expression(&member_access.object)?;
        
        // For now, just return the object
        // In a real implementation, we would access the member
        Ok(object)
    }

    fn generate_enum_variant_access(&mut self, _enum_name: &str, _variant_name: &str) -> Result<BasicValueEnum, CompilerError> {
        // For now, just return void
        Ok(self.context.void_type().into())
    }

    fn generate_builtin_function(&mut self, name: &str, arguments: &[Expression]) -> Result<BasicValueEnum, CompilerError> {
        match name {
            "print" => {
                // For now, just return void
                // In a real implementation, we would call printf
                Ok(self.context.void_type().into())
            }
            _ => Err(CompilerError::codegen_error(&format!("Unknown builtin function: {}", name))),
        }
    }

    // Memory Management
    fn generate_box_expression(&mut self, box_expr: &crate::ast::BoxExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&box_expr.value)?;
        
        // For now, just return the value
        // In a real implementation, we would allocate on the heap
        Ok(value)
    }

    fn generate_rc_expression(&mut self, rc_expr: &crate::ast::RcExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&rc_expr.value)?;
        
        // For now, just return the value
        // In a real implementation, we would create an Rc
        Ok(value)
    }

    fn generate_arc_expression(&mut self, arc_expr: &crate::ast::ArcExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&arc_expr.value)?;
        
        // For now, just return the value
        // In a real implementation, we would create an Arc
        Ok(value)
    }

    fn generate_cell_expression(&mut self, cell_expr: &crate::ast::CellExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&cell_expr.value)?;
        
        // For now, just return the value
        // In a real implementation, we would create a Cell
        Ok(value)
    }

    fn generate_refcell_expression(&mut self, refcell_expr: &crate::ast::RefCellExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&refcell_expr.value)?;
        
        // For now, just return the value
        // In a real implementation, we would create a RefCell
        Ok(value)
    }

    fn generate_malloc_expression(&mut self, malloc_expr: &crate::ast::MallocExpression) -> Result<BasicValueEnum, CompilerError> {
        let size = self.generate_expression(&malloc_expr.size)?;
        
        // For now, just return a null pointer
        // In a real implementation, we would call malloc
        let ptr_type = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        Ok(ptr_type.const_null().into())
    }

    fn generate_free_expression(&mut self, free_expr: &crate::ast::FreeExpression) -> Result<BasicValueEnum, CompilerError> {
        let _pointer = self.generate_expression(&free_expr.pointer)?;
        
        // For now, just return void
        // In a real implementation, we would call free
        Ok(self.context.void_type().into())
    }

    fn generate_realloc_expression(&mut self, realloc_expr: &crate::ast::ReallocExpression) -> Result<BasicValueEnum, CompilerError> {
        let _pointer = self.generate_expression(&realloc_expr.pointer)?;
        let _new_size = self.generate_expression(&realloc_expr.new_size)?;
        
        // For now, just return a null pointer
        // In a real implementation, we would call realloc
        let ptr_type = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        Ok(ptr_type.const_null().into())
    }

    // Ownership and Borrowing
    fn generate_borrow_expression(&mut self, borrow_expr: &crate::ast::BorrowExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&borrow_expr.value)?;
        
        // For now, just return a pointer to the value
        let ptr = self.builder.build_alloca(value.get_type(), "borrow");
        self.builder.build_store(ptr, value);
        Ok(ptr.into())
    }

    fn generate_borrow_mut_expression(&mut self, borrow_mut_expr: &crate::ast::BorrowMutExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&borrow_mut_expr.value)?;
        
        // For now, just return a pointer to the value
        let ptr = self.builder.build_alloca(value.get_type(), "borrow_mut");
        self.builder.build_store(ptr, value);
        Ok(ptr.into())
    }

    fn generate_move_expression(&mut self, move_expr: &crate::ast::MoveExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&move_expr.value)?;
        
        // For now, just return the value
        // In a real implementation, we would handle ownership transfer
        Ok(value)
    }

    fn generate_clone_expression(&mut self, clone_expr: &crate::ast::CloneExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&clone_expr.value)?;
        
        // For now, just return the value
        // In a real implementation, we would clone the value
        Ok(value)
    }

    fn generate_drop_expression(&mut self, drop_expr: &crate::ast::DropExpression) -> Result<BasicValueEnum, CompilerError> {
        let _value = self.generate_expression(&drop_expr.value)?;
        
        // For now, just return void
        // In a real implementation, we would call the destructor
        Ok(self.context.void_type().into())
    }

    // Lifetime Management
    fn generate_lifetime_expression(&mut self, lifetime_expr: &crate::ast::LifetimeExpression) -> Result<BasicValueEnum, CompilerError> {
        let value = self.generate_expression(&lifetime_expr.expression)?;
        
        // For now, just return the value
        // In a real implementation, we would handle lifetime annotations
        Ok(value)
    }

    // Pattern Matching
    fn generate_match_expression(&mut self, match_expr: &crate::ast::MatchExpression) -> Result<BasicValueEnum, CompilerError> {
        let _value = self.generate_expression(&match_expr.value)?;
        
        // For now, just return void
        // In a real implementation, we would generate a switch statement
        Ok(self.context.void_type().into())
    }

    // Concurrency
    fn generate_spawn_expression(&mut self, spawn_expr: &crate::ast::SpawnExpression) -> Result<BasicValueEnum, CompilerError> {
        let _function = self.generate_expression(&spawn_expr.function)?;
        
        // For now, just return void
        // In a real implementation, we would spawn a thread
        Ok(self.context.void_type().into())
    }

    fn generate_join_expression(&mut self, join_expr: &crate::ast::JoinExpression) -> Result<BasicValueEnum, CompilerError> {
        let _thread_id = self.generate_expression(&join_expr.thread_id)?;
        
        // For now, just return void
        // In a real implementation, we would join the thread
        Ok(self.context.void_type().into())
    }

    fn generate_channel_expression(&mut self, channel_expr: &crate::ast::ChannelExpression) -> Result<BasicValueEnum, CompilerError> {
        let _sender = self.generate_expression(&channel_expr.sender)?;
        let _receiver = self.generate_expression(&channel_expr.receiver)?;
        
        // For now, just return void
        // In a real implementation, we would create a channel
        Ok(self.context.void_type().into())
    }

    // Error Handling
    fn generate_try_expression(&mut self, try_expr: &crate::ast::TryExpression) -> Result<BasicValueEnum, CompilerError> {
        let _expression = self.generate_expression(&try_expr.expression)?;
        
        // For now, just return void
        // In a real implementation, we would handle error propagation
        Ok(self.context.void_type().into())
    }

    fn generate_try_catch_expression(&mut self, try_catch_expr: &crate::ast::TryCatchExpression) -> Result<BasicValueEnum, CompilerError> {
        let _try_block = self.generate_expression(&try_catch_expr.try_block)?;
        let _catch_block = self.generate_expression(&try_catch_expr.catch_block)?;
        
        // For now, just return void
        // In a real implementation, we would handle exception handling
        Ok(self.context.void_type().into())
    }

    // Pipeline
    fn generate_pipeline_expression(&mut self, pipeline_expr: &crate::ast::PipelineExpression) -> Result<BasicValueEnum, CompilerError> {
        let mut value = self.generate_expression(&pipeline_expr.initial)?;
        
        for step in &pipeline_expr.steps {
            value = self.generate_expression(step)?;
        }
        
        Ok(value)
    }

    pub fn verify_module(&self) -> Result<(), CompilerError> {
        if self.module.verify().is_ok() {
            Ok(())
        } else {
            Err(CompilerError::codegen_error("Module verification failed"))
        }
    }

    pub fn print_module(&self) {
        self.module.print_to_stderr();
    }

    pub fn write_to_file(&self, filename: &str) -> Result<(), CompilerError> {
        if self.module.write_bitcode_to_path(std::path::Path::new(filename)).is_ok() {
            Ok(())
        } else {
            Err(CompilerError::codegen_error(&format!("Failed to write module to {}", filename)))
        }
    }
}

#[cfg(feature = "llvm-backend")]
pub fn generate_llvm_ir(program: &Program) -> Result<(), CompilerError> {
    let mut backend = LLVMBackend::new("neksis_module");
    backend.generate(program)?;
    backend.verify_module()?;
    Ok(())
}

#[cfg(not(feature = "llvm-backend"))]
pub fn generate_llvm_ir(_program: &Program) -> Result<(), CompilerError> {
    Err(CompilerError::codegen_error("LLVM backend not available - compile with 'llvm-backend' feature"))
} 