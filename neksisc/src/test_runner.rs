use neksisc::{FastCompiler, CompilerOptions};
use neksisc::error::CompilerError;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("=== neksis Compiler Test Suite ===");
    println!("========================================");
    
    // Read the comprehensive test file
    let test_content = fs::read_to_string("../examples/essential_features_test.nx")
        .expect("Failed to read essential features test file");
    
    println!("Test file loaded successfully!");
    println!("File size: {} bytes", test_content.len());
    println!();
    
    // Test different optimization levels
    let optimization_levels = vec![0, 1, 2, 3];
    
    for opt_level in optimization_levels {
        println!("Testing with optimization level: {}", opt_level);
        println!("----------------------------------------");
        
        let options = CompilerOptions {
            incremental: false,
            parallel: false,
            cache_enabled: false,
            optimization_level: opt_level,
            max_workers: 1,
        };
        
        let compiler = FastCompiler::new(options);
        
        let start_time = Instant::now();
        
        match compiler.compile_sequential("comprehensive_tests.nx", &test_content) {
            Ok(output) => {
                let compile_time = start_time.elapsed();
                println!("✓ Compilation successful!");
                println!("  Compile time: {:?}", compile_time);
                println!("  Output size: {} bytes", output.len());
                
                // Get optimization statistics
                let stats = compiler.get_optimization_stats();
                println!("  Optimization stats:");
                println!("    Transformations made: {}", stats.transformations_made);
                println!("    Passes applied: {}", stats.passes_applied.len());
                if stats.code_size_before > 0 {
                    let reduction = ((stats.code_size_before - stats.code_size_after) as f64 / stats.code_size_before as f64) * 100.0;
                    println!("    Code size reduction: {:.1}%", reduction);
                }
                
                // Get optimization report
                let report = compiler.get_optimization_report();
                println!("  Optimization report:");
                println!("    {}", report);
                
                // Try to extract the result from the output
                // The main function should return the number of passed tests
                if let Some(result) = extract_test_result(&output) {
                    println!("  Test result: {}/{} tests passed", result, 25); // 25 total tests
                    if result == 25 {
                        println!("  ✓ ALL TESTS PASSED!");
                    } else {
                        println!("  ✗ Some tests failed");
                    }
                } else {
                    println!("  ? Could not extract test result from output");
                }
                
            },
            Err(e) => {
                println!("✗ Compilation failed: {:?}", e);
            }
        }
        
        println!();
    }
    
    println!("========================================");
    println!("Test suite completed!");
    println!("========================================");
    
    Ok(())
}

fn extract_test_result(output: &str) -> Option<i32> {
    // Look for patterns that might indicate the test result
    // This is a simple heuristic - in a real implementation, you'd want
    // more sophisticated output parsing
    
    // Look for numbers that could be test results
    let lines: Vec<&str> = output.lines().collect();
    for line in &lines {
        if line.contains("passed") || line.contains("tests") {
            // Try to extract a number
            if let Some(num) = extract_number(line) {
                return Some(num);
            }
        }
    }
    
    // If no specific pattern found, try to find any number that could be a result
    for line in &lines {
        if let Some(num) = extract_number(line) {
            if num >= 0 && num <= 25 { // Reasonable range for test results
                return Some(num);
            }
        }
    }
    
    None
}

fn extract_number(text: &str) -> Option<i32> {
    // Simple number extraction
    let words: Vec<&str> = text.split_whitespace().collect();
    for word in words {
        if let Ok(num) = word.parse::<i32>() {
            return Some(num);
        }
    }
    None
} 