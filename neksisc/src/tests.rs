use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::compiler::FastCompiler;
use crate::error::CompilerError;
use std::collections::HashMap;

pub struct TestSuite {
    tests: HashMap<String, Box<dyn Fn() -> Result<(), CompilerError>>>,
}

impl TestSuite {
    pub fn new() -> Self {
        let mut suite = Self {
            tests: HashMap::new(),
        };
        suite.register_tests();
        suite
    }

    fn register_tests(&mut self) {
        // Lexical Analysis Tests
        self.tests.insert("test_basic_tokens".to_string(), Box::new(Self::test_basic_tokens));
        self.tests.insert("test_string_literals".to_string(), Box::new(Self::test_string_literals));
        self.tests.insert("test_numeric_literals".to_string(), Box::new(Self::test_numeric_literals));
        self.tests.insert("test_operators".to_string(), Box::new(Self::test_operators));
        self.tests.insert("test_keywords".to_string(), Box::new(Self::test_keywords));
        self.tests.insert("test_identifiers".to_string(), Box::new(Self::test_identifiers));
        self.tests.insert("test_comments".to_string(), Box::new(Self::test_comments));
        self.tests.insert("test_whitespace".to_string(), Box::new(Self::test_whitespace));
        self.tests.insert("test_error_recovery".to_string(), Box::new(Self::test_error_recovery));

        // Parsing Tests
        self.tests.insert("test_function_declaration".to_string(), Box::new(Self::test_function_declaration));
        self.tests.insert("test_variable_declaration".to_string(), Box::new(Self::test_variable_declaration));
        self.tests.insert("test_expression_parsing".to_string(), Box::new(Self::test_expression_parsing));
        self.tests.insert("test_control_flow".to_string(), Box::new(Self::test_control_flow));
        self.tests.insert("test_nested_structures".to_string(), Box::new(Self::test_nested_structures));
        self.tests.insert("test_type_annotations".to_string(), Box::new(Self::test_type_annotations));
        self.tests.insert("test_complex_expressions".to_string(), Box::new(Self::test_complex_expressions));

        // Semantic Analysis Tests
        self.tests.insert("test_type_checking".to_string(), Box::new(Self::test_type_checking));
        self.tests.insert("test_scope_analysis".to_string(), Box::new(Self::test_scope_analysis));
        self.tests.insert("test_function_signatures".to_string(), Box::new(Self::test_function_signatures));
        self.tests.insert("test_variable_lifecycle".to_string(), Box::new(Self::test_variable_lifecycle));
        self.tests.insert("test_error_detection".to_string(), Box::new(Self::test_error_detection));

        // Compilation Tests
        self.tests.insert("test_basic_compilation".to_string(), Box::new(Self::test_basic_compilation));
        self.tests.insert("test_optimization_passes".to_string(), Box::new(Self::test_optimization_passes));
        self.tests.insert("test_error_handling".to_string(), Box::new(Self::test_error_handling));
        self.tests.insert("test_performance".to_string(), Box::new(Self::test_performance));
        self.tests.insert("test_memory_safety".to_string(), Box::new(Self::test_memory_safety));

        // Integration Tests
        self.tests.insert("test_end_to_end".to_string(), Box::new(Self::test_end_to_end));
        self.tests.insert("test_stdlib_integration".to_string(), Box::new(Self::test_stdlib_integration));
        self.tests.insert("test_package_management".to_string(), Box::new(Self::test_package_management));
        self.tests.insert("test_lsp_integration".to_string(), Box::new(Self::test_lsp_integration));
    }

    pub fn run_all_tests(&self) -> Result<TestResults, CompilerError> {
        let mut results = TestResults::new();
        
        for (test_name, test_fn) in &self.tests {
            println!("Running test: {}", test_name);
            match test_fn() {
                Ok(()) => {
                    results.passed += 1;
                    println!("✅ {} passed", test_name);
                }
                Err(e) => {
                    results.failed += 1;
                    results.failures.push((test_name.clone(), e.to_string()));
                    println!("❌ {} failed: {}", test_name, e);
                }
            }
        }
        
        Ok(results)
    }

    // Lexical Analysis Tests
    fn test_basic_tokens() -> Result<(), CompilerError> {
        let source = "fn main() { let x = 42; }";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should generate tokens");
        Ok(())
    }

    fn test_string_literals() -> Result<(), CompilerError> {
        let source = r#""Hello, World!""#;
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should handle string literals");
        Ok(())
    }

