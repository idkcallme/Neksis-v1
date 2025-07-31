use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::ast::*;
use crate::error::CompilerError;
use crate::codegen::llvm::LLVMBackend;

#[derive(Debug, Clone)]
pub struct WASMCompiler {
    pub target_triple: String,
    pub optimization_level: OptimizationLevel,
    pub features: WASMFeatures,
    pub exports: HashMap<String, WASMExport>,
    pub imports: HashMap<String, WASMImport>,
}

#[derive(Debug, Clone)]
pub struct WASMFeatures {
    pub threads: bool,
    pub simd: bool,
    pub bulk_memory: bool,
    pub reference_types: bool,
    pub exception_handling: bool,
    pub tail_calls: bool,
    pub function_references: bool,
}

#[derive(Debug, Clone)]
pub struct WASMExport {
    pub name: String,
    pub wasm_type: WASMType,
    pub visibility: ExportVisibility,
}

#[derive(Debug, Clone)]
pub struct WASMImport {
    pub module: String,
    pub name: String,
    pub wasm_type: WASMType,
    pub kind: ImportKind,
}

#[derive(Debug, Clone)]
pub enum WASMType {
    I32,
    I64,
    F32,
    F64,
    V128, // SIMD
    FuncRef,
    ExternRef,
    Func(Vec<WASMType>, Vec<WASMType>), // params, results
    Memory(MemoryType),
    Table(TableType),
    Global(GlobalType),
    Array(Box<WASMType>, u32),
    Pointer(Box<WASMType>),
    Reference(Box<WASMType>),
    Function(Box<WASMType>),
    Struct(String),
    Enum(String),
    Trait(String),
    Generic(String, Vec<WASMType>),
    Unknown,
    GenericType(String, Vec<WASMType>),
    Owned(Box<WASMType>),
    Shared(Box<WASMType>),
    Weak(Box<WASMType>),
    Unique(Box<WASMType>),
    Result(Box<WASMType>, Box<WASMType>),
    Option(Box<WASMType>),
    Slice(Box<WASMType>),
    Tuple(Vec<WASMType>),
    Union(Vec<WASMType>),
}

#[derive(Debug, Clone)]
pub struct MemoryType {
    pub initial: u32,
    pub maximum: Option<u32>,
    pub shared: bool,
}

