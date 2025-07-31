use std::fs;
use std::path::Path;
use std::time::Instant;
use neksisc::compiler::{FastCompiler, CompilerOptions};
use neksisc::optimization_analysis::OptimizationAnalyzer;
use neksisc::optimizer::{OptimizationLevel, Optimizer};
use crate::enhanced_optimization_analyzer::{EnhancedOptimizationAnalyzer, EnhancedOptimizationReport};

pub struct ComprehensiveOptimizationRunner {
    enhanced_analyzer: EnhancedOptimizationAnalyzer,
    basic_analyzer: OptimizationAnalyzer,
    compiler: FastCompiler,
    optimizer: Optimizer,
}

impl ComprehensiveOptimizationRunner {
    pub fn new() -> Self {
        let options = CompilerOptions {
            optimization_level: 3, // Aggressive optimization
            incremental: true,
            parallel: true,
            cache_enabled: true,
            max_workers: 4,
        };
        
        Self {
            enhanced_analyzer: EnhancedOptimizationAnalyzer::new(),
            basic_analyzer: OptimizationAnalyzer::new(),
            compiler: FastCompiler::new(options),
            optimizer: Optimizer::new(options),
        }
    }
    
    pub fn run_comprehensive_tests(&mut self) -> Result<ComprehensiveTestResults, Box<dyn std::error::Error>> {
        println!("=== Comprehensive Optimization Test Suite ===");
        println!();
        
        let mut results = ComprehensiveTestResults::new();
        
        // Test 1: Basic Optimization Test
        results.basic_test = self.run_basic_optimization_test()?;
        
        // Test 2: Advanced Optimization Test
        results.advanced_test = self.run_advanced_optimization_test()?;
        
        // Test 3: Matrix Operations Test
        results.matrix_test = self.run_matrix_optimization_test()?;
        
        // Test 4: Recursive Algorithms Test
        results.recursive_test = self.run_recursive_optimization_test()?;
        
        // Test 5: Sorting Algorithms Test
        results.sorting_test = self.run_sorting_optimization_test()?;
        
        // Test 6: Mathematical Computations Test
        results.math_test = self.run_math_optimization_test()?;
        
        // Test 7: String Processing Test
        results.string_test = self.run_string_optimization_test()?;
        
        // Test 8: Function Composition Test
        results.function_composition_test = self.run_function_composition_test()?;
        
        // Test 9: Vector Operations Test
        results.vector_test = self.run_vector_optimization_test()?;
        
        // Test 10: Memory-Intensive Operations Test
        results.memory_test = self.run_memory_optimization_test()?;
        
        // Test 11: Complex Conditional Logic Test
        results.conditional_test = self.run_conditional_optimization_test()?;
        
        // Test 12: Cache-Friendly Operations Test
        results.cache_test = self.run_cache_optimization_test()?;
        
        // Generate comprehensive report
        results.generate_comprehensive_report();
        
        println!("=== Comprehensive Optimization Tests Complete ===");
        println!();
        println!("{}", results.generate_summary_report());
        
        Ok(results)
    }
    
    fn run_basic_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Basic Optimization Test...");
        
        let source = r#"
        fn add(a: Int, b: Int) -> Int {
            a + b
        }
        
        fn constant_math() -> Int {
            let x = 5 + 3 * 2
            let y = 10 / 2
            x + y
        }
        
        fn dead_code_test() -> Int {
            let result = 42
            if false {
                result = 100
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
        
        self.run_single_test("Basic Optimization", source)
    }
    
    fn run_advanced_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Advanced Optimization Test...");
        
        let source = fs::read_to_string("advanced_test.nx")?;
        self.run_single_test("Advanced Optimization", &source)
    }
    
    fn run_matrix_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Matrix Operations Test...");
        
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
        
