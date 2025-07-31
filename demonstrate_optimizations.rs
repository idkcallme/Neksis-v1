use neksisc::compiler::{FastCompiler, CompilerOptions};
use neksisc::optimizer::OptimizationLevel;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== neksis Compiler Optimization Demonstration ===\n");

    // Read the test file
    let source_code = fs::read_to_string("optimization_test.nx")?;
    println!("Source code loaded from optimization_test.nx\n");

    // Test different optimization levels
    let optimization_levels = vec![
        ("No Optimization", 0),
        ("Basic Optimization", 1),
        ("Standard Optimization", 2),
        ("Aggressive Optimization", 3),
    ];

    for (level_name, level) in optimization_levels {
        println!("=== {} ===", level_name);
        
        // Create compiler with specific optimization level
        let options = CompilerOptions {
            optimization_level: level,
            ..Default::default()
        };
        
        let mut compiler = FastCompiler::new(options);
        
        // Compile the code
        match compiler.compile(&source_code) {
            Ok(output) => {
                println!("Compilation successful!");
                
                // Get optimization statistics
                let opt_stats = compiler.get_optimization_stats();
                println!("Optimization passes applied: {}", opt_stats.passes_applied.len());
                println!("Transformations made: {}", opt_stats.transformations_made);
                println!("Code size before: {}", opt_stats.code_size_before);
                println!("Code size after: {}", opt_stats.code_size_after);
                println!("Optimization time: {:?}", opt_stats.optimization_time);
                
                // Get detailed optimization report
                let report = compiler.get_optimization_report();
                println!("\nOptimization Analysis Report:");
                println!("{}", report);
                
                // Show generated code (first 500 characters)
                let code_preview = if output.len() > 500 {
                    format!("{}...", &output[..500])
                } else {
                    output
                };
                println!("\nGenerated code preview:");
                println!("{}", code_preview);
            }
            Err(e) => {
                println!("Compilation failed: {:?}", e);
            }
        }
        
        println!("\n" + "=".repeat(50) + "\n");
    }

    // Demonstrate optimization analysis
    println!("=== Detailed Optimization Analysis ===");
    let options = CompilerOptions {
        optimization_level: 3, // Aggressive optimization
        ..Default::default()
    };
    
    let mut compiler = FastCompiler::new(options);
    
    if let Ok(_) = compiler.compile(&source_code) {
        let report = compiler.get_optimization_report();
        println!("{}", report);
    }

    println!("\n=== Optimization Demonstration Complete ===");
    Ok(())
} 