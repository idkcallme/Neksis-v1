use crate::ast::{
    Expression, Statement, Literal, Type, Program, LetStatement, FunctionStatement, ReturnStatement,
    BorrowExpression, BorrowMutExpression, CloneExpression, MoveStatement, DropStatement,
    BorrowType, ModuleStatement, StructStatement, EnumStatement, BoxExpression,
    RcExpression, ArcExpression, CellExpression, RefCellExpression, MallocExpression,
    FreeExpression, ReallocExpression, LifetimeExpression, MatchExpression, SpawnExpression,
    JoinExpression, ChannelExpression, TryExpression, TryCatchExpression, PipelineExpression,
    ClassStatement, InterpolatedPart, CallArgument, UnaryOperator
};
use crate::error::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeValue {
    Int,
    Float,
    Bool,
    String,
    Void,
    Function(Vec<TypeValue>, Box<TypeValue>),
    Struct(String), // struct name
    Enum(String),   // enum name
    Generic(String, Vec<TypeValue>), // generic type with type arguments
    Reference(Box<TypeValue>, BorrowType, Option<String>), // reference with lifetime
    Owned(Box<TypeValue>),
    Shared(Box<TypeValue>),
    Weak(Box<TypeValue>),
    Unique(Box<TypeValue>),
    Result(Box<TypeValue>, Box<TypeValue>), // Result<T, E>
    Option(Box<TypeValue>), // Option<T>
    Array(Box<TypeValue>, usize), // Fixed-size array
    Slice(Box<TypeValue>), // Dynamic slice
    Tuple(Vec<TypeValue>), // Tuple type
    Union(Vec<TypeValue>), // Union type
    Never, // Never type
    Unknown,
    Pointer, // Add pointer type support
}

impl TypeValue {
    pub fn from_literal(literal: &Literal) -> Self {
        match literal {
            Literal::Int(_) => TypeValue::Int,
            Literal::Float(_) => TypeValue::Float,
            Literal::Bool(_) => TypeValue::Bool,
            Literal::String(_) => TypeValue::String,
            Literal::Char(_) => TypeValue::Int, // char as int for now
            _ => TypeValue::Unknown,
        }
    }

    pub fn is_compatible_with(&self, other: &TypeValue) -> bool {
        match (self, other) {
            (TypeValue::Int, TypeValue::Int) => true,
            (TypeValue::Float, TypeValue::Float) => true,
            (TypeValue::Bool, TypeValue::Bool) => true,
            (TypeValue::String, TypeValue::String) => true,
            (TypeValue::Int, TypeValue::Float) => true, // Implicit conversion
            (TypeValue::Float, TypeValue::Int) => true, // Implicit conversion
            (TypeValue::Struct(name1), TypeValue::Struct(name2)) => name1 == name2,
            (TypeValue::Enum(name1), TypeValue::Enum(name2)) => name1 == name2,
            (TypeValue::Generic(name1, args1), TypeValue::Generic(name2, args2)) => {
                name1 == name2 && args1.len() == args2.len() && 
                args1.iter().zip(args2.iter()).all(|(a, b)| a.is_compatible_with(b))
            }
            (TypeValue::Reference(inner1, borrow1, lifetime1), TypeValue::Reference(inner2, borrow2, lifetime2)) => {
                inner1.is_compatible_with(inner2) && borrow1 == borrow2 && lifetime1 == lifetime2
            }
            (TypeValue::Owned(inner1), TypeValue::Owned(inner2)) => inner1.is_compatible_with(inner2),
            (TypeValue::Shared(inner1), TypeValue::Shared(inner2)) => inner1.is_compatible_with(inner2),
            (TypeValue::Weak(inner1), TypeValue::Weak(inner2)) => inner1.is_compatible_with(inner2),
            (TypeValue::Unique(inner1), TypeValue::Unique(inner2)) => inner1.is_compatible_with(inner2),
            (TypeValue::Result(ok1, err1), TypeValue::Result(ok2, err2)) => {
                ok1.is_compatible_with(ok2) && err1.is_compatible_with(err2)
            }
            (TypeValue::Option(inner1), TypeValue::Option(inner2)) => inner1.is_compatible_with(inner2),
            (TypeValue::Array(inner1, size1), TypeValue::Array(inner2, size2)) => {
                inner1.is_compatible_with(inner2) && size1 == size2
            }
            (TypeValue::Slice(inner1), TypeValue::Slice(inner2)) => inner1.is_compatible_with(inner2),
            (TypeValue::Tuple(types1), TypeValue::Tuple(types2)) => {
                types1.len() == types2.len() && 
                types1.iter().zip(types2.iter()).all(|(a, b)| a.is_compatible_with(b))
            }
            (TypeValue::Union(types1), TypeValue::Union(types2)) => {
                types1.len() == types2.len() && 
                types1.iter().zip(types2.iter()).all(|(a, b)| a.is_compatible_with(b))
            }
            (TypeValue::Never, TypeValue::Never) => true,
            (TypeValue::Unknown, _) => true, // Unknown is compatible with everything
            (_, TypeValue::Unknown) => true, // Everything is compatible with Unknown
            _ => false,
        }
    }
}