#[derive(Debug, Clone)]
pub struct TableType {
    pub element_type: WASMType,
    pub initial: u32,
    pub maximum: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GlobalType {
    pub value_type: WASMType,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub enum ExportVisibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    Function,
    Memory,
    Table,
    Global,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

impl WASMCompiler {
    pub fn new() -> Self {
        Self {
            target_triple: "wasm32-unknown-unknown".to_string(),
            optimization_level: OptimizationLevel::Basic,
            features: WASMFeatures::default(),
            exports: HashMap::new(),
            imports: HashMap::new(),
        }
    }

    pub fn with_features(mut self, features: WASMFeatures) -> Self {
        self.features = features;
        self
    }

    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    pub fn compile_to_wasm(&mut self, program: &Program) -> Result<Vec<u8>, CompilerError> {
        // Initialize LLVM with WASM target
        let context = inkwell::context::Context::create();
        let module = context.create_module("neksis_wasm");
        
        // Set target triple for WASM
        module.set_triple(&self.target_triple);
        
        // Create LLVM backend
        let backend = LLVMBackend::new(&context, &module)?;
        
        // Generate LLVM IR
        self.generate_wasm_ir(&backend, program)?;
        
        // Optimize the module
        self.optimize_module(&backend)?;
        
        // Generate WASM bytecode
        let wasm_bytes = self.generate_wasm_bytecode(&backend)?;
        
        Ok(wasm_bytes)
    }

    fn generate_wasm_ir(&self, backend: &LLVMBackend, program: &Program) -> Result<(), CompilerError> {
        // Generate WASM-specific LLVM IR
        for statement in &program.statements {
            match statement {
                Statement::Function(func) => {
                    self.generate_wasm_function(backend, func)?;
                }
                Statement::Struct(struct_def) => {
                    self.generate_wasm_struct(backend, struct_def)?;
                }
                Statement::Enum(enum_def) => {
                    self.generate_wasm_enum(backend, enum_def)?;
                }
                Statement::Module(module) => {
                    self.generate_wasm_module(backend, module)?;
                }
                _ => {
                    // Handle other statement types
                }
            }
        }
        Ok(())
    }

    fn generate_wasm_function(&self, backend: &LLVMBackend, func: &FunctionStatement) -> Result<(), CompilerError> {
        // Convert Neksis function to WASM function
        let wasm_signature = self.convert_function_signature(&func.signature)?;
        
        // Generate WASM function
        let function = backend.generate_wasm_function(&func.name, &wasm_signature)?;
        
        // Generate function body
        self.generate_wasm_function_body(backend, &function, &func.body)?;
        
        // Export function if needed
        if func.visibility == Visibility::Public {
            self.exports.insert(func.name.clone(), WASMExport {
                name: func.name.clone(),
                wasm_type: WASMType::Func(wasm_signature.0, wasm_signature.1),
                visibility: ExportVisibility::Public,
            });
        }
        
        Ok(())
    }

    fn convert_function_signature(&self, signature: &FunctionSignature) -> Result<(Vec<WASMType>, Vec<WASMType>), CompilerError> {
        let mut params = Vec::new();
        let mut results = Vec::new();
        
        // Convert parameters
        for param in &signature.parameters {
            let wasm_type = self.convert_neksis_type_to_wasm(&param.param_type)?;
            params.push(wasm_type);
        }
        
        // Convert return type
        if let Some(return_type) = &signature.return_type {
            let wasm_type = self.convert_neksis_type_to_wasm(return_type)?;
            results.push(wasm_type);
        }
        
        Ok((params, results))
    }

    fn convert_neksis_type_to_wasm(&self, neksis_type: &Type) -> Result<WASMType, CompilerError> {
        match neksis_type {
            Type::Int => Ok(WASMType::I32),
            Type::Float => Ok(WASMType::F64),
            Type::Bool => Ok(WASMType::I32),
            Type::String => Ok(WASMType::I32), // String pointer
            Type::Char => Ok(WASMType::I32),
            Type::Void => Ok(WASMType::Void),
            Type::Never => Ok(WASMType::Void),
            Type::Array(element_type, size) => {
                let element_wasm_type = self.convert_neksis_type_to_wasm(element_type)?;
                Ok(WASMType::Array(Box::new(element_wasm_type), *size))
            }
            Type::Pointer(pointee_type) => {
                let pointee_wasm_type = self.convert_neksis_type_to_wasm(pointee_type)?;
                Ok(WASMType::Pointer(Box::new(pointee_wasm_type)))
            }
            Type::Reference(referent_type, _, _) => {
                let referent_wasm_type = self.convert_neksis_type_to_wasm(referent_type)?;
                Ok(WASMType::Reference(Box::new(referent_wasm_type)))
            }
            Type::Function(param_types, return_type) => {
                let param_wasm_types: Result<Vec<WASMType>, CompilerError> = param_types.iter()
                    .map(|t| self.convert_neksis_type_to_wasm(t))
                    .collect();
                let return_wasm_type = self.convert_neksis_type_to_wasm(return_type)?;
                Ok(WASMType::Func(param_wasm_types?, vec![return_wasm_type]))
            }
            Type::Struct(name) => Ok(WASMType::Struct(name.clone())),
            Type::Enum(name) => Ok(WASMType::Enum(name.clone())),
            Type::Trait(name) => Ok(WASMType::Trait(name.clone())),
            Type::Generic(name, type_args) => {
                let type_arg_wasm_types: Result<Vec<WASMType>, CompilerError> = type_args.iter()
                    .map(|t| self.convert_neksis_type_to_wasm(t))
                    .collect();
                Ok(WASMType::Generic(name.clone(), type_arg_wasm_types?))
            }
            Type::Unknown => Ok(WASMType::I32), // Default to i32
            Type::GenericType(name, type_args) => {
                let type_arg_wasm_types: Result<Vec<WASMType>, CompilerError> = type_args.iter()
                    .map(|t| self.convert_neksis_type_to_wasm(t))
                    .collect();
                Ok(WASMType::Generic(name.clone(), type_arg_wasm_types?))
            }
            Type::Owned(inner_type) => {
                let inner_wasm_type = self.convert_neksis_type_to_wasm(inner_type)?;
                Ok(WASMType::Owned(Box::new(inner_wasm_type)))
            }
            Type::Shared(inner_type) => {
                let inner_wasm_type = self.convert_neksis_type_to_wasm(inner_type)?;
                Ok(WASMType::Shared(Box::new(inner_wasm_type)))
            }
            Type::Weak(inner_type) => {
                let inner_wasm_type = self.convert_neksis_type_to_wasm(inner_type)?;
                Ok(WASMType::Weak(Box::new(inner_wasm_type)))
            }
            Type::Unique(inner_type) => {
                let inner_wasm_type = self.convert_neksis_type_to_wasm(inner_type)?;
                Ok(WASMType::Unique(Box::new(inner_wasm_type)))
            }
            Type::Result(ok_type, err_type) => {
                let ok_wasm_type = self.convert_neksis_type_to_wasm(ok_type)?;
                let err_wasm_type = self.convert_neksis_type_to_wasm(err_type)?;
                Ok(WASMType::Result(Box::new(ok_wasm_type), Box::new(err_wasm_type)))
            }
            Type::Option(inner_type) => {
                let inner_wasm_type = self.convert_neksis_type_to_wasm(inner_type)?;
                Ok(WASMType::Option(Box::new(inner_wasm_type)))
            }
            Type::Slice(inner_type) => {
                let inner_wasm_type = self.convert_neksis_type_to_wasm(inner_type)?;
                Ok(WASMType::Slice(Box::new(inner_wasm_type)))
            }
            Type::Tuple(types) => {
                let type_wasm_types: Result<Vec<WASMType>, CompilerError> = types.iter()
                    .map(|t| self.convert_neksis_type_to_wasm(t))
                    .collect();
                Ok(WASMType::Tuple(type_wasm_types?))
            }
            Type::Union(types) => {
                let type_wasm_types: Result<Vec<WASMType>, CompilerError> = types.iter()
                    .map(|t| self.convert_neksis_type_to_wasm(t))
                    .collect();
                Ok(WASMType::Union(type_wasm_types?))
            }
        }
    }

    fn convert_function_to_wasm(&self, func: &FunctionStatement) -> Result<WASMFunction, CompilerError> {
        // Convert function parameters
        let mut wasm_params = Vec::new();
        for param in &func.parameters {
            let wasm_type = self.convert_neksis_type_to_wasm(&param.type_annotation)?;
            wasm_params.push(WASMParameter {
                name: param.name.clone(),
                param_type: wasm_type,
            });
        }

        // Convert return type
        let return_type = if let Some(return_type) = &func.return_type {
            self.convert_neksis_type_to_wasm(return_type)?
        } else {
            WASMType::Void
        };

        Ok(WASMFunction {
            name: func.name.clone(),
            parameters: wasm_params,
            return_type,
            body: func.body.clone(),
            is_exported: true, // Simplified
        })
    }

    fn convert_expression_to_wasm(&self, expression: &Expression) -> Result<WASMExpression, CompilerError> {
        match expression {
            Expression::Literal(literal) => {
                let wasm_literal = self.convert_literal_to_wasm(literal)?;
                Ok(WASMExpression::Literal(wasm_literal))
            }
            Expression::Identifier(name) => {
                Ok(WASMExpression::Variable(name.clone()))
            }
            Expression::BinaryOp(bin_op) => {
                let left = self.convert_expression_to_wasm(&bin_op.left)?;
                let right = self.convert_expression_to_wasm(&bin_op.right)?;
                let operator = self.convert_binary_operator_to_wasm(&bin_op.operator)?;
                Ok(WASMExpression::BinaryOperation { left: Box::new(left), operator, right: Box::new(right) })
            }
            Expression::UnaryOp(unary_op) => {
                let operand = self.convert_expression_to_wasm(&unary_op.operand)?;
                let operator = self.convert_unary_operator_to_wasm(&unary_op.operator)?;
                Ok(WASMExpression::UnaryOperation { operator, operand: Box::new(operand) })
            }
            Expression::FunctionCall(func, args) => {
                let func_expr = self.convert_expression_to_wasm(func)?;
                let mut wasm_args = Vec::new();
                for arg in args {
                    let wasm_arg = self.convert_expression_to_wasm(arg)?;
                    wasm_args.push(wasm_arg);
                }
                Ok(WASMExpression::FunctionCall { name: Box::new(func_expr), arguments: wasm_args })
            }
            Expression::If(if_expr) => {
                let condition = self.convert_expression_to_wasm(&if_expr.condition)?;
                let then_branch = self.convert_expression_to_wasm(&if_expr.then_branch)?;
                let else_branch = if let Some(else_branch) = &if_expr.else_branch {
                    self.convert_expression_to_wasm(else_branch)?
                } else {
                    WASMExpression::Literal(WASMLiteral::Void)
                };
                Ok(WASMExpression::If { condition: Box::new(condition), then_branch: Box::new(then_branch), else_branch: Box::new(else_branch) })
            }
            Expression::While(while_expr) => {
                let condition = self.convert_expression_to_wasm(&while_expr.condition)?;
                let body = self.convert_expression_to_wasm(&while_expr.body)?;
                Ok(WASMExpression::While { condition: Box::new(condition), body: Box::new(body) })
            }
            Expression::Block(statements) => {
                let mut wasm_statements = Vec::new();
                for stmt in statements {
                    let wasm_stmt = self.convert_statement_to_wasm(stmt)?;
                    wasm_statements.push(wasm_stmt);
                }
                Ok(WASMExpression::Block { statements: wasm_statements })
            }
            Expression::Return(expr) => {
                let return_value = if let Some(expr) = expr {
                    self.convert_expression_to_wasm(expr)?
                } else {
                    WASMExpression::Literal(WASMLiteral::Void)
                };
                Ok(WASMExpression::Return { value: Box::new(return_value) })
            }
            Expression::Let(let_stmt) => {
                let value = self.convert_expression_to_wasm(&let_stmt.value)?;
                Ok(WASMExpression::Let { name: let_stmt.name.clone(), value: Box::new(value) })
            }
            Expression::Assignment(assign_stmt) => {
                let value = self.convert_expression_to_wasm(&assign_stmt.value)?;
                Ok(WASMExpression::Assignment { target: assign_stmt.target.clone(), value: Box::new(value) })
            }
            Expression::Malloc(malloc_expr) => {
                let size = self.convert_expression_to_wasm(&malloc_expr.size)?;
                Ok(WASMExpression::Malloc { size: Box::new(size) })
            }
            Expression::Free(free_expr) => {
                let pointer = self.convert_expression_to_wasm(&free_expr.pointer)?;
                Ok(WASMExpression::Free { pointer: Box::new(pointer) })
            }
            Expression::Realloc(realloc_expr) => {
                let pointer = self.convert_expression_to_wasm(&realloc_expr.pointer)?;
                let new_size = self.convert_expression_to_wasm(&realloc_expr.new_size)?;
                Ok(WASMExpression::Realloc { pointer: Box::new(pointer), new_size: Box::new(new_size) })
            }
            Expression::TryCatch(try_catch_expr) => {
                let try_block = self.convert_expression_to_wasm(&try_catch_expr.try_block)?;
                let catch_block = self.convert_expression_to_wasm(&try_catch_expr.catch_block)?;
                Ok(WASMExpression::TryCatch { try_block: Box::new(try_block), catch_block: Box::new(catch_block) })
            }
            Expression::Move(move_stmt) => {
                Ok(WASMExpression::Move { from: move_stmt.from.clone(), to: move_stmt.to.clone() })
            }
            Expression::Drop(drop_stmt) => {
                Ok(WASMExpression::Drop { variable: drop_stmt.variable.clone() })
            }
            Expression::Borrow(borrow_expr) => {
                let expression = self.convert_expression_to_wasm(&borrow_expr.expression)?;
                Ok(WASMExpression::Borrow { expression: Box::new(expression) })
            }
            Expression::BorrowMut(borrow_mut_expr) => {
                let expression = self.convert_expression_to_wasm(&borrow_mut_expr.expression)?;
                Ok(WASMExpression::BorrowMut { expression: Box::new(expression) })
            }
            Expression::Clone(clone_expr) => {
                let expression = self.convert_expression_to_wasm(&clone_expr.expression)?;
                Ok(WASMExpression::Clone { expression: Box::new(expression) })
            }
            Expression::BinaryOperation { left, operator, right } => {
                let left_expr = self.convert_expression_to_wasm(left)?;
                let right_expr = self.convert_expression_to_wasm(right)?;
                let wasm_operator = self.convert_binary_operator_to_wasm(operator)?;
                Ok(WASMExpression::BinaryOperation { left: Box::new(left_expr), operator: wasm_operator, right: Box::new(right_expr) })
            }
            Expression::Loop(loop_expr) => {
                let body = self.convert_expression_to_wasm(&loop_expr.body)?;
                Ok(WASMExpression::Loop { body: Box::new(body) })
            }
            Expression::StructLiteral(struct_literal_expr) => {
                let mut fields = Vec::new();
                for (name, expr) in &struct_literal_expr.fields {
                    let wasm_expr = self.convert_expression_to_wasm(expr)?;
                    fields.push((name.clone(), wasm_expr));
                }
                Ok(WASMExpression::StructLiteral { struct_name: struct_literal_expr.struct_name.clone(), fields })
            }
            Expression::MemberAccess(member_access_expr) => {
                let object = self.convert_expression_to_wasm(&member_access_expr.object)?;
                Ok(WASMExpression::MemberAccess { object: Box::new(object), member: member_access_expr.member.clone() })
            }
            Expression::EnumVariantAccess { enum_name, variant_name } => {
                Ok(WASMExpression::EnumVariantAccess { enum_name: enum_name.clone(), variant_name: variant_name.clone() })
            }
            Expression::BuiltinFunction { name, arguments } => {
                let mut wasm_args = Vec::new();
                for arg in arguments {
                    let wasm_arg = self.convert_expression_to_wasm(arg)?;
                    wasm_args.push(wasm_arg);
                }
                Ok(WASMExpression::BuiltinFunction { name: name.clone(), arguments: wasm_args })
            }
            Expression::Box(box_expr) => {
                let value = self.convert_expression_to_wasm(&box_expr.value)?;
                Ok(WASMExpression::Box { value: Box::new(value) })
            }
            Expression::Rc(rc_expr) => {
                let value = self.convert_expression_to_wasm(&rc_expr.value)?;
                Ok(WASMExpression::Rc { value: Box::new(value) })
            }
            Expression::Arc(arc_expr) => {
                let value = self.convert_expression_to_wasm(&arc_expr.value)?;
                Ok(WASMExpression::Arc { value: Box::new(value) })
            }
            Expression::Cell(cell_expr) => {
                let value = self.convert_expression_to_wasm(&cell_expr.value)?;
                Ok(WASMExpression::Cell { value: Box::new(value) })
            }
            Expression::RefCell(ref_cell_expr) => {
                let value = self.convert_expression_to_wasm(&ref_cell_expr.value)?;
                Ok(WASMExpression::RefCell { value: Box::new(value) })
            }
            Expression::Lifetime(lifetime_expr) => {
                let expression = self.convert_expression_to_wasm(&lifetime_expr.expression)?;
                Ok(WASMExpression::Lifetime { lifetime: lifetime_expr.lifetime.name.clone(), expression: Box::new(expression) })
            }
            Expression::Match(match_expr) => {
                let expression = self.convert_expression_to_wasm(&match_expr.expression)?;
                let mut arms = Vec::new();
                for arm in &match_expr.arms {
                    let pattern = self.convert_pattern_to_wasm(&arm.pattern)?;
                    let expression = self.convert_expression_to_wasm(&arm.expression)?;
                    let body = self.convert_expression_to_wasm(&arm.body)?;
                    let guard = if let Some(guard) = &arm.guard {
                        Some(Box::new(self.convert_expression_to_wasm(guard)?))
                    } else {
                        None
                    };
                    arms.push(WASMMatchArm { pattern, expression, body: Box::new(body), guard });
                }
                Ok(WASMExpression::Match { value: Box::new(expression), arms })
            }
            Expression::Spawn(spawn_expr) => {
                let expression = self.convert_expression_to_wasm(&spawn_expr.expression)?;
                Ok(WASMExpression::Spawn { expression: Box::new(expression) })
            }
            Expression::Join(join_expr) => {
                let handle = self.convert_expression_to_wasm(&join_expr.handle)?;
                Ok(WASMExpression::Join { handle: Box::new(handle) })
            }
            Expression::Channel(channel_expr) => {
                let capacity = if let Some(capacity) = &channel_expr.capacity {
                    Some(Box::new(self.convert_expression_to_wasm(capacity)?))
                } else {
                    None
                };
                Ok(WASMExpression::Channel { channel_type: channel_expr.channel_type.clone(), capacity })
            }
            Expression::Try(try_expr) => {
                let expression = self.convert_expression_to_wasm(&try_expr.expression)?;
                Ok(WASMExpression::Try { expression: Box::new(expression) })
            }
            Expression::Pipeline(pipeline_expr) => {
                let mut stages = Vec::new();
                for stage in &pipeline_expr.stages {
                    let wasm_stage = self.convert_expression_to_wasm(stage)?;
                    stages.push(wasm_stage);
                }
                Ok(WASMExpression::Pipeline { stages })
            }
        }
    }

    fn generate_wasm_function_body(&self, backend: &LLVMBackend, function: &inkwell::values::FunctionValue, body: &Expression) -> Result<(), CompilerError> {
        // Generate WASM-specific function body
        let builder = backend.context.create_builder();
        let entry_block = backend.context.append_basic_block(*function, "entry");
        builder.position_at_end(entry_block);
        
        // Generate expression as WASM instructions
        self.generate_wasm_expression(backend, &builder, body)?;
        
        Ok(())
    }

    fn generate_wasm_expression(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, expr: &Expression) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        match expr {
            Expression::Literal(literal) => {
                self.generate_wasm_literal(backend, builder, literal)
            }
            Expression::BinaryOperation { left, operator, right } => {
                self.generate_wasm_binary_op(backend, builder, left, operator, right)
            }
            Expression::UnaryOperation { operator, operand } => {
                self.generate_wasm_unary_op(backend, builder, operator, operand)
            }
            Expression::FunctionCall { name, arguments } => {
                self.generate_wasm_function_call(backend, builder, name, arguments)
            }
            Expression::Identifier(name) => {
                self.generate_wasm_identifier(backend, builder, name)
            }
            Expression::Block(expressions) => {
                self.generate_wasm_block(backend, builder, expressions)
            }
            Expression::If { condition, then_branch, else_branch } => {
                self.generate_wasm_if(backend, builder, condition, then_branch, else_branch)
            }
            Expression::While { condition, body } => {
                self.generate_wasm_while(backend, builder, condition, body)
            }
            Expression::For { variable, iterator, body } => {
                self.generate_wasm_for(backend, builder, variable, iterator, body)
            }
            Expression::Match { value, arms } => {
                self.generate_wasm_match(backend, builder, value, arms)
            }
            _ => {
                Err(CompilerError::codegen_error("wasm", &format!("Unsupported expression for WASM: {:?}", expr)))
            }
        }
    }

    fn generate_wasm_literal(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, literal: &Literal) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        match literal {
            Literal::Int(value) => {
                let int_value = backend.context.i32_type().const_int(*value as u64, false);
                Ok(int_value.into())
            }
            Literal::Float(value) => {
                let float_value = backend.context.f32_type().const_float(*value as f64);
                Ok(float_value.into())
            }
            Literal::Bool(value) => {
                let bool_value = backend.context.bool_type().const_int(*value as u64, false);
                Ok(bool_value.into())
            }
            Literal::String(value) => {
                // Create global string constant
                let string_value = backend.context.const_string(value.as_bytes(), false);
                let global = backend.module.add_global(string_value.get_type(), None, "string_literal");
                global.set_initializer(&string_value);
                let ptr = builder.build_pointer_cast(global.as_pointer_value(), backend.context.i32_type().ptr_type(inkwell::AddressSpace::default()), "string_ptr");
                Ok(ptr.into())
            }
            _ => {
                Err(CompilerError::codegen_error("wasm", &format!("Unsupported literal for WASM: {:?}", literal)))
            }
        }
    }

