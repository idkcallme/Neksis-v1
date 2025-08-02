use crate::ast::{Expression, Statement, Type, Literal, UnaryOperator, BinaryOp};
use crate::error::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeContext {
    functions: HashMap<String, Type>,
    current_scope: usize,
    scopes: Vec<HashMap<String, Type>>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            current_scope: 0,
            scopes: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.current_scope += 1;
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            self.scopes.pop();
            self.current_scope -= 1;
        }
    }

    pub fn declare_variable(&mut self, name: &str, var_type: Type) {
        self.scopes[self.current_scope].insert(name.to_string(), var_type);
    }

    pub fn get_variable_type(&self, name: &str) -> Option<Type> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(var_type) = scope.get(name) {
                return Some(var_type.clone());
            }
        }
        None
    }

    pub fn declare_function(&mut self, name: &str, func_type: Type) {
        self.functions.insert(name.to_string(), func_type);
    }

    pub fn get_function_type(&self, name: &str) -> Option<Type> {
        self.functions.get(name).cloned()
    }
}

pub struct TypeInferrer {
    context: TypeContext,
}

impl TypeInferrer {
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
        }
    }

    pub fn infer_program(&mut self, statements: &[Statement]) -> Result<(), CompilerError> {
        for statement in statements {
            self.infer_statement(statement)?;
        }
        Ok(())
    }

    fn infer_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::LetStatement { name, value, var_type } => {
                let inferred_type = self.infer_expression(value)?;
                
                // If type annotation is provided, check compatibility
                if let Some(annotated_type) = var_type {
                    if !self.types_compatible(&inferred_type, annotated_type) {
                        return Err(CompilerError::type_error(
                            &format!("Type mismatch: expected {}, got {}", 
                                    annotated_type.to_string(), inferred_type.to_string())
                        ));
                    }
                }
                
                self.context.declare_variable(name, inferred_type);
            }
            Statement::AssignmentStatement { name, value } => {
                let variable_type = self.context.get_variable_type(name)
                    .ok_or_else(|| CompilerError::type_error(&format!("Undefined variable: {}", name)))?;
                
                let value_type = self.infer_expression(value)?;
                
                if !self.types_compatible(&value_type, &variable_type) {
                    return Err(CompilerError::type_error(
                        &format!("Cannot assign {} to variable {} of type {}", 
                                value_type.to_string(), name, variable_type.to_string())
                    ));
                }
            }
            Statement::ReturnStatement { value } => {
                if let Some(expr) = value {
                    self.infer_expression(expr)?;
                }
            }
            Statement::FunctionStatement { name, parameters, return_type, body } => {
                // Infer parameter types
                let mut param_types = Vec::new();
                for param in parameters {
                    param_types.push(param.type_annotation.clone());
                }
                
                // Create function type
                let func_type = Type::Function(param_types, Box::new(return_type.clone().unwrap_or(Type::Void)));
                self.context.declare_function(name, func_type);
                
                // Enter function scope
                self.context.enter_scope();
                
                // Declare parameters in function scope
                for param in parameters {
                    let param_type = param.type_annotation.clone();
                    self.context.declare_variable(&param.name, param_type);
                }
                
                // Infer function body
                // Function body is a single expression, not a list of statements
                let body_type = self.infer_expression(body)?;
                
                // Check if body type matches return type
                if let Some(return_type) = return_type {
                    if !self.types_compatible(&body_type, &return_type) {
                        return Err(CompilerError::type_error(
                            &format!("Function body type {:?} does not match return type {:?}", body_type, return_type)
                        ));
                    }
                }
                
                let _ = return_type.clone().unwrap_or(body_type.clone());
                return Ok(());
            }
            Statement::ExpressionStatement { expression } => {
                self.infer_expression(expression)?;
            }
            _ => {
                // For other statement types, do basic checking
                return Ok(());
            }
        }
        Ok(())
    }

    fn infer_expression(&mut self, expression: &Expression) -> Result<Type, CompilerError> {
        match expression {
            Expression::Literal(literal) => {
                match literal {
                    Literal::Int(_) => Ok(Type::Int),
                    Literal::Float(_) => Ok(Type::Float),
                    Literal::String(_) => Ok(Type::String),
                    Literal::Bool(_) => Ok(Type::Bool),
                    Literal::Null => Ok(Type::Null),
                    Literal::Char(_) => Ok(Type::Char),
                    Literal::Array(_) => Ok(Type::Array(Box::new(Type::Unknown), 0)),
                }
            }
            Expression::Identifier(name) => {
                self.context.get_variable_type(name)
                    .ok_or_else(|| CompilerError::type_error(&format!("Undefined variable: {}", name)))
            }

            Expression::BinaryExpression { left, operator, right } => {
                // Delegate to BinaryOp handling by creating a temporary BinaryOp
                let bin_op = BinaryOp {
                    left: left.clone(),
                    operator: operator.clone(),
                    right: right.clone(),
                };
                self.infer_expression(&Expression::BinaryOp(bin_op))
            }
            Expression::UnaryExpression { operator, operand } => {
                let operand_type = self.infer_expression(operand)?;
                
                match operator {
                    UnaryOperator::Neg => {
                        if operand_type == Type::Int || operand_type == Type::Float {
                            Ok(operand_type)
                        } else {
                            Err(CompilerError::type_error("Negation requires numeric operand"))
                        }
                    }
                    UnaryOperator::Not => {
                        if operand_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(CompilerError::type_error("Logical NOT requires boolean operand"))
                        }
                    }
                    _ => Ok(Type::Unknown),
                }
            }
            Expression::CallExpression { function, arguments } => {
                let func_type = self.context.get_function_type(function)
                    .ok_or_else(|| CompilerError::type_error(&format!("Undefined function: {}", function)))?;
                
                if let Type::Function(param_types, return_type) = func_type {
                    if arguments.len() != param_types.len() {
                        return Err(CompilerError::type_error(
                            &format!("Function {} expects {} arguments, got {}", 
                                    function, param_types.len(), arguments.len())
                        ));
                    }
                    
                    // Check argument types
                    for (arg, expected_type) in arguments.iter().zip(param_types.iter()) {
                        let arg_type = self.infer_expression(arg)?;
                        if !self.types_compatible(&arg_type, expected_type) {
                            return Err(CompilerError::type_error(
                                &format!("Argument type mismatch in function {}", function)
                            ));
                        }
                    }
                    
                    Ok(*return_type)
                } else {
                    Err(CompilerError::type_error(&format!("{} is not a function", function)))
                }
            }
            Expression::IfExpression { condition, then_branch, else_branch } => {
                let condition_type = self.infer_expression(condition)?;
                if condition_type != Type::Bool {
                    return Err(CompilerError::type_error("If condition must be boolean"));
                }
                
                let then_type = self.infer_expression(then_branch)?;
                let else_type = if let Some(else_expr) = else_branch {
                    self.infer_expression(else_expr)?
                } else {
                    Type::Void
                };
                
                if !self.types_compatible(&then_type, &else_type) {
                    return Err(CompilerError::type_error("If branches must have compatible types"));
                }
                
                Ok(then_type)
            }
            Expression::BlockExpression { statements } => {
                self.context.enter_scope();
                
                for stmt in statements {
                    self.infer_statement(stmt)?;
                }
                
                self.context.exit_scope();
                
                // Block expressions return the type of the last expression
                if let Some(last_stmt) = statements.last() {
                    if let Statement::ExpressionStatement { expression } = last_stmt {
                        self.infer_expression(expression)
                    } else {
                        Ok(Type::Void)
                    }
                } else {
                    Ok(Type::Void)
                }
            }
            _ => {
                // For other expression types, return a default type
                Ok(Type::Any)
            }
        }
    }

    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            (a, b) => a == b,
        }
    }
}

impl Default for TypeInferrer {
    fn default() -> Self {
        Self::new()
    }
} 