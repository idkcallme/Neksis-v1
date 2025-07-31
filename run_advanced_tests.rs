use std::fs;
use std::time::Instant;
use neksisc::compiler::{FastCompiler, CompilerOptions};
use neksisc::optimization_analysis::OptimizationAnalyzer;
use neksisc::optimizer::{OptimizationLevel, Optimizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Advanced Optimization Test Runner ===");
    println!();
    
    // Test 1: Basic optimization test
    test_basic_optimizations()?;
    
    // Test 2: Advanced optimization test
    test_advanced_optimizations()?;
    
    // Test 3: Matrix operations optimization
    test_matrix_optimizations()?;
    
    // Test 4: Recursive algorithms optimization
    test_recursive_optimizations()?;
    
    // Test 5: Mathematical computations optimization
    test_math_optimizations()?;
    
    println!("=== All Advanced Tests Complete ===");
    Ok(())
}

fn test_basic_optimizations() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Basic Optimizations...");
    
    let source = r#"
    fn add(a: Int, b: Int) -> Int {
        a + b
    }
    
    fn constant_math() -> Int {
        let x = 5 + 3 * 2  // Should be folded to 11
        let y = 10 / 2     // Should be folded to 5
        x + y
    }
    
    fn dead_code_test() -> Int {
        let result = 42
        if false {
            result = 100  // This should be eliminated
        }
        result
    }
    
    fn main() -> Int {
        let result1 = add(5, 3)
        let result2 = constant_math()
        let result3 = dead_code_test()
        result1 + result2 + result3
    }
    "#;
    
    run_optimization_test("Basic Optimizations", source)
}

fn test_advanced_optimizations() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Advanced Optimizations...");
    
    if let Ok(source) = fs::read_to_string("advanced_test.nx") {
        run_optimization_test("Advanced Optimizations", &source)
    } else {
        println!("  Skipping advanced test (file not found)");
        Ok(())
    }
}

fn test_matrix_optimizations() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Matrix Operations...");
    
    let source = r#"
    struct Matrix {
        data: Array<Array<Float>>,
        rows: Int,
        cols: Int,
    }
    
    fn matrix_multiply(a: Matrix, b: Matrix) -> Matrix {
        let result = Matrix {
            data: Array::new(),
            rows: a.rows,
            cols: b.cols,
        };
        
        let i = 0;
        while i < a.rows {
            let j = 0;
            while j < b.cols {
                let k = 0;
                let sum = 0.0;
                while k < a.cols {
                    sum = sum + a.data[i][k] * b.data[k][j];
                    k = k + 1;
                }
                result.data[i][j] = sum;
                j = j + 1;
            }
            i = i + 1;
        }
        
        result
    }
    
    fn matrix_transpose(m: Matrix) -> Matrix {
        let result = Matrix {
            data: Array::new(),
            rows: m.cols,
            cols: m.rows,
        };
        
        let i = 0;
        while i < m.rows {
            let j = 0;
            while j < m.cols {
                result.data[j][i] = m.data[i][j];
                j = j + 1;
            }
            i = i + 1;
        }
        
        result
    }
    
    fn main() -> Int {
        let matrix_a = Matrix { data: Array::new(), rows: 2, cols: 2 };
        let matrix_b = Matrix { data: Array::new(), rows: 2, cols: 2 };
        matrix_multiply(matrix_a, matrix_b);
        matrix_transpose(matrix_a);
        42
    }
    "#;
    
    run_optimization_test("Matrix Operations", source)
}

fn test_recursive_optimizations() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Recursive Algorithms...");
    
    let source = r#"
    fn fibonacci_tail(n: Int, a: Int, b: Int) -> Int {
        if n <= 1 {
            b
        } else {
            fibonacci_tail(n - 1, b, a + b)  // Tail call
        }
    }
    
    fn factorial_tail(n: Int, acc: Int) -> Int {
        if n <= 1 {
            acc
        } else {
            factorial_tail(n - 1, n * acc)  // Tail call
        }
    }
    
    fn fibonacci(n: Int) -> Int {
        fibonacci_tail(n, 0, 1)
    }
    
    fn factorial(n: Int) -> Int {
        factorial_tail(n, 1)
    }
    
    fn main() -> Int {
        fibonacci(10) + factorial(5)
    }
    "#;
    
    run_optimization_test("Recursive Algorithms", source)
}

fn test_math_optimizations() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Mathematical Computations...");
    
    let source = r#"
    fn compute_pi(iterations: Int) -> Float {
        let pi = 0.0;
        let i = 0;
        while i < iterations {
            let term = 4.0 / (2.0 * i + 1.0);
            if i % 2 == 0 {
                pi = pi + term;
            } else {
                pi = pi - term;
            }
            i = i + 1;
        }
        pi
    }
    
    fn complex_math() -> Float {
        let x = 2.0 * 3.14159;  // Should be folded
        let y = 1.0 / 2.0;      // Should be folded
        let z = (x + y) * (x - y);  // Complex expression
        z
    }
    
    fn power_series(x: Float, terms: Int) -> Float {
        let result = 0.0;
        let i = 0;
        while i < terms {
            let term = x.pow(i);
            result = result + term;
            i = i + 1;
        }
        result
    }
    
    fn main() -> Int {
        compute_pi(100);
        complex_math();
        power_series(2.0, 10);
        42
    }
    "#;
    
    run_optimization_test("Mathematical Computations", source)
}

fn run_optimization_test(test_name: &str, source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let options = CompilerOptions {
        optimization_level: 3, // Aggressive optimization
        incremental: true,
        parallel: true,
        cache_enabled: true,
        max_workers: 4,
    };
    
    let mut compiler = FastCompiler::new(options);
    let mut analyzer = OptimizationAnalyzer::new();
    let mut optimizer = Optimizer::new(options);
    
    let start_time = Instant::now();
    
    // Compile the program
    let result = compiler.compile(source)?;
    let compilation_time = start_time.elapsed();
    
    // Analyze the program
    let analysis = analyzer.analyze_program(&result.ast)?;
    
    // Optimize the program
    let mut optimized_program = result.ast.clone();
    optimizer.optimize(&mut optimized_program)?;
    
    let optimization_time = optimizer.get_optimization_stats().optimization_time;
    let stats = optimizer.get_optimization_stats();
    
    println!("  ✓ {} test completed:", test_name);
    println!("    - Compilation time: {:?}", compilation_time);
    println!("    - Optimization time: {:?}", optimization_time);
    println!("    - Transformations made: {}", stats.transformations_made);
    println!("    - Code size: {} -> {} ({}% reduction)", 
             stats.code_size_before,
             stats.code_size_after,
             if stats.code_size_before > 0 {
                 ((stats.code_size_before - stats.code_size_after) * 100) / stats.code_size_before
             } else {
                 0
             });
    println!("    - Functions analyzed: {}", analysis.call_graph.nodes.len());
    println!("    - Loops detected: {}", analysis.control_flow.loops.len());
    println!("    - Optimization opportunities: {}", analysis.optimization_opportunities.len());
    
    Ok(())
} 