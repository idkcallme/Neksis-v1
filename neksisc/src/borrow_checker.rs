use crate::ast::{Expression, Statement, Type};
use crate::error::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BorrowType {
    Immutable,
    Mutable,
    Owned,
    Shared,
    Weak,
}

#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub borrow_type: BorrowType,
    pub lifetime: Option<String>,
    pub is_active: bool,
    pub scope_start: usize,
    pub scope_end: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct VariableState {
    pub borrow_type: BorrowType,
    pub borrows: Vec<BorrowInfo>,
    pub is_moved: bool,
    pub is_frozen: bool,
}

pub struct BorrowChecker {
    variables: HashMap<String, VariableState>,
    current_scope: usize,
    scopes: Vec<HashMap<String, VariableState>>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            current_scope: 0,
            scopes: vec![HashMap::new()],
        }
    }

    pub fn check_program(&mut self, statements: &[Statement]) -> Result<(), CompilerError> {
        for statement in statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::LetStatement { name, value, var_type } => {
                self.check_expression(value)?;
                
                // Determine borrow type based on type annotation
                let borrow_type = if let Some(var_type) = var_type {
                    self.get_borrow_type_from_type(var_type)
                } else {
                    BorrowType::Owned
                };
                
                let variable_state = VariableState {
                    borrow_type,
                    borrows: Vec::new(),
                    is_moved: false,
                    is_frozen: false,
                };
                
                self.variables.insert(name.clone(), variable_state);
            }
            Statement::AssignmentStatement { name, value } => {
                self.check_expression(value)?;
                
                if let Some(var_state) = self.variables.get_mut(name) {
                    if var_state.is_frozen {
                        return Err(CompilerError::borrow_error(
                            &format!("Cannot assign to frozen variable: {}", name)
                        ));
                    }
                    
                    if var_state.borrow_type == BorrowType::Immutable {
                        return Err(CompilerError::borrow_error(
                            &format!("Cannot assign to immutable variable: {}", name)
                        ));
                    }
                } else {
                    return Err(CompilerError::borrow_error(
                        &format!("Undefined variable: {}", name)
                    ));
                }
            }
            Statement::FunctionStatement { name: _, parameters, return_type: _, body } => {
                // Enter function scope
                self.enter_scope();
                
                // Check parameters
                for param in parameters {
                    let borrow_type = self.get_borrow_type_from_type(&param.type_annotation);
                    
                    let variable_state = VariableState {
                        borrow_type,
                        borrows: Vec::new(),
                        is_moved: false,
                        is_frozen: false,
                    };
                    
                    self.variables.insert(param.name.clone(), variable_state);
                }
                
                // Check function body
                self.check_expression(body)?;
                
                self.exit_scope();
            }
            Statement::ReturnStatement { value } => {
                if let Some(expr) = value {
                    self.check_expression(expr)?;
                }
            }
            Statement::ExpressionStatement { expression } => {
                self.check_expression(expression)?;
            }
            _ => {
                // For other statement types, do basic checking
                return Ok(());
            }
        }
        Ok(())
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<(), CompilerError> {
        match expression {
            Expression::Literal(_) => {
                // Literals don't have borrow requirements
                Ok(())
            }
            Expression::Identifier(name) => {
                if let Some(var_state) = self.variables.get(name) {
                    if var_state.is_moved {
                        return Err(CompilerError::borrow_error(
                            &format!("Cannot use moved variable: {}", name)
                        ));
                    }
                    
                    // Check if variable is borrowed
                    if !var_state.borrows.is_empty() {
                        let active_borrows: Vec<_> = var_state.borrows.iter()
                            .filter(|b| b.is_active)
                            .collect();
                        
                        if active_borrows.len() > 1 {
                            return Err(CompilerError::borrow_error(
                                &format!("Multiple active borrows of variable: {}", name)
                            ));
                        }
                    }
                    Ok(())
                } else {
                    return Err(CompilerError::borrow_error(
                        &format!("Undefined variable: {}", name)
                    ));
                }
            }

            Expression::BinaryExpression { left, operator: _, right } => {
                self.check_expression(left)?;
                self.check_expression(right)?;
                Ok(())
            }
            Expression::UnaryExpression { operator: _, operand } => {
                self.check_expression(operand)?;
                Ok(())
            }
            Expression::CallExpression { function, arguments } => {
                // Check function arguments
                for arg in arguments {
                    self.check_expression(arg)?;
                }
                
                // Check if function exists (simplified)
                if !self.is_builtin_function(function) {
                    return Err(CompilerError::borrow_error(
                        &format!("Undefined function: {}", function)
                    ));
                }
                Ok(())
            }
            Expression::IfExpression { condition, then_branch, else_branch } => {
                self.check_expression(condition)?;
                
                // Enter then branch scope
                self.enter_scope();
                self.check_expression(then_branch)?;
                self.exit_scope();
                
                // Enter else branch scope
                if let Some(else_expr) = else_branch {
                    self.enter_scope();
                    self.check_expression(else_expr)?;
                    self.exit_scope();
                }
                Ok(())
            }
            Expression::BlockExpression { statements } => {
                self.enter_scope();
                
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                
                self.exit_scope();
                Ok(())
            }
            Expression::ReferenceExpression { target, borrow_type: _ } => {
                self.check_expression(target)?;
                
                // Check borrow rules - target is an expression, not a string
                // For now, we'll skip the detailed borrow checking for references
                // since we need to extract the variable name from the target expression
                // TODO: Implement proper variable name extraction from target expression
                Ok(())
            }
            Expression::DereferenceExpression { target } => {
                self.check_expression(target)?;
                Ok(())
            }
            _ => {
                // For other expression types, do basic checking
                Ok(())
            }
        }
    }

    fn enter_scope(&mut self) {
        self.current_scope += 1;
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            // End all borrows in this scope
            for var_state in self.variables.values_mut() {
                for borrow in &mut var_state.borrows {
                    if borrow.scope_start == self.current_scope {
                        borrow.is_active = false;
                        borrow.scope_end = Some(self.current_scope);
                    }
                }
            }
            
            self.scopes.pop();
            self.current_scope -= 1;
        }
    }

    fn get_borrow_type_from_type(&self, var_type: &Type) -> BorrowType {
        match var_type {
            Type::Reference(_, borrow_type, _) => self.convert_borrow_type(borrow_type),
            Type::Pointer(_) => BorrowType::Owned,
            Type::Shared(_) => BorrowType::Shared,
            Type::Weak(_) => BorrowType::Weak,
            _ => BorrowType::Owned,
        }
    }

    fn convert_borrow_type(&self, borrow_type: &crate::ast::BorrowType) -> BorrowType {
        match borrow_type {
            crate::ast::BorrowType::Immutable => BorrowType::Immutable,
            crate::ast::BorrowType::Mutable => BorrowType::Mutable,
            crate::ast::BorrowType::ImmutableBorrow => BorrowType::Immutable,
            crate::ast::BorrowType::MutableBorrow => BorrowType::Mutable,
            crate::ast::BorrowType::Move => BorrowType::Owned,
            crate::ast::BorrowType::Copy => BorrowType::Owned,
            crate::ast::BorrowType::Borrowed => BorrowType::Immutable,
            crate::ast::BorrowType::MutableBorrowed => BorrowType::Mutable,
        }
    }



    fn is_builtin_function(&self, name: &str) -> bool {
        matches!(name, "print" | "println" | "read_line" | "len" | "concat")
    }

    pub fn check_lifetime_validity(&self) -> Result<(), CompilerError> {
        for (var_name, var_state) in &self.variables {
            for borrow in &var_state.borrows {
                if borrow.is_active {
                    // Check if the borrow outlives the variable's scope
                    if let Some(scope_end) = borrow.scope_end {
                        if scope_end > self.current_scope {
                            return Err(CompilerError::borrow_error(
                                &format!("Borrow of {} outlives variable scope", var_name)
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_borrow_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        
        for (var_name, var_state) in &self.variables {
            let active_borrows: Vec<_> = var_state.borrows.iter()
                .filter(|b| b.is_active)
                .collect();
            
            // Check for multiple mutable borrows
            let mutable_borrows: Vec<_> = active_borrows.iter()
                .filter(|b| b.borrow_type == BorrowType::Mutable)
                .collect();
            
            if mutable_borrows.len() > 1 {
                violations.push(format!(
                    "Multiple mutable borrows of variable: {}", var_name
                ));
            }
            
            // Check for mutable and immutable borrows simultaneously
            let has_mutable = active_borrows.iter().any(|b| b.borrow_type == BorrowType::Mutable);
            let has_immutable = active_borrows.iter().any(|b| b.borrow_type == BorrowType::Immutable);
            
            if has_mutable && has_immutable {
                violations.push(format!(
                    "Simultaneous mutable and immutable borrows of variable: {}", var_name
                ));
            }
        }
        
        violations
    }

    pub fn print_borrow_summary(&self) {
        println!("=== Borrow Checker Summary ===");
        println!("Variables checked: {}", self.variables.len());
        
        let violations = self.get_borrow_violations();
        if violations.is_empty() {
            println!("✅ No borrow violations detected");
        } else {
            println!("❌ Found {} borrow violations:", violations.len());
            for violation in violations {
                println!("  - {}", violation);
            }
        }
        
        for (var_name, var_state) in &self.variables {
            println!("Variable '{}':", var_name);
            println!("  Borrow type: {:?}", var_state.borrow_type);
            println!("  Active borrows: {}", var_state.borrows.iter().filter(|b| b.is_active).count());
            println!("  Total borrows: {}", var_state.borrows.len());
        }
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
} 