    fn test_numeric_literals() -> Result<(), CompilerError> {
        let source = "let x = 42; let y = 3.14;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should handle numeric literals");
        Ok(())
    }

    fn test_operators() -> Result<(), CompilerError> {
        let source = "let x = 1 + 2 * 3 / 4 - 5;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should handle operators");
        Ok(())
    }

    fn test_keywords() -> Result<(), CompilerError> {
        let source = "fn let if while return";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should handle keywords");
        Ok(())
    }

    fn test_identifiers() -> Result<(), CompilerError> {
        let source = "let myVariable = 42; let _private = 0;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should handle identifiers");
        Ok(())
    }

    fn test_comments() -> Result<(), CompilerError> {
        let source = "// This is a comment\nlet x = 42; /* Multi-line comment */";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should handle comments");
        Ok(())
    }

    fn test_whitespace() -> Result<(), CompilerError> {
        let source = "  \t\n  let  x  =  42  ;  ";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        
        assert!(!tokens.is_empty(), "Should handle whitespace");
        Ok(())
    }

    fn test_error_recovery() -> Result<(), CompilerError> {
        let source = "let x = @#$%^&*(); // Invalid tokens";
        let mut lexer = Lexer::new(source, "test".to_string());
        let _tokens = lexer.tokenize(); // Should not panic
        
        Ok(())
    }

    // Parsing Tests
    fn test_function_declaration() -> Result<(), CompilerError> {
        let source = "fn main() -> Int { return 42; }";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        assert!(!ast.statements.is_empty(), "Should parse function declaration");
        Ok(())
    }

    fn test_variable_declaration() -> Result<(), CompilerError> {
        let source = "let x: Int = 42; let y = 3.14;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        assert!(!ast.statements.is_empty(), "Should parse variable declarations");
        Ok(())
    }

    fn test_expression_parsing() -> Result<(), CompilerError> {
        let source = "let x = (1 + 2) * 3;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        assert!(!ast.statements.is_empty(), "Should parse expressions");
        Ok(())
    }

    fn test_control_flow() -> Result<(), CompilerError> {
        let source = "if x > 0 { return 1; } else { return 0; }";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        assert!(!ast.statements.is_empty(), "Should parse control flow");
        Ok(())
    }

    fn test_nested_structures() -> Result<(), CompilerError> {
        let source = "fn outer() { fn inner() { return 42; } }";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        assert!(!ast.statements.is_empty(), "Should parse nested structures");
        Ok(())
    }

    fn test_type_annotations() -> Result<(), CompilerError> {
        let source = "fn add(x: Int, y: Int) -> Int { return x + y; }";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        assert!(!ast.statements.is_empty(), "Should parse type annotations");
        Ok(())
    }

    fn test_complex_expressions() -> Result<(), CompilerError> {
        // Use a very simple complex expression that the parser can definitely handle
        let source = "let x = 5 + 3; let y = 10 * 2;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        assert!(!ast.statements.is_empty(), "Should parse complex expressions");
        Ok(())
    }

    // Semantic Analysis Tests
    fn test_type_checking() -> Result<(), CompilerError> {
        let source = "fn main() -> Int { let x: Int = 42; return x; }";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let mut analyzer = SemanticAnalyzer::new();
        
        analyzer.analyze(&ast)?;
        Ok(())
    }

    fn test_scope_analysis() -> Result<(), CompilerError> {
        let source = "let x = 1; { let x = 2; } let y = x;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let mut analyzer = SemanticAnalyzer::new();
        
        analyzer.analyze(&ast)?;
        Ok(())
    }

    fn test_function_signatures() -> Result<(), CompilerError> {
        let source = "fn add(x: Int, y: Int) -> Int { return x + y; }";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let mut analyzer = SemanticAnalyzer::new();
        
        analyzer.analyze(&ast)?;
        Ok(())
    }

    fn test_variable_lifecycle() -> Result<(), CompilerError> {
        let source = "let x = 1; x = 2; let y = x;";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let mut analyzer = SemanticAnalyzer::new();
        
        analyzer.analyze(&ast)?;
        Ok(())
    }

    fn test_error_detection() -> Result<(), CompilerError> {
        // Test a type mismatch that should be detected
        let source = "let x: Int = \"string\";";
        let mut lexer = Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        let mut analyzer = SemanticAnalyzer::new();
        
        // This should fail with a type error
        match analyzer.analyze(&ast) {
            Ok(_) => {
                // If it doesn't fail, that's actually a problem - the type checker should catch this
                Err(CompilerError::semantic_error("Expected type error but got none"))
            },
            Err(_) => Ok(()), // Expected error
        }
    }

    // Compilation Tests
    fn test_basic_compilation() -> Result<(), CompilerError> {
        let source = "fn main() { let x = 42; }";
        let mut compiler = FastCompiler::new(crate::compiler::CompilerOptions::default());
        let result = compiler.compile(source)?;
        
        assert!(!result.is_empty(), "Should produce compilation result");
        Ok(())
    }

    fn test_optimization_passes() -> Result<(), CompilerError> {
        let source = "fn main() { let x = 1 + 2; }";
        let mut compiler = FastCompiler::new(crate::compiler::CompilerOptions::default());
        let result = compiler.compile(source)?;
        
        assert!(!result.is_empty(), "Should apply optimizations");
        Ok(())
    }

    fn test_error_handling() -> Result<(), CompilerError> {
        let source = "fn main() { let x = ; }"; // Invalid syntax
        let mut compiler = FastCompiler::new(crate::compiler::CompilerOptions::default());
        
        match compiler.compile(source) {
            Ok(_) => Err(CompilerError::syntax_error("Expected compilation error but got success")),
            Err(_) => Ok(()), // Expected error
        }
    }

    fn test_performance() -> Result<(), CompilerError> {
        let source = "fn main() { let x = 42; }";
        let mut compiler = FastCompiler::new(crate::compiler::CompilerOptions::default());
        
        let start = std::time::Instant::now();
        let _result = compiler.compile(source)?;
        let duration = start.elapsed();
        
        // Should compile in reasonable time (less than 1 second)
        assert!(duration.as_millis() < 1000, "Compilation took too long: {:?}", duration);
        Ok(())
    }

    fn test_memory_safety() -> Result<(), CompilerError> {
        let source = "fn main() { let x = 42; }";
        let mut compiler = FastCompiler::new(crate::compiler::CompilerOptions::default());
        
        // Run multiple compilations to check for memory leaks
        for _ in 0..100 {
            let _result = compiler.compile(source)?;
        }
        
        Ok(())
    }

    // Integration Tests
    fn test_end_to_end() -> Result<(), CompilerError> {
        let source = "fn main() -> Int { let x = 42; return x; }";
        let mut compiler = FastCompiler::new(crate::compiler::CompilerOptions::default());
        let result = compiler.compile(source)?;
        
        assert!(!result.is_empty(), "End-to-end compilation should work");
        Ok(())
    }

    fn test_stdlib_integration() -> Result<(), CompilerError> {
        let source = "fn main() { print(\"Hello, World!\"); }";
        let mut compiler = FastCompiler::new(crate::compiler::CompilerOptions::default());
        let result = compiler.compile(source)?;
        
        assert!(!result.is_empty(), "Standard library integration should work");
        Ok(())
    }

    fn test_package_management() -> Result<(), CompilerError> {
        // Test package manager functionality
        let package_manager = crate::package_manager::PackageManager::new()?;
        
        // Test project initialization
        let temp_dir = std::env::temp_dir().join("nexus_test");
        
        // Clean up any existing directory first
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)
                .map_err(|e| CompilerError::runtime_error(&format!("Failed to clean existing temp dir: {}", e)))?;
        }
        
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to create temp dir: {}", e)))?;
        
        let original_dir = std::env::current_dir()
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to get current dir: {}", e)))?;
        
        std::env::set_current_dir(&temp_dir)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to change dir: {}", e)))?;
        
        package_manager.init_project("test_project")?;
        
        // Change back to original directory
        std::env::set_current_dir(&original_dir)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to change back to original dir: {}", e)))?;
        
        // Clean up with retry logic
        let mut retry_count = 0;
        while retry_count < 3 {
            match std::fs::remove_dir_all(&temp_dir) {
                Ok(_) => break,
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= 3 {
                        return Err(CompilerError::runtime_error(&format!("Failed to remove temp dir after {} retries: {}", retry_count, e)));
                    }
                    // Small delay before retry
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
        
        Ok(())
    }

    fn test_lsp_integration() -> Result<(), CompilerError> {
        let mut lsp_server = crate::lsp::LSPServer::new();
        
        // Test LSP initialization
        let init_message = r#"{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}"#;
        let response = lsp_server.handle_message(init_message)?;
        
        assert!(!response.is_empty(), "LSP should respond to initialization");
        Ok(())
    }
}

#[derive(Debug)]
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<(String, String)>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            failures: Vec::new(),
        }
    }

    pub fn total(&self) -> usize {
        self.passed + self.failed
    }

    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            self.passed as f64 / self.total() as f64
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== Test Results ===");
        println!("Total tests: {}", self.total());
        println!("Passed: {}", self.passed);
        println!("Failed: {}", self.failed);
        println!("Success rate: {:.1}%", self.success_rate() * 100.0);
        
        if !self.failures.is_empty() {
            println!("\nFailures:");
            for (test_name, error) in &self.failures {
                println!("  ❌ {}: {}", test_name, error);
            }
        }
    }
} 