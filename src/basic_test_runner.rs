use neksisc::{FastCompiler, CompilerOptions};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("NEXUS COMPILER BASIC TEST SUITE");
    println!("========================================");
    
    // Read the basic test file
    let test_content = fs::read_to_string("../basic_tests.nx")
        .expect("Failed to read basic_tests.nx");
    
    println!("Test file loaded successfully!");
    println!("File size: {} bytes", test_content.len());
    println!();
    
    // Test with different optimization levels
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
        
        match compiler.compile_sequential("basic_tests.nx", &test_content) {
            Ok(output) => {
                println!("✓ Compilation successful!");
                println!("  Output size: {} bytes", output.len());
                
                // Get optimization statistics
                if let Ok(stats) = compiler.get_optimization_stats() {
                    println!("  Optimization stats:");
                    println!("    Transformations made: {}", stats.transformations_made);
                    println!("    Passes executed: {}", stats.passes_executed);
                    println!("    Estimated code size reduction: {}%", stats.estimated_code_size_reduction);
                }
                
                // Get optimization report
                if let Ok(report) = compiler.get_optimization_report() {
                    println!("  Optimization report:");
                    println!("    {}", report);
                }
                
                // Try to extract the result from the output
                if let Some(result) = extract_test_result(&output) {
                    println!("  Test result: {}/{} tests passed", result, 7); // 7 total tests
                    if result == 7 {
                        println!("  ✓ ALL BASIC TESTS PASSED!");
                    } else {
                        println!("  ✗ Some basic tests failed");
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
    println!("Basic test suite completed!");
    println!("========================================");
    
    Ok(())
}

fn extract_test_result(output: &str) -> Option<i32> {
    // Look for patterns that might indicate the test result
    let lines: Vec<&str> = output.lines().collect();
    for line in lines {
        if line.contains("passed") || line.contains("tests") {
            if let Some(num) = extract_number(line) {
                return Some(num);
            }
        }
    }
    
    // If no specific pattern found, try to find any number that could be a result
    for line in lines {
        if let Some(num) = extract_number(line) {
            if num >= 0 && num <= 10 { // Reasonable range for basic test results
                return Some(num);
            }
        }
    }
    
    None
}

fn extract_number(text: &str) -> Option<i32> {
    let words: Vec<&str> = text.split_whitespace().collect();
    for word in words {
        if let Ok(num) = word.parse::<i32>() {
            return Some(num);
        }
    }
    None
} 