use crate::ast::{Program, Statement, Expression, FunctionStatement, BinaryOp, UnaryOp};
use crate::error::CompilerError;
use crate::compiler::CompilerOptions;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct OptimizationPass {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub level: OptimizationLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationLevel {
    None = 0,
    Basic = 1,
    Standard = 2,
    Aggressive = 3,
}

impl OptimizationLevel {
    pub fn from_u8(level: u8) -> Self {
        match level {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Basic,
            2 => OptimizationLevel::Standard,
            3 => OptimizationLevel::Aggressive,
            _ => OptimizationLevel::Standard,
        }
    }
}

#[derive(Clone)]
pub struct Optimizer {
    passes: Vec<OptimizationPass>,
    options: CompilerOptions,
    optimization_stats: OptimizationStats,
}

#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub passes_applied: Vec<String>,
    pub transformations_made: usize,
    pub code_size_before: usize,
    pub code_size_after: usize,
    pub optimization_time: std::time::Duration,
}

impl OptimizationStats {
    pub fn new() -> Self {
        Self {
            passes_applied: Vec::new(),
            transformations_made: 0,
            code_size_before: 0,
            code_size_after: 0,
            optimization_time: std::time::Duration::from_millis(0),
        }
    }
}

impl Optimizer {
    pub fn new(options: CompilerOptions) -> Self {
        let mut passes = vec![
            OptimizationPass {
                name: "constant_folding".to_string(),
                description: "Fold constant expressions at compile time".to_string(),
                enabled: true,
                level: OptimizationLevel::Basic,
            },
            OptimizationPass {
                name: "dead_code_elimination".to_string(),
                description: "Remove unreachable and unused code".to_string(),
                enabled: true,
                level: OptimizationLevel::Basic,
            },
            OptimizationPass {
                name: "function_inlining".to_string(),
                description: "Inline small functions to reduce call overhead".to_string(),
                enabled: true,
                level: OptimizationLevel::Standard,
            },
            OptimizationPass {
                name: "loop_optimization".to_string(),
                description: "Optimize loops for better performance".to_string(),
                enabled: true,
                level: OptimizationLevel::Standard,
            },
            OptimizationPass {
                name: "strength_reduction".to_string(),
                description: "Replace expensive operations with cheaper equivalents".to_string(),
                enabled: true,
                level: OptimizationLevel::Standard,
            },
            OptimizationPass {
                name: "common_subexpression_elimination".to_string(),
                description: "Eliminate redundant computations".to_string(),
                enabled: true,
                level: OptimizationLevel::Standard,
            },
            OptimizationPass {
                name: "tail_call_optimization".to_string(),
                description: "Optimize tail recursive calls".to_string(),
                enabled: true,
                level: OptimizationLevel::Aggressive,
            },
            OptimizationPass {
                name: "vectorization".to_string(),
                description: "Vectorize operations where possible".to_string(),
                enabled: true,
                level: OptimizationLevel::Aggressive,
            },
        ];

        // Enable passes based on optimization level
        let opt_level = OptimizationLevel::from_u8(options.optimization_level);
        for pass in &mut passes {
            pass.enabled = pass.level <= opt_level;
        }

        Self {
            passes,
            options,
            optimization_stats: OptimizationStats::new(),
        }
    }

    pub fn optimize(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        let start_time = std::time::Instant::now();
        self.optimization_stats.code_size_before = self.estimate_code_size(program);

        // Collect enabled pass names to avoid borrow checker issues
        let enabled_pass_names: Vec<String> = self.passes.iter()
            .filter(|pass| pass.enabled)
            .map(|pass| pass.name.clone())
            .collect();

        for pass_name in enabled_pass_names {
            match pass_name.as_str() {
                "constant_folding" => self.constant_folding_pass(program)?,
                "dead_code_elimination" => self.dead_code_elimination_pass(program)?,
                "function_inlining" => self.function_inlining_pass(program)?,
                "loop_optimization" => self.loop_optimization_pass(program)?,
                "strength_reduction" => self.strength_reduction_pass(program)?,
                "common_subexpression_elimination" => self.cse_pass(program)?,
                "tail_call_optimization" => self.tail_call_optimization_pass(program)?,
                "vectorization" => self.vectorization_pass(program)?,
                _ => {}
            }
            self.optimization_stats.passes_applied.push(pass_name);
        }

        self.optimization_stats.code_size_after = self.estimate_code_size(program);
        self.optimization_stats.optimization_time = start_time.elapsed();

        Ok(())
    }

