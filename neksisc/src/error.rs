use std::fmt;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSuggestion {
    pub message: String,
    pub replacement: Option<String>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub suggestions: Vec<ErrorSuggestion>,
    pub help: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Semantic,
    Type,
    Borrow,
    Lifetime,
    Memory,
    Linker,
    Runtime,
    Internal,
    Configuration,
    IO,
    Network,
    Validation,
}

impl CompilerError {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            location: None,
            suggestions: Vec::new(),
            help: None,
            code: None,
        }
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_suggestion(mut self, suggestion: ErrorSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn lexical_error(message: &str) -> Self {
        Self::new(ErrorKind::Lexical, message.to_string())
    }

    pub fn syntax_error(message: &str) -> Self {
        Self::new(ErrorKind::Syntax, message.to_string())
    }

    pub fn semantic_error(message: &str) -> Self {
        Self::new(ErrorKind::Semantic, message.to_string())
    }

    pub fn type_error(message: &str) -> Self {
        Self::new(ErrorKind::Type, message.to_string())
    }

    pub fn borrow_error(message: &str) -> Self {
        Self::new(ErrorKind::Borrow, message.to_string())
    }

    pub fn lifetime_error(message: &str) -> Self {
        Self::new(ErrorKind::Lifetime, message.to_string())
    }

    pub fn memory_error(message: &str) -> Self {
        Self::new(ErrorKind::Memory, message.to_string())
    }

    pub fn linker_error(message: &str) -> Self {
        Self::new(ErrorKind::Linker, message.to_string())
    }

    pub fn runtime_error(message: &str) -> Self {
        Self::new(ErrorKind::Runtime, message.to_string())
    }

    pub fn internal_error(message: &str) -> Self {
        Self::new(ErrorKind::Internal, message.to_string())
    }

    pub fn config_error(message: &str) -> Self {
        Self::new(ErrorKind::Configuration, message.to_string())
    }

    pub fn io_error(message: &str) -> Self {
        Self::new(ErrorKind::IO, message.to_string())
    }

    pub fn network_error(message: &str) -> Self {
        Self::new(ErrorKind::Network, message.to_string())
    }

    pub fn validation_error(message: &str) -> Self {
        Self::new(ErrorKind::Validation, message.to_string())
    }

    pub fn parse_error(file: &str, message: &str) -> Self {
        Self::new(ErrorKind::Syntax, format!("Parse error in {}: {}", file, message))
    }

    pub fn codegen_error(backend: &str, message: &str) -> Self {
        Self::new(ErrorKind::Internal, format!("Code generation error in {}: {}", backend, message))
    }

    pub fn ai_error(message: &str) -> Self {
        Self::new(ErrorKind::Internal, format!("AI integration error: {}", message))
    }

    pub fn ffi_error(component: &str, message: &str) -> Self {
        Self::new(ErrorKind::Internal, format!("FFI error in {}: {}", component, message))
    }

    pub fn format_detailed(&self) -> String {
        let mut output = String::new();
        
        // Error header
        output.push_str(&format!("error[{}]: {}\n", 
            self.code.as_ref().unwrap_or(&"E0000".to_string()), 
            self.message
        ));
        
        // Location information
        if let Some(location) = &self.location {
            output.push_str(&format!("  --> {}:{}:{}\n", 
                location.file.display(), location.line, location.column
            ));
            
            // Show source line with error
            if let Ok(content) = std::fs::read_to_string(&location.file) {
                let lines: Vec<&str> = content.lines().collect();
                if location.line > 0 && location.line <= lines.len() {
                    let line_num = location.line;
                    let line_content = lines[line_num - 1];
                    
                    output.push_str(&format!("   |\n"));
                    output.push_str(&format!("{} | {}\n", line_num, line_content));
                    output.push_str(&format!("   | {}^\n", " ".repeat(location.column)));
                }
            }
        }
        
        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str("\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("help: {}\n", suggestion.message));
                if let Some(replacement) = &suggestion.replacement {
                    output.push_str(&format!("     = note: {}", replacement));
                }
            }
        }
        
        // Help information
        if let Some(help) = &self.help {
            output.push_str(&format!("\nhelp: {}\n", help));
        }
        
        output
    }

    pub fn format_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| {
            format!("{{\"error\": \"Failed to serialize error: {}\"}}", self.message)
        })
    }

    pub fn is_fatal(&self) -> bool {
        matches!(self.kind, 
            ErrorKind::Internal | 
            ErrorKind::Configuration | 
            ErrorKind::IO | 
            ErrorKind::Network
        )
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(self.kind,
            ErrorKind::Lexical |
            ErrorKind::Syntax |
            ErrorKind::Semantic |
            ErrorKind::Type |
            ErrorKind::Borrow |
            ErrorKind::Lifetime |
            ErrorKind::Memory |
            ErrorKind::Validation
        )
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_detailed())
    }
}

impl std::error::Error for CompilerError {}

impl From<String> for CompilerError {
    fn from(s: String) -> Self {
        CompilerError::syntax_error(&s)
    }
}

impl From<&str> for CompilerError {
    fn from(s: &str) -> Self {
        CompilerError::syntax_error(s)
    }
}

// Error reporting utilities
pub struct ErrorReporter {
    errors: Vec<CompilerError>,
    warnings: Vec<CompilerError>,
    max_errors: usize,
    max_warnings: usize,
    format: ErrorFormat,
}