    fn generate_wasm_binary_op(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, left: &Expression, operator: &BinaryOperator, right: &Expression) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        let left_val = self.generate_wasm_expression(backend, builder, left)?;
        let right_val = self.generate_wasm_expression(backend, builder, right)?;
        
        match operator {
            BinaryOperator::Add => {
                if left_val.is_int_value() {
                    let result = builder.build_int_add(left_val.into_int_value(), right_val.into_int_value(), "add");
                    Ok(result.into())
                } else if left_val.is_float_value() {
                    let result = builder.build_float_add(left_val.into_float_value(), right_val.into_float_value(), "fadd");
                    Ok(result.into())
                } else {
                    Err(CompilerError::codegen_error("wasm", "Invalid types for addition"))
                }
            }
            BinaryOperator::Subtract => {
                if left_val.is_int_value() {
                    let result = builder.build_int_sub(left_val.into_int_value(), right_val.into_int_value(), "sub");
                    Ok(result.into())
                } else if left_val.is_float_value() {
                    let result = builder.build_float_sub(left_val.into_float_value(), right_val.into_float_value(), "fsub");
                    Ok(result.into())
                } else {
                    Err(CompilerError::codegen_error("wasm", "Invalid types for subtraction"))
                }
            }
            BinaryOperator::Multiply => {
                if left_val.is_int_value() {
                    let result = builder.build_int_mul(left_val.into_int_value(), right_val.into_int_value(), "mul");
                    Ok(result.into())
                } else if left_val.is_float_value() {
                    let result = builder.build_float_mul(left_val.into_float_value(), right_val.into_float_value(), "fmul");
                    Ok(result.into())
                } else {
                    Err(CompilerError::codegen_error("wasm", "Invalid types for multiplication"))
                }
            }
            BinaryOperator::Divide => {
                if left_val.is_int_value() {
                    let result = builder.build_int_signed_div(left_val.into_int_value(), right_val.into_int_value(), "div");
                    Ok(result.into())
                } else if left_val.is_float_value() {
                    let result = builder.build_float_div(left_val.into_float_value(), right_val.into_float_value(), "fdiv");
                    Ok(result.into())
                } else {
                    Err(CompilerError::codegen_error("wasm", "Invalid types for division"))
                }
            }
            _ => {
                Err(CompilerError::codegen_error("wasm", &format!("Unsupported binary operator for WASM: {:?}", operator)))
            }
        }
    }

    fn generate_wasm_unary_op(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, operator: &UnaryOperator, operand: &Expression) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        let operand_val = self.generate_wasm_expression(backend, builder, operand)?;
        
        match operator {
            UnaryOperator::Negate => {
                if operand_val.is_int_value() {
                    let result = builder.build_int_neg(operand_val.into_int_value(), "neg");
                    Ok(result.into())
                } else if operand_val.is_float_value() {
                    let result = builder.build_float_neg(operand_val.into_float_value(), "fneg");
                    Ok(result.into())
                } else {
                    Err(CompilerError::codegen_error("wasm", "Invalid type for negation"))
                }
            }
            UnaryOperator::Not => {
                if operand_val.is_int_value() {
                    let result = builder.build_not(operand_val.into_int_value(), "not");
                    Ok(result.into())
                } else {
                    Err(CompilerError::codegen_error("wasm", "Invalid type for logical not"))
                }
            }
            _ => {
                Err(CompilerError::codegen_error("wasm", &format!("Unsupported unary operator for WASM: {:?}", operator)))
            }
        }
    }

    fn generate_wasm_function_call(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, name: &str, arguments: &[Expression]) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        // Look up function
        let function = backend.module.get_function(name)
            .ok_or_else(|| CompilerError::codegen_error("wasm", &format!("Function '{}' not found", name)))?;
        
        // Generate arguments
        let mut args = Vec::new();
        for arg in arguments {
            let arg_val = self.generate_wasm_expression(backend, builder, arg)?;
            args.push(arg_val);
        }
        
        // Call function
        let call_result = builder.build_call(function, &args, "call");
        Ok(call_result.try_as_basic_value().left().unwrap_or_else(|| backend.context.i32_type().const_zero().into()))
    }

    fn generate_wasm_identifier(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, name: &str) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        // Look up variable in current scope
        // This is a simplified implementation
        let zero = backend.context.i32_type().const_zero();
        Ok(zero.into())
    }

    fn generate_wasm_block(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, expressions: &[Expression]) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        let mut last_result = None;
        
        for expr in expressions {
            last_result = Some(self.generate_wasm_expression(backend, builder, expr)?);
        }
        
        Ok(last_result.unwrap_or_else(|| backend.context.i32_type().const_zero().into()))
    }

    fn generate_wasm_if(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, condition: &Expression, then_branch: &Expression, else_branch: &Option<Box<Expression>>) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        let condition_val = self.generate_wasm_expression(backend, builder, condition)?;
        
        let current_function = builder.get_insert_block().unwrap().get_parent().unwrap();
        let then_block = backend.context.append_basic_block(*current_function, "then");
        let else_block = backend.context.append_basic_block(*current_function, "else");
        let merge_block = backend.context.append_basic_block(*current_function, "merge");
        
        // Branch based on condition
        builder.build_conditional_branch(condition_val.into_int_value(), then_block, else_block);
        
        // Generate then branch
        builder.position_at_end(then_block);
        let then_result = self.generate_wasm_expression(backend, builder, then_branch)?;
        builder.build_unconditional_branch(merge_block);
        
        // Generate else branch
        builder.position_at_end(else_block);
        let else_result = if let Some(else_expr) = else_branch {
            self.generate_wasm_expression(backend, builder, else_expr)?
        } else {
            backend.context.i32_type().const_zero().into()
        };
        builder.build_unconditional_branch(merge_block);
        
        // Merge
        builder.position_at_end(merge_block);
        Ok(then_result) // Simplified - should use phi node
    }

    fn generate_wasm_while(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, condition: &Expression, body: &Expression) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        let current_function = builder.get_insert_block().unwrap().get_parent().unwrap();
        let loop_header = backend.context.append_basic_block(*current_function, "loop_header");
        let loop_body = backend.context.append_basic_block(*current_function, "loop_body");
        let loop_exit = backend.context.append_basic_block(*current_function, "loop_exit");
        
        // Jump to loop header
        builder.build_unconditional_branch(loop_header);
        
        // Loop header - check condition
        builder.position_at_end(loop_header);
        let condition_val = self.generate_wasm_expression(backend, builder, condition)?;
        builder.build_conditional_branch(condition_val.into_int_value(), loop_body, loop_exit);
        
        // Loop body
        builder.position_at_end(loop_body);
        self.generate_wasm_expression(backend, builder, body)?;
        builder.build_unconditional_branch(loop_header);
        
        // Loop exit
        builder.position_at_end(loop_exit);
        Ok(backend.context.i32_type().const_zero().into())
    }

    fn generate_wasm_for(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, variable: &str, iterator: &Expression, body: &Expression) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        // Simplified for loop implementation
        // In a real implementation, this would handle different iterator types
        self.generate_wasm_while(backend, builder, iterator, body)
    }

    fn generate_wasm_match(&self, backend: &LLVMBackend, builder: &inkwell::builder::Builder, value: &Expression, arms: &[MatchArm]) -> Result<inkwell::values::BasicValueEnum, CompilerError> {
        // Simplified match implementation
        // In a real implementation, this would generate a switch statement
        if let Some(first_arm) = arms.first() {
            self.generate_wasm_expression(backend, builder, &first_arm.body)
        } else {
            Ok(backend.context.i32_type().const_zero().into())
        }
    }

    fn generate_wasm_struct(&self, backend: &LLVMBackend, struct_def: &StructStatement) -> Result<(), CompilerError> {
        // Generate WASM struct type
        // This would create a struct type in LLVM IR
        Ok(())
    }

    fn generate_wasm_enum(&self, backend: &LLVMBackend, enum_def: &EnumStatement) -> Result<(), CompilerError> {
        // Generate WASM enum type
        // This would create an enum type in LLVM IR
        Ok(())
    }

    fn generate_wasm_module(&self, backend: &LLVMBackend, module: &ModuleStatement) -> Result<(), CompilerError> {
        // Generate WASM module
        // This would handle module-level constructs
        Ok(())
    }

    fn optimize_module(&self, backend: &LLVMBackend) -> Result<(), CompilerError> {
        // Apply WASM-specific optimizations
        match self.optimization_level {
            OptimizationLevel::None => {
                // No optimization
            }
            OptimizationLevel::Basic => {
                // Basic optimizations
                backend.module.verify().map_err(|e| {
                    CompilerError::codegen_error("wasm", &format!("Module verification failed: {}", e))
                })?;
            }
            OptimizationLevel::Aggressive => {
                // Aggressive optimizations
                backend.module.verify().map_err(|e| {
                    CompilerError::codegen_error("wasm", &format!("Module verification failed: {}", e))
                })?;
                
                // Apply optimization passes
                let pass_manager = inkwell::passes::PassManager::create(&backend.module);
                pass_manager.add_instruction_combining_pass();
                pass_manager.add_reassociate_pass();
                pass_manager.add_gvn_pass();
                pass_manager.add_cfg_simplification_pass();
                pass_manager.run(&backend.module);
            }
        }
        
        Ok(())
    }

    fn generate_wasm_bytecode(&self, backend: &LLVMBackend) -> Result<Vec<u8>, CompilerError> {
        // Generate WASM bytecode from LLVM IR
        // This would use LLVM's WASM target to generate bytecode
        
        // For now, return a placeholder
        // In a real implementation, this would:
        // 1. Use LLVM's WASM target
        // 2. Generate WASM bytecode
        // 3. Apply WASM-specific optimizations
        
        Ok(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]) // WASM magic number + version
    }
}