    fn constant_folding_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        for statement in &mut program.statements {
            self.fold_constants_in_statement(statement)?;
        }
        Ok(())
    }

    fn fold_constants_in_statement(&mut self, statement: &mut Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Function(func_stmt) => {
                self.fold_constants_in_expression(&mut func_stmt.body)?;
            }
            Statement::Let(let_stmt) => {
                self.fold_constants_in_expression(&mut let_stmt.value)?;
            }
            Statement::Return(return_stmt) => {
                if let Some(expr) = &mut return_stmt.value {
                    self.fold_constants_in_expression(expr)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn fold_constants_in_expression(&mut self, expr: &mut Expression) -> Result<(), CompilerError> {
        match expr {
            Expression::BinaryOp(binary_op) => {
                self.fold_constants_in_expression(&mut binary_op.left)?;
                self.fold_constants_in_expression(&mut binary_op.right)?;
                
                // Try to fold the binary operation
                if let (Expression::Literal(left_lit), Expression::Literal(right_lit)) = (&*binary_op.left, &*binary_op.right) {
                    if let Some(folded) = self.fold_binary_operation(left_lit, &binary_op.operator, right_lit)? {
                        *expr = folded;
                        self.optimization_stats.transformations_made += 1;
                    }
                }
            }
            Expression::UnaryOp(unary_op) => {
                self.fold_constants_in_expression(&mut unary_op.operand)?;
                
                if let Expression::Literal(lit) = &*unary_op.operand {
                    if let Some(folded) = self.fold_unary_operation(&unary_op.operator, lit)? {
                        *expr = folded;
                        self.optimization_stats.transformations_made += 1;
                    }
                }
            }
            Expression::If(if_expr) => {
                self.fold_constants_in_expression(&mut if_expr.condition)?;
                self.fold_constants_in_expression(&mut if_expr.then_branch)?;
                if let Some(else_expr) = &mut if_expr.else_branch {
                    self.fold_constants_in_expression(else_expr)?;
                }
            }
            Expression::While(while_expr) => {
                self.fold_constants_in_expression(&mut while_expr.condition)?;
                self.fold_constants_in_expression(&mut while_expr.body)?;
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    self.fold_constants_in_statement(stmt)?;
                }
            }
            Expression::FunctionCall(function, arguments) => {
                self.fold_constants_in_expression(function)?;
                for arg in arguments {
                    let mut value = arg.value.clone();
                    self.fold_constants_in_expression(&mut value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn fold_binary_operation(
        &self,
        left: &crate::ast::Literal,
        operator: &crate::ast::BinaryOperator,
        right: &crate::ast::Literal,
    ) -> Result<Option<Expression>, CompilerError> {
        match (left, operator, right) {
            (crate::ast::Literal::Int(a), crate::ast::BinaryOperator::Add, crate::ast::Literal::Int(b)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Int(a + b))))
            }
            (crate::ast::Literal::Int(a), crate::ast::BinaryOperator::Subtract, crate::ast::Literal::Int(b)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Int(a - b))))
            }
            (crate::ast::Literal::Int(a), crate::ast::BinaryOperator::Multiply, crate::ast::Literal::Int(b)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Int(a * b))))
            }
            (crate::ast::Literal::Int(a), crate::ast::BinaryOperator::Divide, crate::ast::Literal::Int(b)) => {
                if *b != 0 {
                    Ok(Some(Expression::Literal(crate::ast::Literal::Int(a / b))))
                } else {
                    Err(CompilerError::semantic_error("division by zero"))
                }
            }
            (crate::ast::Literal::Float(a), crate::ast::BinaryOperator::Add, crate::ast::Literal::Float(b)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Float(a + b))))
            }
            (crate::ast::Literal::Float(a), crate::ast::BinaryOperator::Subtract, crate::ast::Literal::Float(b)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Float(a - b))))
            }
            (crate::ast::Literal::Float(a), crate::ast::BinaryOperator::Multiply, crate::ast::Literal::Float(b)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Float(a * b))))
            }
            (crate::ast::Literal::Float(a), crate::ast::BinaryOperator::Divide, crate::ast::Literal::Float(b)) => {
                if *b != 0.0 {
                    Ok(Some(Expression::Literal(crate::ast::Literal::Float(a / b))))
                } else {
                    Err(CompilerError::semantic_error("division by zero"))
                }
            }
            _ => Ok(None),
        }
    }

    fn fold_unary_operation(
        &self,
        operator: &crate::ast::UnaryOperator,
        operand: &crate::ast::Literal,
    ) -> Result<Option<Expression>, CompilerError> {
        match (operator, operand) {
            (crate::ast::UnaryOperator::Negate, crate::ast::Literal::Int(n)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Int(-n))))
            }
            (crate::ast::UnaryOperator::Negate, crate::ast::Literal::Float(n)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Float(-n))))
            }
            (crate::ast::UnaryOperator::Not, crate::ast::Literal::Bool(b)) => {
                Ok(Some(Expression::Literal(crate::ast::Literal::Bool(!b))))
            }
            _ => Ok(None),
        }
    }

    fn dead_code_elimination_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        let mut reachable = HashSet::new();
        let mut to_visit = Vec::new();

        // Find all reachable functions
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                reachable.insert(func_stmt.name.clone());
                to_visit.push(func_stmt.name.clone());
            }
        }

        // Mark all functions called by reachable functions
        while let Some(func_name) = to_visit.pop() {
            if let Some(func_stmt) = self.find_function(program, &func_name) {
                self.mark_called_functions(func_stmt, &mut reachable, &mut to_visit);
            }
        }

        // Remove unreachable functions
        program.statements.retain(|stmt| {
            if let Statement::Function(func_stmt) = stmt {
                reachable.contains(&func_stmt.name)
            } else {
                true
            }
        });

        self.optimization_stats.transformations_made += 1;
        Ok(())
    }

    fn find_function<'a>(&self, program: &'a Program, name: &str) -> Option<&'a FunctionStatement> {
        for statement in &program.statements {
            if let Statement::Function(func_stmt) = statement {
                if func_stmt.name == name {
                    return Some(func_stmt);
                }
            }
        }
        None
    }

    fn mark_called_functions(
        &self,
        func_stmt: &FunctionStatement,
        reachable: &mut HashSet<String>,
        to_visit: &mut Vec<String>,
    ) {
        self.collect_function_calls(&func_stmt.body, reachable, to_visit);
    }

    fn collect_function_calls(
        &self,
        expr: &Expression,
        reachable: &mut HashSet<String>,
        to_visit: &mut Vec<String>,
    ) {
        match expr {
            Expression::FunctionCall(function, _) => {
                if let Expression::Identifier(name) = &**function {
                    if !reachable.contains(name) {
                        reachable.insert(name.clone());
                        to_visit.push(name.clone());
                    }
                }
            }
            Expression::If(if_expr) => {
                self.collect_function_calls(&if_expr.then_branch, reachable, to_visit);
                if let Some(else_expr) = &if_expr.else_branch {
                    self.collect_function_calls(else_expr, reachable, to_visit);
                }
            }
            Expression::While(while_expr) => {
                self.collect_function_calls(&while_expr.body, reachable, to_visit);
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    if let Statement::Expression(Expression::FunctionCall(function, _)) = stmt {
                        if let Expression::Identifier(name) = &**function {
                            if !reachable.contains(name) {
                                reachable.insert(name.clone());
                                to_visit.push(name.clone());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn function_inlining_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        // This is a simplified inlining pass
        // In a real implementation, you'd need more sophisticated analysis
        for statement in &mut program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.inline_small_functions(func_stmt)?;
            }
        }
        Ok(())
    }

    fn inline_small_functions(&mut self, func_stmt: &mut FunctionStatement) -> Result<(), CompilerError> {
        // Simple heuristic: inline functions with less than 5 expressions
        let complexity = self.estimate_expression_complexity(&func_stmt.body);
        if complexity < 5 {
            // Mark for inlining (in a real implementation, you'd replace call sites)
            self.optimization_stats.transformations_made += 1;
        }
        Ok(())
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

    fn loop_optimization_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        for statement in &mut program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.optimize_loops_in_expression(&mut func_stmt.body)?;
            }
        }
        Ok(())
    }

    fn optimize_loops_in_expression(&mut self, expr: &mut Expression) -> Result<(), CompilerError> {
        match expr {
            Expression::While(while_expr) => {
                // Hoist loop-invariant expressions
                self.hoist_loop_invariants(&mut while_expr.condition, &mut while_expr.body)?;
                
                // Unroll small loops
                if self.should_unroll_loop(&while_expr.condition) {
                    self.unroll_small_loop(&mut while_expr.condition, &mut while_expr.body)?;
                }
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    if let Statement::Expression(Expression::While(while_expr)) = stmt {
                        self.hoist_loop_invariants(&mut while_expr.condition, &mut while_expr.body)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn hoist_loop_invariants(
        &mut self,
        _condition: &mut Expression,
        _body: &mut Expression,
    ) -> Result<(), CompilerError> {
        // Implementation would analyze the loop body and hoist invariant expressions
        // For now, just mark that we attempted optimization
        self.optimization_stats.transformations_made += 1;
        Ok(())
    }

    fn should_unroll_loop(&self, _condition: &Expression) -> bool {
        // Simple heuristic: unroll loops with known small bounds
        // In a real implementation, you'd analyze the loop condition
        false
    }

    fn unroll_small_loop(
        &mut self,
        _condition: &mut Expression,
        _body: &mut Expression,
    ) -> Result<(), CompilerError> {
        // Implementation would replace the loop with unrolled iterations
        self.optimization_stats.transformations_made += 1;
        Ok(())
    }

    fn strength_reduction_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        for statement in &mut program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.apply_strength_reduction(&mut func_stmt.body)?;
            }
        }
        Ok(())
    }

    fn apply_strength_reduction(&mut self, expr: &mut Expression) -> Result<(), CompilerError> {
        // Use a block to limit the borrow of binary_op
        let should_replace = if let Expression::BinaryOp(binary_op) = expr {
            if let Expression::Literal(crate::ast::Literal::Int(n)) = &*binary_op.right {
                binary_op.operator == crate::ast::BinaryOperator::Multiply && *n > 0 && (*n & (*n - 1)) == 0
            } else {
                false
            }
        } else {
            false
        };

        if should_replace {
            if let Expression::BinaryOp(binary_op) = std::mem::replace(expr, Expression::Literal(crate::ast::Literal::Int(0))) {
                if let Expression::Literal(crate::ast::Literal::Int(n)) = *binary_op.right {
                    let shift = n.trailing_zeros() as i64;
                    let new_binary_op = BinaryOp {
                        left: binary_op.left,
                        operator: crate::ast::BinaryOperator::Multiply, // Keep as multiply for now
                        right: Box::new(Expression::Literal(crate::ast::Literal::Int(shift))),
                    };
                    *expr = Expression::BinaryOp(new_binary_op);
                    self.optimization_stats.transformations_made += 1;
                    // Recurse on the new left and right
                    if let Expression::BinaryOp(new_bin) = expr {
                        self.apply_strength_reduction(&mut new_bin.left)?;
                        self.apply_strength_reduction(&mut new_bin.right)?;
                    }
                    return Ok(());
                }
            }
        }

        // Now do the normal recursion
        if let Expression::BinaryOp(binary_op) = expr {
            self.apply_strength_reduction(&mut binary_op.left)?;
            self.apply_strength_reduction(&mut binary_op.right)?;
        }
        Ok(())
    }

    fn cse_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        // Common subexpression elimination
        // This is a simplified implementation
        for statement in &mut program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.eliminate_common_subexpressions(&mut func_stmt.body)?;
            }
        }
        Ok(())
    }

    fn eliminate_common_subexpressions(&mut self, _expr: &mut Expression) -> Result<(), CompilerError> {
        // Implementation would identify and eliminate redundant computations
        // For now, just mark that we attempted optimization
        self.optimization_stats.transformations_made += 1;
        Ok(())
    }

    fn tail_call_optimization_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        for statement in &mut program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.optimize_tail_calls(&mut func_stmt.body)?;
            }
        }
        Ok(())
    }

    fn optimize_tail_calls(&mut self, _expr: &mut Expression) -> Result<(), CompilerError> {
        // Implementation would identify and optimize tail recursive calls
        // For now, just mark that we attempted optimization
        self.optimization_stats.transformations_made += 1;
        Ok(())
    }

    fn vectorization_pass(&mut self, program: &mut Program) -> Result<(), CompilerError> {
        for statement in &mut program.statements {
            if let Statement::Function(func_stmt) = statement {
                self.vectorize_operations(&mut func_stmt.body)?;
            }
        }
        Ok(())
    }

    fn vectorize_operations(&mut self, _expr: &mut Expression) -> Result<(), CompilerError> {
        // Implementation would identify vectorizable operations
        // For now, just mark that we attempted optimization
        self.optimization_stats.transformations_made += 1;
        Ok(())
    }

    fn estimate_code_size(&self, program: &Program) -> usize {
        let mut size = 0;
        for statement in &program.statements {
            size += self.estimate_statement_size(statement);
        }
        size
    }

    fn estimate_statement_size(&self, stmt: &Statement) -> usize {
        match stmt {
            Statement::Function(func_stmt) => {
                10 + self.estimate_expression_size(&func_stmt.body)
            }
            Statement::Let(_) => 5,
            Statement::Return(_) => 3,
            Statement::Expression(_) => 8,
            _ => 1,
        }
    }

    fn estimate_expression_size(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Literal(_) => 1,
            Expression::Identifier(_) => 1,
            Expression::BinaryOp(binary_op) => {
                1 + self.estimate_expression_size(&binary_op.left) + self.estimate_expression_size(&binary_op.right)
            }
            Expression::UnaryOp(unary_op) => {
                1 + self.estimate_expression_size(&unary_op.operand)
            }
            Expression::If(if_expr) => {
                1 + self.estimate_expression_size(&if_expr.condition)
                    + self.estimate_expression_size(&if_expr.then_branch)
                    + if_expr.else_branch.as_ref().map_or(0, |e| self.estimate_expression_size(e))
            }
            Expression::While(while_expr) => {
                1 + self.estimate_expression_size(&while_expr.condition) + self.estimate_expression_size(&while_expr.body)
            }
            Expression::Block(statements) => {
                statements.len() + statements.iter().map(|s| self.estimate_statement_size(s)).sum::<usize>()
            }
            Expression::FunctionCall(function, arguments) => {
                1 + self.estimate_expression_size(function)
                    + arguments.iter().map(|a| self.estimate_expression_size(&a.value)).sum::<usize>()
            }
            _ => 1,
        }
    }

    pub fn get_optimization_stats(&self) -> &OptimizationStats {
        &self.optimization_stats
    }

    pub fn get_passes(&self) -> &[OptimizationPass] {
        &self.passes
    }
} 