#[derive(Debug, Clone)]
pub enum ErrorFormat {
    Human,
    Json,
    Machine,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            max_errors: 100,
            max_warnings: 50,
            format: ErrorFormat::Human,
        }
    }

    pub fn with_format(mut self, format: ErrorFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_limits(mut self, max_errors: usize, max_warnings: usize) -> Self {
        self.max_errors = max_errors;
        self.max_warnings = max_warnings;
        self
    }

    pub fn add_error(&mut self, error: CompilerError) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error);
        }
    }

    pub fn add_warning(&mut self, warning: CompilerError) {
        if self.warnings.len() < self.max_warnings {
            self.warnings.push(warning);
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }

    pub fn report(&self) -> String {
        match self.format {
            ErrorFormat::Human => self.format_human(),
            ErrorFormat::Json => self.format_json(),
            ErrorFormat::Machine => self.format_machine(),
        }
    }

    fn format_human(&self) -> String {
        let mut output = String::new();
        
        // Report errors
        if !self.errors.is_empty() {
            output.push_str(&format!("error: {} error(s) found\n\n", self.errors.len()));
            
            for (i, error) in self.errors.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, error.format_detailed()));
                if i < self.errors.len() - 1 {
                    output.push_str("\n");
                }
            }
        }
        
        // Report warnings
        if !self.warnings.is_empty() {
            if !self.errors.is_empty() {
                output.push_str("\n");
            }
            output.push_str(&format!("warning: {} warning(s) found\n\n", self.warnings.len()));
            
            for (i, warning) in self.warnings.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, warning.format_detailed()));
                if i < self.warnings.len() - 1 {
                    output.push_str("\n");
                }
            }
        }
        
        // Summary
        if !self.errors.is_empty() || !self.warnings.is_empty() {
            output.push_str(&format!("\ncompilation {}: {} error(s), {} warning(s)\n",
                if self.errors.is_empty() { "succeeded" } else { "failed" },
                self.errors.len(),
                self.warnings.len()
            ));
        }
        
        output
    }

    fn format_json(&self) -> String {
        let report = ErrorReport {
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
            summary: ErrorSummary {
                error_count: self.errors.len(),
                warning_count: self.warnings.len(),
                success: self.errors.is_empty(),
            },
        };
        
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| {
            "{\"error\": \"Failed to serialize error report\"}".to_string()
        })
    }

    fn format_machine(&self) -> String {
        let mut output = String::new();
        
        for error in &self.errors {
            output.push_str(&format!("error:{}\n", error.message));
            if let Some(location) = &error.location {
                output.push_str(&format!("location:{}:{}:{}\n", 
                    location.file.display(), location.line, location.column));
            }
            if let Some(code) = &error.code {
                output.push_str(&format!("code:{}\n", code));
            }
            output.push_str("\n");
        }
        
        for warning in &self.warnings {
            output.push_str(&format!("warning:{}\n", warning.message));
            if let Some(location) = &warning.location {
                output.push_str(&format!("location:{}:{}:{}\n", 
                    location.file.display(), location.line, location.column));
            }
            if let Some(code) = &warning.code {
                output.push_str(&format!("code:{}\n", code));
            }
            output.push_str("\n");
        }
        
        output
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorReport {
    errors: Vec<CompilerError>,
    warnings: Vec<CompilerError>,
    summary: ErrorSummary,
}

#[derive(Serialize, Deserialize)]
struct ErrorSummary {
    error_count: usize,
    warning_count: usize,
    success: bool,
}

// Common error constructors
pub fn unexpected_token(token: &str, expected: &str, location: SourceLocation) -> CompilerError {
    CompilerError::syntax_error(&format!("unexpected token `{}`, expected `{}`", token, expected))
        .with_location(location)
        .with_code("E0001".to_string())
        .with_help("Check your syntax and ensure all tokens are properly matched".to_string())
}

pub fn undefined_variable(name: &str, location: SourceLocation) -> CompilerError {
    CompilerError::semantic_error(&format!("undefined variable `{}`", name))
        .with_location(location)
        .with_code("E0002".to_string())
        .with_help("Make sure the variable is declared before use".to_string())
}

pub fn type_mismatch(expected: &str, found: &str, location: SourceLocation) -> CompilerError {
    CompilerError::type_error(&format!("expected `{}`, found `{}`", expected, found))
        .with_location(location)
        .with_code("E0003".to_string())
        .with_help("Check the types of your variables and expressions".to_string())
}

pub fn borrow_error(message: &str, location: SourceLocation) -> CompilerError {
    CompilerError::borrow_error(message)
        .with_location(location)
        .with_code("E0004".to_string())
        .with_help("Check your borrowing rules and lifetime annotations".to_string())
}

pub fn lifetime_error(message: &str, location: SourceLocation) -> CompilerError {
    CompilerError::lifetime_error(message)
        .with_location(location)
        .with_code("E0005".to_string())
        .with_help("Ensure proper lifetime annotations and borrowing rules".to_string())
}

pub fn memory_error(message: &str, location: SourceLocation) -> CompilerError {
    CompilerError::memory_error(message)
        .with_location(location)
        .with_code("E0006".to_string())
        .with_help("Check your memory management and allocation patterns".to_string())
}

pub fn file_not_found(path: &str) -> CompilerError {
    CompilerError::io_error(&format!("file not found: `{}`", path))
        .with_code("E0007".to_string())
        .with_help("Check the file path and ensure the file exists".to_string())
}

pub fn permission_denied(path: &str) -> CompilerError {
    CompilerError::io_error(&format!("permission denied: `{}`", path))
        .with_code("E0008".to_string())
        .with_help("Check file permissions and ensure you have access".to_string())
}

pub fn network_error(message: &str) -> CompilerError {
    CompilerError::network_error(message)
        .with_code("E0009".to_string())
        .with_help("Check your network connection and try again".to_string())
}

pub fn internal_error(message: &str) -> CompilerError {
    CompilerError::internal_error(message)
        .with_code("E9999".to_string())
        .with_help("This is an internal compiler error. Please report this issue.".to_string())
} 