// Integration module for Modern Neksis 2025
// This module provides the main interface for the modernized language features

use crate::modern_ast::*;
use crate::modern_lexer::Lexer;
use crate::modern_parser::Parser;
use crate::modern_stdlib::{NeksisError, NeksisResult};
use crate::collections::*;
use crate::networking::*;
use crate::modern_async;
use crate::error_handling::*;
use std::collections::HashMap;

/// The main Neksis 2025 interpreter
pub struct NeksisInterpreter {
    global_variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    classes: HashMap<String, Class>,
    modules: HashMap<String, Module>,
    error_handler: ErrorHandler,
}

/// Runtime value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(NeksisVec<Value>),
    HashMap(NeksisHashMap<String, Value>),
    Function(Function),
    Object(Object),
    Promise(Box<Promise>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub body: Vec<Statement>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: String,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, Function>,
    pub constructor: Option<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub functions: HashMap<String, Function>,
    pub variables: HashMap<String, Value>,
    pub classes: HashMap<String, Class>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Promise {
    pub id: u64,
    pub status: PromiseStatus,
    pub value: Option<Box<Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PromiseStatus {
    Pending,
    Resolved,
    Rejected,
}

impl NeksisInterpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            global_variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            modules: HashMap::new(),
            error_handler: ErrorHandler::new(),
        };
        
        // Initialize built-in functions
        interpreter.init_builtins();
        interpreter
    }
    
    /// Parse and execute Neksis code
    pub fn execute(&mut self, source: &str) -> Result<Value> {
        // Parse the source code
        let mut lexer = ModernLexer::new(source);
        let tokens = lexer.tokenize()?;
        
        let mut parser = ModernParser::new(tokens);
        let statements = parser.parse()?;
        
        // Execute statements
        let mut last_value = Value::Null;
        for statement in statements {
            last_value = self.execute_statement(statement)?;
        }
        
        Ok(last_value)
    }
    
    /// Execute a single statement
    pub fn execute_statement(&mut self, stmt: Statement) -> NeksisResult<Value> {
        match stmt {
            Statement::Expression(expr) => self.evaluate_expression(expr),
            Statement::Let(let_stmt) => {
                let val = self.evaluate_expression(*let_stmt.value)?;
                self.global_variables.insert(let_stmt.name, val.clone());
                Ok(val)
            }
            Statement::Function(func_stmt) => {
                let function = Function {
                    name: name.clone(),
                    parameters,
                    return_type,
                    body,
                    is_async,
                };
                self.functions.insert(name, function.clone());
                Ok(Value::Function(function))
            }
            Statement::ClassDeclaration { name, fields, methods, constructor } => {
                let class = Class {
                    name: name.clone(),
                    fields: HashMap::new(), // Initialize empty, fields are templates
                    methods,
                    constructor,
                };
                self.classes.insert(name, class.clone());
                Ok(Value::Null)
            }
            Statement::If { condition, then_branch, else_branch } => {
                let condition_value = self.evaluate_expression(condition)?;
                if self.is_truthy(&condition_value) {
                    self.execute_statement(*then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.execute_statement(*else_stmt)
                } else {
                    Ok(Value::Null)
                }
            }
            Statement::While { condition, body } => {
                let mut last_value = Value::Null;
                loop {
                    let condition_value = self.evaluate_expression(condition.clone())?;
                    if !self.is_truthy(&condition_value) {
                        break;
                    }
                    last_value = self.execute_statement(*body.clone())?;
                }
                Ok(last_value)
            }
            Statement::Return(expr) => {
                if let Some(expression) = expr {
                    self.evaluate_expression(expression)
                } else {
                    Ok(Value::Null)
                }
            }
            Statement::Block(statements) => {
                let mut last_value = Value::Null;
                for stmt in statements {
                    last_value = self.execute_statement(stmt)?;
                }
                Ok(last_value)
            }
            _ => Ok(Value::Null), // Handle other statement types
        }
    }
    
    /// Evaluate an expression
    pub fn evaluate_expression(&mut self, expr: Expression) -> Result<Value> {
        match expr {
            Expression::Literal(lit) => Ok(self.literal_to_value(lit)),
            Expression::Variable(name) => {
                self.global_variables.get(&name)
                    .cloned()
                    .ok_or_else(|| NeksisError::RuntimeError(format!("Undefined variable: {}", name)))
            }
            Expression::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(*left)?;
                let right_val = self.evaluate_expression(*right)?;
                self.apply_binary_operator(left_val, operator, right_val)
            }
            Expression::Unary { operator, operand } => {
                let operand_val = self.evaluate_expression(*operand)?;
                self.apply_unary_operator(operator, operand_val)
            }
            Expression::Call { callee, arguments } => {
                let function_name = match *callee {
                    Expression::Variable(name) => name,
                    _ => return Err(NeksisError::RuntimeError("Invalid function call".to_string())),
                };
                
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.evaluate_expression(arg)?);
                }
                
                self.call_function(&function_name, arg_values)
            }
            Expression::Array(elements) => {
                let mut array = NeksisVec::new();
                for element in elements {
                    array.push(self.evaluate_expression(element)?);
                }
                Ok(Value::Array(array))
            }
            Expression::HashMap(pairs) => {
                let mut map = NeksisHashMap::new();
                for (key_expr, value_expr) in pairs {
                    let key = match self.evaluate_expression(key_expr)? {
                        Value::String(s) => s,
                        _ => return Err(NeksisError::TypeError("HashMap key must be string".to_string())),
                    };
                    let value = self.evaluate_expression(value_expr)?;
                    map.insert(key, value);
                }
                Ok(Value::HashMap(map))
            }
            Expression::Index { object, index } => {
                let obj_val = self.evaluate_expression(*object)?;
                let index_val = self.evaluate_expression(*index)?;
                self.index_access(obj_val, index_val)
            }
            Expression::Await(expr) => {
                // Simple await implementation - would need proper async runtime
                self.evaluate_expression(*expr)
            }
            _ => Ok(Value::Null), // Handle other expression types
        }
    }
    
    /// Convert literal to value
    fn literal_to_value(&self, lit: Literal) -> Value {
        match lit {
            Literal::Null => Value::Null,
            Literal::Bool(b) => Value::Bool(b),
            Literal::Int(i) => Value::Int(i),
            Literal::Float(f) => Value::Float(f),
            Literal::String(s) => Value::String(s),
        }
    }
    
    /// Apply binary operator
    fn apply_binary_operator(&self, left: Value, op: BinaryOperator, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                match op {
                    BinaryOperator::Add => Ok(Value::Int(a + b)),
                    BinaryOperator::Subtract => Ok(Value::Int(a - b)),
                    BinaryOperator::Multiply => Ok(Value::Int(a * b)),
                    BinaryOperator::Divide => {
                        if b == 0 {
                            Err(NeksisError::RuntimeError("Division by zero".to_string()))
                        } else {
                            Ok(Value::Int(a / b))
                        }
                    }
                    BinaryOperator::Modulo => Ok(Value::Int(a % b)),
                    BinaryOperator::Equal => Ok(Value::Bool(a == b)),
                    BinaryOperator::NotEqual => Ok(Value::Bool(a != b)),
                    BinaryOperator::Less => Ok(Value::Bool(a < b)),
                    BinaryOperator::LessEqual => Ok(Value::Bool(a <= b)),
                    BinaryOperator::Greater => Ok(Value::Bool(a > b)),
                    BinaryOperator::GreaterEqual => Ok(Value::Bool(a >= b)),
                    _ => Err(NeksisError::TypeError("Invalid operator for integers".to_string())),
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                match op {
                    BinaryOperator::Add => Ok(Value::Float(a + b)),
                    BinaryOperator::Subtract => Ok(Value::Float(a - b)),
                    BinaryOperator::Multiply => Ok(Value::Float(a * b)),
                    BinaryOperator::Divide => Ok(Value::Float(a / b)),
                    BinaryOperator::Equal => Ok(Value::Bool((a - b).abs() < f64::EPSILON)),
                    BinaryOperator::NotEqual => Ok(Value::Bool((a - b).abs() >= f64::EPSILON)),
                    BinaryOperator::Less => Ok(Value::Bool(a < b)),
                    BinaryOperator::LessEqual => Ok(Value::Bool(a <= b)),
                    BinaryOperator::Greater => Ok(Value::Bool(a > b)),
                    BinaryOperator::GreaterEqual => Ok(Value::Bool(a >= b)),
                    _ => Err(NeksisError::TypeError("Invalid operator for floats".to_string())),
                }
            }
            (Value::String(a), Value::String(b)) => {
                match op {
                    BinaryOperator::Add => Ok(Value::String(a + &b)),
                    BinaryOperator::Equal => Ok(Value::Bool(a == b)),
                    BinaryOperator::NotEqual => Ok(Value::Bool(a != b)),
                    _ => Err(NeksisError::TypeError("Invalid operator for strings".to_string())),
                }
            }
            (Value::Bool(a), Value::Bool(b)) => {
                match op {
                    BinaryOperator::And => Ok(Value::Bool(a && b)),
                    BinaryOperator::Or => Ok(Value::Bool(a || b)),
                    BinaryOperator::Equal => Ok(Value::Bool(a == b)),
                    BinaryOperator::NotEqual => Ok(Value::Bool(a != b)),
                    _ => Err(NeksisError::TypeError("Invalid operator for booleans".to_string())),
                }
            }
            _ => Err(NeksisError::TypeError("Type mismatch in binary operation".to_string())),
        }
    }
    
    /// Apply unary operator
    fn apply_unary_operator(&self, op: UnaryOperator, operand: Value) -> Result<Value> {
        match (op, operand) {
            (UnaryOperator::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (UnaryOperator::Negate, Value::Int(i)) => Ok(Value::Int(-i)),
            (UnaryOperator::Negate, Value::Float(f)) => Ok(Value::Float(-f)),
            _ => Err(NeksisError::TypeError("Invalid unary operation".to_string())),
        }
    }
    
    /// Check if value is truthy
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    }
    
    /// Index access for arrays and hashmaps
    fn index_access(&self, object: Value, index: Value) -> Result<Value> {
        match (object, index) {
            (Value::Array(arr), Value::Int(i)) => {
                if i < 0 {
                    return Err(NeksisError::IndexOutOfBounds("Negative index".to_string()));
                }
                arr.get(i as usize).cloned()
                    .map_err(|e| e)
            }
            (Value::HashMap(map), Value::String(key)) => {
                Ok(map.get(&key).cloned().unwrap_or(Value::Null))
            }
            _ => Err(NeksisError::TypeError("Invalid index operation".to_string())),
        }
    }
    
    /// Call a function
    fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Check built-in functions first
        match name {
            "print" => {
                for arg in args {
                    print!("{} ", self.value_to_string(&arg));
                }
                println!();
                Ok(Value::Null)
            }
            "println" => {
                for arg in args {
                    print!("{} ", self.value_to_string(&arg));
                }
                println!();
                Ok(Value::Null)
            }
            "len" => {
                if args.len() != 1 {
                    return Err(NeksisError::RuntimeError("len() takes exactly 1 argument".to_string()));
                }
                match &args[0] {
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
                    Value::HashMap(map) => Ok(Value::Int(map.len() as i64)),
                    _ => Err(NeksisError::TypeError("len() argument must be string, array, or hashmap".to_string())),
                }
            }
            "type_of" => {
                if args.len() != 1 {
                    return Err(NeksisError::RuntimeError("type_of() takes exactly 1 argument".to_string()));
                }
                let type_name = match &args[0] {
                    Value::Null => "null",
                    Value::Bool(_) => "bool",
                    Value::Int(_) => "int",
                    Value::Float(_) => "float",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::HashMap(_) => "hashmap",
                    Value::Function(_) => "function",
                    Value::Object(_) => "object",
                    Value::Promise(_) => "promise",
                };
                Ok(Value::String(type_name.to_string()))
            }
            _ => {
                // Check user-defined functions
                if let Some(function) = self.functions.get(name).cloned() {
                    // Execute function body (simplified)
                    let mut last_value = Value::Null;
                    for stmt in function.body {
                        last_value = self.execute_statement(stmt)?;
                    }
                    Ok(last_value)
                } else {
                    Err(NeksisError::RuntimeError(format!("Undefined function: {}", name)))
                }
            }
        }
    }
    
    /// Convert value to string representation
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| self.value_to_string(v))
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            Value::HashMap(map) => {
                let pairs: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Function(f) => format!("function {}", f.name),
            Value::Object(obj) => format!("object {}", obj.class_name),
            Value::Promise(p) => format!("promise {:?}", p.status),
        }
    }
    
    /// Initialize built-in functions and variables
    fn init_builtins(&mut self) {
        // Add built-in constants
        self.global_variables.insert("true".to_string(), Value::Bool(true));
        self.global_variables.insert("false".to_string(), Value::Bool(false));
        self.global_variables.insert("null".to_string(), Value::Null);
        
        // Add built-in functions will be handled in call_function
    }
    
    /// Load and execute a module
    pub fn load_module(&mut self, module_name: &str, source: &str) -> Result<()> {
        // Parse module
        let mut lexer = ModernLexer::new(source);
        let tokens = lexer.tokenize()?;
        
        let mut parser = ModernParser::new(tokens);
        let statements = parser.parse()?;
        
        // Create module
        let mut module = Module {
            name: module_name.to_string(),
            functions: HashMap::new(),
            variables: HashMap::new(),
            classes: HashMap::new(),
        };
        
        // Process module statements
        for statement in statements {
            match statement {
                Statement::FunctionDeclaration { name, parameters, return_type, body, is_async } => {
                    let function = Function {
                        name: name.clone(),
                        parameters,
                        return_type,
                        body,
                        is_async,
                    };
                    module.functions.insert(name, function);
                }
                Statement::VarDeclaration { name, value, .. } => {
                    let val = if let Some(expr) = value {
                        self.evaluate_expression(expr)?
                    } else {
                        Value::Null
                    };
                    module.variables.insert(name, val);
                }
                Statement::ClassDeclaration { name, fields, methods, constructor } => {
                    let class = Class {
                        name: name.clone(),
                        fields: HashMap::new(),
                        methods,
                        constructor,
                    };
                    module.classes.insert(name, class);
                }
                _ => {
                    // Execute other statements in module context
                    self.execute_statement(statement)?;
                }
            }
        }
        
        self.modules.insert(module_name.to_string(), module);
        Ok(())
    }
}