        self.run_single_test("Matrix Operations", source)
    }
    
    fn run_recursive_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Recursive Algorithms Test...");
        
        let source = r#"
        fn fibonacci_tail(n: Int, a: Int, b: Int) -> Int {
            if n <= 1 {
                b
            } else {
                fibonacci_tail(n - 1, b, a + b)
            }
        }
        
        fn factorial_tail(n: Int, acc: Int) -> Int {
            if n <= 1 {
                acc
            } else {
                factorial_tail(n - 1, n * acc)
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
        
        self.run_single_test("Recursive Algorithms", source)
    }
    
    fn run_sorting_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Sorting Algorithms Test...");
        
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
        
        fn bubble_sort(arr: Array<Int>) -> Array<Int> {
            let n = arr.length();
            let i = 0;
            while i < n {
                let j = 0;
                while j < n - i - 1 {
                    if arr[j] > arr[j + 1] {
                        let temp = arr[j];
                        arr[j] = arr[j + 1];
                        arr[j + 1] = temp;
                    }
                    j = j + 1;
                }
                i = i + 1;
            }
            arr
        }
        
        fn main() -> Int {
            let arr1 = Array::new();
            let arr2 = Array::new();
            quicksort(arr1, 0, 0);
            bubble_sort(arr2);
            42
        }
        "#;
        
        self.run_single_test("Sorting Algorithms", source)
    }
    
    fn run_math_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Mathematical Computations Test...");
        
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
        
        self.run_single_test("Mathematical Computations", source)
    }
    
    fn run_string_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running String Processing Test...");
        
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
        
        fn count_chars(s: String, target: String) -> Int {
            let count = 0;
            let i = 0;
            while i < s.length() {
                if s[i] == target[0] {
                    count = count + 1;
                }
                i = i + 1;
            }
            count
        }
        
        fn substring(s: String, start: Int, end: Int) -> String {
            let result = "";
            let i = start;
            while i < end && i < s.length() {
                result = result + s[i];
                i = i + 1;
            }
            result
        }
        
        fn main() -> Int {
            reverse_string("hello world");
            count_chars("hello world", "l");
            substring("hello world", 0, 5);
            42
        }
        "#;
        
        self.run_single_test("String Processing", source)
    }
    
    fn run_function_composition_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Function Composition Test...");
        
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
        
        fn compose(f: fn(Int) -> Int, g: fn(Int) -> Int, x: Int) -> Int {
            f(g(x))
        }
        
        fn apply_twice(f: fn(Int) -> Int, x: Int) -> Int {
            f(f(x))
        }
        
        fn main() -> Int {
            power4(3);
            compose(square, cube, 2);
            apply_twice(square, 3);
            42
        }
        "#;
        
        self.run_single_test("Function Composition", source)
    }
    
    fn run_vector_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Vector Operations Test...");
        
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
        
        fn vector_dot_product(a: Array<Float>, b: Array<Float>) -> Float {
            let result = 0.0;
            let i = 0;
            while i < a.length() {
                result = result + a[i] * b[i];
                i = i + 1;
            }
            result
        }
        
        fn main() -> Int {
            let vec_a = Array::new();
            let vec_b = Array::new();
            vector_add(vec_a, vec_b);
            vector_multiply(vec_a, vec_b);
            vector_dot_product(vec_a, vec_b);
            42
        }
        "#;
        
        self.run_single_test("Vector Operations", source)
    }
    
    fn run_memory_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Memory-Intensive Operations Test...");
        
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
        
        fn process_large_data(data: Array<Int>) -> Array<Int> {
            let result = Array::new();
            let i = 0;
            while i < data.length() {
                result.push(data[i] * 2 + 1);
                i = i + 1;
            }
            result
        }
        
        fn memory_intensive_operation(size: Int) -> Array<Int> {
            let arr1 = create_large_array(size);
            let arr2 = create_large_array(size);
            let result = Array::new();
            let i = 0;
            while i < size {
                result.push(arr1[i] + arr2[i]);
                i = i + 1;
            }
            result
        }
        
        fn main() -> Int {
            create_large_array(1000);
            memory_intensive_operation(500);
            42
        }
        "#;
        
        self.run_single_test("Memory-Intensive Operations", source)
    }
    
    fn run_conditional_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Complex Conditional Logic Test...");
        
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
        
        fn nested_conditions(a: Int, b: Int, c: Int) -> Int {
            let result = 0;
            
            if a > 0 {
                if b > 0 {
                    if c > 0 {
                        result = 1;
                    } else {
                        result = 2;
                    }
                } else {
                    result = 3;
                }
            } else {
                result = 4;
            }
            
            result
        }
        
        fn switch_like_condition(x: Int) -> Int {
            let result = 0;
            
            if x == 1 {
                result = 10;
            } else if x == 2 {
                result = 20;
            } else if x == 3 {
                result = 30;
            } else {
                result = 0;
            }
            
            result
        }
        
        fn main() -> Int {
            complex_condition(50, 25);
            nested_conditions(1, 1, 1);
            switch_like_condition(2);
            42
        }
        "#;
        
        self.run_single_test("Complex Conditional Logic", source)
    }
    
    fn run_cache_optimization_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        println!("Running Cache-Friendly Operations Test...");
        
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
        
        fn cache_friendly_sum(arr: Array<Array<Int>>) -> Int {
            let sum = 0;
            let i = 0;
            while i < arr.length() {
                let j = 0;
                while j < arr[i].length() {
                    sum = sum + arr[i][j];
                    j = j + 1;
                }
                i = i + 1;
            }
            sum
        }
        
        fn main() -> Int {
            let matrix = Matrix { data: Array::new(), rows: 2, cols: 2 };
            matrix_transpose(matrix);
            let arr = Array::new();
            cache_friendly_sum(arr);
            42
        }
        "#;
        
        self.run_single_test("Cache-Friendly Operations", source)
    }
    
    fn run_single_test(&mut self, test_name: &str, source: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Compile and analyze
        let compilation_result = self.compiler.compile(source)?;
        let analysis = self.basic_analyzer.analyze_program(&compilation_result.ast)?;
        
        // Optimize
        let mut optimized_program = compilation_result.ast.clone();
        self.optimizer.optimize(&mut optimized_program)?;
        
        let total_time = start_time.elapsed();
        let stats = self.optimizer.get_optimization_stats();
        
        Ok(TestResult {
            test_name: test_name.to_string(),
            compilation_time: total_time,
            optimization_time: stats.optimization_time,
            transformations_made: stats.transformations_made,
            code_size_before: stats.code_size_before,
            code_size_after: stats.code_size_after,
            functions_analyzed: analysis.call_graph.nodes.len(),
            function_calls: analysis.call_graph.edges.len(),
            loops_detected: analysis.control_flow.loops.len(),
            optimization_opportunities: analysis.optimization_opportunities.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub compilation_time: std::time::Duration,
    pub optimization_time: std::time::Duration,
    pub transformations_made: usize,
    pub code_size_before: usize,
    pub code_size_after: usize,
    pub functions_analyzed: usize,
    pub function_calls: usize,
    pub loops_detected: usize,
    pub optimization_opportunities: usize,
}

impl TestResult {
    pub fn code_size_reduction_percentage(&self) -> f64 {
        if self.code_size_before > 0 {
            ((self.code_size_before - self.code_size_after) as f64 / self.code_size_before as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn optimization_effectiveness(&self) -> f64 {
        let size_reduction = self.code_size_reduction_percentage();
        let transformation_efficiency = if self.transformations_made > 0 {
            (self.optimization_opportunities as f64 / self.transformations_made as f64) * 100.0
        } else {
            0.0
        };
        
        (size_reduction + transformation_efficiency) / 2.0
    }
}

#[derive(Debug)]
pub struct ComprehensiveTestResults {
    pub basic_test: TestResult,
    pub advanced_test: TestResult,
    pub matrix_test: TestResult,
    pub recursive_test: TestResult,
    pub sorting_test: TestResult,
    pub math_test: TestResult,
    pub string_test: TestResult,
    pub function_composition_test: TestResult,
    pub vector_test: TestResult,
    pub memory_test: TestResult,
    pub conditional_test: TestResult,
    pub cache_test: TestResult,
    pub summary_report: String,
}

impl ComprehensiveTestResults {
    pub fn new() -> Self {
        Self {
            basic_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            advanced_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            matrix_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            recursive_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            sorting_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            math_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            string_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            function_composition_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            vector_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            memory_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            conditional_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            cache_test: TestResult {
                test_name: "".to_string(),
                compilation_time: std::time::Duration::from_millis(0),
                optimization_time: std::time::Duration::from_millis(0),
                transformations_made: 0,
                code_size_before: 0,
                code_size_after: 0,
                functions_analyzed: 0,
                function_calls: 0,
                loops_detected: 0,
                optimization_opportunities: 0,
            },
            summary_report: String::new(),
        }
    }
    
    pub fn generate_comprehensive_report(&mut self) {
        let all_tests = vec![
            &self.basic_test,
            &self.advanced_test,
            &self.matrix_test,
            &self.recursive_test,
            &self.sorting_test,
            &self.math_test,
            &self.string_test,
            &self.function_composition_test,
            &self.vector_test,
            &self.memory_test,
            &self.conditional_test,
            &self.cache_test,
        ];
        
        let total_compilation_time: std::time::Duration = all_tests.iter()
            .map(|test| test.compilation_time)
            .sum();
        
        let total_optimization_time: std::time::Duration = all_tests.iter()
            .map(|test| test.optimization_time)
            .sum();
        
        let total_transformations: usize = all_tests.iter()
            .map(|test| test.transformations_made)
            .sum();
        
        let total_functions: usize = all_tests.iter()
            .map(|test| test.functions_analyzed)
            .sum();
        
        let total_loops: usize = all_tests.iter()
            .map(|test| test.loops_detected)
            .sum();
        
        let total_opportunities: usize = all_tests.iter()
            .map(|test| test.optimization_opportunities)
            .sum();
        
        let avg_code_size_reduction: f64 = all_tests.iter()
            .map(|test| test.code_size_reduction_percentage())
            .sum::<f64>() / all_tests.len() as f64;
        
        let avg_effectiveness: f64 = all_tests.iter()
            .map(|test| test.optimization_effectiveness())
            .sum::<f64>() / all_tests.len() as f64;
        
        self.summary_report = format!(
            "=== Comprehensive Optimization Test Summary ===\n\
             \n\
             OVERALL STATISTICS:\n\
             - Total compilation time: {:?}\n\
             - Total optimization time: {:?}\n\
             - Total transformations made: {}\n\
             - Total functions analyzed: {}\n\
             - Total loops detected: {}\n\
             - Total optimization opportunities: {}\n\
             \n\
             AVERAGE METRICS:\n\
             - Average code size reduction: {:.2}%\n\
             - Average optimization effectiveness: {:.2}%\n\
             \n\
             TEST BREAKDOWN:\n\
             {}",
            total_compilation_time,
            total_optimization_time,
            total_transformations,
            total_functions,
            total_loops,
            total_opportunities,
            avg_code_size_reduction,
            avg_effectiveness,
            self.generate_test_breakdown(),
        );
    }
    
    fn generate_test_breakdown(&self) -> String {
        let tests = vec![
            ("Basic Optimization", &self.basic_test),
            ("Advanced Optimization", &self.advanced_test),
            ("Matrix Operations", &self.matrix_test),
            ("Recursive Algorithms", &self.recursive_test),
            ("Sorting Algorithms", &self.sorting_test),
            ("Mathematical Computations", &self.math_test),
            ("String Processing", &self.string_test),
            ("Function Composition", &self.function_composition_test),
            ("Vector Operations", &self.vector_test),
            ("Memory-Intensive Operations", &self.memory_test),
            ("Complex Conditional Logic", &self.conditional_test),
            ("Cache-Friendly Operations", &self.cache_test),
        ];
        
        tests.iter()
            .map(|(name, test)| {
                format!(
                    "  {}:\n\
                       - Compilation time: {:?}\n\
                       - Transformations: {}\n\
                       - Code size reduction: {:.2}%\n\
                       - Effectiveness: {:.2}%\n\
                       - Functions: {}, Loops: {}, Opportunities: {}\n",
                    name,
                    test.compilation_time,
                    test.transformations_made,
                    test.code_size_reduction_percentage(),
                    test.optimization_effectiveness(),
                    test.functions_analyzed,
                    test.loops_detected,
                    test.optimization_opportunities,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    pub fn generate_summary_report(&self) -> String {
        self.summary_report.clone()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runner = ComprehensiveOptimizationRunner::new();
    let _results = runner.run_comprehensive_tests()?;
    
    Ok(())
} 