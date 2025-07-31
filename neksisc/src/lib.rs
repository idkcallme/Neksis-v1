pub mod ast;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod codegen;
pub mod error;
pub mod compiler;
pub mod repl;
pub mod stdlib;
pub mod optimizer;
pub mod optimization_analysis;
pub mod vm;
pub mod bytecode_compiler;
pub mod package_manager;
pub mod lsp;
pub mod tests;
pub mod cli;
pub mod formatter;
pub mod linter;

// Temporarily comment out problematic modules for core compilation
// pub mod python_bridge;
// pub mod wasm_bridge;
pub mod type_inference;
pub mod memory_profiler;
pub mod borrow_checker;
// pub mod wasm;
pub mod macro_system;
pub mod ffi;
pub mod concurrency;

use crate::error::CompilerError;
use crate::semantic::SemanticAnalyzer;
use crate::codegen::simple::{SimpleCodeGen, CodeGenerator};

pub struct Compiler {
    pub semantic_analyzer: SemanticAnalyzer,
    pub code_generator: SimpleCodeGen,
}

// Re-export FastCompiler for convenience
pub use crate::compiler::FastCompiler;
pub use crate::compiler::CompilerOptions;

impl Compiler {
    pub fn new() -> Result<Self, CompilerError> {
        Ok(Self {
            semantic_analyzer: SemanticAnalyzer::new(),
            code_generator: SimpleCodeGen::new(CompilerOptions::default())?,
        })
    }

    pub fn compile(&mut self, source: &str) -> Result<(), CompilerError> {
        // Parse the source code
        let mut lexer = lexer::Lexer::new(source, "input".to_string());
        let tokens = lexer.tokenize()
            .map_err(|e| CompilerError::parse_error("lexer", &e))?;
        
        let mut parser = parser::Parser::new(tokens);
        let program = parser.parse()
            .map_err(|e| CompilerError::parse_error("parser", &e))?;
        
        // Perform semantic analysis
        self.semantic_analyzer.analyze(&program)?;
        
        // Generate code
        self.code_generator.generate(&program)?;
        
        Ok(())
    }
}

 