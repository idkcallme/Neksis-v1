use std::collections::{HashMap, HashSet};
use std::time::Instant;
use neksisc::ast::{Program, Statement, Expression, FunctionStatement};
use neksisc::optimization_analysis::{OptimizationAnalyzer, OptimizationAnalysis, OptimizationType};
use neksisc::optimizer::{Optimizer, OptimizationLevel, OptimizationStats};
use neksisc::compiler::{FastCompiler, CompilerOptions};

#[derive(Debug, Clone)]
pub struct EnhancedOptimizationReport {
    pub overall_stats: OptimizationStats,
    pub detailed_analysis: OptimizationAnalysis,
    pub performance_metrics: PerformanceMetrics,
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    pub code_quality_score: f64,
    pub optimization_effectiveness: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub compilation_time: std::time::Duration,
    pub optimization_time: std::time::Duration,
    pub memory_usage: usize,
    pub code_size_reduction_percentage: f64,
    pub expected_runtime_improvement: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: OptimizationCategory,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_complexity: ImplementationComplexity,
    pub priority: Priority,
    pub code_examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationCategory {
    LoopOptimization,
    FunctionOptimization,
    MemoryOptimization,
    AlgorithmOptimization,
    DataStructureOptimization,
    CompilerOptimization,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplementationComplexity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

pub struct EnhancedOptimizationAnalyzer {
    analyzer: OptimizationAnalyzer,
    compiler: FastCompiler,
    optimizer: Optimizer,
}

impl EnhancedOptimizationAnalyzer {
    pub fn new() -> Self {
        let options = CompilerOptions {
            optimization_level: 3,
            incremental: true,
            parallel: true,
            cache_enabled: true,
            max_workers: 4,
        };
        
        Self {
            analyzer: OptimizationAnalyzer::new(),
            compiler: FastCompiler::new(options),
            optimizer: Optimizer::new(options),
        }
    }
    
    pub fn analyze_program(&mut self, source: &str) -> Result<EnhancedOptimizationReport, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Compile the program
        let compilation_result = self.compiler.compile(source)?;
        let compilation_time = start_time.elapsed();
        
        // Analyze the program
        let analysis_start = Instant::now();
        let analysis = self.analyzer.analyze_program(&compilation_result.ast)?;
        let analysis_time = analysis_start.elapsed();
        
        // Optimize the program
        let optimization_start = Instant::now();
        let mut optimized_program = compilation_result.ast.clone();
        self.optimizer.optimize(&mut optimized_program)?;
        let optimization_time = optimization_start.elapsed();
        
        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(
            compilation_time,
            optimization_time,
            &compilation_result.ast,
            &optimized_program,
        );
        
        // Generate optimization recommendations
        let recommendations = self.generate_optimization_recommendations(&analysis);
        
        // Calculate quality scores
        let code_quality_score = self.calculate_code_quality_score(&analysis);
        let optimization_effectiveness = self.calculate_optimization_effectiveness(&performance_metrics);
        
        Ok(EnhancedOptimizationReport {
            overall_stats: self.optimizer.get_optimization_stats().clone(),
            detailed_analysis: analysis.clone(),
            performance_metrics,
            optimization_recommendations: recommendations,
            code_quality_score,
            optimization_effectiveness,
        })
    }
    
    fn calculate_performance_metrics(
        &self,
        compilation_time: std::time::Duration,
        optimization_time: std::time::Duration,
        original_program: &Program,
        optimized_program: &Program,
    ) -> PerformanceMetrics {
        let original_size = self.estimate_program_size(original_program);
        let optimized_size = self.estimate_program_size(optimized_program);
        let size_reduction = if original_size > 0 {
            ((original_size - optimized_size) as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };
        
        let expected_runtime_improvement = self.estimate_runtime_improvement(original_program, optimized_program);
        
        PerformanceMetrics {
            compilation_time,
            optimization_time,
            memory_usage: optimized_size,
            code_size_reduction_percentage: size_reduction,
            expected_runtime_improvement,
        }
    }
    
    fn estimate_program_size(&self, program: &Program) -> usize {
        let mut size = 0;
        for statement in &program.statements {
            size += self.estimate_statement_size(statement);
        }
        size
    }
    
    fn estimate_statement_size(&self, statement: &Statement) -> usize {
        match statement {
            Statement::Function(func) => {
                let mut size = func.name.len() + func.params.len() * 10;
                for stmt in &func.body {
                    size += self.estimate_statement_size(stmt);
                }
                size
            }
            Statement::Let(let_stmt) => {
                let_stmt.name.len() + self.estimate_expression_size(&let_stmt.value)
            }
            Statement::Return(ret_stmt) => {
                self.estimate_expression_size(&ret_stmt.value)
            }
            _ => 10,
        }
    }
    
    fn estimate_expression_size(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Literal(_) => 5,
            Expression::Binary { left, right, .. } => {
                self.estimate_expression_size(left) + self.estimate_expression_size(right) + 2
            }
            Expression::Unary { operand, .. } => {
                self.estimate_expression_size(operand) + 1
            }
            Expression::Call { function, arguments } => {
                function.len() + arguments.len() * 5
            }
            Expression::If { condition, then_branch, else_branch } => {
                self.estimate_expression_size(condition) +
                self.estimate_expression_size(then_branch) +
                self.estimate_expression_size(else_branch) + 5
            }
            Expression::While { condition, body } => {
                self.estimate_expression_size(condition) +
                self.estimate_expression_size(body) + 3
            }
            _ => 10,
        }
    }
    
    fn estimate_runtime_improvement(&self, original: &Program, optimized: &Program) -> f64 {
        // Simple heuristic based on optimization opportunities
        let original_loops = self.count_loops(original);
        let optimized_loops = self.count_loops(optimized);
        let function_calls = self.count_function_calls(original);
        let constant_expressions = self.count_constant_expressions(original);
        
        let loop_improvement = if original_loops > 0 {
            (original_loops - optimized_loops) as f64 / original_loops as f64 * 20.0
        } else {
            0.0
        };
        
        let call_improvement = function_calls as f64 * 5.0;
        let constant_improvement = constant_expressions as f64 * 2.0;
        
        (loop_improvement + call_improvement + constant_improvement).min(50.0)
    }
    
    fn count_loops(&self, program: &Program) -> usize {
        let mut count = 0;
        for statement in &program.statements {
            count += self.count_loops_in_statement(statement);
        }
        count
    }
    
    fn count_loops_in_statement(&self, statement: &Statement) -> usize {
        match statement {
            Statement::Function(func) => {
                let mut count = 0;
                for stmt in &func.body {
                    count += self.count_loops_in_statement(stmt);
                }
                count
            }
            Statement::Let(let_stmt) => {
                self.count_loops_in_expression(&let_stmt.value)
            }
            Statement::Return(ret_stmt) => {
                self.count_loops_in_expression(&ret_stmt.value)
            }
            _ => 0,
        }
    }
    
    fn count_loops_in_expression(&self, expr: &Expression) -> usize {
        match expr {
            Expression::While { .. } => 1,
            Expression::Binary { left, right, .. } => {
                self.count_loops_in_expression(left) + self.count_loops_in_expression(right)
            }
            Expression::Unary { operand, .. } => {
                self.count_loops_in_expression(operand)
            }
            Expression::If { condition, then_branch, else_branch } => {
                self.count_loops_in_expression(condition) +
                self.count_loops_in_expression(then_branch) +
                self.count_loops_in_expression(else_branch)
            }
            _ => 0,
        }
    }
    
    fn count_function_calls(&self, program: &Program) -> usize {
        let mut count = 0;
        for statement in &program.statements {
            count += self.count_function_calls_in_statement(statement);
        }
        count
    }
    
    fn count_function_calls_in_statement(&self, statement: &Statement) -> usize {
        match statement {
            Statement::Function(func) => {
                let mut count = 0;
                for stmt in &func.body {
                    count += self.count_function_calls_in_statement(stmt);
                }
                count
            }
            Statement::Let(let_stmt) => {
                self.count_function_calls_in_expression(&let_stmt.value)
            }
            Statement::Return(ret_stmt) => {
                self.count_function_calls_in_expression(&ret_stmt.value)
            }
            _ => 0,
        }
    }
    
    fn count_function_calls_in_expression(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Call { .. } => 1,
            Expression::Binary { left, right, .. } => {
                self.count_function_calls_in_expression(left) + self.count_function_calls_in_expression(right)
            }
            Expression::Unary { operand, .. } => {
                self.count_function_calls_in_expression(operand)
            }
            Expression::If { condition, then_branch, else_branch } => {
                self.count_function_calls_in_expression(condition) +
                self.count_function_calls_in_expression(then_branch) +
                self.count_function_calls_in_expression(else_branch)
            }
            Expression::While { condition, body } => {
                self.count_function_calls_in_expression(condition) +
                self.count_function_calls_in_expression(body)
            }
            _ => 0,
        }
    }
    
