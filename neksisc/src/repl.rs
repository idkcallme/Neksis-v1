use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use crate::ast::*;
use crate::error::CompilerError;
use crate::lexer::{Lexer, Token, TokenInfo};
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::codegen::simple::{SimpleCodeGen, CodeGenerator};
use crate::compiler::CompilerOptions;
use std::fs;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct REPL {
    pub variables: HashMap<String, REPLValue>,
    pub functions: HashMap<String, FunctionStatement>,
    pub history: Vec<String>,
    pub session_id: String,
    pub config: REPLConfig,
    pub debug_mode: bool,
    pub auto_complete: bool,
    pub syntax_highlighting: bool,
    file_watchers: Arc<Mutex<HashMap<String, std::time::SystemTime>>>,
    hot_reload_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct REPLConfig {
    pub prompt: String,
    pub history_file: Option<PathBuf>,
    pub max_history: usize,
    pub auto_save_history: bool,
    pub show_types: bool,
    pub show_memory: bool,
    pub color_output: bool,
}

impl Default for REPLConfig {
    fn default() -> Self {
        Self {
            prompt: "neksis> ".to_string(),
            history_file: Some(PathBuf::from(".neksis_history")),
            max_history: 1000,
            auto_save_history: true,
            show_types: true,
            show_memory: false,
            color_output: true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum REPLValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<REPLValue>),
    Function(FunctionStatement),
    Struct(StructValue),
    Enum(Box<EnumValue>),
    Void,
    Error(String),
}

impl Clone for REPLValue {
    fn clone(&self) -> Self {
        match self {
            REPLValue::Int(i) => REPLValue::Int(*i),
            REPLValue::Float(f) => REPLValue::Float(*f),
            REPLValue::Bool(b) => REPLValue::Bool(*b),
            REPLValue::String(s) => REPLValue::String(s.clone()),
            REPLValue::Array(arr) => REPLValue::Array(arr.clone()),
            REPLValue::Function(f) => REPLValue::Function(f.clone()),
            REPLValue::Struct(s) => REPLValue::Struct(s.clone()),
            REPLValue::Enum(e) => REPLValue::Enum(e.clone()),
            REPLValue::Void => REPLValue::Void,
            REPLValue::Error(e) => REPLValue::Error(e.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructValue {
    pub name: String,
    pub fields: HashMap<String, REPLValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: String,
    pub variant: String,
    pub data: Option<Box<REPLValue>>,
}

impl REPL {
    pub fn new() -> Self {
        let session_id = format!("repl_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());

        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            history: Vec::new(),
            session_id,
            config: REPLConfig::default(),
            debug_mode: false,
            auto_complete: true,
            syntax_highlighting: true,
            file_watchers: Arc::new(Mutex::new(HashMap::new())),
            hot_reload_enabled: true,
        }
    }

    pub fn with_config(mut self, config: REPLConfig) -> Self {
        self.config = config;
        self
    }

    pub fn run(&mut self) -> Result<(), CompilerError> {
        self.load_history()?;
        
        println!("{}", self.colorize("Welcome to Neksis REPL v0.1.0", Color::Green));
        println!("{}", self.colorize("Type 'help' for commands, 'quit' to exit", Color::Cyan));
        println!("{}", self.colorize(&format!("Session ID: {}", self.session_id), Color::Yellow));
        println!("{}", self.colorize(&format!("Debug mode: {}", self.debug_mode), Color::Magenta));
        println!();

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut buffer = String::new();

        loop {
            print!("{}", self.config.prompt);
            stdout.flush().map_err(|e| CompilerError::runtime_error(&format!("IO error: {}", e)))?;

            buffer.clear();
            stdin.read_line(&mut buffer)
                .map_err(|e| CompilerError::runtime_error(&format!("IO error: {}", e)))?;

            let input = buffer.trim();
            if input.is_empty() {
                continue;
            }

            // Add to history
            self.history.push(input.to_string());
            if self.history.len() > self.config.max_history {
                self.history.remove(0);
            }
            
            // Auto-save history if enabled
            if self.config.auto_save_history {
                let _ = self.save_history();
            }

            // Handle special commands
            if input.starts_with(':') {
                self.handle_command(&input[1..])?;
                continue;
            }

            // Evaluate expression
            match self.evaluate(input) {
                Ok(result) => {
                    if result != REPLValue::Void {
                        let output = self.format_value(&result);
                        println!("{}", output);
                    }
                }
                Err(error) => {
                    eprintln!("{}", self.colorize(&format!("Error: {}", error), Color::Red));
                }
            }
        }
    }

    fn handle_command(&mut self, command: &str) -> Result<(), CompilerError> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&":help") => self.show_help(),
            Some(&":history") => self.show_history(),
            Some(&":vars") => self.show_variables(),
            Some(&":funcs") => self.show_functions(),
            Some(&":types") => self.show_types(),
            Some(&":memory") => self.show_memory_usage(),
            Some(&":config") => self.show_config(),
            Some(&":info") => self.show_info(),
            Some(&":debug") => {
                self.debug_mode = !self.debug_mode;
                println!("Debug mode: {}", if self.debug_mode { "ON" } else { "OFF" });
                Ok(())
            }
            Some(&":reload") => {
                if let Some(file) = parts.get(1) {
                    self.hot_reload_file(file)
                } else {
                    println!("Usage: :reload <filename>");
                    Ok(())
                }
            }
            Some(&":watch") => {
                if let Some(file) = parts.get(1) {
                    self.watch_file(file)
                } else {
                    println!("Usage: :watch <filename>");
                    Ok(())
                }
            }
            Some(&":unwatch") => {
                if let Some(file) = parts.get(1) {
                    self.unwatch_file(file)
                } else {
                    println!("Usage: :unwatch <filename>");
                    Ok(())
                }
            }
            Some(&":list") => {
                self.list_watched_files()
            }
            Some(&":load") => {
                if let Some(file) = parts.get(1) {
                    self.load_file(file)
                } else {
                    println!("Usage: :load <filename>");
                    Ok(())
                }
            }
            Some(&":save") => {
                if let Some(file) = parts.get(1) {
                    self.save_session(file)
                } else {
                    println!("Usage: :save <filename>");
                    Ok(())
                }
            }
            Some(&":hot") => {
                self.hot_reload_enabled = !self.hot_reload_enabled;
                println!("Hot reloading: {}", if self.hot_reload_enabled { "ON" } else { "OFF" });
                Ok(())
            }
            _ => {
                println!("Unknown command. Type :help for available commands.");
                Ok(())
            }
        }
    }

    fn show_help(&self) -> Result<(), CompilerError> {
        println!("Neksis REPL Commands:");
        println!("  :help          - Show this help");
        println!("  :history       - Show command history");
        println!("  :vars          - Show variables");
        println!("  :funcs         - Show functions");
        println!("  :types         - Show type information");
        println!("  :memory        - Show memory usage");
        println!("  :config        - Show configuration");
        println!("  :info          - Show system information");
        println!("  :debug         - Toggle debug mode");
        println!("  :hot           - Toggle hot reloading");
        println!("  :reload <file> - Hot reload a file");
        println!("  :load <file>   - Load and execute a file");
        println!("  :save <file>   - Save current session to file");
        println!("  :watch <file>  - Start watching a file for changes");
        println!("  :unwatch <file> - Stop watching a file");
        println!("  :list          - List watched files");
        println!("  :quit          - Exit REPL");
        println!();
        println!("Hot Reloading Features:");
        println!("  - Automatic file change detection");
        println!("  - Background compilation");
        println!("  - Real-time error reporting");
        println!("  - Session persistence");
        Ok(())
    }

    fn evaluate(&mut self, input: &str) -> Result<REPLValue, CompilerError> {
        if self.debug_mode {
            println!("{}", self.colorize(&format!("Evaluating: {}", input), Color::Blue));
        }

        // Tokenize
        let tokens = self.tokenize(input)?;
        if self.debug_mode {
            println!("{}", self.colorize(&format!("Tokens: {:?}", tokens), Color::Blue));
        }

        // Parse
        let program = self.parse(&tokens)?;
        if self.debug_mode {
            println!("{}", self.colorize(&format!("AST: {:?}", program), Color::Blue));
        }

        // Execute
        self.execute(&program)
    }

    fn tokenize(&self, input: &str) -> Result<Vec<Token>, CompilerError> {
        let mut lexer = Lexer::new(input, "repl".to_string());
        let token_infos = lexer.tokenize().map_err(|e| CompilerError::syntax_error(&e))?;
        Ok(token_infos.into_iter().map(|ti| ti.token).collect())
    }
    
    fn parse(&self, tokens: &[Token]) -> Result<Program, CompilerError> {
        let token_infos: Vec<TokenInfo> = tokens.iter().enumerate().map(|(i, token)| {
            TokenInfo {
                token: token.clone(),
                line: 1, // TODO: Get actual line numbers
                column: i + 1,
                lexeme: format!("{:?}", token),
            }
        }).collect();
        
        let mut parser = Parser::new(token_infos);
        parser.parse().map_err(|e| CompilerError::syntax_error(&e))
    }

    fn execute(&mut self, program: &Program) -> Result<REPLValue, CompilerError> {
        let mut result = REPLValue::Void;

        for statement in &program.statements {
            match statement {
                Statement::Function(func_stmt) => {
                    self.functions.insert(func_stmt.name.clone(), func_stmt.clone());
                    result = REPLValue::Function(func_stmt.clone());
                }
                Statement::Let(let_stmt) => {
                    let value = self.evaluate_expression(&let_stmt.value)?;
                    self.variables.insert(let_stmt.name.clone(), value.clone());
                    result = value;
                }
                Statement::Expression(expr) => {
                    result = self.evaluate_expression(expr)?;
                }
                _ => {}
            }
        }

        Ok(result)
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<REPLValue, CompilerError> {
        match expr {
            Expression::Literal(literal) => self.evaluate_literal(literal),
            Expression::Identifier(name) => self.lookup_variable(name),
            Expression::BinaryOp(binary_op) => {
                let _left_val = self.evaluate_expression(&binary_op.left)?;
                let _right_val = self.evaluate_expression(&binary_op.right)?;
                self.evaluate_binary_op(&binary_op.left, &binary_op.operator, &binary_op.right)
            }
            Expression::FunctionCall(func_name, arguments) => {
                let args: Vec<Box<Expression>> = arguments.iter().map(|arg| Box::new(arg.value.clone())).collect();
                self.evaluate_function_call(&format!("{:?}", func_name), &args)
            }
            Expression::If(if_expr) => {
                let cond_val = self.evaluate_expression(&if_expr.condition)?;
                match cond_val {
                    REPLValue::Bool(true) => self.evaluate_expression(&if_expr.then_branch),
                    REPLValue::Bool(false) => {
                        if let Some(else_expr) = &if_expr.else_branch {
                            self.evaluate_expression(else_expr)
                        } else {
                            Ok(REPLValue::Void)
                        }
                    }
                    _ => Err(CompilerError::type_error("Condition must be boolean"))
                }
            }
            Expression::While(while_expr) => {
                let mut result = REPLValue::Void;
                while let REPLValue::Bool(true) = self.evaluate_expression(&while_expr.condition)? {
                    result = self.evaluate_expression(&while_expr.body)?;
                }
                Ok(result)
            }
            Expression::Block(statements) => {
                let mut result = REPLValue::Void;
                for stmt in statements {
                    match stmt {
                        Statement::Expression(expr) => {
                            result = self.evaluate_expression(expr)?;
                        }
                        Statement::Let(let_stmt) => {
                            let value = self.evaluate_expression(&let_stmt.value)?;
                            self.variables.insert(let_stmt.name.clone(), value);
                        }
                        _ => {}
                    }
                }
                Ok(result)
            }
            _ => {
                if self.debug_mode {
                    println!("{}", self.colorize(&format!("Unsupported expression: {:?}", expr), Color::Yellow));
                }
                Ok(REPLValue::Void)
            }
        }
    }

    fn evaluate_literal(&self, literal: &Literal) -> Result<REPLValue, CompilerError> {
        match literal {
            Literal::Int(value) => Ok(REPLValue::Int(*value)),
            Literal::Float(value) => Ok(REPLValue::Float(*value)),
            Literal::Bool(value) => Ok(REPLValue::Bool(*value)),
            Literal::String(value) => Ok(REPLValue::String(value.clone())),
            Literal::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_literal(element)?);
                }
                Ok(REPLValue::Array(values))
            },
            Literal::Char(c) => Ok(REPLValue::String(c.to_string())),
            Literal::Null => Ok(REPLValue::Void),
        }
    }

    fn lookup_variable(&self, name: &str) -> Result<REPLValue, CompilerError> {
        self.variables.get(name)
            .cloned()
            .ok_or_else(|| CompilerError::semantic_error(&format!("Undefined variable: {}", name)))
    }

    fn evaluate_binary_op(&mut self, left: &Expression, operator: &BinaryOperator, right: &Expression) -> Result<REPLValue, CompilerError> {
        let left_val = self.evaluate_expression(left)?;
        let right_val = self.evaluate_expression(right)?;
        
        match (left_val.clone(), operator, right_val.clone()) {
            (REPLValue::Int(a), BinaryOperator::Add, REPLValue::Int(b)) => Ok(REPLValue::Int(a + b)),
            (REPLValue::Int(a), BinaryOperator::Subtract, REPLValue::Int(b)) => Ok(REPLValue::Int(a - b)),
            (REPLValue::Int(a), BinaryOperator::Multiply, REPLValue::Int(b)) => Ok(REPLValue::Int(a * b)),
            (REPLValue::Int(a), BinaryOperator::Divide, REPLValue::Int(b)) => {
                if b == 0 {
                    Err(CompilerError::runtime_error("Division by zero"))
                } else {
                    Ok(REPLValue::Int(a / b))
                }
            }
            (REPLValue::Float(a), BinaryOperator::Add, REPLValue::Float(b)) => Ok(REPLValue::Float(a + b)),
            (REPLValue::Float(a), BinaryOperator::Subtract, REPLValue::Float(b)) => Ok(REPLValue::Float(a - b)),
            (REPLValue::Float(a), BinaryOperator::Multiply, REPLValue::Float(b)) => Ok(REPLValue::Float(a * b)),
            (REPLValue::Float(a), BinaryOperator::Divide, REPLValue::Float(b)) => {
                if b == 0.0 {
                    Err(CompilerError::runtime_error("Division by zero"))
                } else {
                    Ok(REPLValue::Float(a / b))
                }
            }
            (REPLValue::Bool(a), BinaryOperator::Equal, REPLValue::Bool(b)) => Ok(REPLValue::Bool(a == b)),
            (REPLValue::Bool(a), BinaryOperator::NotEqual, REPLValue::Bool(b)) => Ok(REPLValue::Bool(a != b)),
            (REPLValue::Int(a), BinaryOperator::Equal, REPLValue::Int(b)) => Ok(REPLValue::Bool(a == b)),
            (REPLValue::Int(a), BinaryOperator::NotEqual, REPLValue::Int(b)) => Ok(REPLValue::Bool(a != b)),
            (REPLValue::Int(a), BinaryOperator::LessThan, REPLValue::Int(b)) => Ok(REPLValue::Bool(a < b)),
            (REPLValue::Int(a), BinaryOperator::LessThanOrEqual, REPLValue::Int(b)) => Ok(REPLValue::Bool(a <= b)),
            (REPLValue::Int(a), BinaryOperator::GreaterThan, REPLValue::Int(b)) => Ok(REPLValue::Bool(a > b)),
            (REPLValue::Int(a), BinaryOperator::GreaterThanOrEqual, REPLValue::Int(b)) => Ok(REPLValue::Bool(a >= b)),
            _ => Err(CompilerError::type_error(&format!("Invalid binary operation: {:?} {:?} {:?}", left_val, operator, right_val))),
        }
    }

    fn evaluate_function_call(&mut self, name: &str, arguments: &[Box<Expression>]) -> Result<REPLValue, CompilerError> {
        // Check for built-in functions
        if let Some(result) = self.call_builtin_function(name, arguments)? {
            return Ok(result);
        }

        // Check for user-defined functions
        if let Some(_func) = self.functions.get(name) {
            // TODO: Implement function calling with arguments
            return Ok(REPLValue::Void);
        }

        Err(CompilerError::semantic_error(&format!("Undefined function: {}", name)))
    }

    fn call_builtin_function(&mut self, name: &str, args: &[Box<Expression>]) -> Result<Option<REPLValue>, CompilerError> {
        let evaluated_args: Result<Vec<REPLValue>, CompilerError> = args.iter()
            .map(|arg| self.evaluate_expression(arg))
            .collect();
        
        let evaluated_args = evaluated_args?;
        
        match name {
            "print" => {
                for arg in evaluated_args {
                    print!("{:?}", arg);
                }
                Ok(Some(REPLValue::Void))
            }
            "println" => {
                for arg in evaluated_args {
                    print!("{:?}", arg);
                }
                println!();
                Ok(Some(REPLValue::Void))
            }
            "len" => {
                if evaluated_args.len() != 1 {
                    return Err(CompilerError::runtime_error("len() takes exactly one argument"));
                }
                match &evaluated_args[0] {
                    REPLValue::String(s) => Ok(Some(REPLValue::Int(s.len() as i64))),
                    REPLValue::Array(arr) => Ok(Some(REPLValue::Int(arr.len() as i64))),
                    _ => Err(CompilerError::type_error("len() can only be called on strings or arrays")),
                }
            }
            _ => Ok(None),
        }
    }

    fn format_value(&self, value: &REPLValue) -> String {
        let formatted = match value {
            REPLValue::Int(i) => format!("{}", i),
            REPLValue::Float(f) => format!("{}", f),
            REPLValue::Bool(b) => format!("{}", b),
            REPLValue::String(s) => format!("\"{}\"", s),
            REPLValue::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| self.format_value(v)).collect();
                format!("[{}]", elements.join(", "))
            }
            REPLValue::Function(func) => format!("<function {}>", func.name),
            REPLValue::Struct(s) => {
                let fields: Vec<String> = s.fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_value(v)))
                    .collect();
                format!("{} {{ {} }}", s.name, fields.join(", "))
            }
            REPLValue::Enum(e) => {
                if let Some(data) = &e.data {
                    format!("{}.{}({})", e.name, e.variant, self.format_value(data))
                } else {
                    format!("{}.{}", e.name, e.variant)
                }
            }
            REPLValue::Void => "()".to_string(),
            REPLValue::Error(msg) => format!("<error: {}>", msg),
        };

        if self.config.show_types {
            let type_name = match value {
                REPLValue::Int(_) => "i64",
                REPLValue::Float(_) => "f64",
                REPLValue::Bool(_) => "bool",
                REPLValue::String(_) => "String",
                REPLValue::Array(_) => "Array",
                REPLValue::Function(_) => "Function",
                REPLValue::Struct(_) => "Struct",
                REPLValue::Enum(_) => "Enum",
                REPLValue::Void => "void",
                REPLValue::Error(_) => "error",
            };
            format!("{}: {}", formatted, self.colorize(type_name, Color::Blue))
        } else {
            formatted
        }
    }

    fn show_history(&self) -> Result<(), CompilerError> {
        println!("{}", self.colorize("Command History:", Color::Cyan));
        for (i, command) in self.history.iter().enumerate() {
            println!("{:3}: {}", i + 1, command);
        }
        Ok(())
    }

    fn show_variables(&self) -> Result<(), CompilerError> {
        println!("{}", self.colorize("Variables:", Color::Cyan));
        for (name, value) in &self.variables {
            println!("  {} = {}", name, self.format_value(value));
        }
        Ok(())
    }

    fn show_functions(&self) -> Result<(), CompilerError> {
        println!("{}", self.colorize("Functions:", Color::Cyan));
        for (name, func) in &self.functions {
            println!("  fn {}({}) -> {:?}", 
                name, 
                func.parameters.iter().map(|p| format!("{}: {:?}", p.name, p.type_annotation)).collect::<Vec<_>>().join(", "),
                func.return_type
            );
        }
        Ok(())
    }

    fn show_types(&self) -> Result<(), CompilerError> {
        println!("{}", self.colorize("Type Information:", Color::Cyan));
        for (name, value) in &self.variables {
            let type_name = match value {
                REPLValue::Int(_) => "i64",
                REPLValue::Float(_) => "f64",
                REPLValue::Bool(_) => "bool",
                REPLValue::String(_) => "String",
                REPLValue::Array(_) => "Array",
                REPLValue::Function(_) => "Function",
                REPLValue::Struct(_) => "Struct",
                REPLValue::Enum(_) => "Enum",
                REPLValue::Void => "void",
                REPLValue::Error(_) => "error",
            };
            println!("  {}: {}", name, type_name);
        }
        Ok(())
    }

    fn show_memory_usage(&self) -> Result<(), CompilerError> {
        println!("{}", self.colorize("Memory Usage:", Color::Cyan));
        println!("  Variables: {}", self.variables.len());
        println!("  Functions: {}", self.functions.len());
        println!("  History entries: {}", self.history.len());
        Ok(())
    }

    fn show_config(&self) -> Result<(), CompilerError> {
        println!("{}", self.colorize("Configuration:", Color::Cyan));
        println!("  Prompt: {}", self.config.prompt);
        println!("  History file: {:?}", self.config.history_file);
        println!("  Max history: {}", self.config.max_history);
        println!("  Auto-save history: {}", self.config.auto_save_history);
        println!("  Show types: {}", self.config.show_types);
        println!("  Show memory: {}", self.config.show_memory);
        println!("  Color output: {}", self.config.color_output);
        println!("  Debug mode: {}", self.debug_mode);
        println!("  Auto-complete: {}", self.auto_complete);
        println!("  Syntax highlighting: {}", self.syntax_highlighting);
        Ok(())
    }

    fn show_info(&self) -> Result<(), CompilerError> {
        println!("{}", self.colorize("Session Information:", Color::Cyan));
        println!("  Session ID: {}", self.session_id);
        println!("  Variables: {}", self.variables.len());
        println!("  Functions: {}", self.functions.len());
        println!("  History entries: {}", self.history.len());
        println!("  Debug mode: {}", self.debug_mode);
        Ok(())
    }

    fn load_file(&mut self, filename: &str) -> Result<(), CompilerError> {
        let content = std::fs::read_to_string(filename)
            .map_err(|e| CompilerError::io_error(&format!("Failed to read file: {}", e)))?;
        
        println!("{}", self.colorize(&format!("Loading file: {}", filename), Color::Green));
        
        match self.evaluate(&content) {
            Ok(_) => println!("{}", self.colorize("File loaded successfully", Color::Green)),
            Err(e) => eprintln!("{}", self.colorize(&format!("Error loading file: {}", e), Color::Red)),
        }
        
        Ok(())
    }

    fn save_session(&self, filename: &str) -> Result<(), CompilerError> {
        let mut session_data = String::new();
        
        // Save variables
        session_data.push_str("// Variables\n");
        for (name, value) in &self.variables {
            session_data.push_str(&format!("let {} = {};\n", name, self.format_value(value)));
        }
        
        // Save functions
        session_data.push_str("\n// Functions\n");
        for (_, func) in &self.functions {
            session_data.push_str(&format!("fn {}({}) -> {:?} {{\n", 
                func.name,
                func.parameters.iter().map(|p| format!("{}: {:?}", p.name, p.type_annotation)).collect::<Vec<_>>().join(", "),
                func.return_type
            ));
            // TODO: Add function body
            session_data.push_str("}\n\n");
        }
        
        std::fs::write(filename, session_data)
            .map_err(|e| CompilerError::io_error(&format!("Failed to write file: {}", e)))?;
        
        println!("{}", self.colorize(&format!("Session saved to: {}", filename), Color::Green));
        Ok(())
    }

    fn load_history(&mut self) -> Result<(), CompilerError> {
        if let Some(ref history_file) = self.config.history_file {
            if let Ok(content) = std::fs::read_to_string(history_file) {
                for line in content.lines() {
                    if !line.trim().is_empty() {
                        self.history.push(line.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    fn save_history(&self) -> Result<(), CompilerError> {
        if self.config.auto_save_history {
            if let Some(ref history_file) = self.config.history_file {
                let content = self.history.join("\n");
                std::fs::write(history_file, content)
                    .map_err(|e| CompilerError::io_error(&format!("Failed to save history: {}", e)))?;
            }
        }
        Ok(())
    }

    fn colorize(&self, text: &str, color: Color) -> String {
        if self.config.color_output {
            match color {
                Color::Red => format!("\x1b[31m{}\x1b[0m", text),
                Color::Green => format!("\x1b[32m{}\x1b[0m", text),
                Color::Yellow => format!("\x1b[33m{}\x1b[0m", text),
                Color::Blue => format!("\x1b[34m{}\x1b[0m", text),
                Color::Magenta => format!("\x1b[35m{}\x1b[0m", text),
                Color::Cyan => format!("\x1b[36m{}\x1b[0m", text),
            }
        } else {
            text.to_string()
        }
    }

    fn watch_file(&mut self, filename: &str) -> Result<(), CompilerError> {
        if let Ok(metadata) = fs::metadata(filename) {
            if let Ok(modified) = metadata.modified() {
                let mut watchers = self.file_watchers.lock().unwrap();
                watchers.insert(filename.to_string(), modified);
                println!("Now watching: {}", filename);
                
                // Start background watcher thread
                let file_watchers = Arc::clone(&self.file_watchers);
                let filename = filename.to_string();
                thread::spawn(move || {
                    Self::watch_file_background(filename, file_watchers);
                });
            }
        } else {
            println!("Error: Could not access file {}", filename);
        }
        Ok(())
    }

    fn watch_file_background(filename: String, file_watchers: Arc<Mutex<HashMap<String, std::time::SystemTime>>>) {
        loop {
            thread::sleep(Duration::from_millis(1000)); // Check every second
            
            if let Ok(metadata) = fs::metadata(&filename) {
                if let Ok(modified) = metadata.modified() {
                    let mut watchers = file_watchers.lock().unwrap();
                    if let Some(last_modified) = watchers.get(&filename) {
                        if modified > *last_modified {
                            println!("\n[Hot Reload] File {} has changed!", filename);
                            watchers.insert(filename.clone(), modified);
                        }
                    }
                }
            }
        }
    }

    fn unwatch_file(&mut self, filename: &str) -> Result<(), CompilerError> {
        let mut watchers = self.file_watchers.lock().unwrap();
        if watchers.remove(filename).is_some() {
            println!("Stopped watching: {}", filename);
        } else {
            println!("File {} was not being watched", filename);
        }
        Ok(())
    }

    fn list_watched_files(&self) -> Result<(), CompilerError> {
        let watchers = self.file_watchers.lock().unwrap();
        if watchers.is_empty() {
            println!("No files are being watched");
        } else {
            println!("Watched files:");
            for filename in watchers.keys() {
                println!("  {}", filename);
            }
        }
        Ok(())
    }

    fn hot_reload_file(&mut self, filename: &str) -> Result<(), CompilerError> {
        println!("Hot reloading: {}", filename);
        
        match self.compile_and_run_file(filename) {
            Ok(_) => println!("Successfully reloaded {}", filename),
            Err(e) => println!("Error reloading {}: {}", filename, e),
        }
        Ok(())
    }

    fn compile_and_run_file(&mut self, file_path: &str) -> Result<(), String> {
        // Read the file
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Compile the source
        let mut lexer = Lexer::new(&source, "repl_input.nx".to_string());
        let tokens = lexer.tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| format!("Parser error: {}", e))?;
        
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)
            .map_err(|e| format!("Semantic analysis error: {}", e))?;
        
        let mut codegen = SimpleCodeGen::new(CompilerOptions::default())
            .map_err(|e| format!("Code generation error: {}", e))?;
        
        let code = codegen.generate(&ast)
            .map_err(|e| format!("Code generation error: {}", e))?;
        
        // Execute the generated code
        self.execute_generated_code(&format!("{:?}", code))?;
        
        Ok(())
    }

    fn execute_generated_code(&mut self, code: &str) -> Result<(), String> {
        // In a real implementation, this would execute the generated code
        // For now, we'll just print it
        println!("Generated code: {}", code);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
}

impl REPL {
    pub fn start_repl() -> Result<(), CompilerError> {
        let mut repl = REPL::new();
        repl.run()
    }
} 