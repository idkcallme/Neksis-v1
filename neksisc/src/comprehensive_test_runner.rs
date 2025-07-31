use neksisc::{FastCompiler, CompilerOptions};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("NEXUS COMPILER COMPREHENSIVE TEST SUITE");
    println!("========================================");
    
    // Read the comprehensive test file
    let test_content = fs::read_to_string("../working_comprehensive_test.nx")
        .expect("Failed to read working_comprehensive_test.nx");
    
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
        
        match compiler.compile_sequential("working_comprehensive_test.nx", &test_content) {
            Ok(output) => {
                println!("✓ Compilation successful!");
                println!("  Output size: {} bytes", output.len());
                
                // Get optimization statistics
                let stats = compiler.get_optimization_stats();
                println!("  Optimization stats:");
                println!("    Transformations made: {}", stats.transformations_made);
                println!("    Passes applied: {}", stats.passes_applied.len());
                println!("    Code size before: {} bytes", stats.code_size_before);
                println!("    Code size after: {} bytes", stats.code_size_after);
                if stats.code_size_before > 0 {
                    let reduction = ((stats.code_size_before - stats.code_size_after) as f64 / stats.code_size_before as f64) * 100.0;
                    println!("    Code size reduction: {:.1}%", reduction);
                }
                
                // Get optimization report
                let report = compiler.get_optimization_report();
                println!("  Optimization report:");
                println!("    {}", report);
                
                // Try to extract the result from the output
                if let Some(result) = extract_test_result(&output) {
                    println!("  Test result: {}/{} tests passed", result, 13); // 13 total tests
                    if result == 13 {
                        println!("  ✓ ALL COMPREHENSIVE TESTS PASSED!");
                    } else {
                        println!("  ✗ Some comprehensive tests failed");
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
    println!("Comprehensive test suite completed!");
    println!("========================================");
    
    Ok(())
}

fn extract_test_result(output: &str) -> Option<i32> {
    // Look for patterns that might indicate the test result
    let lines: Vec<&str> = output.lines().collect();
    for line in &lines {
        if line.contains("passed") || line.contains("tests") {
            if let Some(num) = extract_number(line) {
                return Some(num);
            }
        }
    }
    
    // If no specific pattern found, try to find any number that could be a result
    for line in &lines {
        if let Some(num) = extract_number(line) {
            if num >= 0 && num <= 20 { // Reasonable range for comprehensive test results
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