use neksisc::compiler::{FastCompiler, CompilerOptions};

fn main() {
    println!("=== neksis Compiler Optimization Demo ===\n");

    // Simple test code
    let source_code = "fn main() -> Int { 5 + 3 }";

    // Test with optimization level 2
    let options = CompilerOptions {
        optimization_level: 2,
        ..Default::default()
    };
    
    let mut compiler = FastCompiler::new(options);
    
    match compiler.compile(source_code) {
        Ok(_output) => {
            println!("✓ Compilation successful!");
            
            // Get optimization statistics
            let opt_stats = compiler.get_optimization_stats();
            println!("✓ Optimization passes applied: {}", opt_stats.passes_applied.len());
            println!("✓ Transformations made: {}", opt_stats.transformations_made);
            println!("✓ Code size before: {}", opt_stats.code_size_before);
            println!("✓ Code size after: {}", opt_stats.code_size_after);
            println!("✓ Optimization time: {:?}", opt_stats.optimization_time);
            
            // Get optimization report
            let report = compiler.get_optimization_report();
            println!("\n=== Optimization Report ===");
            println!("{}", report);
        }
        Err(e) => {
            println!("✗ Compilation failed: {:?}", e);
        }
    }

    println!("\n=== Demo Complete ===");
} 