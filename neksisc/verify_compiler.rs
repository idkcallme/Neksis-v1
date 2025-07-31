use neksisc::lexer::Lexer;
use neksisc::parser::Parser;
use neksisc::semantic::SemanticAnalyzer;
use neksisc::bytecode_compiler::BytecodeCompiler;
use neksisc::vm::VM;

fn main() {
    println!("=== neksis Compiler Verification ===");
    
    // Test 1: Basic Lexical Analysis
    println!("1. Testing Lexical Analysis...");
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    match lexer.tokenize() {
        Ok(tokens) => println!("   ✅ Success: {} tokens generated", tokens.len()),
        Err(e) => println!("   ❌ Failed: {}", e),
    }
    
    // Test 2: Basic Parsing
    println!("2. Testing Parsing...");
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(_) => println!("   ✅ Success: AST generated"),
            Err(e) => println!("   ❌ Failed: {}", e),
        }
    } else {
        println!("   ❌ Failed: Could not tokenize");
    }
    
    // Test 3: Semantic Analysis
    println!("3. Testing Semantic Analysis...");
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        if let Ok(ast) = parser.parse() {
            let mut analyzer = SemanticAnalyzer::new();
            match analyzer.analyze(&ast) {
                Ok(_) => println!("   ✅ Success: Semantic analysis completed"),
                Err(e) => println!("   ❌ Failed: {}", e),
            }
        } else {
            println!("   ❌ Failed: Could not parse");
        }
    } else {
        println!("   ❌ Failed: Could not tokenize");
    }
    
    // Test 4: Bytecode Compilation
    println!("4. Testing Bytecode Compilation...");
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        if let Ok(ast) = parser.parse() {
            let mut compiler = BytecodeCompiler::new();
            match compiler.compile_program(&ast) {
                Ok(instructions) => println!("   ✅ Success: {} instructions generated", instructions.len()),
                Err(e) => println!("   ❌ Failed: {}", e),
            }
        } else {
            println!("   ❌ Failed: Could not parse");
        }
    } else {
        println!("   ❌ Failed: Could not tokenize");
    }
    
    // Test 5: Virtual Machine
    println!("5. Testing Virtual Machine...");
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source, "test".to_string());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        if let Ok(ast) = parser.parse() {
            let mut compiler = BytecodeCompiler::new();
            if let Ok(instructions) = compiler.compile_program(&ast) {
                let mut vm = VM::new();
                vm.load_instructions(instructions);
                match vm.run() {
                    Ok(_) => println!("   ✅ Success: VM execution completed"),
                    Err(e) => println!("   ❌ Failed: {}", e),
                }
            } else {
                println!("   ❌ Failed: Could not compile to bytecode");
            }
        } else {
            println!("   ❌ Failed: Could not parse");
        }
    } else {
        println!("   ❌ Failed: Could not tokenize");
    }
    
    println!("=== Verification Complete ===");
} 