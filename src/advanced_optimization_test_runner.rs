use std::fs;
use std::path::Path;
use std::time::Instant;
use neksisc::compiler::{FastCompiler, CompilerOptions};
use neksisc::optimization_analysis::OptimizationAnalyzer;
use neksisc::optimizer::OptimizationLevel;

pub struct AdvancedOptimizationTestRunner {
    compiler: FastCompiler,
    analyzer: OptimizationAnalyzer,
}

impl AdvancedOptimizationTestRunner {
    pub fn new() -> Self {
        let options = CompilerOptions {
            optimization_level: 3, // Aggressive optimization
            incremental: true,
            parallel: true,
            cache_enabled: true,
            max_workers: 4,
        };
        
        Self {
            compiler: FastCompiler::new(options),
            analyzer: OptimizationAnalyzer::new(),
        }
    }
    
    pub fn run_advanced_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("=== Advanced Optimization Test Suite ===");
        println!();
        
        // Test 1: Matrix Operations
        self.test_matrix_optimizations()?;
        
        // Test 2: Recursive Algorithms
        self.test_recursive_optimizations()?;
        
        // Test 3: Sorting Algorithms
        self.test_sorting_optimizations()?;
        
        // Test 4: Mathematical Computations
        self.test_math_optimizations()?;
        
        // Test 5: String Processing
        self.test_string_optimizations()?;
        
        // Test 6: Function Composition
        self.test_function_composition()?;
        
        // Test 7: Vector Operations
        self.test_vector_optimizations()?;
        
        // Test 8: Memory-Intensive Operations
        self.test_memory_optimizations()?;
        
        // Test 9: Complex Conditional Logic
        self.test_conditional_optimizations()?;
        
        // Test 10: Cache-Friendly Operations
        self.test_cache_optimizations()?;
        
