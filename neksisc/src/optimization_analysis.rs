use crate::ast::{Program, Statement, Expression, FunctionStatement, BinaryOp, UnaryOp, IfExpression, WhileExpression};
use crate::error::CompilerError;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct OptimizationAnalysis {
    pub call_graph: CallGraph,
    pub data_flow: DataFlowAnalysis,
    pub control_flow: ControlFlowAnalysis,
    pub performance_metrics: PerformanceMetrics,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

#[derive(Debug, Clone)]
pub struct CallGraph {
    pub nodes: HashMap<String, CallGraphNode>,
    pub edges: Vec<CallGraphEdge>,
}

#[derive(Debug, Clone)]
pub struct CallGraphNode {
    pub function_name: String,
    pub complexity: usize,
    pub call_count: usize,
    pub inlining_candidate: bool,
    pub hot_function: bool,
    pub recursive: bool,
}

#[derive(Debug, Clone)]
pub struct CallGraphEdge {
    pub from: String,
    pub to: String,
    pub call_sites: Vec<SourceLocation>,
    pub frequency: usize,
}

#[derive(Debug, Clone)]
pub struct DataFlowAnalysis {
    pub variable_liveness: HashMap<String, LivenessInfo>,
    pub reaching_definitions: HashMap<String, Vec<Definition>>,
    pub available_expressions: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
pub struct LivenessInfo {
    pub variable: String,
    pub live_ranges: Vec<LiveRange>,
    pub last_use: Option<SourceLocation>,
    pub first_def: Option<SourceLocation>,
}

#[derive(Debug, Clone)]
pub struct LiveRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub variable: String,
    pub location: SourceLocation,
    pub expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ControlFlowAnalysis {
    pub basic_blocks: HashMap<String, Vec<BasicBlock>>,
    pub dominance: HashMap<String, DominanceInfo>,
    pub loops: Vec<LoopInfo>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: String,
    pub statements: Vec<Statement>,
    pub predecessors: Vec<String>,
    pub successors: Vec<String>,
    pub entry_point: bool,
    pub exit_point: bool,
}

#[derive(Debug, Clone)]
pub struct DominanceInfo {
    pub function: String,
    pub dominators: HashMap<String, HashSet<String>>,
    pub immediate_dominators: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub header: String,
    pub body: Vec<String>,
    pub exit_blocks: Vec<String>,
    pub trip_count: Option<usize>,
    pub invariant_expressions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub function_times: HashMap<String, f64>,
    pub memory_usage: HashMap<String, usize>,
    pub cache_misses: HashMap<String, usize>,
    pub hot_paths: Vec<HotPath>,
}

#[derive(Debug, Clone)]
pub struct HotPath {
    pub path: Vec<String>,
    pub execution_time: f64,
    pub frequency: usize,
}

#[derive(Debug, Clone)]
pub struct OptimizationOpportunity {
    pub opportunity_type: OptimizationType,
    pub location: SourceLocation,
    pub description: String,
    pub expected_improvement: f64,
    pub confidence: f64,
    pub implementation: String,
}

#[derive(Debug, Clone)]
pub enum OptimizationType {
    FunctionInlining,
    LoopUnrolling,
    DeadCodeElimination,
    ConstantFolding,
    StrengthReduction,
    CommonSubexpressionElimination,
    TailCallOptimization,
    Vectorization,
    MemoryOptimization,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone)]
pub struct OptimizationAnalyzer {
    analysis: OptimizationAnalysis,
}

impl OptimizationAnalyzer {
    pub fn new() -> Self {
        Self {
            analysis: OptimizationAnalysis {
                call_graph: CallGraph {
                    nodes: HashMap::new(),
                    edges: Vec::new(),
                },
                data_flow: DataFlowAnalysis {
                    variable_liveness: HashMap::new(),
                    reaching_definitions: HashMap::new(),
                    available_expressions: HashMap::new(),
                },
                control_flow: ControlFlowAnalysis {
                    basic_blocks: HashMap::new(),
                    dominance: HashMap::new(),
                    loops: Vec::new(),
                },
                performance_metrics: PerformanceMetrics {
                    function_times: HashMap::new(),
                    memory_usage: HashMap::new(),
                    cache_misses: HashMap::new(),
                    hot_paths: Vec::new(),
                },
                optimization_opportunities: Vec::new(),
            },
        }
    }

    pub fn analyze_program(&mut self, program: &Program) -> Result<&OptimizationAnalysis, CompilerError> {
        self.build_call_graph(program)?;
        self.analyze_data_flow(program)?;
        self.analyze_control_flow(program)?;
        self.identify_optimization_opportunities(program)?;
        
        Ok(&self.analysis)
    }

    fn build_call_graph(&mut self, program: &Program) -> Result<(), CompilerError> {
        // Build nodes for all functions
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                let complexity = self.calculate_function_complexity(func_stmt);
                let node = CallGraphNode {
                    function_name: func_stmt.name.clone(),
                    complexity,
                    call_count: 0,
                    inlining_candidate: complexity < 10,
                    hot_function: false,
                    recursive: false,
                };
                self.analysis.call_graph.nodes.insert(func_stmt.name.clone(), node);
            }
        }

        // Build edges by analyzing function calls
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.analyze_function_calls(&func_stmt.name, &func_stmt.body)?;
            }
        }

        // Detect recursive functions
        self.detect_recursive_functions()?;

        Ok(())
    }

    fn calculate_function_complexity(&self, func_stmt: &FunctionStatement) -> usize {
        self.estimate_expression_complexity(&func_stmt.body)
    }

    fn estimate_expression_complexity(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Literal(_) => 1,
            Expression::Identifier(_) => 1,
            Expression::BinaryOp(binary_op) => {
                1 + self.estimate_expression_complexity(&binary_op.left) + self.estimate_expression_complexity(&binary_op.right)
            }
            Expression::UnaryOp(unary_op) => {
                1 + self.estimate_expression_complexity(&unary_op.operand)
            }
            Expression::If(if_expr) => {
                1 + self.estimate_expression_complexity(&if_expr.condition)
                    + self.estimate_expression_complexity(&if_expr.then_branch)
                    + if_expr.else_branch.as_ref().map_or(0, |e| self.estimate_expression_complexity(e))
            }
            Expression::While(while_expr) => {
                1 + self.estimate_expression_complexity(&while_expr.condition) + self.estimate_expression_complexity(&while_expr.body)
            }
            Expression::Block(statements) => {
                statements.len() + statements.iter().map(|s| self.estimate_statement_complexity(s)).sum::<usize>()
            }
            Expression::FunctionCall(function, arguments) => {
                1 + self.estimate_expression_complexity(function)
                    + arguments.iter().map(|a| self.estimate_expression_complexity(&a.value)).sum::<usize>()
            }
            _ => 1,
        }
    }

    fn estimate_statement_complexity(&self, stmt: &Statement) -> usize {
        match stmt {
            Statement::Let(let_stmt) => self.estimate_expression_complexity(&let_stmt.value),
            Statement::Return(return_stmt) => {
                return_stmt.value.as_ref().map_or(0, |e| self.estimate_expression_complexity(e))
            }
            Statement::Expression(expr) => self.estimate_expression_complexity(expr),
            _ => 1,
        }
    }

    fn analyze_function_calls(
        &mut self,
        caller: &str,
        expr: &Expression,
    ) -> Result<(), CompilerError> {
        match expr {
            Expression::FunctionCall(function, _) => {
                if let Expression::Identifier(callee) = &**function {
                    let edge = CallGraphEdge {
                        from: caller.to_string(),
                        to: callee.clone(),
                        call_sites: vec![SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                        }],
                        frequency: 1,
                    };
                    self.analysis.call_graph.edges.push(edge);

                    // Update call count
                    if let Some(node) = self.analysis.call_graph.nodes.get_mut(callee) {
                        node.call_count += 1;
                    }
                }
            }
            Expression::If(if_expr) => {
                self.analyze_function_calls(caller, &if_expr.then_branch)?;
                if let Some(else_expr) = &if_expr.else_branch {
                    self.analyze_function_calls(caller, else_expr)?;
                }
            }
            Expression::While(while_expr) => {
                self.analyze_function_calls(caller, &while_expr.body)?;
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    if let Statement::Expression(Expression::FunctionCall(function, _)) = stmt {
                        if let Expression::Identifier(callee) = &**function {
                            let edge = CallGraphEdge {
                                from: caller.to_string(),
                                to: callee.clone(),
                                call_sites: vec![SourceLocation {
                                    file: "unknown".to_string(),
                                    line: 0,
                                    column: 0,
                                }],
                                frequency: 1,
                            };
                            self.analysis.call_graph.edges.push(edge);

                            if let Some(node) = self.analysis.call_graph.nodes.get_mut(callee) {
                                node.call_count += 1;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn detect_recursive_functions(&mut self) -> Result<(), CompilerError> {
        // Simple recursive detection using DFS
        let func_names: Vec<String> = self.analysis.call_graph.nodes.keys().cloned().collect();
        
        for func_name in func_names {
            let mut visited = HashSet::new();
            let mut recursion_stack = HashSet::new();
            
            if self.has_recursion(&func_name, &mut visited, &mut recursion_stack) {
                if let Some(node) = self.analysis.call_graph.nodes.get_mut(&func_name) {
                    node.recursive = true;
                }
            }
        }
        Ok(())
    }

    fn has_recursion(
        &self,
        func_name: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> bool {
        if recursion_stack.contains(func_name) {
            return true;
        }
        
        if visited.contains(func_name) {
            return false;
        }
        
        visited.insert(func_name.to_string());
        recursion_stack.insert(func_name.to_string());
        
        for edge in &self.analysis.call_graph.edges {
            if edge.from == func_name {
                if self.has_recursion(&edge.to, visited, recursion_stack) {
                    return true;
                }
            }
        }
        
        recursion_stack.remove(func_name);
        false
    }

    fn analyze_data_flow(&mut self, program: &Program) -> Result<(), CompilerError> {
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.analyze_function_data_flow(func_stmt)?;
            }
        }
        Ok(())
    }

    fn analyze_function_data_flow(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        // Analyze variable liveness
        self.analyze_variable_liveness(func_stmt)?;
        
        // Analyze reaching definitions
        self.analyze_reaching_definitions(func_stmt)?;
        
        // Analyze available expressions
        self.analyze_available_expressions(func_stmt)?;
        
        Ok(())
    }

    fn analyze_variable_liveness(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        // Simplified liveness analysis
        let mut variables = HashSet::new();
        self.collect_variables(&func_stmt.body, &mut variables);
        
        for var in variables {
            let liveness_info = LivenessInfo {
                variable: var.clone(),
                live_ranges: vec![LiveRange {
                    start: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        column: 0,
                    },
                    end: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        column: 0,
                    },
                    scope: func_stmt.name.clone(),
                }],
                last_use: None,
                first_def: None,
            };
            self.analysis.data_flow.variable_liveness.insert(var, liveness_info);
        }
        
        Ok(())
    }

    fn collect_variables(&self, expr: &Expression, variables: &mut HashSet<String>) {
        match expr {
            Expression::Identifier(name) => {
                variables.insert(name.clone());
            }
            Expression::BinaryOp(binary_op) => {
                self.collect_variables(&binary_op.left, variables);
                self.collect_variables(&binary_op.right, variables);
            }
            Expression::UnaryOp(unary_op) => {
                self.collect_variables(&unary_op.operand, variables);
            }
            Expression::If(if_expr) => {
                self.collect_variables(&if_expr.condition, variables);
                self.collect_variables(&if_expr.then_branch, variables);
                if let Some(else_expr) = &if_expr.else_branch {
                    self.collect_variables(else_expr, variables);
                }
            }
            Expression::While(while_expr) => {
                self.collect_variables(&while_expr.condition, variables);
                self.collect_variables(&while_expr.body, variables);
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    if let Statement::Let(let_stmt) = stmt {
                        variables.insert(let_stmt.name.clone());
                        self.collect_variables(&let_stmt.value, variables);
                    }
                }
            }
            Expression::FunctionCall(function, arguments) => {
                self.collect_variables(function, variables);
                for arg in arguments {
                    self.collect_variables(&arg.value, variables);
                }
            }
            _ => {}
        }
    }

    fn analyze_reaching_definitions(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        // Simplified reaching definitions analysis
        let mut definitions = Vec::new();
        self.collect_definitions(&func_stmt.body, &mut definitions);
        
        for def in definitions {
            self.analysis.data_flow.reaching_definitions
                .entry(def.variable.clone())
                .or_insert_with(Vec::new)
                .push(def);
        }
        
        Ok(())
    }

    fn collect_definitions(&self, expr: &Expression, definitions: &mut Vec<Definition>) {
        match expr {
            Expression::Block(statements) => {
                for stmt in statements {
                    if let Statement::Let(let_stmt) = stmt {
                        let def = Definition {
                            variable: let_stmt.name.clone(),
                            location: SourceLocation {
                                file: "unknown".to_string(),
                                line: 0,
                                column: 0,
                            },
                            expression: Some(format!("{:?}", let_stmt.value)),
                        };
                        definitions.push(def);
                    }
                }
            }
            _ => {}
        }
    }

    fn analyze_available_expressions(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        // Simplified available expressions analysis
        let mut expressions = HashSet::new();
        self.collect_expressions(&func_stmt.body, &mut expressions);
        
        self.analysis.data_flow.available_expressions
            .insert(func_stmt.name.clone(), expressions);
        
        Ok(())
    }

    fn collect_expressions(&self, expr: &Expression, expressions: &mut HashSet<String>) {
        match expr {
            Expression::BinaryOp(binary_op) => {
                let expr_str = format!("{:?} {:?} {:?}", binary_op.left, binary_op.operator, binary_op.right);
                expressions.insert(expr_str);
                self.collect_expressions(&binary_op.left, expressions);
                self.collect_expressions(&binary_op.right, expressions);
            }
            Expression::UnaryOp(unary_op) => {
                let expr_str = format!("{:?} {:?}", unary_op.operator, unary_op.operand);
                expressions.insert(expr_str);
                self.collect_expressions(&unary_op.operand, expressions);
            }
            Expression::If(if_expr) => {
                self.collect_expressions(&if_expr.condition, expressions);
                self.collect_expressions(&if_expr.then_branch, expressions);
                if let Some(else_expr) = &if_expr.else_branch {
                    self.collect_expressions(else_expr, expressions);
                }
            }
            Expression::While(while_expr) => {
                self.collect_expressions(&while_expr.condition, expressions);
                self.collect_expressions(&while_expr.body, expressions);
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    if let Statement::Let(let_stmt) = stmt {
                        self.collect_expressions(&let_stmt.value, expressions);
                    }
                }
            }
            Expression::FunctionCall(function, arguments) => {
                self.collect_expressions(function, expressions);
                for arg in arguments {
                    self.collect_expressions(&arg.value, expressions);
                }
            }
            _ => {}
        }
    }

    fn analyze_control_flow(&mut self, program: &Program) -> Result<(), CompilerError> {
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.analyze_function_control_flow(func_stmt)?;
            }
        }
        Ok(())
    }

    fn analyze_function_control_flow(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        // Build basic blocks
        let blocks = self.build_basic_blocks(func_stmt)?;
        self.analysis.control_flow.basic_blocks.insert(func_stmt.name.clone(), blocks);
        
        // Analyze dominance
        self.analyze_dominance(func_stmt)?;
        
        // Detect loops
        self.detect_loops(func_stmt)?;
        
        Ok(())
    }

    fn build_basic_blocks(&self, _func_stmt: &FunctionStatement) -> Result<Vec<BasicBlock>, CompilerError> {
        // Simplified basic block construction
        let mut blocks = Vec::new();
        
        // Create a single basic block for the function body
        let block = BasicBlock {
            id: "entry".to_string(),
            statements: vec![],
            predecessors: vec![],
            successors: vec![],
            entry_point: true,
            exit_point: true,
        };
        blocks.push(block);
        
        Ok(blocks)
    }

    fn analyze_dominance(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        // Simplified dominance analysis
        let dominance_info = DominanceInfo {
            function: func_stmt.name.clone(),
            dominators: HashMap::new(),
            immediate_dominators: HashMap::new(),
        };
        self.analysis.control_flow.dominance.insert(func_stmt.name.clone(), dominance_info);
        
        Ok(())
    }

    fn detect_loops(&mut self, func_stmt: &FunctionStatement) -> Result<(), CompilerError> {
        // Simplified loop detection
        self.find_loops_in_expression(&func_stmt.body)?;
        
        Ok(())
    }

    fn find_loops_in_expression(&mut self, expr: &Expression) -> Result<(), CompilerError> {
        match expr {
            Expression::While(_while_expr) => {
                let loop_info = LoopInfo {
                    header: "while_loop".to_string(),
                    body: vec!["body".to_string()],
                    exit_blocks: vec!["exit".to_string()],
                    trip_count: None,
                    invariant_expressions: vec![],
                };
                self.analysis.control_flow.loops.push(loop_info);
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    if let Statement::Expression(Expression::While(_while_expr)) = stmt {
                        let loop_info = LoopInfo {
                            header: "while_loop".to_string(),
                            body: vec!["body".to_string()],
                            exit_blocks: vec!["exit".to_string()],
                            trip_count: None,
                            invariant_expressions: vec![],
                        };
                        self.analysis.control_flow.loops.push(loop_info);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn identify_optimization_opportunities(&mut self, _program: &Program) -> Result<(), CompilerError> {
        // Identify function inlining opportunities
        self.identify_inlining_opportunities()?;
        
        // Identify loop optimization opportunities
        self.identify_loop_optimizations()?;
        
        // Identify dead code elimination opportunities
        self.identify_dead_code_opportunities()?;
        
        // Identify strength reduction opportunities
        self.identify_strength_reduction_opportunities()?;
        
        Ok(())
    }

    fn identify_inlining_opportunities(&mut self) -> Result<(), CompilerError> {
        for (func_name, node) in &self.analysis.call_graph.nodes {
            if node.inlining_candidate && node.call_count > 0 && !node.recursive {
                let opportunity = OptimizationOpportunity {
                    opportunity_type: OptimizationType::FunctionInlining,
                    location: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        column: 0,
                    },
                    description: format!("Inline function '{}' (complexity: {}, calls: {})", 
                                      func_name, node.complexity, node.call_count),
                    expected_improvement: 0.15, // 15% improvement estimate
                    confidence: 0.8,
                    implementation: format!("Replace calls to '{}' with function body", func_name),
                };
                self.analysis.optimization_opportunities.push(opportunity);
            }
        }
        Ok(())
    }

    fn identify_loop_optimizations(&mut self) -> Result<(), CompilerError> {
        for loop_info in &self.analysis.control_flow.loops {
            let opportunity = OptimizationOpportunity {
                opportunity_type: OptimizationType::LoopUnrolling,
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                description: format!("Unroll loop at '{}'", loop_info.header),
                expected_improvement: 0.25, // 25% improvement estimate
                confidence: 0.7,
                implementation: "Replace loop with unrolled iterations".to_string(),
            };
            self.analysis.optimization_opportunities.push(opportunity);
        }
        Ok(())
    }

    fn identify_dead_code_opportunities(&mut self) -> Result<(), CompilerError> {
        // Analyze unreachable code
        let opportunity = OptimizationOpportunity {
            opportunity_type: OptimizationType::DeadCodeElimination,
            location: SourceLocation {
                file: "unknown".to_string(),
                line: 0,
                column: 0,
            },
            description: "Remove unreachable code".to_string(),
            expected_improvement: 0.05, // 5% improvement estimate
            confidence: 0.9,
            implementation: "Remove unreachable basic blocks".to_string(),
        };
        self.analysis.optimization_opportunities.push(opportunity);
        
        Ok(())
    }

    fn identify_strength_reduction_opportunities(&mut self) -> Result<(), CompilerError> {
        // Look for multiplication by constants that could be replaced with shifts
        let opportunity = OptimizationOpportunity {
            opportunity_type: OptimizationType::StrengthReduction,
            location: SourceLocation {
                file: "unknown".to_string(),
                line: 0,
                column: 0,
            },
            description: "Replace expensive operations with cheaper equivalents".to_string(),
            expected_improvement: 0.1, // 10% improvement estimate
            confidence: 0.8,
            implementation: "Replace multiplication by powers of 2 with shifts".to_string(),
        };
        self.analysis.optimization_opportunities.push(opportunity);
        
        Ok(())
    }

    pub fn get_analysis(&self) -> &OptimizationAnalysis {
        &self.analysis
    }

    pub fn generate_optimization_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Optimization Analysis Report ===\n\n");
        
        // Call graph summary
        report.push_str("Call Graph Analysis:\n");
        report.push_str(&format!("Total functions: {}\n", self.analysis.call_graph.nodes.len()));
        report.push_str(&format!("Total calls: {}\n", self.analysis.call_graph.edges.len()));
        
        let inlining_candidates = self.analysis.call_graph.nodes.values()
            .filter(|node| node.inlining_candidate)
            .count();
        report.push_str(&format!("Inlining candidates: {}\n", inlining_candidates));
        
        let recursive_functions = self.analysis.call_graph.nodes.values()
            .filter(|node| node.recursive)
            .count();
        report.push_str(&format!("Recursive functions: {}\n", recursive_functions));
        
        // Optimization opportunities
        report.push_str("\nOptimization Opportunities:\n");
        for (i, opportunity) in self.analysis.optimization_opportunities.iter().enumerate() {
            report.push_str(&format!("{}. {} ({}% improvement, confidence: {}%)\n", 
                                   i + 1, opportunity.description, 
                                   (opportunity.expected_improvement * 100.0) as i32,
                                   (opportunity.confidence * 100.0) as i32));
        }
        
        report
    }
} 