    fn count_constant_expressions(&self, program: &Program) -> usize {
        let mut count = 0;
        for statement in &program.statements {
            count += self.count_constant_expressions_in_statement(statement);
        }
        count
    }
    
    fn count_constant_expressions_in_statement(&self, statement: &Statement) -> usize {
        match statement {
            Statement::Function(func) => {
                let mut count = 0;
                for stmt in &func.body {
                    count += self.count_constant_expressions_in_statement(stmt);
                }
                count
            }
            Statement::Let(let_stmt) => {
                self.count_constant_expressions_in_expression(&let_stmt.value)
            }
            Statement::Return(ret_stmt) => {
                self.count_constant_expressions_in_expression(&ret_stmt.value)
            }
            _ => 0,
        }
    }
    
    fn count_constant_expressions_in_expression(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Literal(_) => 1,
            Expression::Binary { left, right, .. } => {
                self.count_constant_expressions_in_expression(left) + 
                self.count_constant_expressions_in_expression(right)
            }
            Expression::Unary { operand, .. } => {
                self.count_constant_expressions_in_expression(operand)
            }
            _ => 0,
        }
    }
    
    fn generate_optimization_recommendations(&self, analysis: &OptimizationAnalysis) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        // Loop optimization recommendations
        if !analysis.control_flow.loops.is_empty() {
            recommendations.push(OptimizationRecommendation {
                category: OptimizationCategory::LoopOptimization,
                description: "Consider loop unrolling for small loops with known iteration counts".to_string(),
                expected_improvement: 15.0,
                implementation_complexity: ImplementationComplexity::Medium,
                priority: Priority::High,
                code_examples: vec![
                    "// Before: while i < 4 { sum += arr[i]; i += 1; }".to_string(),
                    "// After: sum += arr[0] + arr[1] + arr[2] + arr[3];".to_string(),
                ],
            });
        }
        
        // Function optimization recommendations
        let inlining_candidates = analysis.call_graph.nodes.values()
            .filter(|node| node.inlining_candidate)
            .count();
        
        if inlining_candidates > 0 {
            recommendations.push(OptimizationRecommendation {
                category: OptimizationCategory::FunctionOptimization,
                description: format!("{} functions are candidates for inlining", inlining_candidates),
                expected_improvement: 10.0,
                implementation_complexity: ImplementationComplexity::Low,
                priority: Priority::Medium,
                code_examples: vec![
                    "// Consider inlining small, frequently called functions".to_string(),
                ],
            });
        }
        
        // Memory optimization recommendations
        if analysis.optimization_opportunities.iter()
            .any(|opp| opp.opportunity_type == OptimizationType::MemoryOptimization) {
            recommendations.push(OptimizationRecommendation {
                category: OptimizationCategory::MemoryOptimization,
                description: "Memory access patterns can be optimized for better cache performance".to_string(),
                expected_improvement: 20.0,
                implementation_complexity: ImplementationComplexity::High,
                priority: Priority::Medium,
                code_examples: vec![
                    "// Consider cache-friendly data access patterns".to_string(),
                    "// Use contiguous memory layouts where possible".to_string(),
                ],
            });
        }
        
        // Algorithm optimization recommendations
        if analysis.call_graph.nodes.values()
            .any(|node| node.recursive) {
            recommendations.push(OptimizationRecommendation {
                category: OptimizationCategory::AlgorithmOptimization,
                description: "Recursive functions detected - consider iterative alternatives".to_string(),
                expected_improvement: 25.0,
                implementation_complexity: ImplementationComplexity::Medium,
                priority: Priority::High,
                code_examples: vec![
                    "// Convert recursive functions to iterative where possible".to_string(),
                    "// Use tail call optimization for recursive functions".to_string(),
                ],
            });
        }
        
        recommendations
    }
    
    fn calculate_code_quality_score(&self, analysis: &OptimizationAnalysis) -> f64 {
        let mut score = 100.0;
        
        // Penalize for complex functions
        for node in analysis.call_graph.nodes.values() {
            if node.complexity > 10 {
                score -= 5.0;
            }
        }
        
        // Penalize for too many function calls
        if analysis.call_graph.edges.len() > 20 {
            score -= 10.0;
        }
        
        // Penalize for recursive functions without tail call optimization
        let recursive_count = analysis.call_graph.nodes.values()
            .filter(|node| node.recursive)
            .count();
        score -= recursive_count as f64 * 3.0;
        
        // Bonus for optimization opportunities
        score += analysis.optimization_opportunities.len() as f64 * 2.0;
        
        score.max(0.0).min(100.0)
    }
    
    fn calculate_optimization_effectiveness(&self, metrics: &PerformanceMetrics) -> f64 {
        let mut effectiveness = 0.0;
        
        // Code size reduction contribution
        effectiveness += metrics.code_size_reduction_percentage * 0.3;
        
        // Expected runtime improvement contribution
        effectiveness += metrics.expected_runtime_improvement * 0.5;
        
        // Compilation time efficiency (penalty for slow compilation)
        let compilation_efficiency = if metrics.compilation_time.as_millis() > 1000 {
            100.0 - (metrics.compilation_time.as_millis() as f64 / 1000.0) * 10.0
        } else {
            100.0
        };
        effectiveness += compilation_efficiency * 0.2;
        
        effectiveness.max(0.0).min(100.0)
    }
    
    pub fn generate_detailed_report(&self, report: &EnhancedOptimizationReport) -> String {
        format!(
            "=== Enhanced Optimization Analysis Report ===\n\
             \n\
             PERFORMANCE METRICS:\n\
             - Compilation time: {:?}\n\
             - Optimization time: {:?}\n\
             - Code size reduction: {:.2}%\n\
             - Expected runtime improvement: {:.2}%\n\
             - Memory usage: {} bytes\n\
             \n\
             QUALITY SCORES:\n\
             - Code quality score: {:.1}/100\n\
             - Optimization effectiveness: {:.1}/100\n\
             \n\
             OPTIMIZATION STATISTICS:\n\
             - Transformations made: {}\n\
             - Passes applied: {}\n\
             \n\
             ANALYSIS DETAILS:\n\
             - Functions analyzed: {}\n\
             - Function calls: {}\n\
             - Loops detected: {}\n\
             - Optimization opportunities: {}\n\
             \n\
             RECOMMENDATIONS:\n\
             {}",
            report.performance_metrics.compilation_time,
            report.performance_metrics.optimization_time,
            report.performance_metrics.code_size_reduction_percentage,
            report.performance_metrics.expected_runtime_improvement,
            report.performance_metrics.memory_usage,
            report.code_quality_score,
            report.optimization_effectiveness,
            report.overall_stats.transformations_made,
            report.overall_stats.passes_applied.join(", "),
            report.detailed_analysis.call_graph.nodes.len(),
            report.detailed_analysis.call_graph.edges.len(),
            report.detailed_analysis.control_flow.loops.len(),
            report.detailed_analysis.optimization_opportunities.len(),
            self.format_recommendations(&report.optimization_recommendations),
        )
    }
    
    fn format_recommendations(&self, recommendations: &[OptimizationRecommendation]) -> String {
        if recommendations.is_empty() {
            "No specific recommendations at this time.".to_string()
        } else {
            recommendations.iter()
                .map(|rec| {
                    format!(
                        "- [{}] {} (Priority: {:?}, Expected improvement: {:.1}%)\n  Complexity: {:?}\n  Examples:\n{}",
                        rec.category.to_string(),
                        rec.description,
                        rec.priority,
                        rec.expected_improvement,
                        rec.implementation_complexity,
                        rec.code_examples.iter()
                            .map(|ex| format!("    {}", ex))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        }
    }
}

impl std::fmt::Display for OptimizationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationCategory::LoopOptimization => write!(f, "Loop Optimization"),
            OptimizationCategory::FunctionOptimization => write!(f, "Function Optimization"),
            OptimizationCategory::MemoryOptimization => write!(f, "Memory Optimization"),
            OptimizationCategory::AlgorithmOptimization => write!(f, "Algorithm Optimization"),
            OptimizationCategory::DataStructureOptimization => write!(f, "Data Structure Optimization"),
            OptimizationCategory::CompilerOptimization => write!(f, "Compiler Optimization"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = EnhancedOptimizationAnalyzer::new();
    
    // Test with the advanced optimization test
    let source = std::fs::read_to_string("advanced_test.nx")?;
    let report = analyzer.analyze_program(&source)?;
    
    println!("{}", analyzer.generate_detailed_report(&report));
    
    Ok(())
} 