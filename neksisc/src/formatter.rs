use crate::ast::{Program, Statement, Expression, FunctionStatement, LetStatement, ReturnStatement};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::error::CompilerError;
// Removed unused import

pub struct CodeFormatter {
    indent_size: usize,
    max_line_length: usize,
    use_spaces: bool,
}

impl CodeFormatter {
    pub fn new() -> Self {
        Self {
            indent_size: 4,
            max_line_length: 100,
            use_spaces: true,
        }
    }

    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    pub fn with_max_line_length(mut self, length: usize) -> Self {
        self.max_line_length = length;
        self
    }

    pub fn with_use_spaces(mut self, use_spaces: bool) -> Self {
        self.use_spaces = use_spaces;
        self
    }

    pub fn format_source(&self, source: &str) -> Result<String, CompilerError> {
        // Parse the source code
        let mut lexer = Lexer::new(source, "format".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        // Format the AST
        let mut formatted = String::new();
        self.format_program(&ast, &mut formatted, 0)?;
        
        Ok(formatted)
    }

    fn format_program(&self, program: &Program, output: &mut String, indent: usize) -> Result<(), CompilerError> {
        for (i, statement) in program.statements.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            self.format_statement(statement, output, indent)?;
        }
        Ok(())
    }

    fn format_statement(&self, statement: &Statement, output: &mut String, indent: usize) -> Result<(), CompilerError> {
        match statement {
            Statement::Function(func) => self.format_function(func, output, indent)?,
            Statement::Let(let_stmt) => self.format_let_statement(let_stmt, output, indent)?,
            Statement::Return(return_stmt) => self.format_return_statement(return_stmt, output, indent)?,
            Statement::Expression(expr) => {
                self.add_indent(output, indent);
                self.format_expression(expr, output)?;
                output.push(';');
            }
            _ => {
                // For other statement types, add a placeholder
                self.add_indent(output, indent);
                output.push_str("// TODO: Format this statement type");
            }
        }
        Ok(())
    }

    fn format_function(&self, func: &FunctionStatement, output: &mut String, indent: usize) -> Result<(), CompilerError> {
        self.add_indent(output, indent);
        output.push_str("fn ");
        output.push_str(&func.name);
        output.push('(');
        
        // Format parameters
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&param.name);
            output.push_str(": ");
            self.format_type(&param.type_annotation, output)?;
        }
        output.push(')');
        
        // Format return type
        if let Some(return_type) = &func.return_type {
            output.push_str(" -> ");
            self.format_type(return_type, output)?;
        }
        
        output.push_str(" {\n");
        
        // Format function body
        if let Expression::Block(statements) = &*func.body {
            for statement in statements {
                self.format_statement(statement, output, indent + 1)?;
                output.push('\n');
            }
        }
        
        self.add_indent(output, indent);
        output.push('}');
        Ok(())
    }

    fn format_let_statement(&self, let_stmt: &LetStatement, output: &mut String, indent: usize) -> Result<(), CompilerError> {
        self.add_indent(output, indent);
        output.push_str("let ");
        output.push_str(&let_stmt.name);
        
        if let Some(type_annotation) = &let_stmt.type_annotation {
            output.push_str(": ");
            self.format_type(type_annotation, output)?;
        }
        
        output.push_str(" = ");
        self.format_expression(&let_stmt.value, output)?;
        output.push(';');
        Ok(())
    }

    fn format_return_statement(&self, return_stmt: &ReturnStatement, output: &mut String, indent: usize) -> Result<(), CompilerError> {
        self.add_indent(output, indent);
        output.push_str("return");
        
        if let Some(value) = &return_stmt.value {
            output.push(' ');
            self.format_expression(value, output)?;
        }
        
        output.push(';');
        Ok(())
    }

    fn format_expression(&self, expression: &Expression, output: &mut String) -> Result<(), CompilerError> {
        match expression {
            Expression::Literal(literal) => self.format_literal(literal, output)?,
            Expression::Identifier(name) => output.push_str(name),
            Expression::BinaryOp(binary_op) => {
                self.format_expression(&binary_op.left, output)?;
                output.push(' ');
                output.push_str(&self.format_operator(&binary_op.operator));
                output.push(' ');
                self.format_expression(&binary_op.right, output)?;
            }
            Expression::FunctionCall(callee, args) => {
                self.format_expression(callee, output)?;
                output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    self.format_expression(&arg.value, output)?;
                }
                output.push(')');
            }
            Expression::Block(statements) => {
                output.push_str("{\n");
                for statement in statements {
                    self.format_statement(statement, output, 1)?;
                    output.push('\n');
                }
                output.push('}');
            }
            _ => {
                // For other expression types, add a placeholder
                output.push_str("/* TODO: Format this expression type */");
            }
        }
        Ok(())
    }

    fn format_literal(&self, literal: &crate::ast::Literal, output: &mut String) -> Result<(), CompilerError> {
        match literal {
            crate::ast::Literal::Int(value) => output.push_str(&value.to_string()),
            crate::ast::Literal::Float(value) => output.push_str(&value.to_string()),
            crate::ast::Literal::String(value) => {
                output.push('"');
                output.push_str(value);
                output.push('"');
            }
            crate::ast::Literal::Bool(value) => output.push_str(&value.to_string()),
            _ => output.push_str("/* TODO: Format this literal type */"),
        }
        Ok(())
    }

    fn format_type(&self, type_expr: &crate::ast::Type, output: &mut String) -> Result<(), CompilerError> {
        match type_expr {
            crate::ast::Type::Int => output.push_str("Int"),
            crate::ast::Type::Float => output.push_str("Float"),
            crate::ast::Type::String => output.push_str("String"),
            crate::ast::Type::Bool => output.push_str("Bool"),
            crate::ast::Type::Void => output.push_str("Void"),
            _ => output.push_str("/* TODO: Format this type */"),
        }
        Ok(())
    }

    fn format_operator(&self, op: &crate::ast::BinaryOperator) -> &str {
        match op {
            crate::ast::BinaryOperator::Add => "+",
            crate::ast::BinaryOperator::Subtract => "-",
            crate::ast::BinaryOperator::Multiply => "*",
            crate::ast::BinaryOperator::Divide => "/",
            crate::ast::BinaryOperator::Equal => "==",
            crate::ast::BinaryOperator::NotEqual => "!=",
            crate::ast::BinaryOperator::LessThan => "<",
            crate::ast::BinaryOperator::LessThanOrEqual => "<=",
            crate::ast::BinaryOperator::GreaterThan => ">",
            crate::ast::BinaryOperator::GreaterThanOrEqual => ">=",
            _ => "/* TODO: Format this operator */",
        }
    }

    fn add_indent(&self, output: &mut String, indent: usize) {
        let indent_str = if self.use_spaces {
            " ".repeat(indent * self.indent_size)
        } else {
            "\t".repeat(indent)
        };
        output.push_str(&indent_str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_basic_function() {
        let formatter = CodeFormatter::new();
        let source = "fn main() { let x = 42; return x; }";
        let formatted = formatter.format_source(source).unwrap();
        
        assert!(formatted.contains("fn main()"));
        assert!(formatted.contains("let x = 42;"));
        assert!(formatted.contains("return x;"));
    }

    #[test]
    fn test_format_with_parameters() {
        let formatter = CodeFormatter::new();
        let source = "fn add(x: Int, y: Int) -> Int { return x + y; }";
        let formatted = formatter.format_source(source).unwrap();
        
        assert!(formatted.contains("fn add(x: Int, y: Int) -> Int"));
    }
} 