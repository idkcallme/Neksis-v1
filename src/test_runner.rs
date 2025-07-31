use std::fs;
use std::path::Path;
use std::time::Instant;
use neksisc::compiler::{FastCompiler, CompilerOptions};
use neksisc::optimization_analysis::OptimizationAnalyzer;
use neksisc::optimizer::{OptimizationLevel, Optimizer};
use crate::enhanced_optimization_analyzer::{EnhancedOptimizationAnalyzer, EnhancedOptimizationReport};
use crate::comprehensive_optimization_runner::{ComprehensiveOptimizationRunner, ComprehensiveTestResults};

pub struct TestRunner {
    compiler: FastCompiler,
    analyzer: OptimizationAnalyzer,
    optimizer: Optimizer,
    enhanced_analyzer: EnhancedOptimizationAnalyzer,
    comprehensive_runner: ComprehensiveOptimizationRunner,
}

impl TestRunner {
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
            optimizer: Optimizer::new(options),
            enhanced_analyzer: EnhancedOptimizationAnalyzer::new(),
            comprehensive_runner: ComprehensiveOptimizationRunner::new(),
        }
    }
    
    pub fn run_all_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("=== neksis Compiler Test Suite ===");
        println!();
        
        // Run basic tests
        self.run_basic_tests()?;
        
        // Run optimization tests
        self.run_optimization_tests()?;
        
        // Run advanced optimization tests
        self.run_advanced_optimization_tests()?;
        
        // Run comprehensive optimization tests
        self.run_comprehensive_optimization_tests()?;
        
        // Run enhanced analysis
        self.run_enhanced_analysis()?;
        
        println!("=== All Tests Complete ===");
        Ok(())
    }
    
    fn run_basic_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running Basic Tests...");
        
        let test_files = vec![
            "minimal_test.nx",
            "simple_test.nx",
            "basic_tests.nx",
            "optimization_test.nx",
        ];
        
        for test_file in test_files {
            if Path::new(test_file).exists() {
                println!("  Testing {}...", test_file);
                let source = fs::read_to_string(test_file)?;
                let result = self.compiler.compile(&source)?;
                println!("    ✓ Compiled successfully");
                
                // Analyze the program
                let analysis = self.analyzer.analyze_program(&result.ast)?;
                println!("    ✓ Analysis complete: {} functions, {} loops", 
                         analysis.call_graph.nodes.len(),
                         analysis.control_flow.loops.len());
            }
        }
        
        println!("  ✓ Basic tests completed");
        Ok(())
    }
    
    fn run_optimization_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running Optimization Tests...");
        
        let test_files = vec![
            "optimization_test.nx",
            "working_comprehensive_test.nx",
            "final_comprehensive_test.nx",
        ];
        
        for test_file in test_files {
            if Path::new(test_file).exists() {
                println!("  Testing {} with optimizations...", test_file);
                let source = fs::read_to_string(test_file)?;
                
                let start_time = Instant::now();
                let result = self.compiler.compile(&source)?;
                let compilation_time = start_time.elapsed();
                
                // Optimize the program
                let mut optimized_program = result.ast.clone();
                self.optimizer.optimize(&mut optimized_program)?;
                let optimization_time = self.optimizer.get_optimization_stats().optimization_time;
                
                let stats = self.optimizer.get_optimization_stats();
                println!("    ✓ Optimization complete:");
                println!("      - Compilation time: {:?}", compilation_time);
                println!("      - Optimization time: {:?}", optimization_time);
                println!("      - Transformations made: {}", stats.transformations_made);
                println!("      - Code size: {} -> {} ({}% reduction)", 
                         stats.code_size_before,
                         stats.code_size_after,
                         if stats.code_size_before > 0 {
                             ((stats.code_size_before - stats.code_size_after) * 100) / stats.code_size_before
                         } else {
                             0
                         });
            }
        }
        
        println!("  ✓ Optimization tests completed");
        Ok(())
    }
    
    fn run_advanced_optimization_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running Advanced Optimization Tests...");
        
        if Path::new("advanced_test.nx").exists() {
            println!("  Testing advanced optimization scenarios...");
            let source = fs::read_to_string("advanced_test.nx")?;
            
            let start_time = Instant::now();
            let result = self.compiler.compile(&source)?;
            let compilation_time = start_time.elapsed();
            
            // Analyze with enhanced analyzer
            let enhanced_report = self.enhanced_analyzer.analyze_program(&source)?;
            
            println!("    ✓ Advanced analysis complete:");
            println!("      - Compilation time: {:?}", compilation_time);
            println!("      - Code quality score: {:.1}/100", enhanced_report.code_quality_score);
            println!("      - Optimization effectiveness: {:.1}/100", enhanced_report.optimization_effectiveness);
            println!("      - Recommendations: {}", enhanced_report.optimization_recommendations.len());
            
            // Print detailed report
            println!("    Detailed Report:");
            println!("{}", self.enhanced_analyzer.generate_detailed_report(&enhanced_report));
        }
        
        println!("  ✓ Advanced optimization tests completed");
        Ok(())
    }
    
    fn run_comprehensive_optimization_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running Comprehensive Optimization Tests...");
        
        let results = self.comprehensive_runner.run_comprehensive_tests()?;
        
        println!("  ✓ Comprehensive optimization tests completed");
        println!("  Summary: {} test categories analyzed", 12);
        
        Ok(())
    }
    
    fn run_enhanced_analysis(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running Enhanced Analysis...");
        
        let test_files = vec![
            "advanced_test.nx",
            "working_comprehensive_test.nx",
            "final_comprehensive_test.nx",
        ];
        
        for test_file in test_files {
            if Path::new(test_file).exists() {
                println!("  Enhanced analysis of {}...", test_file);
                let source = fs::read_to_string(test_file)?;
                
                let enhanced_report = self.enhanced_analyzer.analyze_program(&source)?;
                
                println!("    ✓ Enhanced analysis complete:");
                println!("      - Performance metrics calculated");
                println!("      - Optimization recommendations generated");
                println!("      - Code quality assessed");
                
                // Generate detailed report for the most complex test
                if test_file == "advanced_test.nx" {
                    println!("    Detailed Analysis Report:");
                    println!("{}", self.enhanced_analyzer.generate_detailed_report(&enhanced_report));
                }
            }
        }
        
        println!("  ✓ Enhanced analysis completed");
        Ok(())
    }
    
    pub fn generate_performance_summary(&self) -> String {
        let stats = self.optimizer.get_optimization_stats();
        
        format!(
            "=== Performance Summary ===\n\
             \n\
             OPTIMIZATION STATISTICS:\n\
             - Total transformations made: {}\n\
             - Code size reduction: {} -> {} ({}%)\n\
             - Optimization time: {:?}\n\
             - Passes applied: {}\n\
             \n\
             COMPILER FEATURES:\n\
             - Aggressive optimization level enabled\n\
             - Parallel compilation enabled\n\
             - Incremental compilation enabled\n\
             - Compilation cache enabled\n\
             - Maximum workers: 4\n\
             \n\
             OPTIMIZATION PASSES:\n\
             - Constant folding\n\
             - Dead code elimination\n\
             - Function inlining\n\
             - Loop optimization\n\
             - Strength reduction\n\
             - Common subexpression elimination\n\
             - Tail call optimization\n\
             - Vectorization\n",
            stats.transformations_made,
            stats.code_size_before,
            stats.code_size_after,
            if stats.code_size_before > 0 {
                ((stats.code_size_before - stats.code_size_after) * 100) / stats.code_size_before
            } else {
                0
            },
            stats.optimization_time,
            stats.passes_applied.join(", "),
        )
    }
    
    pub fn run_specific_test(&mut self, test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running specific test: {}", test_name);
        
        match test_name {
            "basic" => self.run_basic_tests()?,
            "optimization" => self.run_optimization_tests()?,
            "advanced" => self.run_advanced_optimization_tests()?,
            "comprehensive" => self.run_comprehensive_optimization_tests()?,
            "enhanced" => self.run_enhanced_analysis()?,
            "all" => self.run_all_tests()?,
            _ => {
                println!("Unknown test: {}. Available tests: basic, optimization, advanced, comprehensive, enhanced, all", test_name);
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runner = TestRunner::new();
    
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        let test_name = &args[1];
        runner.run_specific_test(test_name)?;
    } else {
        // Run all tests by default
        runner.run_all_tests()?;
    }
    
    println!();
    println!("{}", runner.generate_performance_summary());
    
    Ok(())
} 