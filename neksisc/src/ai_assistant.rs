use std::collections::HashMap;
use crate::ast::{Expression, Statement, Program};
use crate::vm::VMValue;

pub struct AIAssistant {
    code_patterns: HashMap<String, Vec<String>>,
    optimization_suggestions: Vec<OptimizationSuggestion>,
    performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub location: String,
    pub suggestion_type: OptimizationType,
    pub description: String,
    pub estimated_improvement: f64, // Percentage improvement
    pub code_before: String,
    pub code_after: String,
}

#[derive(Debug, Clone)]
pub enum OptimizationType {
    LoopOptimization,
    MemoryOptimization,
    AlgorithmicImprovement,
    ConcurrencyOpportunity,
    CachingOpportunity,
    InliningOpportunity,
}

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub function_call_counts: HashMap<String, usize>,
    pub execution_times: HashMap<String, Vec<f64>>,
    pub memory_usage: HashMap<String, usize>,
    pub hot_paths: Vec<String>,
}

impl AIAssistant {
    pub fn new() -> Self {
        let mut code_patterns = HashMap::new();
        
        // Pre-populate with common patterns
        code_patterns.insert("fibonacci".to_string(), vec![
            "Recursive implementation detected - consider iterative approach for better performance".to_string(),
            "Consider memoization to avoid redundant calculations".to_string(),
        ]);
        
        code_patterns.insert("array_iteration".to_string(), vec![
            "Consider using parallel iteration for large arrays".to_string(),
            "Use array_slice for better memory efficiency".to_string(),
        ]);
        
        code_patterns.insert("string_concatenation".to_string(), vec![
            "Multiple string concatenations detected - consider using array join".to_string(),
            "Use string builder pattern for better performance".to_string(),
        ]);
        
        Self {
            code_patterns,
            optimization_suggestions: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }
    
    pub fn analyze_code(&mut self, program: &Program) -> Vec<OptimizationSuggestion> {
        self.optimization_suggestions.clear();
        
        for statement in &program.statements {
            self.analyze_statement(statement);
        }
        
        self.generate_suggestions()
    }
    
    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Function(func_stmt) => {
                self.analyze_function(&func_stmt.name, &func_stmt.body);
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr, "global");
            }
            _ => {}
        }
    }
    
    fn analyze_function(&mut self, function_name: &str, body: &Expression) {
        // Track function for performance monitoring
        *self.performance_metrics.function_call_counts.entry(function_name.to_string()).or_insert(0) += 1;
        
        // Analyze function body
        self.analyze_expression(body, function_name);
        
        // Check for specific patterns
        if function_name.contains("fibonacci") {
            self.suggest_fibonacci_optimization(function_name);
        }
        
        if function_name.contains("sort") || function_name.contains("search") {
            self.suggest_algorithmic_improvements(function_name);
        }
    }
    
    fn analyze_expression(&mut self, expr: &Expression, context: &str) {
        match expr {
            Expression::FunctionCall(func, args) => {
                if let Expression::Identifier(func_name) = &**func {
                    self.analyze_function_call(func_name, args, context);
                }
            }
            Expression::BinaryOp(bin_op) => {
                self.analyze_binary_operation(bin_op, context);
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    self.analyze_statement(stmt);
                }
            }
            Expression::While(while_expr) => {
                self.analyze_loop(&while_expr.condition, &while_expr.body, context);
            }
            _ => {}
        }
    }
    
    fn analyze_function_call(&mut self, func_name: &str, args: &[crate::ast::CallArgument], context: &str) {
        // Track function calls
        *self.performance_metrics.function_call_counts.entry(func_name.to_string()).or_insert(0) += 1;
        
        // Suggest optimizations based on usage patterns
        match func_name {
            "array_push" | "array_pop" => {
                if args.len() > 0 {
                    self.suggest_array_optimization(func_name, context);
                }
            }
            "dict_set" | "dict_get" => {
                self.suggest_caching_opportunity(func_name, context);
            }
            "json_stringify" | "json_parse" => {
                self.suggest_serialization_optimization(func_name, context);
            }
            _ => {}
        }
    }
    
    fn analyze_binary_operation(&mut self, bin_op: &crate::ast::BinaryOp, context: &str) {
        use crate::ast::BinaryOperator;
        
        match bin_op.operator {
            BinaryOperator::Add => {
                // Check for string concatenation in loops
                if context.contains("loop") || context.contains("while") {
                    self.optimization_suggestions.push(OptimizationSuggestion {
                        location: format!("{} - string concatenation", context),
                        suggestion_type: OptimizationType::LoopOptimization,
                        description: "String concatenation in loop detected - consider using array join".to_string(),
                        estimated_improvement: 50.0,
                        code_before: "let result = \"\"; while (...) { result = result + item; }".to_string(),
                        code_after: "let parts = []; while (...) { array_push(parts, item); } let result = join(parts, \"\");".to_string(),
                    });
                }
            }
            BinaryOperator::Multiply | BinaryOperator::Divide => {
                // Check for expensive operations
                self.suggest_mathematical_optimization(context);
            }
            _ => {}
        }
    }
    
    fn analyze_loop(&mut self, condition: &Expression, body: &Expression, context: &str) {
        // Check for parallelization opportunities
        if self.can_parallelize_loop(body) {
            self.optimization_suggestions.push(OptimizationSuggestion {
                location: format!("{} - loop parallelization", context),
                suggestion_type: OptimizationType::ConcurrencyOpportunity,
                description: "Loop can be parallelized for better performance".to_string(),
                estimated_improvement: 200.0,
                code_before: "while (condition) { ... }".to_string(),
                code_after: "@parallel while (condition) { ... }".to_string(),
            });
        }
    }
    
    fn can_parallelize_loop(&self, body: &Expression) -> bool {
        // Simple heuristic - check if loop body doesn't have dependencies
        // In a real implementation, this would be much more sophisticated
        match body {
            Expression::Block(statements) => {
                // Check for independent operations
                statements.len() > 1 && !self.has_dependencies(statements)
            }
            _ => false
        }
    }
    
    fn has_dependencies(&self, _statements: &[Statement]) -> bool {
        // Simplified dependency analysis
        // In practice, this would analyze data flow and dependencies
        false
    }
    
    // Specific optimization suggestions
    fn suggest_fibonacci_optimization(&mut self, function_name: &str) {
        self.optimization_suggestions.push(OptimizationSuggestion {
            location: function_name.to_string(),
            suggestion_type: OptimizationType::AlgorithmicImprovement,
            description: "Fibonacci implementation can be optimized using iteration or memoization".to_string(),
            estimated_improvement: 1000.0,
            code_before: "fn fibonacci(n) { if n <= 1 { return n; } return fibonacci(n-1) + fibonacci(n-2); }".to_string(),
            code_after: "fn fibonacci(n) { let a = 0; let b = 1; while n > 1 { let temp = a + b; a = b; b = temp; n = n - 1; } return b; }".to_string(),
        });
    }
    
    fn suggest_array_optimization(&mut self, func_name: &str, context: &str) {
        if self.performance_metrics.function_call_counts.get(func_name).unwrap_or(&0) > &10 {
            self.optimization_suggestions.push(OptimizationSuggestion {
                location: format!("{} - {}", context, func_name),
                suggestion_type: OptimizationType::MemoryOptimization,
                description: "Frequent array operations detected - consider pre-allocating capacity".to_string(),
                estimated_improvement: 30.0,
                code_before: format!("Multiple calls to {}", func_name),
                code_after: "Pre-allocate array with expected capacity".to_string(),
            });
        }
    }
    
    fn suggest_caching_opportunity(&mut self, func_name: &str, context: &str) {
        self.optimization_suggestions.push(OptimizationSuggestion {
            location: format!("{} - {}", context, func_name),
            suggestion_type: OptimizationType::CachingOpportunity,
            description: "Dictionary operations detected - consider caching frequently accessed values".to_string(),
            estimated_improvement: 40.0,
            code_before: "Multiple dict_get calls with same key".to_string(),
            code_after: "Cache result: let cached_value = dict_get(dict, key);".to_string(),
        });
    }
    
    fn suggest_serialization_optimization(&mut self, func_name: &str, context: &str) {
        self.optimization_suggestions.push(OptimizationSuggestion {
            location: format!("{} - {}", context, func_name),
            suggestion_type: OptimizationType::MemoryOptimization,
            description: "JSON operations detected - consider batch processing for multiple objects".to_string(),
            estimated_improvement: 25.0,
            code_before: "Multiple json_stringify calls".to_string(),
            code_after: "Batch process: json_stringify([obj1, obj2, obj3])".to_string(),
        });
    }
    
    fn suggest_algorithmic_improvements(&mut self, function_name: &str) {
        self.optimization_suggestions.push(OptimizationSuggestion {
            location: function_name.to_string(),
            suggestion_type: OptimizationType::AlgorithmicImprovement,
            description: "Consider using more efficient algorithms (e.g., quick sort vs bubble sort)".to_string(),
            estimated_improvement: 500.0,
            code_before: "O(n²) algorithm detected".to_string(),
            code_after: "Use O(n log n) or O(n) algorithm if possible".to_string(),
        });
    }
    
    fn suggest_mathematical_optimization(&mut self, context: &str) {
        self.optimization_suggestions.push(OptimizationSuggestion {
            location: context.to_string(),
            suggestion_type: OptimizationType::LoopOptimization,
            description: "Expensive mathematical operations - consider lookup tables for common values".to_string(),
            estimated_improvement: 60.0,
            code_before: "Repeated expensive calculations".to_string(),
            code_after: "Use lookup table or cache results".to_string(),
        });
    }
    
    fn generate_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = self.optimization_suggestions.clone();
        
        // Sort by estimated improvement
        suggestions.sort_by(|a, b| b.estimated_improvement.partial_cmp(&a.estimated_improvement).unwrap());
        
        suggestions
    }
    
    pub fn get_code_completion(&self, context: &str, partial_code: &str) -> Vec<String> {
        let mut completions = Vec::new();
        
        // Basic completion based on context
        if partial_code.starts_with("dict_") {
            completions.extend(vec![
                "dict_new()".to_string(),
                "dict_set(dict, key, value)".to_string(),
                "dict_get(dict, key)".to_string(),
                "dict_has(dict, key)".to_string(),
                "dict_size(dict)".to_string(),
            ]);
        }
        
        if partial_code.starts_with("array_") {
            completions.extend(vec![
                "array_push(array, value)".to_string(),
                "array_pop(array)".to_string(),
                "array_reverse(array)".to_string(),
                "array_sort(array)".to_string(),
                "array_slice(array, start, end)".to_string(),
            ]);
        }
        
        if partial_code.starts_with("json_") {
            completions.extend(vec![
                "json_parse(json_string)".to_string(),
                "json_stringify(value)".to_string(),
            ]);
        }
        
        // Filter based on partial code
        completions.retain(|completion| {
            completion.to_lowercase().contains(&partial_code.to_lowercase())
        });
        
        completions
    }
    
    pub fn update_performance_metrics(&mut self, function_name: &str, execution_time: f64, memory_used: usize) {
        self.performance_metrics
            .execution_times
            .entry(function_name.to_string())
            .or_insert(Vec::new())
            .push(execution_time);
            
        self.performance_metrics
            .memory_usage
            .insert(function_name.to_string(), memory_used);
    }
    
    pub fn get_performance_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== AI Performance Analysis ===\n");
        
        // Function call statistics
        report.push_str("\nFunction Call Counts:\n");
        for (func, count) in &self.performance_metrics.function_call_counts {
            report.push_str(&format!("  {}: {} calls\n", func, count));
        }
        
        // Average execution times
        report.push_str("\nAverage Execution Times:\n");
        for (func, times) in &self.performance_metrics.execution_times {
            if !times.is_empty() {
                let avg: f64 = times.iter().sum::<f64>() / times.len() as f64;
                report.push_str(&format!("  {}: {:.2}ms\n", func, avg));
            }
        }
        
        // Optimization suggestions
        report.push_str(&format!("\nOptimization Suggestions: {}\n", self.optimization_suggestions.len()));
        for (i, suggestion) in self.optimization_suggestions.iter().take(5).enumerate() {
            report.push_str(&format!(
                "  {}. {} - {:.0}% improvement potential\n",
                i + 1,
                suggestion.description,
                suggestion.estimated_improvement
            ));
        }
        
        report
    }
}