impl WASMFeatures {
    pub fn default() -> Self {
        Self {
            threads: false,
            simd: false,
            bulk_memory: false,
            reference_types: false,
            exception_handling: false,
            tail_calls: false,
            function_references: false,
        }
    }

    pub fn with_threads(mut self) -> Self {
        self.threads = true;
        self
    }

    pub fn with_simd(mut self) -> Self {
        self.simd = true;
        self
    }

    pub fn with_bulk_memory(mut self) -> Self {
        self.bulk_memory = true;
        self
    }

    pub fn with_reference_types(mut self) -> Self {
        self.reference_types = true;
        self
    }

    pub fn with_exception_handling(mut self) -> Self {
        self.exception_handling = true;
        self
    }

    pub fn with_tail_calls(mut self) -> Self {
        self.tail_calls = true;
        self
    }

    pub fn with_function_references(mut self) -> Self {
        self.function_references = true;
        self
    }
}

// WASM runtime utilities
pub struct WASMRuntime {
    #[cfg(feature = "wasm-backend")]
    pub instance: Option<wasmtime::Instance>,
    #[cfg(feature = "wasm-backend")]
    pub store: Option<wasmtime::Store<()>>,
    pub module_loaded: bool,
    pub functions: HashMap<String, WASMFunction>,
}

