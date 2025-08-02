use neksisc::{FastCompiler, CompilerOptions};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("WORKING HELLO WORLD TEST");
    println!("========================================");
    
    // Read the working hello test file
    let test_content = fs::read_to_string("../working_hello.nx")
        .expect("Failed to read working_hello.nx");
    
    println!("Test file loaded successfully!");
    println!("File size: {} bytes", test_content.len());
    println!();
    
    let options = CompilerOptions {
        incremental: false,
        parallel: false,
        cache_enabled: false,
        optimization_level: 0,
        max_workers: 1,
    };
    
    let compiler = FastCompiler::new(options);
    
    match compiler.compile_sequential("working_hello.nx", &test_content) {
        Ok(output) => {
            println!("✓ HELLO WORLD COMPILATION SUCCESSFUL!");
            println!("  Output size: {} bytes", output.len());
            println!("  First 200 chars: {}", output.chars().take(200).collect::<String>());
            println!("  ✓ NEKSIS COMPILER IS WORKING!");
        },
        Err(e) => {
            println!("✗ Compilation failed: {:?}", e);
        }
    }
    
    println!();
    println!("========================================");
    println!("HELLO WORLD TEST COMPLETED!");
    println!("========================================");
    
    Ok(())
}