pub struct SemanticAnalyzer {
    variables: HashMap<String, TypeValue>,
    functions: HashMap<String, TypeValue>,
    structs: HashMap<String, Vec<String>>, // struct name -> field names
    enums: HashMap<String, Vec<String>>,   // enum name -> variant names
    modules: HashMap<String, HashMap<String, TypeValue>>, // module name -> (function name -> type)
    current_function: Option<String>,
    lifetimes: HashMap<String, usize>, // lifetime name -> scope depth
    ownership_info: HashMap<String, OwnershipInfo>,
    pub gradual_ownership: bool, // New field for gradual ownership mode
}

#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    pub ownership: BorrowType,
    pub lifetime: Option<String>,
    pub is_moved: bool,
    pub borrow_count: usize,
    pub mutable_borrow_count: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            modules: HashMap::new(),
            current_function: None,
            lifetimes: HashMap::new(),
            ownership_info: HashMap::new(),
            gradual_ownership: true, // Gradual mode enabled by default
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), CompilerError> {
        // First pass: collect struct, enum, and class definitions
        for statement in &program.statements {
            match statement {
                Statement::Struct(struct_stmt) => self.analyze_struct_statement(struct_stmt)?,
                Statement::Enum(enum_stmt) => self.analyze_enum_statement(enum_stmt)?,
                Statement::Class(class_stmt) => self.analyze_class_statement(class_stmt)?,
                _ => {}
            }
        }

        // Second pass: collect module definitions
        for statement in &program.statements {
            if let Statement::Module(module_stmt) = statement {
                self.analyze_module_statement(module_stmt)?;
            }
        }

        // Third pass: collect function signatures
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.analyze_function_signature(func_stmt)?;
            }
        }

        // Fourth pass: analyze all statements
        for statement in &program.statements {
            self.analyze_statement(statement)?;
        }

        Ok(())
    }

    fn analyze_module_statement(&mut self, module_stmt: &ModuleStatement) -> Result<(), CompilerError> {
        let module_name = module_stmt.name.clone();
        let mut module_functions = HashMap::new();
        
        for statement in &module_stmt.statements {
            if let Statement::Function(func_stmt) = statement {
                let function_type = self.get_function_type(&func_stmt)?;
                module_functions.insert(func_stmt.name.clone(), function_type);
            }
        }
        
        self.modules.insert(module_name, module_functions);
        Ok(())
    }

    fn analyze_function_signature(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        let function_type = self.get_function_type(func_stmt)?;
        self.functions.insert(func_stmt.name.clone(), function_type);
        Ok(())
    }

    fn analyze_function_body(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        self.current_function = Some(func_stmt.name.clone());
        
        // Analyze parameters
        for param in &func_stmt.parameters {
            let param_type = self.convert_ast_type_to_type_value(&param.type_annotation)?;
            self.variables.insert(param.name.clone(), param_type);
            
            // Initialize ownership info
            self.ownership_info.insert(param.name.clone(), OwnershipInfo {
                ownership: param.borrow_type.clone().unwrap_or(BorrowType::Move),
                lifetime: param.lifetime.as_ref().map(|lt| lt.name.clone()),
                is_moved: false,
                borrow_count: 0,
                mutable_borrow_count: 0,
            });
        }
        
        // Analyze function body
        self.analyze_expression(&func_stmt.body)?;
        
        self.current_function = None;
        Ok(())
    }

    fn analyze_struct_statement(&mut self, struct_stmt: &StructStatement) -> Result<(), CompilerError> {
        let field_names: Vec<String> = struct_stmt.fields.iter().map(|f| f.name.clone()).collect();
        self.structs.insert(struct_stmt.name.clone(), field_names);
        Ok(())
    }

    fn analyze_enum_statement(&mut self, enum_stmt: &EnumStatement) -> Result<(), CompilerError> {
        let variant_names: Vec<String> = enum_stmt.variants.iter().map(|v| v.name.clone()).collect();
        self.enums.insert(enum_stmt.name.clone(), variant_names);
        Ok(())
    }

    fn analyze_class_statement(&mut self, class_stmt: &ClassStatement) -> Result<(), CompilerError> {
        // Register class fields and methods
        let field_names: Vec<String> = class_stmt.fields.iter().map(|f| f.name.clone()).collect();
        self.structs.insert(class_stmt.name.clone(), field_names);
        // Optionally, register methods for method resolution
        for method in &class_stmt.methods {
            let method_type = self.get_function_type(method)?;
            self.functions.insert(format!("{}::{}", class_stmt.name, method.name), method_type);
        }
        // Check superclass exists if specified
        if let Some(ref super_name) = class_stmt.superclass {
            if !self.structs.contains_key(super_name) {
                return Err(CompilerError::type_error(&format!("Superclass '{}' not found for class '{}'", super_name, class_stmt.name)));
            }
        }
        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Let(let_stmt) => self.analyze_let_statement(let_stmt),
            Statement::Return(return_stmt) => self.analyze_return_statement(return_stmt),
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
                Ok(())
            }
            Statement::Move(move_stmt) => self.analyze_move_statement(move_stmt),
            Statement::Drop(drop_stmt) => self.analyze_drop_statement(drop_stmt),
            _ => Ok(()), // TODO: Implement other statement types
        }
    }

    fn analyze_let_statement(&mut self, let_stmt: &LetStatement) -> Result<(), CompilerError> {
        let value_type = self.analyze_expression(&let_stmt.value)?;
        
        // Check type annotation if provided
        if let Some(type_annotation) = &let_stmt.type_annotation {
            let expected_type = self.convert_ast_type_to_type_value(type_annotation)?;
            
            if !value_type.is_compatible_with(&expected_type) {
                return Err(CompilerError::type_error(&format!(
                    "Type mismatch: expected {:?}, got {:?}", expected_type, value_type
                )));
            }
        }
        
        self.variables.insert(let_stmt.name.clone(), value_type);
        
        // Initialize ownership info
        self.ownership_info.insert(let_stmt.name.clone(), OwnershipInfo {
            ownership: BorrowType::Move, // Default ownership for let statements
            lifetime: None, // Will be set based on type annotation
            is_moved: false,
            borrow_count: 0,
            mutable_borrow_count: 0,
        });
        
        Ok(())
    }

    fn analyze_return_statement(&mut self, return_stmt: &ReturnStatement) -> Result<(), CompilerError> {
        if let Some(value) = &return_stmt.value {
            let return_type = self.analyze_expression(value)?;
            
            // Check against function return type if available
            if let Some(function_name) = &self.current_function {
                if let Some(function_type) = self.functions.get(function_name) {
                    if let TypeValue::Function(_, return_type_expected) = function_type {
                        if !return_type.is_compatible_with(return_type_expected) {
                            return Err(CompilerError::type_error(&format!(
                                "Return type mismatch: expected {:?}, got {:?}", return_type_expected, return_type
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_move_statement(&mut self, move_stmt: &MoveStatement) -> Result<(), CompilerError> {
        // Check if source variable exists
        if !self.variables.contains_key(&move_stmt.from) {
            return Err(CompilerError::type_error(&format!("Variable {} not found", move_stmt.from)));
        }
        
        // Get source type
        let source_type = self.variables.get(&move_stmt.from).unwrap().clone();
        
        // Register destination variable
        self.variables.insert(move_stmt.to.clone(), source_type);
        
        // Mark source as moved
        if let Some(ownership_info) = self.ownership_info.get_mut(&move_stmt.from) {
            ownership_info.is_moved = true;
        }
        
        // Initialize destination ownership info
        self.ownership_info.insert(move_stmt.to.clone(), OwnershipInfo {
            ownership: BorrowType::Move,
            lifetime: None,
            is_moved: false,
            borrow_count: 0,
            mutable_borrow_count: 0,
        });
        
        Ok(())
    }

    fn analyze_drop_statement(&mut self, drop_stmt: &DropStatement) -> Result<(), CompilerError> {
        // Check if variable exists
        if !self.variables.contains_key(&drop_stmt.variable) {
            return Err(CompilerError::type_error(&format!("Variable {} not found", drop_stmt.variable)));
        }
        
        // Mark as moved
        if let Some(ownership_info) = self.ownership_info.get_mut(&drop_stmt.variable) {
            ownership_info.is_moved = true;
        }
        
        Ok(())
    }

    fn analyze_expression(&mut self, expression: &Expression) -> Result<TypeValue, CompilerError> {
        match expression {
            Expression::Literal(literal) => Ok(TypeValue::from_literal(literal)),
            Expression::Identifier(name) => {
                if name == "print" || name == "println" || name == "read_line" {
                    // Treat as built-in function: type is Function(Any, Any)
                    Ok(TypeValue::Function(vec![TypeValue::Unknown], Box::new(TypeValue::Unknown)))
                } else if let Some(var_type) = self.variables.get(name) {
                    Ok(var_type.clone())
                } else {
                    // Check if it's a function
                    if let Some(function_type) = self.functions.get(name) {
                        Ok(function_type.clone())
                    } else {
                        Err(CompilerError::type_error(&format!("Undefined variable or function: {}", name)))
                    }
                }
            }
            Expression::BinaryOp(bin_op) => {
                let left_type = self.analyze_expression(&bin_op.left)?;
                let right_type = self.analyze_expression(&bin_op.right)?;
                
                // Check type compatibility for binary operations
                match (&left_type, &right_type) {
                    (TypeValue::Int, TypeValue::Int) => Ok(TypeValue::Int),
                    (TypeValue::Float, TypeValue::Float) => Ok(TypeValue::Float),
                    (TypeValue::Int, TypeValue::Float) => Ok(TypeValue::Float),
                    (TypeValue::Float, TypeValue::Int) => Ok(TypeValue::Float),
                    (TypeValue::Bool, TypeValue::Bool) => Ok(TypeValue::Bool),
                    _ => self.ownership_error_or_warning(&format!("Invalid binary operation between {:?} and {:?}", left_type, right_type)),
                }
            }
            Expression::UnaryOp(unary_op) => {
                let operand_type = self.analyze_expression(&unary_op.operand)?;
                
                match &unary_op.operator {
                    UnaryOperator::Negate => {
                        match operand_type {
                            TypeValue::Int | TypeValue::Float => Ok(operand_type),
                            _ => self.ownership_error_or_warning("Cannot negate non-numeric value"),
                        }
                    }
                    UnaryOperator::Not => {
                        match operand_type {
                            TypeValue::Bool => Ok(TypeValue::Bool),
                            _ => self.ownership_error_or_warning("Cannot apply NOT to non-boolean value"),
                        }
                    }
                    UnaryOperator::Copy => Ok(operand_type),
                    UnaryOperator::Borrow => Ok(TypeValue::Reference(Box::new(operand_type), BorrowType::Borrowed, None)),
                    UnaryOperator::BorrowMut => Ok(TypeValue::Reference(Box::new(operand_type), BorrowType::MutableBorrowed, None)),
                    UnaryOperator::Move => Ok(operand_type),
                    UnaryOperator::Drop => Ok(TypeValue::Void),
                    UnaryOperator::Dereference => {
                        match operand_type {
                            TypeValue::Pointer => Ok(TypeValue::Unknown), // TODO: Implement proper pointer dereferencing
                            _ => self.ownership_error_or_warning("Cannot dereference non-pointer value"),
                        }
                    }
                    UnaryOperator::Reference => Ok(TypeValue::Pointer),
                    UnaryOperator::ReferenceMut => Ok(TypeValue::Pointer),
                }
            }
            Expression::FunctionCall(call, args) => self.analyze_function_call(call, args),
            Expression::If(if_expr) => {
                let condition_type = self.analyze_expression(&if_expr.condition)?;
                if condition_type != TypeValue::Bool {
                    return self.ownership_error_or_warning("If condition must be boolean");
                }
                
                let then_type = self.analyze_expression(&if_expr.then_branch)?;
                let else_type = if let Some(ref else_branch) = if_expr.else_branch {
                    self.analyze_expression(else_branch)?
                } else {
                    TypeValue::Void
                };
                
                if then_type.is_compatible_with(&else_type) {
                    Ok(then_type)
                } else {
                    self.ownership_error_or_warning(&format!("If branches have incompatible types: {:?} and {:?}", then_type, else_type))
                }
            }
            Expression::While(while_expr) => {
                let condition_type = self.analyze_expression(&while_expr.condition)?;
                if condition_type != TypeValue::Bool {
                    return self.ownership_error_or_warning("While condition must be boolean");
                }
                
                self.analyze_expression(&while_expr.body)?;
                Ok(TypeValue::Void)
            }
            Expression::Loop(loop_expr) => {
                self.analyze_expression(&loop_expr.body)?;
                Ok(TypeValue::Void)
            }
            Expression::Block(statements) => {
                let mut last_type = TypeValue::Void;
                for statement in statements {
                    match statement {
                        Statement::Expression(expr) => {
                            last_type = self.analyze_expression(expr)?;
                        }
                        _ => {
                            self.analyze_statement(statement)?;
                        }
                    }
                }
                Ok(last_type)
            }
            Expression::StructLiteral(struct_lit) => {
                // For now, just return the struct type
                Ok(TypeValue::Struct(struct_lit.struct_name.clone()))
            }
            Expression::MemberAccess(member_access) => {
                let object_type = self.analyze_expression(&member_access.object)?;
                // For now, just return the object type
                // In a real implementation, we would look up the member type
                Ok(object_type)
            }
            Expression::EnumVariantAccess { enum_name, variant_name: _ } => {
                // For now, just return the enum type
                Ok(TypeValue::Enum(enum_name.clone()))
            }
            Expression::BuiltinFunction { name, arguments } => {
                self.analyze_builtin_function(name, arguments)
            }
            Expression::ArrayAccess(array_access) => {
                // Analyze array access
                let array_type = self.analyze_expression(&array_access.array)?;
                let index_type = self.analyze_expression(&array_access.index)?;
                
                // Check that index is an integer
                if index_type != TypeValue::Int {
                    return Err(CompilerError::type_error("Array index must be an integer"));
                }
                
                // Return the element type of the array
                match array_type {
                    TypeValue::Array(element_type, _) => Ok(*element_type),
                    _ => Err(CompilerError::type_error("Cannot access non-array type")),
                }
            }
            // Memory Management
            Expression::Box(box_expr) => self.analyze_box_expression(box_expr),
            Expression::Rc(rc_expr) => self.analyze_rc_expression(rc_expr),
            Expression::Arc(arc_expr) => self.analyze_arc_expression(arc_expr),
            Expression::Cell(cell_expr) => self.analyze_cell_expression(cell_expr),
            Expression::RefCell(refcell_expr) => self.analyze_refcell_expression(refcell_expr),
            Expression::Malloc(malloc_expr) => self.analyze_malloc_expression(malloc_expr),
            Expression::Free(free_expr) => self.analyze_free_expression(free_expr),
            Expression::Realloc(realloc_expr) => self.analyze_realloc_expression(realloc_expr),
            // Ownership and Borrowing
            Expression::Borrow(borrow_expr) => self.analyze_borrow_expression(borrow_expr),
            Expression::BorrowMut(borrow_mut_expr) => self.analyze_borrow_mut_expression(borrow_mut_expr),
            Expression::Move(move_expr) => self.analyze_move_expression(move_expr),
            Expression::Clone(clone_expr) => self.analyze_clone_expression(clone_expr),
            Expression::Drop(drop_expr) => self.analyze_drop_expression(drop_expr),
            // Lifetime Management
            Expression::Lifetime(lifetime_expr) => self.analyze_lifetime_expression(lifetime_expr),
            // Pattern Matching
            Expression::Match(match_expr) => self.analyze_match_expression(match_expr),
            // Concurrency
            Expression::Spawn(spawn_expr) => self.analyze_spawn_expression(spawn_expr),
            Expression::Join(join_expr) => self.analyze_join_expression(join_expr),
            Expression::Channel(channel_expr) => self.analyze_channel_expression(channel_expr),
            // Error Handling
            Expression::Try(try_expr) => self.analyze_try_expression(try_expr),
            Expression::TryCatch(try_catch_expr) => self.analyze_try_catch_expression(try_catch_expr),
            // Pipeline
            Expression::Pipeline(pipeline_expr) => self.analyze_pipeline_expression(pipeline_expr),
            // Missing expression cases
            Expression::Return(return_expr) => {
                if let Some(expr) = return_expr {
                    self.analyze_expression(expr)
                } else {
                    Ok(TypeValue::Void)
                }
            }
            Expression::Let(let_stmt) => {
                self.analyze_let_statement(let_stmt)?;
                Ok(TypeValue::Void)
            }
            Expression::Assignment(assignment_stmt) => {
                let value_type = self.analyze_expression(&assignment_stmt.value)?;
                // Check if target variable exists and has compatible type
                if let Some(target_type) = self.variables.get(&assignment_stmt.target) {
                    if !value_type.is_compatible_with(target_type) {
                        return self.ownership_error_or_warning(&format!(
                            "Assignment type mismatch: expected {:?}, got {:?}", target_type, value_type
                        ));
                    }
                }
                Ok(value_type)
            }
            Expression::BinaryOperation { left, operator: _, right } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;
                
                // Check type compatibility for binary operations
                match (&left_type, &right_type) {
                    (TypeValue::Int, TypeValue::Int) => Ok(TypeValue::Int),
                    (TypeValue::Float, TypeValue::Float) => Ok(TypeValue::Float),
                    (TypeValue::Int, TypeValue::Float) => Ok(TypeValue::Float),
                    (TypeValue::Float, TypeValue::Int) => Ok(TypeValue::Float),
                    (TypeValue::Bool, TypeValue::Bool) => Ok(TypeValue::Bool),
                    _ => self.ownership_error_or_warning(&format!("Invalid binary operation between {:?} and {:?}", left_type, right_type)),
                }
            }
            Expression::Throw(throw_expr) => {
                // Analyze the thrown value (could be any type, but often an error/exception type)
                let _ = self.analyze_expression(&throw_expr.value)?;
                // Throw never returns normally, so use Never type
                Ok(TypeValue::Never)
            }
            Expression::Lambda(lambda_expr) => {
                // Infer parameter types (use Unknown if not annotated)
                let param_types: Vec<TypeValue> = lambda_expr.parameters.iter().map(|p| {
                    self.convert_ast_type_to_type_value(&p.type_annotation).unwrap_or(TypeValue::Unknown)
                }).collect();
                // Analyze body type
                let body_type = self.analyze_expression(&lambda_expr.body)?;
                Ok(TypeValue::Function(param_types, Box::new(body_type)))
            }
            Expression::DictLiteral(dict_expr) => {
                // TODO: Type-check keys and values
                for (k, v) in &dict_expr.entries {
                    self.analyze_expression(k)?;
                    self.analyze_expression(v)?;
                }
                Ok(TypeValue::Unknown)
            }
            Expression::SetLiteral(set_expr) => {
                // TODO: Type-check elements
                for elem in &set_expr.elements {
                    self.analyze_expression(elem)?;
                }
                Ok(TypeValue::Unknown)
            }
            Expression::InterpolatedString(interp_expr) => {
                for part in &interp_expr.parts {
                    if let InterpolatedPart::Expr(expr) = part {
                        self.analyze_expression(expr)?;
                    }
                }
                Ok(TypeValue::String)
            }
            Expression::ListComprehension(list_comp) => {
                // Analyze the iterable
                let _iterable_type = self.analyze_expression(&list_comp.iterable)?;
                // Analyze the element expression
                let _ = self.analyze_expression(&list_comp.element)?;
                // Optionally analyze the condition
                if let Some(cond) = &list_comp.condition {
                    let cond_type = self.analyze_expression(cond)?;
                    if cond_type != TypeValue::Bool {
                        return Err(CompilerError::type_error("List comprehension condition must be boolean"));
                    }
                }
                // For now, return a dynamic array type
                Ok(TypeValue::Array(Box::new(TypeValue::Unknown), 0))
            }
            Expression::Slice(slice_expr) => {
                // Analyze the collection being sliced
                let _ = self.analyze_expression(&slice_expr.collection)?;
                // Optionally analyze start, end, step
                if let Some(start) = &slice_expr.start {
                    let _ = self.analyze_expression(start)?;
                }
                if let Some(end) = &slice_expr.end {
                    let _ = self.analyze_expression(end)?;
                }
                if let Some(step) = &slice_expr.step {
                    let _ = self.analyze_expression(step)?;
                }
                // For now, return a slice type
                Ok(TypeValue::Slice(Box::new(TypeValue::Unknown)))
            }
        }
    }

    fn convert_ast_type_to_type_value(&self, ast_type: &Type) -> Result<TypeValue, CompilerError> {
        match ast_type {
            Type::Int => Ok(TypeValue::Int),
            Type::Float => Ok(TypeValue::Float),
            Type::Bool => Ok(TypeValue::Bool),
            Type::String => Ok(TypeValue::String),
            Type::Void => Ok(TypeValue::Void),
            Type::Function(params, return_type) => {
                let param_types: Vec<TypeValue> = params
                    .iter()
                    .map(|t| self.convert_ast_type_to_type_value(t))
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                let return_type_value = self.convert_ast_type_to_type_value(return_type)?;
                Ok(TypeValue::Function(param_types, Box::new(return_type_value)))
            }
            Type::Unknown => Ok(TypeValue::Unknown),
            Type::Struct(name) => Ok(TypeValue::Struct(name.clone())),
            Type::Enum(name) => Ok(TypeValue::Enum(name.clone())),
            Type::GenericType(name, args) => {
                let type_args: Vec<TypeValue> = args
                    .iter()
                    .map(|t| self.convert_ast_type_to_type_value(t))
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                Ok(TypeValue::Generic(name.clone(), type_args))
            }
            Type::Reference(inner_type, borrow_type, lifetime) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                let lifetime_name = lifetime.as_ref().map(|lt| lt.name.clone());
                Ok(TypeValue::Reference(Box::new(inner_type_value), borrow_type.clone(), lifetime_name))
            }
            Type::Owned(inner_type) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                Ok(TypeValue::Owned(Box::new(inner_type_value)))
            }
            Type::Shared(inner_type) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                Ok(TypeValue::Shared(Box::new(inner_type_value)))
            }
            Type::Weak(inner_type) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                Ok(TypeValue::Weak(Box::new(inner_type_value)))
            }
            Type::Unique(inner_type) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                Ok(TypeValue::Unique(Box::new(inner_type_value)))
            }
            Type::Result(ok_type, err_type) => {
                let ok_type_value = self.convert_ast_type_to_type_value(ok_type)?;
                let err_type_value = self.convert_ast_type_to_type_value(err_type)?;
                Ok(TypeValue::Result(Box::new(ok_type_value), Box::new(err_type_value)))
            }
            Type::Option(inner_type) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                Ok(TypeValue::Option(Box::new(inner_type_value)))
            }
            Type::Array(inner_type, size) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                Ok(TypeValue::Array(Box::new(inner_type_value), *size))
            }
            Type::Slice(inner_type) => {
                let inner_type_value = self.convert_ast_type_to_type_value(inner_type)?;
                Ok(TypeValue::Slice(Box::new(inner_type_value)))
            }
            Type::Tuple(types) => {
                let type_values: Vec<TypeValue> = types
                    .iter()
                    .map(|t| self.convert_ast_type_to_type_value(t))
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                Ok(TypeValue::Tuple(type_values))
            }
            Type::Union(types) => {
                let type_values: Vec<TypeValue> = types
                    .iter()
                    .map(|t| self.convert_ast_type_to_type_value(t))
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                Ok(TypeValue::Union(type_values))
            }
            Type::Never => Ok(TypeValue::Never),
            Type::Pointer(_) => Ok(TypeValue::Pointer), // Add pointer type support
            Type::Char => Ok(TypeValue::String), // Treat char as string for now
            Type::Trait(_) => Ok(TypeValue::Unknown), // Treat traits as unknown for now
            Type::Generic(_, _) => Ok(TypeValue::Unknown), // Treat generics as unknown for now
        }
    }

    fn analyze_function_call(&mut self, call: &Expression, args: &[CallArgument]) -> Result<TypeValue, CompilerError> {
        let function_type = self.analyze_expression(call)?;
        match function_type {
            TypeValue::Function(param_types, return_type) => {
                // For now, assume we can get parameter names and default values from a stub
                // In a real implementation, this would look up the function definition
                // Here, we just check argument count and types
                let _used_params = vec![false; param_types.len()];
                let _matched_args: Vec<Option<Expression>> = vec![None; param_types.len()];
                let mut param_names = Vec::new();
                for i in 0..param_types.len() {
                    param_names.push(format!("arg{}", i));
                }
                // TODO: Get real parameter names and default values from function definition
                // For now, only support positional arguments
                if args.len() != param_types.len() {
                    return self.ownership_error_or_warning(&format!(
                        "Function call has {} arguments but expected {}",
                        args.len(), param_types.len()
                    ));
                }
                for (i, arg) in args.iter().enumerate() {
                    let arg_type = self.analyze_expression(&arg.value)?;
                    if !arg_type.is_compatible_with(&param_types[i]) {
                        return self.ownership_error_or_warning(&format!(
                            "Argument type mismatch: expected {:?}, got {:?}",
                            param_types[i], arg_type
                        ));
                    }
                }
                Ok(*return_type)
            }
            _ => self.ownership_error_or_warning("Cannot call non-function value"),
        }
    }

    // Memory Management
    fn analyze_box_expression(&mut self, box_expr: &BoxExpression) -> Result<TypeValue, CompilerError> {
        let inner_type = self.analyze_expression(&box_expr.value)?;
        Ok(TypeValue::Unique(Box::new(inner_type)))
    }

    fn analyze_rc_expression(&mut self, rc_expr: &RcExpression) -> Result<TypeValue, CompilerError> {
        let inner_type = self.analyze_expression(&rc_expr.value)?;
        Ok(TypeValue::Shared(Box::new(inner_type)))
    }

    fn analyze_arc_expression(&mut self, arc_expr: &ArcExpression) -> Result<TypeValue, CompilerError> {
        let inner_type = self.analyze_expression(&arc_expr.value)?;
        Ok(TypeValue::Generic("Arc".to_string(), vec![inner_type]))
    }

    fn analyze_cell_expression(&mut self, cell_expr: &CellExpression) -> Result<TypeValue, CompilerError> {
        let inner_type = self.analyze_expression(&cell_expr.value)?;
        Ok(TypeValue::Owned(Box::new(inner_type)))
    }

    fn analyze_refcell_expression(&mut self, refcell_expr: &RefCellExpression) -> Result<TypeValue, CompilerError> {
        let inner_type = self.analyze_expression(&refcell_expr.value)?;
        Ok(TypeValue::Owned(Box::new(inner_type)))
    }

    fn analyze_malloc_expression(&mut self, malloc_expr: &MallocExpression) -> Result<TypeValue, CompilerError> {
        let size_type = self.analyze_expression(&malloc_expr.size)?;
        if size_type != TypeValue::Int {
            return Err(CompilerError::type_error("Malloc size must be an integer"));
        }
        
        // Return a pointer type based on the type annotation
        if let Some(type_annotation) = &malloc_expr.type_annotation {
            let inner_type = self.convert_ast_type_to_type_value(type_annotation)?;
            Ok(TypeValue::Reference(Box::new(inner_type), BorrowType::MutableBorrowed, None))
        } else {
            // Default to void pointer
            Ok(TypeValue::Reference(Box::new(TypeValue::Void), BorrowType::MutableBorrowed, None))
        }
    }

    fn analyze_free_expression(&mut self, free_expr: &FreeExpression) -> Result<TypeValue, CompilerError> {
        let pointer_type = self.analyze_expression(&free_expr.pointer)?;
        
        // Check if it's a pointer type
        match pointer_type {
            TypeValue::Reference(_, _, _) => Ok(TypeValue::Void),
            _ => Err(CompilerError::type_error("Free can only be applied to pointers")),
        }
    }

    fn analyze_realloc_expression(&mut self, realloc_expr: &ReallocExpression) -> Result<TypeValue, CompilerError> {
        let pointer_type = self.analyze_expression(&realloc_expr.pointer)?;
        let new_size_type = self.analyze_expression(&realloc_expr.new_size)?;
        
        if new_size_type != TypeValue::Int {
            return Err(CompilerError::type_error("Realloc new size must be an integer"));
        }
        
        // Check if it's a pointer type
        match pointer_type {
            TypeValue::Reference(inner_type, _, _) => {
                Ok(TypeValue::Reference(inner_type, BorrowType::MutableBorrowed, None))
            }
            _ => Err(CompilerError::type_error("Realloc can only be applied to pointers")),
        }
    }

    // Ownership and Borrowing
    fn analyze_borrow_expression(&mut self, borrow_expr: &BorrowExpression) -> Result<TypeValue, CompilerError> {
        let value_type = self.analyze_expression(&borrow_expr.expression)?;
        let lifetime_name = borrow_expr.lifetime.as_ref().map(|lt| lt.name.clone());
        Ok(TypeValue::Reference(Box::new(value_type), BorrowType::Borrowed, lifetime_name))
    }

    fn analyze_borrow_mut_expression(&mut self, borrow_mut_expr: &BorrowMutExpression) -> Result<TypeValue, CompilerError> {
        let value_type = self.analyze_expression(&borrow_mut_expr.expression)?;
        let lifetime_name = borrow_mut_expr.lifetime.as_ref().map(|lt| lt.name.clone());
        Ok(TypeValue::Reference(Box::new(value_type), BorrowType::MutableBorrowed, lifetime_name))
    }

    fn analyze_move_expression(&mut self, _move_expr: &MoveStatement) -> Result<TypeValue, CompilerError> {
        // Move statements don't have a value field, they just transfer ownership
        // The actual value analysis would be done in the expression that uses the move
        Ok(TypeValue::Void)
    }

    fn analyze_clone_expression(&mut self, clone_expr: &CloneExpression) -> Result<TypeValue, CompilerError> {
        let value_type = self.analyze_expression(&clone_expr.expression)?;
        Ok(value_type)
    }

    fn analyze_drop_expression(&mut self, _drop_expr: &DropStatement) -> Result<TypeValue, CompilerError> {
        // Drop statements don't have a value field, they just mark variables for cleanup
        // The actual value analysis would be done in the expression that uses the drop
        Ok(TypeValue::Void)
    }

    // Lifetime Management
    fn analyze_lifetime_expression(&mut self, lifetime_expr: &LifetimeExpression) -> Result<TypeValue, CompilerError> {
        let value_type = self.analyze_expression(&lifetime_expr.expression)?;
        Ok(value_type)
    }

    // Pattern Matching
    fn analyze_match_expression(&mut self, match_expr: &MatchExpression) -> Result<TypeValue, CompilerError> {
        let value_type = self.analyze_expression(&match_expr.expression)?;
        
        // For now, just return the value type
        // In a real implementation, we would analyze the match arms
        Ok(value_type)
    }

    // Concurrency
    fn analyze_spawn_expression(&mut self, spawn_expr: &SpawnExpression) -> Result<TypeValue, CompilerError> {
        let _function_type = self.analyze_expression(&spawn_expr.expression)?;
        
        // For now, just return void
        // In a real implementation, we would return a thread handle type
        Ok(TypeValue::Void)
    }

    fn analyze_join_expression(&mut self, join_expr: &JoinExpression) -> Result<TypeValue, CompilerError> {
        let _thread_id_type = self.analyze_expression(&join_expr.handle)?;
        
        // For now, just return void
        // In a real implementation, we would return the thread result type
        Ok(TypeValue::Void)
    }

    fn analyze_channel_expression(&mut self, channel_expr: &ChannelExpression) -> Result<TypeValue, CompilerError> {
        let _channel_type = &channel_expr.channel_type;
        let _capacity = &channel_expr.capacity;
        
        // For now, just return void
        // In a real implementation, we would return a channel type
        Ok(TypeValue::Void)
    }

    // Error Handling
    fn analyze_try_expression(&mut self, try_expr: &TryExpression) -> Result<TypeValue, CompilerError> {
        let expression_type = self.analyze_expression(&try_expr.expression)?;
        
        // For now, just return the expression type
        // In a real implementation, we would handle Result types
        Ok(expression_type)
    }

    fn analyze_try_catch_expression(&mut self, try_catch_expr: &TryCatchExpression) -> Result<TypeValue, CompilerError> {
        let try_type = self.analyze_expression(&try_catch_expr.try_block)?;
        let _catch_type = self.analyze_expression(&try_catch_expr.catch_block)?;
        
        // For now, just return the try type
        // In a real implementation, we would ensure both branches return the same type
        Ok(try_type)
    }

    // Pipeline
    fn analyze_pipeline_expression(&mut self, pipeline_expr: &PipelineExpression) -> Result<TypeValue, CompilerError> {
        let mut value_type = TypeValue::Unknown;
        
        for stage in &pipeline_expr.stages {
            value_type = self.analyze_expression(stage)?;
        }
        
        Ok(value_type)
    }

    fn get_function_type(&self, func_stmt: &FunctionStatement) -> Result<TypeValue, CompilerError> {
        let param_types: Vec<TypeValue> = func_stmt
            .parameters
            .iter()
            .map(|param| self.convert_ast_type_to_type_value(&param.type_annotation))
            .collect::<Result<Vec<_>, CompilerError>>()?;
        
        let return_type = if let Some(return_type) = &func_stmt.return_type {
            self.convert_ast_type_to_type_value(return_type)?
        } else {
            TypeValue::Void
        };
        
        Ok(TypeValue::Function(param_types, Box::new(return_type)))
    }

    fn analyze_builtin_function(&mut self, name: &str, arguments: &[Expression]) -> Result<TypeValue, CompilerError> {
        match name {
            "print" => {
                // Check that all arguments are printable
                for arg in arguments {
                    let arg_type = self.analyze_expression(arg)?;
                    match arg_type {
                        TypeValue::Int | TypeValue::Float | TypeValue::Bool | TypeValue::String => {}
                        TypeValue::Generic(ref name, ref _args) => {
                            // Allow printing of generic types like Arc, Box, etc.
                            match name.as_str() {
                                "Arc" | "Box" | "Rc" | "Cell" | "RefCell" => {}
                                _ => return self.ownership_error_or_warning(&format!("Cannot print value of type {:?}", arg_type)),
                            }
                        }
                        _ => return self.ownership_error_or_warning(&format!("Cannot print value of type {:?}", arg_type)),
                    }
                }
                Ok(TypeValue::Void)
            }
            _ => self.ownership_error_or_warning(&format!("Unknown builtin function: {}", name)),
        }
    }

    fn ownership_error_or_warning(&self, msg: &str) -> Result<TypeValue, CompilerError> {
        if self.gradual_ownership {
            println!("[Warning] Ownership: {}", msg);
            Ok(TypeValue::Unknown)
        } else {
            Err(CompilerError::type_error(msg))
        }
    }
} 