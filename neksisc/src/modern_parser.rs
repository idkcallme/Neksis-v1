// Modern Parser for Neksis 2025
use crate::modern_ast::*;
use crate::modern_lexer::{Token, TokenInfo};

pub struct Parser {
    tokens: Vec<TokenInfo>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }
    
    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        let mut modules = std::collections::HashMap::new();
        
        while !self.is_at_end() {
            // Skip newlines at top level
            if self.check(&Token::Newline) {
                self.advance();
                continue;
            }
            
            match self.parse_statement() {
                Ok(stmt) => {
                    if let Statement::Module(module_stmt) = &stmt {
                        modules.insert(module_stmt.name.clone(), Module {
                            name: module_stmt.name.clone(),
                            statements: module_stmt.statements.clone(),
                            exports: Vec::new(), // TODO: Parse exports
                            imports: Vec::new(), // TODO: Parse imports
                        });
                    }
                    statements.push(stmt);
                },
                Err(e) => {
                    // Error recovery: skip to next statement
                    eprintln!("Parse error: {}", e);
                    self.recover_to_next_statement();
                }
            }
        }
        
        Ok(Program { statements, modules })
    }
    
    // Parser utilities
    fn advance(&mut self) -> &TokenInfo {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().token, Token::Eof)
    }
    
    fn peek(&self) -> &TokenInfo {
        self.tokens.get(self.current).unwrap_or(&TokenInfo {
            token: Token::Eof,
            line: 0,
            column: 0,
            span: crate::modern_lexer::Span { start: 0, end: 0 },
        })
    }
    
    fn previous(&self) -> &TokenInfo {
        if self.current > 0 {
            &self.tokens[self.current - 1]
        } else {
            &self.tokens[0]
        }
    }
    
    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token) == std::mem::discriminant(token_type)
        }
    }
    
    fn match_token(&mut self, token_type: &Token) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn consume(&mut self, token_type: &Token, message: &str) -> Result<&TokenInfo, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(format!("{} at line {}", message, self.peek().line))
        }
    }
    
    fn recover_to_next_statement(&mut self) {
        while !self.is_at_end() {
            match &self.peek().token {
                Token::Let | Token::Fn | Token::Struct | Token::Enum | 
                Token::Class | Token::Module | Token::Use | Token::Import => break,
                Token::Semicolon => {
                    self.advance();
                    break;
                },
                _ => {
                    self.advance();
                }
            }
        }
    }
    
    // Statement parsing
    fn parse_statement(&mut self) -> Result<Statement, String> {
        match &self.peek().token {
            Token::Let => {
                self.advance();
                Ok(Statement::Let(self.parse_let_statement()?))
            },
            Token::Fn => {
                self.advance();
                Ok(Statement::Function(self.parse_function_statement()?))
            },
            Token::Struct => {
                self.advance();
                Ok(Statement::Struct(self.parse_struct_statement()?))
            },
            Token::Enum => {
                self.advance();
                Ok(Statement::Enum(self.parse_enum_statement()?))
            },
            Token::Class => {
                self.advance();
                Ok(Statement::Class(self.parse_class_statement()?))
            },
            Token::Module => {
                self.advance();
                Ok(Statement::Module(self.parse_module_statement()?))
            },
            Token::Use => {
                self.advance();
                Ok(Statement::Use(self.parse_use_statement()?))
            },
            Token::Return => {
                self.advance();
                Ok(Statement::Return(self.parse_return_statement()?))
            },
            Token::Break => {
                self.advance();
                self.consume(&Token::Semicolon, "Expected ';' after 'break'")?;
                Ok(Statement::Break)
            },
            Token::Continue => {
                self.advance();
                self.consume(&Token::Semicolon, "Expected ';' after 'continue'")?;
                Ok(Statement::Continue)
            },
            Token::Throw => {
                self.advance();
                Ok(Statement::Throw(self.parse_throw_statement()?))
            },
            _ => {
                // Expression statement
                let expr = self.parse_expression()?;
                
                // Only require semicolon for non-block expressions
                if !matches!(expr, Expression::Block { .. } | Expression::If { .. } | 
                           Expression::While { .. } | Expression::For { .. } | 
                           Expression::Loop { .. } | Expression::Match { .. }) {
                    self.consume(&Token::Semicolon, "Expected ';' after expression")?;
                }
                
                Ok(Statement::Expression(expr))
            }
        }
    }
    
    fn parse_let_statement(&mut self) -> Result<LetStatement, String> {
        let is_mutable = self.match_token(&Token::Mut);
        
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected identifier after 'let'".to_string()),
        };
        
        let type_annotation = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(&Token::Assign, "Expected '=' in let statement")?;
        let value = Box::new(self.parse_expression()?);
        
        self.consume(&Token::Semicolon, "Expected ';' after let statement")?;
        
        Ok(LetStatement {
            name,
            type_annotation,
            value,
            is_mutable,
        })
    }
    
    fn parse_function_statement(&mut self) -> Result<FunctionStatement, String> {
        let is_async = if self.current >= 2 {
            matches!(self.tokens.get(self.current - 2), Some(TokenInfo { token: Token::Async, .. }))
        } else {
            false
        };
        
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected function name".to_string()),
        };
        
        // Parse generic parameters
        let generic_params = if self.check(&Token::LeftAngle) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        
        self.consume(&Token::LeftParen, "Expected '(' after function name")?;
        
        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                parameters.push(self.parse_parameter()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightParen, "Expected ')' after parameters")?;
        
        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = Box::new(self.parse_block_expression()?);
        
        Ok(FunctionStatement {
            name,
            parameters,
            return_type,
            body,
            generic_params,
            is_async,
        })
    }
    
    fn parse_generic_params(&mut self) -> Result<Vec<String>, String> {
        self.consume(&Token::LeftAngle, "Expected '<'")?;
        
        let mut params = Vec::new();
        if !self.check(&Token::RightAngle) {
            loop {
                match &self.peek().token {
                    Token::Identifier(name) => {
                        params.push(name.clone());
                        self.advance();
                    },
                    _ => return Err("Expected type parameter name".to_string()),
                }
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightAngle, "Expected '>'")?;
        Ok(params)
    }
    
    fn parse_parameter(&mut self) -> Result<Parameter, String> {
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected parameter name".to_string()),
        };
        
        self.consume(&Token::Colon, "Expected ':' after parameter name")?;
        let type_annotation = self.parse_type()?;
        
        let default_value = if self.match_token(&Token::Assign) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        
        Ok(Parameter {
            name,
            type_annotation,
            default_value,
        })
    }
    
    fn parse_struct_statement(&mut self) -> Result<StructStatement, String> {
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected struct name".to_string()),
        };
        
        let generic_params = if self.check(&Token::LeftAngle) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        
        self.consume(&Token::LeftBrace, "Expected '{' after struct name")?;
        
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Skip newlines in struct body
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            let is_public = self.match_token(&Token::Pub);
            
            let field_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                },
                _ => return Err("Expected field name".to_string()),
            };
            
            self.consume(&Token::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;
            
            fields.push(StructField {
                name: field_name,
                field_type,
                is_public,
            });
            
            // Allow trailing comma
            if self.match_token(&Token::Comma) || self.match_token(&Token::Newline) {
                continue;
            } else if self.check(&Token::RightBrace) {
                break;
            } else {
                return Err("Expected ',' or '}' after struct field".to_string());
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after struct fields")?;
        
        Ok(StructStatement {
            name,
            fields,
            generic_params,
        })
    }
    
    fn parse_enum_statement(&mut self) -> Result<EnumStatement, String> {
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected enum name".to_string()),
        };
        
        let generic_params = if self.check(&Token::LeftAngle) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        
        self.consume(&Token::LeftBrace, "Expected '{' after enum name")?;
        
        let mut variants = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            let variant_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                },
                _ => return Err("Expected variant name".to_string()),
            };
            
            let fields = if self.match_token(&Token::LeftParen) {
                let mut field_types = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        field_types.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(&Token::RightParen, "Expected ')' after variant fields")?;
                field_types
            } else {
                Vec::new()
            };
            
            variants.push(EnumVariant {
                name: variant_name,
                fields,
            });
            
            if self.match_token(&Token::Comma) || self.match_token(&Token::Newline) {
                continue;
            } else if self.check(&Token::RightBrace) {
                break;
            } else {
                return Err("Expected ',' or '}' after enum variant".to_string());
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after enum variants")?;
        
        Ok(EnumStatement {
            name,
            variants,
            generic_params,
        })
    }
    
    fn parse_class_statement(&mut self) -> Result<ClassStatement, String> {
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected class name".to_string()),
        };
        
        let generic_params = if self.check(&Token::LeftAngle) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        
        let superclass = if self.match_token(&Token::Colon) {
            match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    Some(name)
                },
                _ => return Err("Expected superclass name".to_string()),
            }
        } else {
            None
        };
        
        self.consume(&Token::LeftBrace, "Expected '{' after class declaration")?;
        
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            let is_public = self.match_token(&Token::Pub);
            
            if self.check(&Token::Fn) {
                self.advance();
                let method = self.parse_function_statement()?;
                methods.push(method);
            } else {
                // Field
                let field_name = match &self.peek().token {
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    },
                    _ => return Err("Expected field or method name".to_string()),
                };
                
                self.consume(&Token::Colon, "Expected ':' after field name")?;
                let field_type = self.parse_type()?;
                
                fields.push(StructField {
                    name: field_name,
                    field_type,
                    is_public,
                });
                
                if !self.match_token(&Token::Comma) && !self.match_token(&Token::Newline) && !self.check(&Token::RightBrace) {
                    return Err("Expected ',' or newline after class field".to_string());
                }
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after class body")?;
        
        Ok(ClassStatement {
            name,
            fields,
            methods,
            superclass,
            generic_params,
        })
    }
    
    fn parse_module_statement(&mut self) -> Result<ModuleStatement, String> {
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected module name".to_string()),
        };
        
        self.consume(&Token::LeftBrace, "Expected '{' after module name")?;
        
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            statements.push(self.parse_statement()?);
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after module body")?;
        
        Ok(ModuleStatement {
            name,
            statements,
        })
    }
    
    fn parse_use_statement(&mut self) -> Result<UseStatement, String> {
        let mut items = Vec::new();
        let mut path: String;
        
        if self.match_token(&Token::LeftBrace) {
            // use { item1, item2, ... } from "path"
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                match &self.peek().token {
                    Token::Identifier(name) => {
                        items.push(name.clone());
                        self.advance();
                    },
                    _ => return Err("Expected identifier in use statement".to_string()),
                }
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::RightBrace, "Expected '}'")?;
            self.consume(&Token::From, "Expected 'from'")?;
        }
        
        // Parse path
        match &self.peek().token {
            Token::String(p) => {
                path = p.clone();
                self.advance();
            },
            Token::Identifier(p) => {
                path = p.clone();
                self.advance();
                
                // Handle dot notation: use std.collections.HashMap
                while self.match_token(&Token::Dot) {
                    match &self.peek().token {
                        Token::Identifier(part) => {
                            path.push('.');
                            path.push_str(part);
                            self.advance();
                        },
                        _ => return Err("Expected identifier after '.'".to_string()),
                    }
                }
            },
            _ => return Err("Expected path in use statement".to_string()),
        }
        
        let alias = if self.match_token(&Token::As) {
            match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    Some(name)
                },
                _ => return Err("Expected alias after 'as'".to_string()),
            }
        } else {
            None
        };
        
        self.consume(&Token::Semicolon, "Expected ';' after use statement")?;
        
        Ok(UseStatement {
            path,
            items,
            alias,
        })
    }
    
    fn parse_return_statement(&mut self) -> Result<ReturnStatement, String> {
        let value = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        
        self.consume(&Token::Semicolon, "Expected ';' after return statement")?;
        
        Ok(ReturnStatement { value })
    }
    
    fn parse_throw_statement(&mut self) -> Result<ThrowStatement, String> {
        let value = Box::new(self.parse_expression()?);
        self.consume(&Token::Semicolon, "Expected ';' after throw statement")?;
        Ok(ThrowStatement { value })
    }
    
    // Type parsing
    fn parse_type(&mut self) -> Result<Type, String> {
        match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                match name.as_str() {
                    "Int" | "i32" | "i64" => Ok(Type::Int),
                    "Float" | "f32" | "f64" => Ok(Type::Float),
                    "String" | "str" => Ok(Type::String),
                    "Bool" | "bool" => Ok(Type::Boolean),
                    "Void" | "void" => Ok(Type::Void),
                    _ => {
                        // Check for generic type arguments
                        if self.match_token(&Token::LeftAngle) {
                            let mut args = Vec::new();
                            if !self.check(&Token::RightAngle) {
                                loop {
                                    args.push(self.parse_type()?);
                                    if !self.match_token(&Token::Comma) {
                                        break;
                                    }
                                }
                            }
                            self.consume(&Token::RightAngle, "Expected '>'")?;
                            Ok(Type::Generic(name, args))
                        } else {
                            // Could be struct, enum, or class
                            Ok(Type::Struct(name))
                        }
                    }
                }
            },
            Token::LeftBracket => {
                self.advance();
                let element_type = Box::new(self.parse_type()?);
                self.consume(&Token::RightBracket, "Expected ']'")?;
                Ok(Type::Array(element_type))
            },
            Token::Ampersand => {
                self.advance();
                if self.match_token(&Token::Mut) {
                    let inner_type = Box::new(self.parse_type()?);
                    Ok(Type::MutableReference(inner_type))
                } else {
                    let inner_type = Box::new(self.parse_type()?);
                    Ok(Type::Reference(inner_type))
                }
            },
            _ => Err(format!("Unexpected token in type: {:?}", self.peek().token)),
        }
    }
    
    // Expression parsing with proper precedence
    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_assignment()
    }
    
    fn parse_assignment(&mut self) -> Result<Expression, String> {
        let expr = self.parse_or()?;
        
        if self.match_token(&Token::Assign) {
            let value = Box::new(self.parse_assignment()?);
            if let Expression::Identifier(name) = expr {
                return Ok(Expression::Assignment {
                    target: name,
                    value,
                });
            } else {
                return Err("Invalid assignment target".to_string());
            }
        }
        
        Ok(expr)
    }
    
    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&Token::Or) {
            let operator = BinaryOperator::Or;
            let right = Box::new(self.parse_and()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&Token::And) {
            let operator = BinaryOperator::And;
            let right = Box::new(self.parse_equality()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;
        
        while let Some(op) = self.match_equality_op() {
            let right = Box::new(self.parse_comparison()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn match_equality_op(&mut self) -> Option<BinaryOperator> {
        if self.match_token(&Token::Equal) {
            Some(BinaryOperator::Equal)
        } else if self.match_token(&Token::NotEqual) {
            Some(BinaryOperator::NotEqual)
        } else {
            None
        }
    }
    
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_term()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = Box::new(self.parse_term()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn match_comparison_op(&mut self) -> Option<BinaryOperator> {
        if self.match_token(&Token::Greater) {
            Some(BinaryOperator::Greater)
        } else if self.match_token(&Token::GreaterEqual) {
            Some(BinaryOperator::GreaterEqual)
        } else if self.match_token(&Token::Less) {
            Some(BinaryOperator::Less)
        } else if self.match_token(&Token::LessEqual) {
            Some(BinaryOperator::LessEqual)
        } else {
            None
        }
    }
    
    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_factor()?;
        
        while let Some(op) = self.match_term_op() {
            let right = Box::new(self.parse_factor()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn match_term_op(&mut self) -> Option<BinaryOperator> {
        if self.match_token(&Token::Minus) {
            Some(BinaryOperator::Subtract)
        } else if self.match_token(&Token::Plus) {
            Some(BinaryOperator::Add)
        } else {
            None
        }
    }
    
    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;
        
        while let Some(op) = self.match_factor_op() {
            let right = Box::new(self.parse_unary()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn match_factor_op(&mut self) -> Option<BinaryOperator> {
        if self.match_token(&Token::Slash) {
            Some(BinaryOperator::Divide)
        } else if self.match_token(&Token::Star) {
            Some(BinaryOperator::Multiply)
        } else if self.match_token(&Token::Percent) {
            Some(BinaryOperator::Modulo)
        } else {
            None
        }
    }
    
    fn parse_unary(&mut self) -> Result<Expression, String> {
        if let Some(op) = self.match_unary_op() {
            let operand = Box::new(self.parse_unary()?);
            Ok(Expression::Unary {
                operator: op,
                operand,
            })
        } else {
            self.parse_call()
        }
    }
    
    fn match_unary_op(&mut self) -> Option<UnaryOperator> {
        if self.match_token(&Token::Not) {
            Some(UnaryOperator::Not)
        } else if self.match_token(&Token::Minus) {
            Some(UnaryOperator::Minus)
        } else if self.match_token(&Token::Plus) {
            Some(UnaryOperator::Plus)
        } else {
            None
        }
    }
    
    fn parse_call(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&Token::LeftParen) {
                // Function call
                let mut arguments = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(&Token::RightParen, "Expected ')' after arguments")?;
                
                expr = Expression::Call {
                    function: Box::new(expr),
                    arguments,
                };
            } else if self.match_token(&Token::Dot) {
                // Member access
                let member = match &self.peek().token {
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    },
                    _ => return Err("Expected member name after '.'".to_string()),
                };
                
                expr = Expression::MemberAccess {
                    object: Box::new(expr),
                    member,
                };
            } else if self.match_token(&Token::LeftBracket) {
                // Array access
                let index = Box::new(self.parse_expression()?);
                self.consume(&Token::RightBracket, "Expected ']'")?;
                
                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expression, String> {
        match &self.peek().token {
            Token::Integer(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(value)))
            },
            Token::Float(f) => {
                let value = *f;
                self.advance();
                Ok(Expression::Literal(Literal::Float(value)))
            },
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(value)))
            },
            Token::Boolean(b) => {
                let value = *b;
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(value)))
            },
            Token::Null => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            },
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Check for struct literal
                if self.check(&Token::LeftBrace) {
                    self.parse_struct_literal(name)
                } else {
                    Ok(Expression::Identifier(name))
                }
            },
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RightParen, "Expected ')'")?;
                Ok(expr)
            },
            Token::LeftBracket => {
                self.parse_array_literal()
            },
            Token::LeftBrace => {
                self.parse_block_expression()
            },
            Token::If => {
                self.advance();
                self.parse_if_expression()
            },
            Token::While => {
                self.advance();
                self.parse_while_expression()
            },
            Token::For => {
                self.advance();
                self.parse_for_expression()
            },
            Token::Loop => {
                self.advance();
                self.parse_loop_expression()
            },
            Token::Match => {
                self.advance();
                self.parse_match_expression()
            },
            _ => Err(format!("Unexpected token: {:?}", self.peek().token)),
        }
    }
    
    fn parse_struct_literal(&mut self, name: String) -> Result<Expression, String> {
        self.consume(&Token::LeftBrace, "Expected '{'")?;
        
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            let field_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                },
                _ => return Err("Expected field name".to_string()),
            };
            
            self.consume(&Token::Colon, "Expected ':' after field name")?;
            let field_value = self.parse_expression()?;
            
            fields.push((field_name, field_value));
            
            if !self.match_token(&Token::Comma) && !self.match_token(&Token::Newline) {
                break;
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}'")?;
        
        Ok(Expression::StructLiteral {
            name,
            fields,
        })
    }
    
    fn parse_array_literal(&mut self) -> Result<Expression, String> {
        self.consume(&Token::LeftBracket, "Expected '['")?;
        
        let mut elements = Vec::new();
        if !self.check(&Token::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightBracket, "Expected ']'")?;
        
        Ok(Expression::Array { elements })
    }
    
    fn parse_block_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        let mut expression = None;
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            // Check if this is the last expression (no semicolon)
            let checkpoint = self.current;
            match self.parse_statement() {
                Ok(Statement::Expression(expr)) => {
                    // If the next token is }, this is the block's return expression
                    if self.check(&Token::RightBrace) {
                        expression = Some(Box::new(expr));
                        break;
                    } else {
                        statements.push(Statement::Expression(expr));
                    }
                },
                Ok(stmt) => {
                    statements.push(stmt);
                },
                Err(_) => {
                    // Try parsing as expression without semicolon
                    self.current = checkpoint;
                    match self.parse_expression() {
                        Ok(expr) => {
                            if self.check(&Token::RightBrace) {
                                expression = Some(Box::new(expr));
                                break;
                            } else {
                                return Err("Expected ';' after expression".to_string());
                            }
                        },
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}'")?;
        
        Ok(Expression::Block {
            statements,
            expression,
        })
    }
    
    fn parse_if_expression(&mut self) -> Result<Expression, String> {
        let condition = Box::new(self.parse_expression()?);
        let then_branch = Box::new(self.parse_block_expression()?);
        
        let else_branch = if self.match_token(&Token::Else) {
            if self.check(&Token::If) {
                // else if
                self.advance();
                Some(Box::new(self.parse_if_expression()?))
            } else {
                // else block
                Some(Box::new(self.parse_block_expression()?))
            }
        } else {
            None
        };
        
        Ok(Expression::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    fn parse_while_expression(&mut self) -> Result<Expression, String> {
        let condition = Box::new(self.parse_expression()?);
        let body = Box::new(self.parse_block_expression()?);
        
        Ok(Expression::While {
            condition,
            body,
        })
    }
    
    fn parse_for_expression(&mut self) -> Result<Expression, String> {
        let variable = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err("Expected variable name in for loop".to_string()),
        };
        
        self.consume(&Token::In, "Expected 'in' in for loop")?;
        let iterable = Box::new(self.parse_expression()?);
        let body = Box::new(self.parse_block_expression()?);
        
        Ok(Expression::For {
            variable,
            iterable,
            body,
        })
    }
    
    fn parse_loop_expression(&mut self) -> Result<Expression, String> {
        let body = Box::new(self.parse_block_expression()?);
        
        Ok(Expression::Loop {
            body,
        })
    }
    
    fn parse_match_expression(&mut self) -> Result<Expression, String> {
        let expression = Box::new(self.parse_expression()?);
        self.consume(&Token::LeftBrace, "Expected '{' after match expression")?;
        
        let mut arms = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            let pattern = self.parse_pattern()?;
            
            let guard = if self.match_token(&Token::If) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            
            self.consume(&Token::FatArrow, "Expected '=>' after match pattern")?;
            let body = Box::new(self.parse_expression()?);
            
            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
            
            if !self.match_token(&Token::Comma) && !self.match_token(&Token::Newline) {
                break;
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after match arms")?;
        
        Ok(Expression::Match {
            expression,
            arms,
        })
    }
    
    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        match &self.peek().token {
            Token::Integer(n) => {
                let value = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Integer(value)))
            },
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(value)))
            },
            Token::Boolean(b) => {
                let value = *b;
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(value)))
            },
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Pattern::Identifier(name))
            },
            _ => Err("Invalid pattern".to_string()),
        }
    }
}
