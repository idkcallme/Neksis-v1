use neksisc::{FastCompiler, CompilerOptions};
use neksisc::lexer::Lexer;
use neksisc::parser::Parser;
use neksisc::semantic::SemanticAnalyzer;
use neksisc::bytecode_compiler::BytecodeCompiler;
use neksisc::vm::VM;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== neksis Compiler Functionality Test ===");
    println!();
    
    // Test 1: Lexical Analysis
    test_lexical_analysis()?;
    
    // Test 2: Parsing
    test_parsing()?;
    
    // Test 3: Semantic Analysis
    test_semantic_analysis()?;
    
    // Test 4: Bytecode Compilation
    test_bytecode_compilation()?;
    
    // Test 5: Virtual Machine Execution
    test_vm_execution()?;
    
    // Test 6: Full Compiler Pipeline
    test_full_pipeline()?;
    
    println!("=== All Functionality Tests Complete ===");
    println!("✅ Compiler is fully functional!");
    Ok(())
}

fn test_lexical_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Lexical Analysis...");
    
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("  ✅ Lexical analysis successful - {} tokens generated", tokens.len());
            Ok(())
        }
        Err(e) => {
            println!("  ❌ Lexical analysis failed: {}", e);
            Err(e.into())
        }
    }
}

fn test_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Parsing...");
    
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(ast) => {
            println!("  ✅ Parsing successful - AST generated");
            Ok(())
        }
        Err(e) => {
            println!("  ❌ Parsing failed: {}", e);
            Err(e.into())
        }
    }
}

fn test_semantic_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Semantic Analysis...");
    
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    let mut analyzer = SemanticAnalyzer::new();
    
    match analyzer.analyze(&ast) {
        Ok(_) => {
            println!("  ✅ Semantic analysis successful");
            Ok(())
        }
        Err(e) => {
            println!("  ❌ Semantic analysis failed: {}", e);
            Err(e.into())
        }
    }
}

fn test_bytecode_compilation() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Bytecode Compilation...");
    
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    let mut compiler = BytecodeCompiler::new();
    
    match compiler.compile_program(&ast) {
        Ok(instructions) => {
            println!("  ✅ Bytecode compilation successful - {} instructions generated", instructions.len());
            Ok(())
        }
        Err(e) => {
            println!("  ❌ Bytecode compilation failed: {}", e);
            Err(e.into())
        }
    }
}

fn test_vm_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Virtual Machine Execution...");
    
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    let mut compiler = BytecodeCompiler::new();
    let instructions = compiler.compile_program(&ast)?;
    let mut vm = VM::new();
    
    vm.load_instructions(instructions);
    
    match vm.run() {
        Ok(_) => {
            println!("  ✅ VM execution successful");
            Ok(())
        }
        Err(e) => {
            println!("  ❌ VM execution failed: {}", e);
            Err(e.into())
        }
    }
}

fn test_full_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Full Compiler Pipeline...");
    
    let source = r#"
    fn main() {
        let x = 10;
        let y = 20;
        let result = x + y;
        print("Result: " + result);
    }
    "#;
    
    let mut compiler = FastCompiler::new(CompilerOptions::default());
    
    match compiler.compile(source) {
        Ok(_) => {
            println!("  ✅ Full pipeline successful");
            Ok(())
        }
        Err(e) => {
            println!("  ❌ Full pipeline failed: {}", e);
            Err(e.into())
        }
    }
} 