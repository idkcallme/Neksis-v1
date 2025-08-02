// Error handling module for Neksis 2025
use std::fmt;
use std::error::Error as StdError;
use crate::modern_stdlib::{NeksisError, NeksisResult};

/// Enhanced error types for Neksis
#[derive(Debug, Clone, PartialEq)]
pub enum DetailedError {
    // Syntax and parsing errors
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
        source_line: Option<String>,
    },
    ParseError {
        message: String,
        position: usize,
        expected: Vec<String>,
        found: String,
    },
    
    // Type system errors
    TypeError {
        message: String,
        expected_type: String,
        actual_type: String,
        location: Option<String>,
    },
    GenericError {
        message: String,
        generic_param: String,
        constraint: String,
    },
    
    // Runtime errors
    RuntimeError {
        message: String,
        stack_trace: Vec<String>,
        error_code: u32,
    },
    NullPointerError {
        message: String,
        variable_name: String,
        location: String,
    },
    IndexOutOfBounds {
        message: String,
        index: i64,
        length: usize,
        container_type: String,
    },
    
    // Memory and resource errors
    MemoryError {
        message: String,
        allocation_size: usize,
        available_memory: Option<usize>,
    },
    ResourceError {
        message: String,
        resource_type: String,
        resource_id: String,
    },
    
    // Concurrency errors
    DeadlockError {
        message: String,
        thread_ids: Vec<u64>,
        resources: Vec<String>,
    },
    RaceConditionError {
        message: String,
        variable_name: String,
        access_type: String,
    },
    
    // I/O and network errors
    IOError {
        message: String,
        operation: String,
        path: Option<String>,
        error_code: Option<i32>,
    },
    NetworkError {
        message: String,
        url: Option<String>,
        status_code: Option<u16>,
        network_operation: String,
    },
    
    // Security errors
    SecurityError {
        message: String,
        violation_type: String,
        attempted_action: String,
    },
    AuthenticationError {
        message: String,
        user_id: Option<String>,
        auth_method: String,
    },
    AuthorizationError {
        message: String,
        required_permission: String,
        current_permissions: Vec<String>,
    },
    
    // Module and import errors
    ImportError {
        message: String,
        module_name: String,
        search_paths: Vec<String>,
    },
    ModuleError {
        message: String,
        module_name: String,
        error_type: String,
    },
    
    // Database errors
    DatabaseError {
        message: String,
        query: Option<String>,
        error_code: Option<String>,
        connection_info: Option<String>,
    },
    
    // Custom user-defined errors
    UserError {
        message: String,
        error_type: String,
        custom_data: std::collections::HashMap<String, String>,
    },
}

impl fmt::Display for DetailedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetailedError::SyntaxError { message, line, column, source_line } => {
                writeln!(f, "Syntax Error at line {}, column {}: {}", line, column, message)?;
                if let Some(source) = source_line {
                    writeln!(f, "  {}", source)?;
                    writeln!(f, "  {}^", " ".repeat(*column - 1))?;
                }
                Ok(())
            }
            DetailedError::ParseError { message, position, expected, found } => {
                writeln!(f, "Parse Error at position {}: {}", position, message)?;
                writeln!(f, "  Expected: {}", expected.join(", "))?;
                writeln!(f, "  Found: {}", found)?;
                Ok(())
            }
            DetailedError::TypeError { message, expected_type, actual_type, location } => {
                write!(f, "Type Error: {}", message)?;
                writeln!(f, "  Expected: {}", expected_type)?;
                writeln!(f, "  Actual: {}", actual_type)?;
                if let Some(loc) = location {
                    writeln!(f, "  Location: {}", loc)?;
                }
                Ok(())
            }
            DetailedError::RuntimeError { message, stack_trace, error_code } => {
                writeln!(f, "Runtime Error ({}): {}", error_code, message)?;
                writeln!(f, "Stack trace:")?;
                for frame in stack_trace {
                    writeln!(f, "  at {}", frame)?;
                }
                Ok(())
            }
            DetailedError::IndexOutOfBounds { message, index, length, container_type } => {
                writeln!(f, "Index Out of Bounds: {}", message)?;
                writeln!(f, "  Index: {}, Length: {}, Container: {}", index, length, container_type)?;
                Ok(())
            }
            DetailedError::NetworkError { message, url, status_code, network_operation } => {
                writeln!(f, "Network Error during {}: {}", network_operation, message)?;
                if let Some(u) = url {
                    writeln!(f, "  URL: {}", u)?;
                }
                if let Some(code) = status_code {
                    writeln!(f, "  Status Code: {}", code)?;
                }
                Ok(())
            }
            DetailedError::SecurityError { message, violation_type, attempted_action } => {
                writeln!(f, "Security Violation ({}): {}", violation_type, message)?;
                writeln!(f, "  Attempted Action: {}", attempted_action)?;
                Ok(())
            }
            DetailedError::DatabaseError { message, query, error_code, connection_info } => {
                writeln!(f, "Database Error: {}", message)?;
                if let Some(code) = error_code {
                    writeln!(f, "  Error Code: {}", code)?;
                }
                if let Some(q) = query {
                    writeln!(f, "  Query: {}", q)?;
                }
                if let Some(conn) = connection_info {
                    writeln!(f, "  Connection: {}", conn)?;
                }
                Ok(())
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl StdError for DetailedError {}

/// Error context for providing additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file: Option<String>,
    pub function: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            file: None,
            function: None,
            line: None,
            column: None,
            additional_info: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());
        self
    }
    
    pub fn with_function(mut self, function: &str) -> Self {
        self.function = Some(function.to_string());
        self
    }
    
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
    
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info.insert(key.to_string(), value.to_string());
        self
    }
}

