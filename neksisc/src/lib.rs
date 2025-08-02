// Modern Neksis 2025 Library
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

// Modern 2025 modules
pub mod modern_ast;
pub mod modern_lexer;
pub mod modern_parser;
pub mod modern_stdlib;
pub mod collections;
pub mod networking;
pub mod async_runtime;
pub mod modern_async;

// Object-Oriented Programming and Module System
pub mod oop;
pub mod module_system;

// NEXUS CORE - Production-Ready Advanced Systems Programming Framework
// Temporarily disabled due to compilation errors - need type definitions
// pub mod nexus_rt;        // Real-time scheduling and deterministic memory
// pub mod nexus_gpu;       // GPU acceleration and parallel computing
// pub mod nexus_secure;    // Security, sandboxing, and cryptography
// pub mod nexus_metal;     // Bare metal and embedded systems programming
// pub mod nexus_ai;        // AI/ML integration and acceleration
// pub mod nexus_production; // Complete production integration

// NEXUS CORE - Next Generation Features
// Temporarily disabled due to compilation errors - need type definitions
// pub mod nexus_build;     // Self-compiling JIT system
// pub mod nexus_stealth;   // Security enumeration and penetration testing
// pub mod nexus_cold;      // Cold execution and state preservation
// pub mod nexus_time;      // Temporal programming and time-travel debugging

// Test modules
pub mod test_oop_modules;

// Integration modules temporarily disabled - will fix AST alignment later
// pub mod error_handling;
// pub mod modern_integration;

// Advanced features
pub mod type_inference;
pub mod memory_profiler;
pub mod borrow_checker;
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

 