use crate::ast::{Program, Statement, Expression, FunctionStatement};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::error::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LintRule {
    pub name: String,
    pub description: String,
    pub severity: LintSeverity,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub rule: String,
    pub message: String,
    pub severity: LintSeverity,
    pub line: usize,
    pub column: usize,
    pub suggestion: Option<String>,
}

pub struct Linter {
    rules: HashMap<String, LintRule>,
    max_line_length: usize,
    allow_unused_variables: bool,
    allow_unused_functions: bool,
}

impl Linter {
    pub fn new() -> Self {
        let mut linter = Self {
            rules: HashMap::new(),
            max_line_length: 100,
            allow_unused_variables: false,
            allow_unused_functions: false,
        };
        linter.register_default_rules();
        linter
    }

    fn register_default_rules(&mut self) {
        let default_rules = vec![
            LintRule {
                name: "unused_variable".to_string(),
                description: "Variable is declared but never used".to_string(),
                severity: LintSeverity::Warning,
                enabled: true,
            },
            LintRule {
                name: "unused_function".to_string(),
                description: "Function is declared but never called".to_string(),
                severity: LintSeverity::Warning,
                enabled: true,
            },
            LintRule {
                name: "long_line".to_string(),
                description: "Line exceeds maximum length".to_string(),
                severity: LintSeverity::Warning,
                enabled: true,
            },
            LintRule {
                name: "missing_return".to_string(),
                description: "Function with return type missing return statement".to_string(),
                severity: LintSeverity::Error,
                enabled: true,
            },
            LintRule {
                name: "unreachable_code".to_string(),
                description: "Code after return statement is unreachable".to_string(),
                severity: LintSeverity::Warning,
                enabled: true,
            },
            LintRule {
                name: "empty_function".to_string(),
                description: "Function has empty body".to_string(),
                severity: LintSeverity::Info,
                enabled: true,
            },
        ];

        for rule in default_rules {
            self.rules.insert(rule.name.clone(), rule);
        }
    }