/// Error with context
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error: DetailedError,
    pub context: ErrorContext,
    pub caused_by: Option<Box<ContextualError>>,
}

impl ContextualError {
    pub fn new(error: DetailedError) -> Self {
        Self {
            error,
            context: ErrorContext::new(),
            caused_by: None,
        }
    }
    
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }
    
    pub fn caused_by(mut self, cause: ContextualError) -> Self {
        self.caused_by = Some(Box::new(cause));
        self
    }
    
    pub fn root_cause(&self) -> &DetailedError {
        match &self.caused_by {
            Some(cause) => cause.root_cause(),
            None => &self.error,
        }
    }
    
    pub fn chain(&self) -> Vec<&DetailedError> {
        let mut chain = vec![&self.error];
        let mut current = &self.caused_by;
        
        while let Some(cause) = current {
            chain.push(&cause.error);
            current = &cause.caused_by;
        }
        
        chain
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.error)?;
        
        if self.context.file.is_some() || self.context.function.is_some() {
            write!(f, "  Context: ")?;
            if let Some(file) = &self.context.file {
                write!(f, "file: {}", file)?;
                if let Some(line) = self.context.line {
                    write!(f, ":{}", line)?;
                    if let Some(column) = self.context.column {
                        write!(f, ":{}", column)?;
                    }
                }
                write!(f, " ")?;
            }
            if let Some(function) = &self.context.function {
                write!(f, "in {}", function)?;
            }
            writeln!(f)?;
        }
        
        for (key, value) in &self.context.additional_info {
            writeln!(f, "  {}: {}", key, value)?;
        }
        
        if let Some(cause) = &self.caused_by {
            writeln!(f, "Caused by:")?;
            write!(f, "{}", cause)?;
        }
        
        Ok(())
    }
}

impl StdError for ContextualError {}

/// Result type for operations that can fail with contextual errors
pub type ContextualResult<T> = std::result::Result<T, ContextualError>;

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Ignore,
    RetryWithBackoff { max_attempts: u32, backoff_ms: u64 },
    FallbackValue(String),
    FallbackFunction(String),
    Propagate,
    Log,
    Custom(String),
}

/// Error handler for managing error recovery
pub struct ErrorHandler {
    strategies: std::collections::HashMap<String, RecoveryStrategy>,
    error_log: Vec<ContextualError>,
    max_log_size: usize,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            strategies: std::collections::HashMap::new(),
            error_log: Vec::new(),
            max_log_size: 1000,
        }
    }
    
    pub fn register_strategy(&mut self, error_type: &str, strategy: RecoveryStrategy) {
        self.strategies.insert(error_type.to_string(), strategy);
    }
    
    pub fn handle_error(&mut self, error: ContextualError) -> Result<Option<String>> {
        // Log the error
        self.log_error(error.clone());
        
        // Determine error type
        let error_type = match &error.error {
            DetailedError::SyntaxError { .. } => "syntax",
            DetailedError::TypeError { .. } => "type",
            DetailedError::RuntimeError { .. } => "runtime",
            DetailedError::NetworkError { .. } => "network",
            DetailedError::IOError { .. } => "io",
            DetailedError::DatabaseError { .. } => "database",
            _ => "unknown",
        };
        
        // Apply recovery strategy
        if let Some(strategy) = self.strategies.get(error_type) {
            match strategy {
                RecoveryStrategy::Ignore => Ok(None),
                RecoveryStrategy::FallbackValue(value) => Ok(Some(value.clone())),
                RecoveryStrategy::Log => {
                    eprintln!("Error logged: {}", error);
                    Ok(None)
                }
                RecoveryStrategy::Propagate => {
                    Err(NeksisError::Other(format!("Unhandled error: {}", error)))
                }
                _ => Err(NeksisError::Other(format!("Recovery strategy not implemented: {:?}", strategy))),
            }
        } else {
            // Default: propagate
            Err(NeksisError::Other(format!("Unhandled error: {}", error)))
        }
    }
    
    pub fn log_error(&mut self, error: ContextualError) {
        self.error_log.push(error);
        
        // Trim log if too large
        if self.error_log.len() > self.max_log_size {
            self.error_log.remove(0);
        }
    }
    
    pub fn get_recent_errors(&self, count: usize) -> &[ContextualError] {
        let start = if self.error_log.len() > count {
            self.error_log.len() - count
        } else {
            0
        };
        &self.error_log[start..]
    }
    
    pub fn clear_log(&mut self) {
        self.error_log.clear();
    }
}

