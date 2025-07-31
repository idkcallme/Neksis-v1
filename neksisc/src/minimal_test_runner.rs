use neksisc::{FastCompiler, CompilerOptions};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("NEXUS COMPILER MINIMAL TEST");
    println!("========================================");
    
    // Read the minimal test file
    let test_content = fs::read_to_string("../minimal_test.nx")
        .expect("Failed to read minimal_test.nx");
    
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
    
    match compiler.compile_sequential("minimal_test.nx", &test_content) {
        Ok(output) => {
            println!("✓ Compilation successful!");
            println!("  Output size: {} bytes", output.len());
            println!("  Output: {}", output);
        },
        Err(e) => {
            println!("✗ Compilation failed: {:?}", e);
        }
    }
    
    println!("========================================");
    println!("Minimal test completed!");
    println!("========================================");
    
    Ok(())
} 