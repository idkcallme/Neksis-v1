use neksisc::compiler::{FastCompiler, CompilerOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== neksis Compiler Optimization Test ===\n");

    // Test source code with optimization opportunities
    let source_code = r#"
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn constant_math() -> Int {
    let x = 5 + 3 * 2
    let y = 10 / 2
    x + y
}

fn main() -> Int {
    let result1 = add(5, 3)
    let result2 = constant_math()
    result1 + result2
}
"#;

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
        match compiler.compile(source_code) {
            Ok(output) => {
                println!("Compilation successful!");
                
                // Get optimization statistics
                let opt_stats = compiler.get_optimization_stats();
                println!("Optimization passes applied: {}", opt_stats.passes_applied.len());
                println!("Transformations made: {}", opt_stats.transformations_made);
                println!("Code size before: {}", opt_stats.code_size_before);
                println!("Code size after: {}", opt_stats.code_size_after);
                println!("Optimization time: {:?}", opt_stats.optimization_time);
                
                // Show generated code (first 300 characters)
                let code_preview = if output.len() > 300 {
                    format!("{}...", &output[..300])
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
    
    if let Ok(_) = compiler.compile(source_code) {
        let report = compiler.get_optimization_report();
        println!("{}", report);
    }

    println!("\n=== Optimization Test Complete ===");
    Ok(())
} 