/// Convenience functions for creating errors
pub fn syntax_error(message: &str, line: usize, column: usize) -> DetailedError {
    DetailedError::SyntaxError {
        message: message.to_string(),
        line,
        column,
        source_line: None,
    }
}

pub fn type_error(message: &str, expected: &str, actual: &str) -> DetailedError {
    DetailedError::TypeError {
        message: message.to_string(),
        expected_type: expected.to_string(),
        actual_type: actual.to_string(),
        location: None,
    }
}

pub fn runtime_error(message: &str, error_code: u32) -> DetailedError {
    DetailedError::RuntimeError {
        message: message.to_string(),
        stack_trace: Vec::new(),
        error_code,
    }
}

pub fn network_error(message: &str, operation: &str) -> DetailedError {
    DetailedError::NetworkError {
        message: message.to_string(),
        url: None,
        status_code: None,
        network_operation: operation.to_string(),
    }
}

pub fn io_error(message: &str, operation: &str) -> DetailedError {
    DetailedError::IOError {
        message: message.to_string(),
        operation: operation.to_string(),
        path: None,
        error_code: None,
    }
}

/// Error conversion utilities
impl From<NeksisError> for DetailedError {
    fn from(err: NeksisError) -> Self {
        match err {
            NeksisError::ParseError(msg) => DetailedError::ParseError {
                message: msg,
                position: 0,
                expected: vec![],
                found: "".to_string(),
            },
            NeksisError::TypeError(msg) => DetailedError::TypeError {
                message: msg,
                expected_type: "unknown".to_string(),
                actual_type: "unknown".to_string(),
                location: None,
            },
            NeksisError::RuntimeError(msg) => DetailedError::RuntimeError {
                message: msg,
                stack_trace: vec![],
                error_code: 1,
            },
            NeksisError::NetworkError(msg) => DetailedError::NetworkError {
                message: msg,
                url: None,
                status_code: None,
                network_operation: "unknown".to_string(),
            },
            NeksisError::IoError(msg) => DetailedError::IOError {
                message: msg,
                operation: "unknown".to_string(),
                path: None,
                error_code: None,
            },
            NeksisError::IndexOutOfBounds(msg) => DetailedError::IndexOutOfBounds {
                message: msg,
                index: 0,
                length: 0,
                container_type: "unknown".to_string(),
            },
            _ => DetailedError::RuntimeError {
                message: format!("{:?}", err),
                stack_trace: vec![],
                error_code: 999,
            },
        }
    }
}

impl From<DetailedError> for ContextualError {
    fn from(err: DetailedError) -> Self {
        ContextualError::new(err)
    }
}

/// Try-catch mechanism for error handling
pub struct TryCatch<T> {
    result: ContextualResult<T>,
}

impl<T> TryCatch<T> {
    pub fn try_operation<F>(operation: F) -> Self
    where
        F: FnOnce() -> ContextualResult<T>,
    {
        Self {
            result: operation(),
        }
    }
    
    pub fn catch<F>(self, handler: F) -> ContextualResult<T>
    where
        F: FnOnce(ContextualError) -> ContextualResult<T>,
    {
        match self.result {
            Ok(value) => Ok(value),
            Err(error) => handler(error),
        }
    }
    
    pub fn finally<F>(self, cleanup: F) -> ContextualResult<T>
    where
        F: FnOnce(),
    {
        cleanup();
        self.result
    }
}