    pub fn lint_source(&self, source: &str, filename: &str) -> Result<Vec<LintIssue>, CompilerError> {
        let mut issues = Vec::new();
        
        // Parse the source code
        let mut lexer = Lexer::new(source, filename.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        // Check line length
        self.check_line_length(source, &mut issues);
        
        // Analyze AST for issues
        self.analyze_program(&ast, source, &mut issues)?;

        Ok(issues)
    }

    fn check_line_length(&self, source: &str, issues: &mut Vec<LintIssue>) {
        for (line_num, line) in source.lines().enumerate() {
            if line.len() > self.max_line_length {
                issues.push(LintIssue {
                    rule: "long_line".to_string(),
                    message: format!("Line {} exceeds maximum length of {} characters", line_num + 1, self.max_line_length),
                    severity: LintSeverity::Warning,
                    line: line_num + 1,
                    column: self.max_line_length + 1,
                    suggestion: Some("Consider breaking the line into multiple lines".to_string()),
                });
            }
        }
    }

    fn analyze_program(&self, program: &Program, _source: &str, issues: &mut Vec<LintIssue>) -> Result<(), CompilerError> {
        let mut used_variables = HashMap::new();
        let mut declared_functions = HashMap::new();
        let mut called_functions = HashMap::new();

        for statement in &program.statements {
            self.analyze_statement(statement, &mut used_variables, &mut declared_functions, &mut called_functions, issues)?;
        }

        // Check for unused variables
        if !self.allow_unused_variables {
            for (var_name, _) in &used_variables {
                if !used_variables.contains_key(var_name) {
                    issues.push(LintIssue {
                        rule: "unused_variable".to_string(),
                        message: format!("Variable '{}' is declared but never used", var_name),
                        severity: LintSeverity::Warning,
                        line: 1, // TODO: Get actual line number
                        column: 1,
                        suggestion: Some("Remove the variable or use it in your code".to_string()),
                    });
                }
            }
        }

        // Check for unused functions
        if !self.allow_unused_functions {
            for (func_name, _) in &declared_functions {
                if !called_functions.contains_key(func_name) && func_name != "main" {
                    issues.push(LintIssue {
                        rule: "unused_function".to_string(),
                        message: format!("Function '{}' is declared but never called", func_name),
                        severity: LintSeverity::Warning,
                        line: 1, // TODO: Get actual line number
                        column: 1,
                        suggestion: Some("Remove the function or call it from your code".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    fn analyze_statement(
        &self,
        statement: &Statement,
        used_variables: &mut HashMap<String, bool>,
        declared_functions: &mut HashMap<String, bool>,
        called_functions: &mut HashMap<String, bool>,
        issues: &mut Vec<LintIssue>,
    ) -> Result<(), CompilerError> {
        match statement {
            Statement::Function(func) => {
                declared_functions.insert(func.name.clone(), true);
                self.analyze_function(func, used_variables, called_functions, issues)?;
            }
            Statement::Let(let_stmt) => {
                used_variables.insert(let_stmt.name.clone(), false);
                self.analyze_expression(&let_stmt.value, used_variables, called_functions)?;
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr, used_variables, called_functions)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_function(
        &self,
        func: &FunctionStatement,
        used_variables: &mut HashMap<String, bool>,
        called_functions: &mut HashMap<String, bool>,
        issues: &mut Vec<LintIssue>,
    ) -> Result<(), CompilerError> {
        // Check for empty function body
        if let Expression::Block(statements) = &*func.body {
            if statements.is_empty() {
                issues.push(LintIssue {
                    rule: "empty_function".to_string(),
                    message: format!("Function '{}' has empty body", func.name),
                    severity: LintSeverity::Info,
                    line: 1, // TODO: Get actual line number
                    column: 1,
                    suggestion: Some("Add implementation or remove the function".to_string()),
                });
            }

            // Check for missing return statement
            if func.return_type.is_some() {
                let has_return = statements.iter().any(|stmt| matches!(stmt, Statement::Return(_)));
                if !has_return {
                    issues.push(LintIssue {
                        rule: "missing_return".to_string(),
                        message: format!("Function '{}' has return type but no return statement", func.name),
                        severity: LintSeverity::Error,
                        line: 1, // TODO: Get actual line number
                        column: 1,
                        suggestion: Some("Add a return statement or change return type to Void".to_string()),
                    });
                }
            }
        }

        // Analyze function body
        self.analyze_expression(&func.body, used_variables, called_functions)?;
        Ok(())
    }

    fn analyze_expression(
        &self,
        expression: &Expression,
        used_variables: &mut HashMap<String, bool>,
        called_functions: &mut HashMap<String, bool>,
    ) -> Result<(), CompilerError> {
        match expression {
            Expression::Identifier(name) => {
                used_variables.insert(name.clone(), true);
            }
            Expression::FunctionCall(callee, _) => {
                if let Expression::Identifier(func_name) = callee.as_ref() {
                    called_functions.insert(func_name.clone(), true);
                }
            }
            Expression::BinaryOp(binary_op) => {
                self.analyze_expression(&binary_op.left, used_variables, called_functions)?;
                self.analyze_expression(&binary_op.right, used_variables, called_functions)?;
            }
            Expression::Block(statements) => {
                for statement in statements {
                    self.analyze_statement(statement, used_variables, &mut HashMap::new(), called_functions, &mut Vec::new())?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn with_max_line_length(mut self, length: usize) -> Self {
        self.max_line_length = length;
        self
    }

    pub fn with_allow_unused_variables(mut self, allow: bool) -> Self {
        self.allow_unused_variables = allow;
        self
    }

    pub fn with_allow_unused_functions(mut self, allow: bool) -> Self {
        self.allow_unused_functions = allow;
        self
    }

    pub fn enable_rule(&mut self, rule_name: &str) {
        if let Some(rule) = self.rules.get_mut(rule_name) {
            rule.enabled = true;
        }
    }

    pub fn disable_rule(&mut self, rule_name: &str) {
        if let Some(rule) = self.rules.get_mut(rule_name) {
            rule.enabled = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_unused_variable() {
        let linter = Linter::new();
        let source = "fn main() { let x = 42; }";
        let issues = linter.lint_source(source, "test.nx").unwrap();
        
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.rule == "unused_variable"));
    }

    #[test]
    fn test_lint_missing_return() {
        let linter = Linter::new();
        let source = "fn main() -> Int { let x = 42; }";
        let issues = linter.lint_source(source, "test.nx").unwrap();
        
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.rule == "missing_return"));
    }

    #[test]
    fn test_lint_long_line() {
        let linter = Linter::new().with_max_line_length(10);
        let source = "fn main() { let very_long_variable_name = 42; }";
        let issues = linter.lint_source(source, "test.nx").unwrap();
        
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.rule == "long_line"));
    }
} 