use std::fs;
use neksisc::compiler::{FastCompiler, CompilerOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing neksis Compiler ===");
    
    // Test 1: Simple compilation
    test_simple_compilation()?;
    
    // Test 2: File compilation
    test_file_compilation()?;
    
    println!("=== All Tests Complete ===");
    Ok(())
}

fn test_simple_compilation() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing simple compilation...");
    
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
            println!("  ✅ Simple compilation successful");
            Ok(())
        }
        Err(e) => {
            println!("  ❌ Simple compilation failed: {}", e);
            Err(e.into())
        }
    }
}

fn test_file_compilation() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing file compilation...");
    
    if let Ok(source) = fs::read_to_string("simple_test.nx") {
        let mut compiler = FastCompiler::new(CompilerOptions::default());
        match compiler.compile(&source) {
            Ok(_) => {
                println!("  ✅ File compilation successful");
                Ok(())
            }
            Err(e) => {
                println!("  ❌ File compilation failed: {}", e);
                Err(e.into())
            }
        }
    } else {
        println!("  ⚠️  File not found, skipping");
        Ok(())
    }
} 