        println!("=== Advanced Optimization Tests Complete ===");
        Ok(())
    }
    
    fn test_matrix_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Matrix Operations Optimization...");
        
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
        
        fn main() -> Int {
            let matrix_a = Matrix { data: Array::new(), rows: 2, cols: 2 };
            let matrix_b = Matrix { data: Array::new(), rows: 2, cols: 2 };
            matrix_multiply(matrix_a, matrix_b);
            42
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        let report = self.analyzer.generate_optimization_report();
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Optimization opportunities found: {}", analysis.optimization_opportunities.len());
        println!("  Loop optimization candidates: {}", 
                 analysis.control_flow.loops.len());
        
        Ok(())
    }
    
    fn test_recursive_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Recursive Algorithm Optimization...");
        
        let source = r#"
        fn fibonacci_tail(n: Int, a: Int, b: Int) -> Int {
            if n <= 1 {
                b
            } else {
                fibonacci_tail(n - 1, b, a + b)
            }
        }
        
        fn fibonacci(n: Int) -> Int {
            fibonacci_tail(n, 0, 1)
        }
        
        fn main() -> Int {
            fibonacci(10)
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Recursive functions detected: {}", 
                 analysis.call_graph.nodes.values()
                     .filter(|node| node.recursive)
                     .count());
        println!("  Tail call optimization opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::TailCallOptimization)
                     .count());
        
        Ok(())
    }
    
    fn test_sorting_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Sorting Algorithm Optimization...");
        
        let source = r#"
        fn quicksort(arr: Array<Int>, low: Int, high: Int) -> Array<Int> {
            if low < high {
                let pivot = partition(arr, low, high);
                quicksort(arr, low, pivot - 1);
                quicksort(arr, pivot + 1, high);
            }
            arr
        }
        
        fn partition(arr: Array<Int>, low: Int, high: Int) -> Int {
            let pivot = arr[high];
            let i = low - 1;
            let j = low;
            
            while j < high {
                if arr[j] <= pivot {
                    i = i + 1;
                    let temp = arr[i];
                    arr[i] = arr[j];
                    arr[j] = temp;
                }
                j = j + 1;
            }
            
            let temp = arr[i + 1];
            arr[i + 1] = arr[high];
            arr[high] = temp;
            
            i + 1
        }
        
        fn main() -> Int {
            let arr = Array::new();
            quicksort(arr, 0, 0);
            42
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Function calls in call graph: {}", analysis.call_graph.edges.len());
        println!("  Loop optimization opportunities: {}", 
                 analysis.control_flow.loops.len());
        
        Ok(())
    }
    
    fn test_math_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Mathematical Computation Optimization...");
        
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
            let x = 2.0 * 3.14159;
            let y = 1.0 / 2.0;
            let z = (x + y) * (x - y);
            z
        }
        
        fn main() -> Int {
            compute_pi(100);
            complex_math();
            42
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Constant folding opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::ConstantFolding)
                     .count());
        println!("  Strength reduction opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::StrengthReduction)
                     .count());
        
        Ok(())
    }
    
    fn test_string_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing String Processing Optimization...");
        
        let source = r#"
        fn reverse_string(s: String) -> String {
            let result = "";
            let i = s.length() - 1;
            while i >= 0 {
                result = result + s[i];
                i = i - 1;
            }
            result
        }
        
        fn main() -> Int {
            reverse_string("hello world");
            42
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Loop optimization opportunities: {}", 
                 analysis.control_flow.loops.len());
        println!("  String operation optimizations: {}", 
                 analysis.optimization_opportunities.len());
        
        Ok(())
    }
    
    fn test_function_composition(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Function Composition Optimization...");
        
        let source = r#"
        fn square(x: Int) -> Int {
            x * x
        }
        
        fn cube(x: Int) -> Int {
            x * x * x
        }
        
        fn power4(x: Int) -> Int {
            square(square(x))
        }
        
        fn main() -> Int {
            power4(3)
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Function inlining candidates: {}", 
                 analysis.call_graph.nodes.values()
                     .filter(|node| node.inlining_candidate)
                     .count());
        println!("  Function composition opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::FunctionInlining)
                     .count());
        
        Ok(())
    }
    
    fn test_vector_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Vector Operations Optimization...");
        
        let source = r#"
        fn vector_add(a: Array<Float>, b: Array<Float>) -> Array<Float> {
            let result = Array::new();
            let i = 0;
            while i < a.length() {
                result.push(a[i] + b[i]);
                i = i + 1;
            }
            result
        }
        
        fn vector_multiply(a: Array<Float>, b: Array<Float>) -> Array<Float> {
            let result = Array::new();
            let i = 0;
            while i < a.length() {
                result.push(a[i] * b[i]);
                i = i + 1;
            }
            result
        }
        
        fn main() -> Int {
            let vec_a = Array::new();
            let vec_b = Array::new();
            vector_add(vec_a, vec_b);
            vector_multiply(vec_a, vec_b);
            42
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Vectorization opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::Vectorization)
                     .count());
        println!("  SIMD optimization candidates: {}", 
                 analysis.control_flow.loops.len());
        
        Ok(())
    }
    
    fn test_memory_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Memory-Intensive Operations...");
        
        let source = r#"
        fn create_large_array(size: Int) -> Array<Int> {
            let arr = Array::new();
            let i = 0;
            while i < size {
                arr.push(i * i);
                i = i + 1;
            }
            arr
        }
        
        fn main() -> Int {
            create_large_array(1000);
            42
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Memory optimization opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::MemoryOptimization)
                     .count());
        println!("  Strength reduction opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::StrengthReduction)
                     .count());
        
        Ok(())
    }
    
    fn test_conditional_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Complex Conditional Logic...");
        
        let source = r#"
        fn complex_condition(x: Int, y: Int) -> Int {
            let result = 0;
            
            if x > 100 && y < 50 {
                result = 1;
            } else if x <= 100 && y >= 50 {
                result = 2;
            } else if false {
                result = 999;
            }
            
            if x == 0 || y == 0 {
                result = result + 10;
            }
            
            result
        }
        
        fn main() -> Int {
            complex_condition(50, 25)
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Dead code elimination opportunities: {}", 
                 analysis.optimization_opportunities.iter()
                     .filter(|opp| opp.opportunity_type == neksisc::optimization_analysis::OptimizationType::DeadCodeElimination)
                     .count());
        println!("  Control flow optimizations: {}", 
                 analysis.control_flow.basic_blocks.len());
        
        Ok(())
    }
    
    fn test_cache_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Cache-Friendly Operations...");
        
        let source = r#"
        struct Matrix {
            data: Array<Array<Float>>,
            rows: Int,
            cols: Int,
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
            let matrix = Matrix { data: Array::new(), rows: 2, cols: 2 };
            matrix_transpose(matrix);
            42
        }
        "#;
        
        let start = Instant::now();
        let result = self.compiler.compile(source)?;
        let compile_time = start.elapsed();
        
        let analysis = self.analyzer.analyze_program(&result.ast)?;
        
        println!("  Compile time: {:?}", compile_time);
        println!("  Cache optimization opportunities: {}", 
                 analysis.optimization_opportunities.len());
        println!("  Loop optimization candidates: {}", 
                 analysis.control_flow.loops.len());
        
        Ok(())
    }
    
    pub fn generate_performance_report(&self) -> String {
        let stats = self.compiler.get_optimization_stats();
        
        format!(
            "=== Advanced Optimization Performance Report ===\n\
             Transformations made: {}\n\
             Code size reduction: {} -> {} ({}%)\n\
             Optimization time: {:?}\n\
             Passes applied: {}\n",
            stats.transformations_made,
            stats.code_size_before,
            stats.code_size_after,
            if stats.code_size_before > 0 {
                ((stats.code_size_before - stats.code_size_after) * 100) / stats.code_size_before
            } else {
                0
            },
            stats.optimization_time,
            stats.passes_applied.join(", ")
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runner = AdvancedOptimizationTestRunner::new();
    runner.run_advanced_tests()?;
    
    println!();
    println!("{}", runner.generate_performance_report());
    
    Ok(())
} 