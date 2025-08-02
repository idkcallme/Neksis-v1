// Modern Standard Library for Neksis 2025
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{self as std_io, Write};
use std::path::Path;

/// Initialize the standard library
pub fn init() {
    println!("Neksis 2025 Standard Library initialized");
}

/// Error handling types for Neksis
#[derive(Debug, Clone, PartialEq)]
pub enum NeksisError {
    RuntimeError(String),
    TypeError(String),
    IoError(String),
    NetworkError(String),
    ParseError(String),
    IndexOutOfBounds(String),
    NullPointerError(String),
    DivisionByZero,
    StackOverflow,
    HeapExhausted,
    Timeout(String),
    ChannelError(String),
    LockError(String),
    TaskError(String),
    Other(String),
}

impl std::fmt::Display for NeksisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeksisError::RuntimeError(msg) => write!(f, "Runtime Error: {}", msg),
            NeksisError::TypeError(msg) => write!(f, "Type Error: {}", msg),
            NeksisError::IoError(msg) => write!(f, "I/O Error: {}", msg),
            NeksisError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            NeksisError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            NeksisError::IndexOutOfBounds(msg) => write!(f, "Index Out of Bounds: {}", msg),
            NeksisError::NullPointerError(msg) => write!(f, "Null Pointer Error: {}", msg),
            NeksisError::DivisionByZero => write!(f, "Division by Zero"),
            NeksisError::StackOverflow => write!(f, "Stack Overflow"),
            NeksisError::HeapExhausted => write!(f, "Heap Exhausted"),
            NeksisError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            NeksisError::ChannelError(msg) => write!(f, "Channel Error: {}", msg),
            NeksisError::LockError(msg) => write!(f, "Lock Error: {}", msg),
            NeksisError::TaskError(msg) => write!(f, "Task Error: {}", msg),
            NeksisError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for NeksisError {}

/// Result type alias for Neksis operations
pub type NeksisResult<T> = Result<T, NeksisError>;

/// Option type alias for cleaner code  
pub type NeksisOption<T> = Option<T>;

/// Core runtime functions
pub mod core {
    use super::*;
    
    /// Get current timestamp
    pub fn now() -> Result<u64, NeksisError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|_| NeksisError::RuntimeError("Failed to get current time".to_string()))
    }
    
    /// Sleep for specified milliseconds
    pub fn sleep(ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
    
    /// Exit the program with specified code
    pub fn exit(code: i32) -> ! {
        std::process::exit(code);
    }
}

/// Basic I/O operations
pub mod io {
    use super::*;
    
    /// Print to stdout
    pub fn print(msg: &str) {
        print!("{}", msg);
        let _ = std_io::stdout().flush();
    }
    
    /// Print line to stdout
    pub fn println(msg: &str) {
        println!("{}", msg);
    }
    
    /// Read line from stdin
    pub fn read_line() -> NeksisResult<String> {
        let mut input = String::new();
        match std_io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Remove the trailing newline
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                Ok(input)
            }
            Err(e) => Err(NeksisError::IoError(format!("Failed to read line: {}", e)))
        }
    }
    
    /// Read entire file as string
    pub fn read_file(path: &str) -> NeksisResult<String> {
        std::fs::read_to_string(path)
            .map_err(|e| NeksisError::IoError(format!("Failed to read file '{}': {}", path, e)))
    }
    
    /// Write string to file
    pub fn write_file(path: &str, content: &str) -> NeksisResult<()> {
        std::fs::write(path, content)
            .map_err(|e| NeksisError::IoError(format!("Failed to write file '{}': {}", path, e)))
    }
    
    /// Check if file exists
    pub fn file_exists(path: &str) -> bool {
        Path::new(path).exists()
    }
}

/// String manipulation utilities
pub mod string {
    /// Check if string contains substring
    pub fn contains(text: &str, substring: &str) -> bool {
        text.contains(substring)
    }
    
    /// Split string by delimiter
    pub fn split(text: &str, delimiter: &str) -> Vec<String> {
        text.split(delimiter).map(|s| s.to_string()).collect()
    }
    
    /// Join strings with delimiter
    pub fn join(strings: &[String], delimiter: &str) -> String {
        strings.join(delimiter)
    }
    
    /// Convert to uppercase
    pub fn to_upper(text: &str) -> String {
        text.to_uppercase()
    }
    
    /// Convert to lowercase
    pub fn to_lower(text: &str) -> String {
        text.to_lowercase()
    }
    
    /// Trim whitespace
    pub fn trim(text: &str) -> String {
        text.trim().to_string()
    }
}

/// Math utilities
pub mod math {
    /// Absolute value
    pub fn abs(x: f64) -> f64 {
        x.abs()
    }
    
    /// Square root
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }
    
    /// Power function
    pub fn pow(base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }
    
    /// Round to nearest integer
    pub fn round(x: f64) -> f64 {
        x.round()
    }
    
    /// Floor function
    pub fn floor(x: f64) -> f64 {
        x.floor()
    }
    
    /// Minimum of two values
    pub fn min(a: f64, b: f64) -> f64 {
        a.min(b)
    }
    
    /// Maximum of two values
    pub fn max(a: f64, b: f64) -> f64 {
        a.max(b)
    }
}