impl WASMRuntime {
    pub fn new() -> Self {
        Self {
            instance: None,
            store: None,
            module_loaded: false,
            functions: HashMap::new(),
        }
    }

    pub fn load_module(&mut self, wasm_bytes: &[u8]) -> Result<(), CompilerError> {
        // Load WASM module using wasmtime
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wasm_bytes)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to load WASM module: {}", e)))?;
        
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[])
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to instantiate WASM module: {}", e)))?;
        
        self.instance = Some(instance);
        self.store = Some(store);
        
        Ok(())
    }

    #[cfg(feature = "wasm-backend")]
    pub fn call_function(&mut self, name: &str, args: &[wasmtime::Val]) -> Result<Vec<wasmtime::Val>, CompilerError> {
        if let (Some(instance), Some(store)) = (&self.instance, &mut self.store) {
            let func = instance.get_func(store, name)
                .ok_or_else(|| CompilerError::runtime_error(&format!("Function '{}' not found", name)))?;
            
            func.call(store, args, &mut vec![])
                .map_err(|e| CompilerError::runtime_error(&format!("Failed to call function: {}", e)))
        } else {
            Err(CompilerError::runtime_error("WASM runtime not initialized"))
        }
    }
}

// WASM utilities
pub fn compile_to_wasm(program: &Program, features: WASMFeatures) -> Result<Vec<u8>, CompilerError> {
    let mut compiler = WASMCompiler::new().with_features(features);
    compiler.compile_to_wasm(program)
}

pub fn create_wasm_runtime() -> WASMRuntime {
    WASMRuntime::new()
} 