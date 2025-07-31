use std::collections::HashMap;
use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::codegen::simple::{SimpleCodeGen, CodeGenerator};
use crate::ast::Program;
use crate::optimizer::Optimizer;
use crate::optimization_analysis::OptimizationAnalyzer;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use num_cpus;

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub incremental: bool,
    pub parallel: bool,
    pub cache_enabled: bool,
    pub optimization_level: u8,
    pub max_workers: usize,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            incremental: true,
            parallel: true,
            cache_enabled: true,
            optimization_level: 1,
            max_workers: num_cpus::get(),
        }
    }
}

#[derive(Debug)]
pub struct CompilationCache {
    pub ast_cache: HashMap<String, Program>,
    pub token_cache: HashMap<String, Vec<crate::lexer::Token>>,
    pub semantic_cache: HashMap<String, crate::semantic::TypeValue>,
    pub last_modified: HashMap<String, std::time::SystemTime>,
}

impl CompilationCache {
    pub fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
            token_cache: HashMap::new(),
            semantic_cache: HashMap::new(),
            last_modified: HashMap::new(),
        }
    }

    pub fn is_file_changed(&self, filename: &str) -> bool {
        if let Ok(metadata) = std::fs::metadata(filename) {
            if let Ok(modified) = metadata.modified() {
                if let Some(last_modified) = self.last_modified.get(filename) {
                    return modified > *last_modified;
                }
            }
        }
        true // Assume changed if we can't determine
    }

    pub fn update_file_timestamp(&mut self, filename: &str) {
        if let Ok(metadata) = std::fs::metadata(filename) {
            if let Ok(modified) = metadata.modified() {
                self.last_modified.insert(filename.to_string(), modified);
            }
        }
    }
}

pub struct FastCompiler {
    options: CompilerOptions,
    cache: Arc<Mutex<CompilationCache>>,
    compilation_stats: Arc<Mutex<CompilationStats>>,
    optimizer: Optimizer,
    optimization_analyzer: OptimizationAnalyzer,
}

#[derive(Debug)]
pub struct CompilationStats {
    pub total_compilations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_compilation_time: Duration,
    pub last_compilation_time: Duration,
}

impl CompilationStats {
    pub fn new() -> Self {
        Self {
            total_compilations: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_compilation_time: Duration::from_millis(0),
            last_compilation_time: Duration::from_millis(0),
        }
    }

    pub fn update_stats(&mut self, compilation_time: Duration, cache_hit: bool) {
        self.total_compilations += 1;
        self.last_compilation_time = compilation_time;
        
        if cache_hit {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }

        // Update average compilation time
        let total_time = self.average_compilation_time * ((self.total_compilations - 1) as u32) + compilation_time;
        self.average_compilation_time = total_time / (self.total_compilations as u32);
    }
}

impl FastCompiler {
    pub fn new(options: CompilerOptions) -> Self {
        Self {
            options: options.clone(),
            cache: Arc::new(Mutex::new(CompilationCache::new())),
            compilation_stats: Arc::new(Mutex::new(CompilationStats::new())),
            optimizer: Optimizer::new(options.clone()),
            optimization_analyzer: OptimizationAnalyzer::new(),
        }
    }

    pub fn compile_file(&self, filename: &str) -> Result<String, CompilerError> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_hit = {
            let cache = self.cache.lock().unwrap();
            !cache.is_file_changed(filename)
        };

        let result = if cache_hit && self.options.cache_enabled {
            self.compile_from_cache(filename)
        } else {
            self.compile_fresh(filename)
        };

        // Update statistics
        let compilation_time = start_time.elapsed();
        let mut stats = self.compilation_stats.lock().unwrap();
        stats.update_stats(compilation_time, cache_hit);

