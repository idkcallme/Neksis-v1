pub mod collections;
pub mod io;
pub mod math;
pub mod string;
pub mod time;
pub mod crypto;
pub mod networking;
pub mod threading;
pub mod error;

use std::collections::HashMap;
use crate::ast::{Expression, Literal, Type};
use crate::error::CompilerError;

pub struct StandardLibrary {
    functions: HashMap<String, BuiltinFunction>,
    types: HashMap<String, Type>,
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = Self {
            functions: HashMap::new(),
            types: HashMap::new(),
        };
        
        stdlib.register_builtins();
        stdlib
    }

    fn register_builtins(&mut self) {
        // I/O functions
        self.register_function("print", Type::Function(vec![Type::String], Box::new(Type::Void)), 
            BuiltinFunction::Print);
        self.register_function("println", Type::Function(vec![Type::String], Box::new(Type::Void)), 
            BuiltinFunction::Println);
        self.register_function("read_line", Type::Function(vec![], Box::new(Type::String)), 
            BuiltinFunction::ReadLine);
        
        // Math functions
        self.register_function("abs", Type::Function(vec![Type::Int], Box::new(Type::Int)), 
            BuiltinFunction::Abs);
        self.register_function("sqrt", Type::Function(vec![Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Sqrt);
        self.register_function("sin", Type::Function(vec![Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Sin);
        self.register_function("cos", Type::Function(vec![Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Cos);
        
        // String functions
        self.register_function("len", Type::Function(vec![Type::String], Box::new(Type::Int)), 
            BuiltinFunction::StringLen);
        self.register_function("substring", Type::Function(vec![Type::String, Type::Int, Type::Int], Box::new(Type::String)), 
            BuiltinFunction::Substring);
        self.register_function("concat", Type::Function(vec![Type::String, Type::String], Box::new(Type::String)), 
            BuiltinFunction::Concat);
        
        // Type conversion
        self.register_function("to_string", Type::Function(vec![Type::Int], Box::new(Type::String)), 
            BuiltinFunction::ToString);
        self.register_function("to_int", Type::Function(vec![Type::String], Box::new(Type::Int)), 
            BuiltinFunction::ToInt);
        self.register_function("to_float", Type::Function(vec![Type::String], Box::new(Type::Float)), 
            BuiltinFunction::ToFloat);
        
        // Memory management
        self.register_function("malloc", Type::Function(vec![Type::Int], Box::new(Type::Pointer(Box::new(Type::Void)))), 
            BuiltinFunction::Malloc);
        self.register_function("free", Type::Function(vec![Type::Pointer(Box::new(Type::Void))], Box::new(Type::Void)), 
            BuiltinFunction::Free);
        
        // Error handling
        self.register_function("panic", Type::Function(vec![Type::String], Box::new(Type::Never)), 
            BuiltinFunction::Panic);
        self.register_function("assert", Type::Function(vec![Type::Bool, Type::String], Box::new(Type::Void)), 
            BuiltinFunction::Assert);
    }

    fn register_function(&mut self, name: &str, signature: Type, implementation: BuiltinFunction) {
        self.functions.insert(name.to_string(), implementation);
        self.types.insert(name.to_string(), signature);
    }

    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn execute_builtin(&self, name: &str, args: &[Expression]) -> Result<Expression, CompilerError> {
        if let Some(builtin) = self.get_builtin(name) {
            builtin.execute(args)
        } else {
            Err(CompilerError::runtime_error(&format!("Unknown builtin function: {}", name)))
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuiltinFunction {
    // I/O
    Print,
    Println,
    ReadLine,
    
    // Math
    Abs,
    Sqrt,
    Sin,
    Cos,
    
    // String
    StringLen,
    Substring,
    Concat,
    
    // Type conversion
    ToString,
    ToInt,
    ToFloat,
    
    // Memory management
    Malloc,
    Free,
    
    // Error handling
    Panic,
    Assert,
}

impl BuiltinFunction {
    pub fn execute(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        match self {
            BuiltinFunction::Print => self.execute_print(args),
            BuiltinFunction::Println => self.execute_println(args),
            BuiltinFunction::ReadLine => self.execute_read_line(args),
            BuiltinFunction::Abs => self.execute_abs(args),
            BuiltinFunction::Sqrt => self.execute_sqrt(args),
            BuiltinFunction::Sin => self.execute_sin(args),
            BuiltinFunction::Cos => self.execute_cos(args),
            BuiltinFunction::StringLen => self.execute_string_len(args),
            BuiltinFunction::Substring => self.execute_substring(args),
            BuiltinFunction::Concat => self.execute_concat(args),
            BuiltinFunction::ToString => self.execute_to_string(args),
            BuiltinFunction::ToInt => self.execute_to_int(args),
            BuiltinFunction::ToFloat => self.execute_to_float(args),
            BuiltinFunction::Malloc => self.execute_malloc(args),
            BuiltinFunction::Free => self.execute_free(args),
            BuiltinFunction::Panic => self.execute_panic(args),
            BuiltinFunction::Assert => self.execute_assert(args),
        }
    }

    fn execute_print(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("print expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            print!("{}", s);
            Ok(Expression::Literal(Literal::String("".to_string())))
        } else {
            Err(CompilerError::runtime_error("print expects a string argument"))
        }
    }

    fn execute_println(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("println expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            println!("{}", s);
            Ok(Expression::Literal(Literal::String("".to_string())))
        } else {
            Err(CompilerError::runtime_error("println expects a string argument"))
        }
    }

    fn execute_read_line(&self, _args: &[Expression]) -> Result<Expression, CompilerError> {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read line: {}", e)))?;
        
        Ok(Expression::Literal(Literal::String(input.trim().to_string())))
    }

    fn execute_abs(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("abs expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::Int(n)) = &args[0] {
            Ok(Expression::Literal(Literal::Int(n.abs())))
        } else {
            Err(CompilerError::runtime_error("abs expects an integer argument"))
        }
    }

    fn execute_sqrt(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("sqrt expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::Float(n)) = &args[0] {
            Ok(Expression::Literal(Literal::Float(n.sqrt())))
        } else {
            Err(CompilerError::runtime_error("sqrt expects a float argument"))
        }
    }

    fn execute_sin(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("sin expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::Float(n)) = &args[0] {
            Ok(Expression::Literal(Literal::Float(n.sin())))
        } else {
            Err(CompilerError::runtime_error("sin expects a float argument"))
        }
    }

    fn execute_cos(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("cos expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::Float(n)) = &args[0] {
            Ok(Expression::Literal(Literal::Float(n.cos())))
        } else {
            Err(CompilerError::runtime_error("cos expects a float argument"))
        }
    }

    fn execute_string_len(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("len expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            Ok(Expression::Literal(Literal::Int(s.len() as i64)))
        } else {
            Err(CompilerError::runtime_error("len expects a string argument"))
        }
    }

    fn execute_substring(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 3 {
            return Err(CompilerError::runtime_error("substring expects exactly 3 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s)), 
                Expression::Literal(Literal::Int(start)), 
                Expression::Literal(Literal::Int(end))) = (&args[0], &args[1], &args[2]) {
            let start = *start as usize;
            let end = *end as usize;
            if start < s.len() && end <= s.len() && start < end {
                Ok(Expression::Literal(Literal::String(s[start..end].to_string())))
            } else {
                Err(CompilerError::runtime_error("Invalid substring indices"))
            }
        } else {
            Err(CompilerError::runtime_error("substring expects string, int, int arguments"))
        }
    }

    fn execute_concat(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("concat expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s1)), 
                Expression::Literal(Literal::String(s2))) = (&args[0], &args[1]) {
            Ok(Expression::Literal(Literal::String(format!("{}{}", s1, s2))))
        } else {
            Err(CompilerError::runtime_error("concat expects two string arguments"))
        }
    }

    fn execute_to_string(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_string expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Int(n)) => {
                Ok(Expression::Literal(Literal::String(n.to_string())))
            }
            Expression::Literal(Literal::Float(f)) => {
                Ok(Expression::Literal(Literal::String(f.to_string())))
            }
            Expression::Literal(Literal::Bool(b)) => {
                Ok(Expression::Literal(Literal::String(b.to_string())))
            }
            _ => Err(CompilerError::runtime_error("to_string expects a primitive type argument"))
        }
    }

    fn execute_to_int(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_int expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            match s.parse::<i64>() {
                Ok(n) => Ok(Expression::Literal(Literal::Int(n))),
                Err(_) => Err(CompilerError::runtime_error("Failed to parse string as integer"))
            }
        } else {
            Err(CompilerError::runtime_error("to_int expects a string argument"))
        }
    }

    fn execute_to_float(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_float expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            match s.parse::<f64>() {
                Ok(f) => Ok(Expression::Literal(Literal::Float(f))),
                Err(_) => Err(CompilerError::runtime_error("Failed to parse string as float"))
            }
        } else {
            Err(CompilerError::runtime_error("to_float expects a string argument"))
        }
    }

    fn execute_malloc(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("malloc expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::Int(size)) = &args[0] {
            if *size > 0 {
                // In a real implementation, this would allocate memory
                Ok(Expression::Literal(Literal::Int(0))) // Placeholder
            } else {
                Err(CompilerError::runtime_error("malloc size must be positive"))
            }
        } else {
            Err(CompilerError::runtime_error("malloc expects an integer argument"))
        }
    }

    fn execute_free(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("free expects exactly 1 argument"));
        }
        
        // In a real implementation, this would free memory
        Ok(Expression::Literal(Literal::String("".to_string())))
    }

    fn execute_panic(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("panic expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(msg)) = &args[0] {
            Err(CompilerError::runtime_error(&format!("Panic: {}", msg)))
        } else {
            Err(CompilerError::runtime_error("panic expects a string argument"))
        }
    }

    fn execute_assert(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("assert expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::Bool(condition)), 
                Expression::Literal(Literal::String(message))) = (&args[0], &args[1]) {
            if !*condition {
                Err(CompilerError::runtime_error(&format!("Assertion failed: {}", message)))
            } else {
                Ok(Expression::Literal(Literal::String("".to_string())))
            }
        } else {
            Err(CompilerError::runtime_error("assert expects bool, string arguments"))
        }
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
} 