/// Convenience function to create and run a Neksis interpreter
pub fn run_neksis_code(source: &str) -> Result<Value> {
    let mut interpreter = NeksisInterpreter::new();
    interpreter.execute(source)
}

/// Run Neksis code with error handling
pub fn run_neksis_code_safe(source: &str) -> Result<Value> {
    match run_neksis_code(source) {
        Ok(value) => Ok(value),
        Err(error) => {
            eprintln!("Neksis Error: {:?}", error);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_arithmetic() {
        let result = run_neksis_code("2 + 3 * 4").unwrap();
        assert_eq!(result, Value::Int(14));
    }
    
    #[test]
    fn test_variables() {
        let result = run_neksis_code("let x = 5; x + 10").unwrap();
        assert_eq!(result, Value::Int(15));
    }
    
    #[test]
    fn test_arrays() {
        let result = run_neksis_code("let arr = [1, 2, 3]; arr[1]").unwrap();
        assert_eq!(result, Value::Int(2));
    }
    
    #[test]
    fn test_functions() {
        let code = r#"
            function add(a, b) {
                return a + b;
            }
            add(5, 3)
        "#;
        let result = run_neksis_code(code).unwrap();
        assert_eq!(result, Value::Int(8));
    }
    
    #[test]
    fn test_string_operations() {
        let result = run_neksis_code(r#""Hello, " + "World!""#).unwrap();
        assert_eq!(result, Value::String("Hello, World!".to_string()));
    }
    
    #[test]
    fn test_builtin_functions() {
        let code = r#"
            let arr = [1, 2, 3, 4, 5];
            len(arr)
        "#;
        let result = run_neksis_code(code).unwrap();
        assert_eq!(result, Value::Int(5));
    }
}