        result
    }

    fn compile_from_cache(&self, filename: &str) -> Result<String, CompilerError> {
        let cache = self.cache.lock().unwrap();
        
        if let Some(ast) = cache.ast_cache.get(filename) {
            // Use cached AST for code generation
            let options = CompilerOptions::default();
            let mut codegen = SimpleCodeGen::new(options)?;
            let _code = codegen.generate(ast)?;
            Ok("Compilation successful".to_string())
        } else {
            Err(CompilerError::syntax_error("No cached AST found"))
        }
    }

    fn compile_fresh(&self, filename: &str) -> Result<String, CompilerError> {
        let content = std::fs::read_to_string(filename)
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to read file: {}", e)))?;

        if self.options.parallel {
            self.compile_parallel(filename, &content)
        } else {
            self.compile_sequential(filename, &content)
        }
    }

    fn compile_parallel(&self, filename: &str, content: &str) -> Result<String, CompilerError> {
        // Parallel lexical analysis
        let lexer_handle = {
            let content = content.to_string();
            let filename = filename.to_string();
            thread::spawn(move || {
                let mut lexer = Lexer::new(&content, filename);
                lexer.tokenize()
            })
        };

        // Wait for lexical analysis to complete
        let tokens = lexer_handle.join()
            .map_err(|_| CompilerError::syntax_error("Lexical analysis failed"))?
            .map_err(|e| CompilerError::syntax_error(&e))?;

        // Parallel parsing and semantic analysis
        let (ast, semantic_result) = {
            let tokens_clone = tokens.clone();
            let ast_handle = thread::spawn(move || {
                let mut parser = Parser::new(tokens_clone);
                parser.parse()
            });

            let semantic_handle = thread::spawn(move || {
                let _analyzer = SemanticAnalyzer::new();
                // For now, we'll do semantic analysis after parsing
                // In a more sophisticated implementation, this could be done in parallel
                Ok(())
            });

            let ast = ast_handle.join()
                .map_err(|_| CompilerError::syntax_error("Parsing failed"))?
                .map_err(|e: String| CompilerError::syntax_error(&e))?;
            
            let _semantic = semantic_handle.join()
                .map_err(|_| CompilerError::syntax_error("Semantic analysis failed"))?
                .map_err(|e: CompilerError| e)?;

            (ast, Ok::<(), CompilerError>(()))
        };

        semantic_result?;

        // Semantic analysis (sequential for now)
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;

        // Code generation
        let options = CompilerOptions::default();
        let mut codegen = SimpleCodeGen::new(options)?;
        let code = codegen.generate(&ast)?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.ast_cache.insert(filename.to_string(), ast);
            cache.update_file_timestamp(filename);
        }

        Ok(code)
    }

    pub fn compile_sequential(&self, filename: &str, content: &str) -> Result<String, CompilerError> {
        let start_time = Instant::now();
        
        // Lexical analysis
        let mut lexer = Lexer::new(content, filename.to_string());
        let tokens = lexer.tokenize()?;

        // Parsing
        let mut parser = Parser::new(tokens);
        let mut ast = parser.parse()?;
        
        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;

        // Optimization analysis
        let mut optimization_analyzer = OptimizationAnalyzer::new();
        let _analysis = optimization_analyzer.analyze_program(&ast)?;
        
        // Apply optimizations
        let mut optimizer = Optimizer::new(self.options.clone());
        optimizer.optimize(&mut ast)?;

        // Code generation
        let mut codegen = SimpleCodeGen::new(self.options.clone())?;
        let code = codegen.generate(&ast)?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.ast_cache.insert(filename.to_string(), ast);
            cache.update_file_timestamp(filename);
        }

        // Update compilation stats
        if let Ok(mut stats) = self.compilation_stats.lock() {
            stats.update_stats(start_time.elapsed(), false);
        }

        Ok(code)
    }

    pub fn get_compilation_stats(&self) -> CompilationStats {
        self.compilation_stats.lock().unwrap().clone()
    }

    pub fn get_optimization_stats(&self) -> crate::optimizer::OptimizationStats {
        self.optimizer.get_optimization_stats().clone()
    }

    pub fn get_optimization_report(&self) -> String {
        self.optimization_analyzer.generate_optimization_report()
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.ast_cache.clear();
        cache.token_cache.clear();
        cache.semantic_cache.clear();
        cache.last_modified.clear();
    }

    pub fn compile_multiple_files(&self, filenames: &[String]) -> Result<Vec<String>, CompilerError> {
        if self.options.parallel && filenames.len() > 1 {
            self.compile_files_parallel(filenames)
        } else {
            self.compile_files_sequential(filenames)
        }
    }

    fn compile_files_parallel(&self, filenames: &[String]) -> Result<Vec<String>, CompilerError> {
        let handles: Vec<_> = filenames.iter().map(|filename| {
            let compiler = self.clone();
            let filename = filename.clone();
            thread::spawn(move || {
                compiler.compile_file(&filename)
            })
        }).collect();

        let mut results = Vec::new();
        for handle in handles {
            let result = handle.join()
                .map_err(|_| CompilerError::syntax_error("Thread join failed"))?;
            results.push(result?);
        }

        Ok(results)
    }

    fn compile_files_sequential(&self, filenames: &[String]) -> Result<Vec<String>, CompilerError> {
        let mut results = Vec::new();
        for filename in filenames {
            let result = self.compile_file(filename)?;
            results.push(result);
        }
        Ok(results)
    }

    pub fn compile(&mut self, source: &str) -> Result<String, CompilerError> {
        let _start_time = Instant::now();
        
        // Lexical analysis
        let mut lexer = Lexer::new(source, "input.nx".to_string());
        let tokens = lexer.tokenize()
            .map_err(|e| CompilerError::syntax_error(&e))?;
        
        // Parsing
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| CompilerError::syntax_error(&e))?;
        
        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;
        
        // Bytecode compilation
        let mut bytecode_compiler = crate::bytecode_compiler::BytecodeCompiler::new();
        let instructions = bytecode_compiler.compile_program(&ast)?;
        
        // Execute bytecode
        let mut vm = crate::vm::VM::new();
        vm.load_instructions(instructions);
        
        let _result = vm.run()?;
        Ok(format!("Execution completed successfully"))
    }
}

impl Clone for FastCompiler {
    fn clone(&self) -> Self {
        Self {
            options: self.options.clone(),
            cache: Arc::clone(&self.cache),
            compilation_stats: Arc::clone(&self.compilation_stats),
            optimizer: self.optimizer.clone(),
            optimization_analyzer: self.optimization_analyzer.clone(),
        }
    }
}

impl Clone for CompilationStats {
    fn clone(&self) -> Self {
        Self {
            total_compilations: self.total_compilations,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            average_compilation_time: self.average_compilation_time,
            last_compilation_time: self.last_compilation_time,
        }
    }
} 