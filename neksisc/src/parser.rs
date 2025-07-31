use crate::ast::*;
use crate::lexer::{Token, TokenInfo, InterpolatedPart as LexerInterpolatedPart};
use crate::ast::InterpolatedPart;

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
        
        while !self.is_at_end() {
            if let Some(statement) = self.parse_statement()? {
                statements.push(statement);
            } else {
                if !self.is_at_end() {
                    self.advance();
                }
            }
        }
        
        Ok(Program {
            statements,
            annotations: Vec::new(),
        })
    }
    
    // Parser Infrastructure
    fn advance(&mut self) -> &TokenInfo {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn peek(&self) -> &Token {
        if self.is_at_end() {
            &Token::Eof
        } else {
            &self.tokens[self.current].token
        }
    }
    
    fn peek_next(&self) -> &Token {
        if self.current + 1 >= self.tokens.len() {
            &Token::Eof
        } else {
            &self.tokens[self.current + 1].token
        }
    }
    
    fn previous(&self) -> &TokenInfo {
        &self.tokens[self.current - 1]
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    fn check(&self, token: &Token) -> bool {
        !self.is_at_end() && self.peek() == token
    }
    
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn consume(&mut self, token: &Token, message: &str) -> Result<&TokenInfo, String> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(format!("{} at line {}", message, self.previous().line))
        }
    }
    
    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            if self.previous().token == Token::Semicolon {
                return;
            }
            
            match self.peek() {
                Token::Fn | Token::Let | Token::Struct | Token::Enum | Token::Use => {
                    return;
                }
                _ => {}
            }
            
            self.advance();
        }
    }
    
    // Parse Annotations
    fn parse_annotation(&mut self) -> Result<Annotation, String> {
        let _start_line = self.previous().line;
        let _start_column = self.previous().column;
        
        // Parse annotation name (identifier)
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected annotation name".to_string());
        };
        
        // Parse arguments in parentheses
        let arguments = if self.match_token(&Token::LeftParen) {
            let mut args = Vec::new();
            
            if !self.check(&Token::RightParen) {
                loop {
                    if let Token::String(arg) = &self.peek() {
                        args.push(arg.clone());
                        self.advance();
                    } else {
                        return Err("Expected string argument in annotation".to_string());
                    }
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(&Token::RightParen, "Expected ')' after annotation arguments")?;
            args
        } else {
            Vec::new()
        };
        
        Ok(Annotation {
            name,
            arguments: arguments.into_iter().map(|arg| Expression::Literal(Literal::String(arg))).collect(),
            attached_to: None, // Will be set later when we know what follows
        })
    }
    
    // Parse Statements
    fn parse_statement(&mut self) -> Result<Option<Statement>, String> {
        if self.match_token(&Token::Let) {
            return Ok(Some(Statement::Let(self.parse_let_statement()?)));
        } else if self.match_token(&Token::Fn) {
            return Ok(Some(Statement::Function(self.parse_function_statement()?)));
        } else if self.match_token(&Token::Struct) {
            return Ok(Some(Statement::Struct(self.parse_struct_statement()?)));
        } else if self.match_token(&Token::Enum) {
            return Ok(Some(Statement::Enum(self.parse_enum_statement()?)));
        } else if self.match_token(&Token::Class) {
            return Ok(Some(Statement::Class(self.parse_class_statement()?)));
        } else if self.match_token(&Token::Use) {
            return Ok(Some(Statement::Use(self.parse_use_statement()?)));
        } else if self.match_token(&Token::Import) {
            // import module or import module as alias
            let module = if let Token::Identifier(name) = &self.peek() {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err("Expected module name after 'import'".to_string());
            };
            let alias = if self.match_token(&Token::As) {
                if let Token::Identifier(name) = &self.peek() {
                    let name = name.clone();
                    self.advance();
                    Some(name)
                } else {
                    return Err("Expected alias after 'as' in import".to_string());
                }
            } else {
                None
            };
            if !self.match_token(&Token::Semicolon) {
                return Err("Expected ';' after import statement".to_string());
            }
            return Ok(Some(Statement::Use(UseStatement { path: module, alias })));
        } else if self.match_token(&Token::From) {
            // from module import name
            let module = if let Token::Identifier(name) = &self.peek() {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err("Expected module name after 'from'".to_string());
            };
            self.consume(&Token::Import, "Expected 'import' after module name")?;
            let name = if let Token::Identifier(name) = &self.peek() {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err("Expected name after 'import' in from-import".to_string());
            };
            let alias = if self.match_token(&Token::As) {
                if let Token::Identifier(name) = &self.peek() {
                    let name = name.clone();
                    self.advance();
                    Some(name)
                } else {
                    return Err("Expected alias after 'as' in from-import".to_string());
                }
            } else {
                None
            };
            if !self.match_token(&Token::Semicolon) {
                return Err("Expected ';' after from-import statement".to_string());
            }
            // For now, treat as UseStatement with path 'module.name'
            return Ok(Some(Statement::Use(UseStatement { path: format!("{}.{}", module, name), alias })));
        } else if self.match_token(&Token::Return) {
            return Ok(Some(Statement::Return(self.parse_return_statement()?)));
        } else if self.match_token(&Token::Module) {
            return Ok(Some(Statement::Module(self.parse_module_statement()?)));
        } else if self.match_token(&Token::If) {
            return Ok(Some(Statement::Expression(self.parse_if_expression()?)));
        } else if self.match_token(&Token::Move) {
            return Ok(Some(Statement::Move(self.parse_move_statement()?)));
        } else if self.match_token(&Token::Drop) {
            return Ok(Some(Statement::Drop(self.parse_drop_statement()?)));
        } else if self.match_token(&Token::Throw) {
            let expr = self.parse_expression()?;
            if !self.match_token(&Token::Semicolon) {
                return Err("Expected ';' after throw expression".to_string());
            }
            return Ok(Some(Statement::Expression(Expression::Throw(ThrowExpression { value: Box::new(expr) }))));
        } else if self.check(&Token::Else) {
            return Err("'else' without 'if'".to_string());
        } else if self.check(&Token::Semicolon) {
            // Skip standalone semicolons
            self.advance();
            return Ok(None);
        }
        
        // Try to parse as expression statement
        let expr = self.parse_expression();
        match &expr {
            Ok(Expression::Block(_)) => {
                // Block statements do not require a semicolon
                Ok(Some(Statement::Expression(expr.unwrap())))
            },
            Ok(expr) => {
                // Non-block expressions require a semicolon
                if !self.match_token(&Token::Semicolon) {
                    return Err("Expected ';' after expression".to_string());
                }
                Ok(Some(Statement::Expression(expr.clone())))
            },
            Err(_e) => {
                // If we can't parse as expression, it's an error
                Err(format!("Unexpected token: {:?}", self.peek()))
            }
        }
    }
    
    fn parse_module_statement(&mut self) -> Result<ModuleStatement, String> {
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected module name".to_string());
        };
        
        self.consume(&Token::LeftBrace, "Expected '{' after module name")?;
        
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Some(statement) = self.parse_statement()? {
                statements.push(statement);
            } else {
                if !self.is_at_end() {
                    self.advance();
                }
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after module body")?;
        
        Ok(ModuleStatement {
            name,
            statements,
        })
    }
    
    // Parse Let Statements
    fn parse_let_statement(&mut self) -> Result<LetStatement, String> {
        let _start_line = self.previous().line;
        let _start_column = self.previous().column;
        
        let mutable = self.match_token(&Token::Mut);
        
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected variable name".to_string());
        };
        
        let _type_annotation = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(&Token::Equal, "Expected '=' after variable declaration")?;
        
        let value = self.parse_expression()?;
        
        if !self.match_token(&Token::Semicolon) {
            return Err("Expected ';' after let statement".to_string());
        }
        
        // Parse ownership annotation
        let _ownership = if self.match_token(&Token::Owned) {
            BorrowType::Move
        } else if self.match_token(&Token::Shared) {
            BorrowType::ImmutableBorrow
        } else if self.match_token(&Token::Ref) {
            let _lifetime = self.parse_lifetime()?;
            BorrowType::ImmutableBorrow
        } else if self.match_token(&Token::MutRef) {
            let _lifetime = self.parse_lifetime()?;
            BorrowType::MutableBorrow
        } else {
            BorrowType::Move // Default to move
        };
        
        Ok(LetStatement {
            name,
            type_annotation: _type_annotation,
            value: Box::new(value),
            is_mutable: mutable,
        })
    }
    
    fn parse_move_statement(&mut self) -> Result<MoveStatement, String> {
        let from = if let Token::Identifier(name) = self.peek() {
            name.clone()
        } else {
            return Err("Expected identifier after 'move'".to_string());
        };
        self.advance();
        
        self.consume(&Token::Equal, "Expected '=' after source variable")?;
        
        let to = if let Token::Identifier(name) = self.peek() {
            name.clone()
        } else {
            return Err("Expected identifier after '='".to_string());
        };
        self.advance();
        
        self.consume(&Token::Semicolon, "Expected ';' after move statement")?;
        
        Ok(MoveStatement {
            from: from,
            to: to,
            ownership_transfer: true,
        })
    }

    fn parse_drop_statement(&mut self) -> Result<DropStatement, String> {
        let variable = if let Token::Identifier(name) = self.peek() {
            name.clone()
        } else {
            return Err("Expected identifier after 'drop'".to_string());
        };
        self.advance();
        
        self.consume(&Token::Semicolon, "Expected ';' after drop statement")?;
        
        Ok(DropStatement {
            variable: variable,
            explicit: true,
        })
    }

    fn parse_lifetime(&mut self) -> Result<Lifetime, String> {
        self.consume(&Token::LeftParen, "Expected '(' for lifetime")?;
        let name = if let Token::Identifier(name) = self.peek() {
            name.clone()
        } else {
            return Err("Expected lifetime name".to_string());
        };
        self.advance();
        self.consume(&Token::RightParen, "Expected ')' after lifetime name")?;
        
        Ok(Lifetime {
            name: name,
            is_inferred: false,
        })
    }

    // Parse Function Statements
    pub fn parse_function_statement(&mut self) -> Result<FunctionStatement, String> {
        let _start_line = self.previous().line;
        let _start_column = self.previous().column;

        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected function name".to_string());
        };

        let _type_params = if self.check(&Token::LeftAngle) {
            self.parse_generic_type_params()?
        } else {
            Vec::new() // TODO: Parse generic type parameters
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
        
        self.consume(&Token::LeftBrace, "Expected '{' before function body")?;
        
        let body = self.parse_block_expression()?;
        
        Ok(FunctionStatement {
            name,
            parameters: parameters.clone(),
            return_type: return_type.clone(),
            body: Box::new(body),
            annotations: Vec::new(), // TODO: Associate annotations
            signature: FunctionSignature {
                parameters: parameters,
                return_type: return_type,
            },
        })
    }
    
    fn parse_generic_type_params(&mut self) -> Result<Vec<String>, String> {
        self.consume(&Token::LeftAngle, "Expected '<' for generic type parameters")?;
        
        let mut type_params = Vec::new();
        if !self.check(&Token::RightAngle) {
            loop {
                let param_name = if let Token::Identifier(name) = &self.peek() {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err("Expected type parameter name".to_string());
                };
                
                type_params.push(param_name);
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightAngle, "Expected '>' after generic type parameters")?;
        
        Ok(type_params)
    }
    
    fn parse_generic_type_arguments(&mut self) -> Result<Vec<Type>, String> {
        self.consume(&Token::LeftAngle, "Expected '<' for generic type arguments")?;
        
        let mut type_args = Vec::new();
        if !self.check(&Token::RightAngle) {
            loop {
                type_args.push(self.parse_type()?);
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightAngle, "Expected '>' after generic type arguments")?;
        
        Ok(type_args)
    }
    
    fn parse_parameter(&mut self) -> Result<Parameter, String> {
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected parameter name".to_string());
        };
        self.consume(&Token::Colon, "Expected ':' after parameter name")?;
        let type_annotation = self.parse_type()?;
        // Parse borrow type
        let borrow_type = if self.match_token(&Token::Ref) {
            BorrowType::Borrowed
        } else if self.match_token(&Token::MutRef) {
            BorrowType::MutableBorrowed
        } else if self.match_token(&Token::Shared) {
            BorrowType::ImmutableBorrow
        } else {
            BorrowType::Move
        };
        // Parse lifetime
        let lifetime = if self.match_token(&Token::Lifetime) {
            Some(self.parse_lifetime()?)
        } else {
            None
        };
        // Parse default value
        let default_value = if self.match_token(&Token::Equal) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(Parameter {
            name,
            type_annotation,
            borrow_type: Some(borrow_type),
            lifetime,
            ownership: None,
            default_value,
        })
    }
    
    // Parse Struct Statements
    fn parse_struct_statement(&mut self) -> Result<StructStatement, String> {
        let _start_line = self.previous().line;
        let _start_column = self.previous().column;
        
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected struct name".to_string());
        };
        
        self.consume(&Token::LeftBrace, "Expected '{' after struct name")?;
        
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            fields.push(self.parse_struct_field()?);
            
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after struct fields")?;
        
        Ok(StructStatement {
            name,
            fields,
            annotations: Vec::new(),
        })
    }
    
    fn parse_struct_field(&mut self) -> Result<StructField, String> {
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected field name".to_string());
        };
        
        self.consume(&Token::Colon, "Expected ':' after field name")?;
        
        let type_annotation = self.parse_type()?;
        
        Ok(StructField {
            name,
            field_type: type_annotation,
            visibility: Visibility::Public,
        })
    }
    
    // Parse Enum Statements
    fn parse_enum_statement(&mut self) -> Result<EnumStatement, String> {
        let _start_line = self.previous().line;
        let _start_column = self.previous().column;
        
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected enum name".to_string());
        };
        
        self.consume(&Token::LeftBrace, "Expected '{' after enum name")?;
        
        let mut variants = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            variants.push(self.parse_enum_variant()?);
            
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after enum variants")?;
        
        Ok(EnumStatement {
            name,
            variants,
            annotations: Vec::new(),
        })
    }
    
    fn parse_enum_variant(&mut self) -> Result<EnumVariant, String> {
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected variant name".to_string());
        };
        
        let data = if self.match_token(&Token::LeftParen) {
            let mut types = Vec::new();
            
            if !self.check(&Token::RightParen) {
                loop {
                    types.push(self.parse_type()?);
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(&Token::RightParen, "Expected ')' after variant data")?;
            Some(types)
        } else {
            None
        };
        
        Ok(EnumVariant { 
            name, 
            fields: data.unwrap_or_else(Vec::new).into_iter().map(|t| StructField {
                name: "field".to_string(),
                field_type: t,
                visibility: Visibility::Public,
            }).collect(),
            visibility: Visibility::Public,
        })
    }
    
    // Parse Use Statements
    fn parse_use_statement(&mut self) -> Result<UseStatement, String> {
        let _start_line = self.previous().line;
        let _start_column = self.previous().column;
        
        let mut items = Vec::new();
        
        if self.match_token(&Token::LeftBrace) {
            // Named imports: use { item1, item2 } from "url"
            if !self.check(&Token::RightBrace) {
                loop {
                    if let Token::Identifier(name) = &self.peek() {
                        items.push(name.clone());
                        self.advance();
                    } else {
                        return Err("Expected identifier in use statement".to_string());
                    }
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(&Token::RightBrace, "Expected '}' in use statement")?;
        } else {
            // Single import: use module from "url"
            if let Token::Identifier(name) = &self.peek() {
                items.push(name.clone());
                self.advance();
            } else {
                return Err("Expected identifier in use statement".to_string());
            }
        }
        
        self.consume(&Token::From, "Expected 'from' in use statement")?;
        
        let from = if let Token::String(url) = &self.peek() {
            let url = url.clone();
            self.advance();
            url
        } else {
            return Err("Expected string URL in use statement".to_string());
        };
        
        let alias = if self.match_token(&Token::At) {
            if let Token::Identifier(name) = &self.peek() {
                let name = name.clone();
                self.advance();
                Some(name)
            } else {
                return Err("Expected alias name after 'as'".to_string());
            }
        } else {
            None
        };
        
        if !self.match_token(&Token::Semicolon) {
            return Err("Expected ';' after use statement".to_string());
        }
        
        Ok(UseStatement {
            path: from,
            alias,
        })
    }
    
    fn parse_return_statement(&mut self) -> Result<ReturnStatement, String> {
        let _start_line = self.previous().line;
        let _start_column = self.previous().column;
        
        let value = if !self.check(&Token::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume(&Token::Semicolon, "Expected ';' after return statement")?;
        
        Ok(ReturnStatement {
            value: value.map(|v| Box::new(v)),
        })
    }
    
    pub fn parse_struct_literal(&mut self, struct_name: String) -> Result<Expression, String> {

        self.consume(&Token::LeftBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let field_name = if let Token::Identifier(name) = &self.peek() {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err("Expected field name".to_string());
            };

            self.consume(&Token::Colon, "Expected ':' after field name")?;
            let field_value = self.parse_expression()?;
            
            fields.push((field_name, field_value));

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume(&Token::RightBrace, "Expected '}' after struct fields")?;

        Ok(Expression::StructLiteral(StructLiteralExpression {
            struct_name,
            fields,
        }))
    }
    
    // Parse Types
    fn parse_type(&mut self) -> Result<Type, String> {
        self.parse_primary_type()
    }
    
    fn parse_primary_type(&mut self) -> Result<Type, String> {
        let name = match &self.peek() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            Token::Box => {
                self.advance();
                "box".to_string()
            }
            Token::Rc => {
                self.advance();
                "rc".to_string()
            }
            Token::Arc => {
                self.advance();
                "arc".to_string()
            }
            Token::Cell => {
                self.advance();
                "cell".to_string()
            }
            Token::RefCell => {
                self.advance();
                "refcell".to_string()
            }
            _ => {
                return Err(format!("Unexpected token in type: {:?}", self.peek()));
            }
        };
        
        match name.as_str() {
            "i32" => Ok(Type::Int),
            "i64" => Ok(Type::Int),
            "Int" => Ok(Type::Int),
            "f32" => Ok(Type::Float),
            "f64" => Ok(Type::Float),
            "Float" => Ok(Type::Float),
            "bool" => Ok(Type::Bool),
            "Bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "String" => Ok(Type::String),
            "void" => Ok(Type::Void),
            "Void" => Ok(Type::Void),
            _ => {
                // Check for generic types
                if self.check(&Token::LeftAngle) {
                    self.advance(); // consume LeftAngle
                    let mut type_args = Vec::new();
                    
                    if !self.check(&Token::RightAngle) {
                        loop {
                            type_args.push(self.parse_type()?);
                            
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(&Token::RightAngle, "Expected '>' in generic type")?;
                    
                    Ok(Type::GenericType(name, type_args))
                } else {
                    // Simple type name - could be struct or enum
                    Ok(Type::Struct(name))
                }
            }
        }
    }
    
    // Parse Expressions
    fn parse_expression(&mut self) -> Result<Expression, String> {
        // Check for control flow expressions first
        if self.match_token(&Token::If) {
            return self.parse_if_expression();
        } else if self.match_token(&Token::While) {
            return self.parse_while_expression();
        } else if self.match_token(&Token::Loop) {
            return self.parse_loop_expression();
        } else if self.match_token(&Token::For) {
            return self.parse_for_expression();
        } else if self.match_token(&Token::Match) {
            return self.parse_match_expression();
        } else if self.match_token(&Token::Throw) {
            let expr = self.parse_expression()?;
            return Ok(Expression::Throw(ThrowExpression { value: Box::new(expr) }));
        }
        
        // Check for ownership/borrowing expressions
        if self.match_token(&Token::Move) {
            let expr = self.parse_expression()?;
            return Ok(expr);
        } else if self.match_token(&Token::Drop) {
            let expr = self.parse_expression()?;
            return Ok(expr);
        } else if self.match_token(&Token::Clone) {
            let expr = self.parse_expression()?;
            return Ok(expr);
        } else if self.match_token(&Token::Ref) {
            let expr = self.parse_expression()?;
            return Ok(expr);
        } else if self.match_token(&Token::MutRef) {
            let expr = self.parse_expression()?;
            return Ok(expr);
        } else if self.match_token(&Token::Box) {
            return self.parse_box_expression();
        } else if self.match_token(&Token::Rc) {
            return self.parse_rc_expression();
        } else if self.match_token(&Token::Arc) {
            self.advance();
            let expr = self.parse_expression()?;
            return Ok(Expression::Arc(ArcExpression {
                value: Box::new(expr),
            }));
        } else if self.match_token(&Token::Cell) {
            return self.parse_cell_expression();
        } else if self.match_token(&Token::RefCell) {
            return self.parse_refcell_expression();
        }
        
        self.parse_assignment()
    }
    
    fn parse_assignment(&mut self) -> Result<Expression, String> {
        let expr = self.parse_or()?;
        
        if self.match_token(&Token::Equal) {
            let value = self.parse_assignment()?;
            return Ok(Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::Equal,
                right: Box::new(value),
            }));
        }
        
        Ok(expr)
    }
    
    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&Token::Or) {
            if self.check(&Token::LeftBrace) {
                // Don't consume LeftBrace as part of a binary expression
                break;
            }
            let operator = BinaryOperator::Or;
            // Check for LeftBrace before parsing the right-hand side
            if self.check(&Token::LeftBrace) {
                break;
            }
            let right = self.parse_and()?;
            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&Token::AmpersandAmpersand) {
            if self.check(&Token::LeftBrace) {
                // Don't consume LeftBrace as part of a binary expression
                break;
            }
            let operator = BinaryOperator::And;
            // Check for LeftBrace before parsing the right-hand side
            if self.check(&Token::LeftBrace) {
                break;
            }
            let right = self.parse_equality()?;
            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;
        
        while self.match_token(&Token::BangEqual) || self.match_token(&Token::EqualEqual) {
            if self.check(&Token::LeftBrace) {
                // Don't consume LeftBrace as part of a binary expression
                break;
            }
            let operator = if self.previous().token == Token::BangEqual {
                BinaryOperator::NotEqual
            } else {
                BinaryOperator::Equal
            };
            // Check for LeftBrace before parsing the right-hand side
            if self.check(&Token::LeftBrace) {
                break;
            }
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_term()?;
        
        while self.match_token(&Token::Greater) || self.match_token(&Token::GreaterEqual) ||
              self.match_token(&Token::Less) || self.match_token(&Token::LessEqual) {
            if self.check(&Token::LeftBrace) {
                // Don't consume LeftBrace as part of a binary expression
                break;
            }
            let operator = match self.previous().token {
                Token::Greater => BinaryOperator::GreaterThan,
                Token::GreaterEqual => BinaryOperator::GreaterThanOrEqual,
                Token::Less => BinaryOperator::LessThan,
                Token::LessEqual => BinaryOperator::LessThanOrEqual,
                _ => unreachable!(),
            };
            // Check for LeftBrace before parsing the right-hand side
            if self.check(&Token::LeftBrace) {
                break;
            }
            let right = self.parse_term()?;
            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_factor()?;
        while self.match_token(&Token::Minus) || self.match_token(&Token::Plus) {
            if self.check(&Token::LeftBrace) {
                // Don't consume LeftBrace as part of a binary expression
                break;
            }
            let operator = if self.previous().token == Token::Minus {
                BinaryOperator::Subtract
            } else {
                BinaryOperator::Add
            };
            // Check for LeftBrace before parsing the right-hand side
            if self.check(&Token::LeftBrace) {
                break;
            }
            let right = self.parse_factor()?;
            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        // Check for LeftBrace before returning
        if self.check(&Token::LeftBrace) {
            // Don't consume the LeftBrace, just return the expression as is
        }
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;
        
        while self.match_token(&Token::Slash) || self.match_token(&Token::Star) ||
              self.match_token(&Token::Percent) {
            if self.check(&Token::LeftBrace) {
                // Don't consume LeftBrace as part of a binary expression
                break;
            }
            let operator = match self.previous().token {
                Token::Slash => BinaryOperator::Divide,
                Token::Star => BinaryOperator::Multiply,
                Token::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            // Check for LeftBrace before parsing the right-hand side
            if self.check(&Token::LeftBrace) {
                break;
            }
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        
        // Check for LeftBrace before returning
        if self.check(&Token::LeftBrace) {
            // Don't consume the LeftBrace, just return the expression as is
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.match_token(&Token::Bang) {
            let right = self.parse_unary()?;
            return Ok(Expression::UnaryOp(UnaryOp {
                operator: UnaryOperator::Not,
                operand: Box::new(right),
            }));
        }
        
        if self.match_token(&Token::Minus) {
            let right = self.parse_unary()?;
            return Ok(Expression::UnaryOp(UnaryOp {
                operator: UnaryOperator::Negate,
                operand: Box::new(right),
            }));
        }
        
        if self.match_token(&Token::Ampersand) {
            let right = self.parse_unary()?;
            return Ok(Expression::Borrow(BorrowExpression {
                expression: Box::new(right.clone()),
                borrow_type: BorrowType::ImmutableBorrow,
                lifetime: None,
            }));
        }
        
        if self.match_token(&Token::Ampersand) {
            let right = self.parse_unary()?;
            return Ok(Expression::BorrowMut(BorrowMutExpression {
                expression: Box::new(right.clone()),
                lifetime: None,
            }));
        }
        
        let result = self.parse_call()?;
        
        // Check for LeftBrace before returning
        if self.check(&Token::LeftBrace) {
            // Don't consume the LeftBrace, just return the expression as is
        }
        
        Ok(result)
    }
    
    fn parse_call(&mut self) -> Result<Expression, String> {
        // Early check for control-flow keywords (but allow Match as it can be an expression)
        match self.peek() {
            Token::If | Token::Else | Token::While | Token::For | Token::Loop => {
                return Err(format!("Unexpected control-flow keyword '{:?}' in expression", self.peek()));
            }
            _ => {}
        }
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.check(&Token::LeftAngle) {
                // Parse generic type arguments
                let type_args = self.parse_generic_type_arguments()?;
                
                // Check for function call parentheses after generic type arguments
                if self.match_token(&Token::LeftParen) {
                    let mut arguments = Vec::new();
                    
                    if !self.check(&Token::RightParen) {
                        loop {
                            arguments.push(CallArgument { name: None, value: self.parse_expression()? });
                            
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(&Token::RightParen, "Expected ')' after arguments")?;
                    
                    // Create a generic function call
                    return Ok(Expression::FunctionCall(
                        Box::new(expr),
                        arguments
                    ));
                } else {
                    // Just generic type arguments without function call
                    return Ok(expr);
                }
            } else if self.match_token(&Token::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.check(&Token::LeftBrace) {
                // Only consume LeftBrace if this is a struct literal (i.e., previous expr is identifier)
                if let Expression::Identifier(struct_name) = &expr {
                    // Check if the next token after { is an identifier (field name)
                    // If not, this might be a match expression or other control flow
                    let next_token = self.peek_next();
                    if let Token::Identifier(_) = next_token {
                        // Check if the token after that is a colon (field: value)
                        let next_next_token = if self.current + 2 < self.tokens.len() {
                            &self.tokens[self.current + 2].token
                        } else {
                            &Token::Eof
                        };
                        if let Token::Colon = next_next_token {
                            self.advance(); // consume LeftBrace
                            expr = self.parse_struct_literal(struct_name.clone())?;
                        } else {
                            // Not a struct literal, do not consume LeftBrace, break loop
                            break;
                        }
                    } else {
                        // Not a struct literal, do not consume LeftBrace, break loop
                        break;
                    }
                } else {
                    // Not a struct literal, do not consume LeftBrace, break loop
                    break;
                }
            } else if self.match_token(&Token::Dot) {
                // Member access: expr.member
                if let Token::Identifier(member) = &self.peek() {
                    let member = member.clone();
                    self.advance();
                    expr = Expression::MemberAccess(MemberAccess {
                        object: Box::new(expr),
                        member,
                    });
                } else {
                    return Err("Expected identifier after '.'".to_string());
                }
            } else if self.match_token(&Token::ColonColon) {
                // Handle module paths like std::io::print or enum variants like Color::Red
                let next_name = if let Token::Identifier(name) = &self.peek() {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err("Expected identifier after '::'".to_string());
                };
                
                // Build the full path
                let current_path = if let Expression::Identifier(name) = expr { name } else { "unknown".to_string() };
                let full_path = format!("{}::{}", current_path, next_name);
                
                // Check if there's another :: after this
                if self.check(&Token::ColonColon) {
                    // This is a module path, continue building it
                    expr = Expression::Identifier(full_path);
                } else {
                    // Check if this is an enum variant with data
                    if self.match_token(&Token::LeftParen) {
                        let mut arguments = Vec::new();
                        
                        if !self.check(&Token::RightParen) {
                            loop {
                                arguments.push(CallArgument { name: None, value: self.parse_expression()? });
                                
                                if !self.match_token(&Token::Comma) {
                                    break;
                                }
                            }
                        }
                        
                        self.consume(&Token::RightParen, "Expected ')' after enum variant arguments")?;
                        
                        // For now, we'll treat this as a function call
                        // TODO: Implement proper enum variant handling
                        expr = Expression::FunctionCall(
                            Box::new(Expression::Identifier(full_path)),
                            arguments
                        );
                    } else {
                        // Simple enum variant without data
                        expr = Expression::EnumVariantAccess {
                            enum_name: current_path,
                            variant_name: next_name,
                        };
                    }
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: Expression) -> Result<Expression, String> {
        let mut arguments = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                // Support keyword arguments: name = expr
                if let Token::Identifier(name) = &self.peek() {
                    let name = name.clone();
                    if self.peek_next() == &Token::Equal {
                        self.advance(); // consume identifier
                        self.advance(); // consume '='
                        let value = self.parse_expression()?;
                        arguments.push(CallArgument { name: Some(name), value });
                    } else {
                        let value = self.parse_expression()?;
                        arguments.push(CallArgument { name: None, value });
                    }
                } else {
                    let value = self.parse_expression()?;
                    arguments.push(CallArgument { name: None, value });
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        self.consume(&Token::RightParen, "Expected ')' after arguments")?;
        Ok(Expression::FunctionCall(Box::new(callee), arguments))
    }
    
    pub fn parse_primary(&mut self) -> Result<Expression, String> {
        if self.match_token(&Token::LeftParen) {
            let expr = self.parse_expression()?;
            self.consume(&Token::RightParen, "Expected ')' after expression")?;
            Ok(expr)
        } else if self.match_token(&Token::LeftBrace) {
            // Check if this is a block expression or dict/set literal
            if self.check(&Token::RightBrace) {
                self.advance();
                // Empty block
                return Ok(Expression::Block(vec![]));
            }
            
            // Peek ahead to see if this looks like a dict/set literal
            let mut _temp_parser = Parser {
                tokens: self.tokens.clone(),
                current: self.current,
            };
            
            // Try to parse as expression first
            let first_token = _temp_parser.peek();
            let is_literal = match first_token {
                Token::String(_) | Token::Number(_) | Token::Identifier(_) => {
                    // Could be dict/set literal
                    true
                },
                _ => false,
            };
            
            if is_literal {
                // Try to parse as dict/set literal
                let mut entries = Vec::new();
                let mut elements = Vec::new();
                let first_expr = self.parse_expression()?;
                if self.match_token(&Token::Colon) {
                    // Dict/map literal
                    let value_expr = self.parse_expression()?;
                    entries.push((first_expr, value_expr));
                    while self.match_token(&Token::Comma) {
                        if self.check(&Token::RightBrace) { break; }
                        let key = self.parse_expression()?;
                        self.consume(&Token::Colon, "Expected ':' in dict literal")?;
                        let value = self.parse_expression()?;
                        entries.push((key, value));
                    }
                    self.consume(&Token::RightBrace, "Expected '}' after dict literal")?;
                    return Ok(Expression::DictLiteral(DictLiteralExpression { entries }));
                } else {
                    // Set literal
                    elements.push(first_expr);
                    while self.match_token(&Token::Comma) {
                        if self.check(&Token::RightBrace) { break; }
                        elements.push(self.parse_expression()?);
                    }
                    self.consume(&Token::RightBrace, "Expected '}' after set literal")?;
                    return Ok(Expression::SetLiteral(SetLiteralExpression { elements }));
                }
            } else {
                // This is a block expression
                return self.parse_block();
            }
        } else if self.match_token(&Token::If) {
            self.parse_if_expression()
        } else if self.match_token(&Token::While) {
            self.parse_while_expression()
        } else if self.match_token(&Token::Loop) {
            self.parse_loop_expression()
        } else if self.match_token(&Token::Match) {
            self.parse_match_expression()
        } else if self.match_token(&Token::Try) {
            self.parse_try_expression()
        } else if self.match_token(&Token::Spawn) {
            self.parse_spawn_expression()
        } else if self.match_token(&Token::Join) {
            self.parse_join_expression()
        } else if self.match_token(&Token::Channel) {
            self.parse_channel_expression()
        } else if self.match_token(&Token::Pipeline) {
            self.parse_pipeline_expression()
        } else if self.match_token(&Token::Box) {
            self.parse_box_expression()
        } else if self.match_token(&Token::Rc) {
            self.parse_rc_expression()
        } else if self.match_token(&Token::Arc) {
            self.parse_arc_expression()
        } else if self.match_token(&Token::Cell) {
            self.parse_cell_expression()
        } else if self.match_token(&Token::RefCell) {
            self.parse_refcell_expression()
        } else if self.match_token(&Token::Malloc) {
            self.parse_malloc_expression()
        } else if self.match_token(&Token::Free) {
            self.parse_free_expression()
        } else if self.match_token(&Token::Realloc) {
            self.parse_realloc_expression()
        } else if self.match_token(&Token::Clone) {
            self.parse_clone_expression()
        } else if self.match_token(&Token::Move) {
            self.parse_move_expression()
        } else if self.match_token(&Token::Drop) {
            self.parse_drop_expression()
        } else if self.match_token(&Token::Borrow) {
            self.parse_borrow_expression()
        } else if self.match_token(&Token::BorrowMut) {
            self.parse_borrow_mut_expression()
        } else if self.match_token(&Token::Lifetime) {
            self.parse_lifetime_expression()
        } else if self.match_token(&Token::LeftBracket) {
            // Could be array literal, list comprehension, or slicing
            let expr = self.parse_expression()?;
            if self.match_token(&Token::For) {
                // List comprehension: [expr for x in xs if cond]
                let iterator = if let Token::Identifier(name) = self.peek() {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err("Expected identifier in list comprehension".to_string());
                };
                self.consume(&Token::In, "Expected 'in' in list comprehension")?;
                let iterable = self.parse_expression()?;
                let condition = if self.match_token(&Token::If) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                self.consume(&Token::RightBracket, "Expected ']' after list comprehension")?;
                return Ok(Expression::ListComprehension(ListComprehensionExpression {
                    element: Box::new(expr),
                    iterator,
                    iterable: Box::new(iterable),
                    condition,
                }));
            } else if self.match_token(&Token::Colon) {
                // Slicing: xs[start:end:step]
                let start = Some(Box::new(expr));
                let end = if !self.check(&Token::RightBracket) && !self.check(&Token::Colon) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                let step = if self.match_token(&Token::Colon) {
                    if !self.check(&Token::RightBracket) {
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.consume(&Token::RightBracket, "Expected ']' after slice")?;
                return Ok(Expression::Slice(SliceExpression {
                    collection: Box::new(Expression::Identifier("<slice_target>".to_string())), // Placeholder, to be replaced in context
                    start,
                    end,
                    step,
                }));
            } else if self.check(&Token::RightBracket) {
                self.advance();
                return Ok(Expression::Literal(Literal::Array(vec![])));
            } else {
                // Fallback to array literal
                let mut elements = vec![expr];
                while self.match_token(&Token::Comma) {
                    if self.check(&Token::RightBracket) { break; }
                    elements.push(self.parse_expression()?);
                }
                self.consume(&Token::RightBrace, "Expected '}' after array literal")?;
                let mut literals = Vec::new();
                for element in elements {
                    if let Expression::Literal(literal) = element {
                        literals.push(literal);
                    } else {
                        return Err("Array elements must be literals".to_string());
                    }
                }
                return Ok(Expression::Literal(Literal::Array(literals)));
            }
        } else if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            
            // Check if this is a struct literal
            if self.check(&Token::LeftBrace) {
                self.parse_struct_literal(name)
            } else {
                Ok(Expression::Identifier(name))
            }
        } else if let Token::Number(n) = &self.peek() {
            let n = n.clone();
            self.advance();
            Ok(Expression::Literal(Literal::Int(n)))
        } else if let Token::Float(f) = &self.peek() {
            let f = f.clone();
            self.advance();
            Ok(Expression::Literal(Literal::Float(f)))
        } else if let Token::String(s) = &self.peek() {
            let s = s.clone();
            self.advance();
            Ok(Expression::Literal(Literal::String(s)))
        } else if let Token::Char(c) = &self.peek() {
            let c = c.clone();
            self.advance();
            Ok(Expression::Literal(Literal::Char(c)))
        } else if self.match_token(&Token::True) {
            Ok(Expression::Literal(Literal::Bool(true)))
        } else if self.match_token(&Token::False) {
            Ok(Expression::Literal(Literal::Bool(false)))
        } else if self.match_token(&Token::Null) {
            Ok(Expression::Literal(Literal::Null))
        } else if self.match_token(&Token::Fn) {
            // Lambda/anonymous function: fn (params) => expr or fn (params) { ... }
            self.consume(&Token::LeftParen, "Expected '(' after 'fn' in lambda expression")?;
            let mut parameters = Vec::new();
            if !self.check(&Token::RightParen) {
                loop {
                    let param_name = if let Token::Identifier(name) = &self.peek() {
                        let name = name.clone();
                        self.advance();
                        name
                    } else {
                        return Err("Expected parameter name in lambda".to_string());
                    };
                    // Optionally parse type annotation
                    let type_annotation = if self.match_token(&Token::Colon) {
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    parameters.push(Parameter {
                        name: param_name,
                        type_annotation: type_annotation.unwrap_or(Type::Unknown),
                        borrow_type: None,
                        lifetime: None,
                        ownership: None,
                        default_value: None,
                    });
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(&Token::RightParen, "Expected ')' after lambda parameters")?;
            // Support both '=>' and '{ ... }' lambda bodies
            let body = if self.match_token(&Token::Arrow) {
                Box::new(self.parse_expression()?)
            } else if self.check(&Token::LeftBrace) {
                Box::new(self.parse_block_expression()?)
            } else {
                return Err("Expected '=>' or '{' after lambda parameters".to_string());
            };
            return Ok(Expression::Lambda(LambdaExpression { parameters, body }));
        } else if let Token::InterpolatedString(parts) = self.peek() {
            // Clone the parts before advancing to avoid borrow checker issues
            let parts = if let Token::InterpolatedString(parts) = self.peek() {
                parts.clone()
            } else { unreachable!() };
            self.advance();
            let mut expr_parts = Vec::new();
            for part in parts {
                match part {
                    LexerInterpolatedPart::String(s) => expr_parts.push(InterpolatedPart::String(s)),
                    LexerInterpolatedPart::Expr(expr_str) => {
                        let mut sub_lexer = crate::lexer::Lexer::new(&expr_str, "<interpolated>".to_string());
                        let sub_tokens = sub_lexer.tokenize()?;
                        let mut sub_parser = Parser::new(sub_tokens);
                        let expr = sub_parser.parse_expression()?;
                        expr_parts.push(InterpolatedPart::Expr(expr));
                    }
                }
            }
            return Ok(Expression::InterpolatedString(InterpolatedStringExpression { parts: expr_parts }));
        } else {
            Err(format!("Unexpected token: {:?}", self.peek()))
        }
    }

    fn parse_try_catch_expression(&mut self) -> Result<Expression, String> {
        let try_block = if self.match_token(&Token::LeftBrace) {
            // Block expression
            let mut statements = Vec::new();
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                if let Some(statement) = self.parse_statement()? {
                    statements.push(statement);
                } else {
                    if !self.is_at_end() {
                        self.advance();
                    }
                }
            }
            self.consume(&Token::RightBrace, "Expected '}' after try block")?;
            Box::new(Expression::Block(statements))
        } else {
            // Single expression
            Box::new(self.parse_expression()?)
        };
        
        self.consume(&Token::Catch, "Expected 'catch' after try block")?;
        
        let _catch_block = if self.match_token(&Token::LeftBrace) {
            // Block expression
            let mut statements = Vec::new();
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                if let Some(statement) = self.parse_statement()? {
                    statements.push(statement);
                } else {
                    if !self.is_at_end() {
                        self.advance();
                    }
                }
            }
            self.consume(&Token::RightBrace, "Expected '}' after catch block")?;
            Box::new(Expression::Block(statements))
        } else {
            // Single expression
            Box::new(self.parse_expression()?)
        };
        
        Ok(*try_block)
    }
    
    fn parse_block(&mut self) -> Result<Expression, String> {
        let mut statements = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Some(statement) = self.parse_statement()? {
                statements.push(statement);
            } else {
                // If no statement was parsed, advance past the current token
                if !self.is_at_end() {
                    self.advance();
                }
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after block")?;
        
        Ok(Expression::Block(statements))
    }
    
    fn parse_block_expression(&mut self) -> Result<Expression, String> {
        self.parse_block()
    }
    
    fn parse_list_literal(&mut self) -> Result<Expression, String> {
        let mut elements = Vec::new();
        
        if !self.check(&Token::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightBracket, "Expected ']' after list elements")?;
        
        // Convert Expression to Literal for array elements
        let mut literals = Vec::new();
        for element in elements {
            if let Expression::Literal(literal) = element {
                literals.push(literal);
            } else {
                return Err("Array elements must be literals".to_string());
            }
        }
        Ok(Expression::Literal(Literal::Array(literals)))
    }
    
    fn parse_map_literal(&mut self) -> Result<Expression, String> {
        let mut map = std::collections::HashMap::new();
        
        if !self.check(&Token::RightBrace) {
            loop {
                let key = if let Token::String(key) = &self.peek() {
                    let key = key.clone();
                    self.advance();
                    key
                } else {
                    return Err("Expected string key in map literal".to_string());
                };
                
                self.consume(&Token::Colon, "Expected ':' after map key")?;
                
                let value = self.parse_expression()?;
                map.insert(key, value);
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after map elements")?;
        
        // TODO: Implement map literal support
        Err("Map literals not yet implemented".to_string())
    }
    
    fn parse_if_condition(&mut self) -> Result<Expression, String> {
        // Use regular expression parsing but stop at LeftBrace
        let mut expr = self.parse_or()?;
        
        // Continue parsing binary expressions until we hit LeftBrace
        while !self.check(&Token::LeftBrace) && !self.is_at_end() {
            if self.match_token(&Token::Greater) || self.match_token(&Token::GreaterEqual) ||
               self.match_token(&Token::Less) || self.match_token(&Token::LessEqual) ||
               self.match_token(&Token::EqualEqual) || self.match_token(&Token::BangEqual) {
                let operator = match self.previous().token {
                    Token::Greater => BinaryOperator::GreaterThan,
                    Token::GreaterEqual => BinaryOperator::GreaterThanOrEqual,
                    Token::Less => BinaryOperator::LessThan,
                    Token::LessEqual => BinaryOperator::LessThanOrEqual,
                    Token::EqualEqual => BinaryOperator::Equal,
                    Token::BangEqual => BinaryOperator::NotEqual,
                    _ => unreachable!(),
                };
                let right = self.parse_or()?;
                expr = Expression::BinaryOp(BinaryOp {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        Ok(expr)
    }
    
    fn parse_if_expression(&mut self) -> Result<Expression, String> {
        // Make parentheses optional for if conditions
        let condition = if self.match_token(&Token::LeftParen) {
            let cond = Box::new(self.parse_expression()?);
            self.consume(&Token::RightParen, "Expected ')' after if condition")?;
            cond
        } else {
            // Parse the condition as a simple expression that stops at LeftBrace
            let cond = Box::new(self.parse_if_condition()?);
            cond
        };
        
        let then_branch = if self.match_token(&Token::LeftBrace) {
            // Block expression
            let mut statements = Vec::new();
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                if let Some(statement) = self.parse_statement()? {
                    statements.push(statement);
                } else {
                    if !self.is_at_end() {
                        self.advance();
                    }
                }
            }
            self.consume(&Token::RightBrace, "Expected '}' after if block")?;
            Box::new(Expression::Block(statements))
        } else {
            // Single expression
            Box::new(self.parse_expression()?)
        };
        
        let else_branch = if self.match_token(&Token::Else) {
            Some(if self.match_token(&Token::If) {
                // else if chain - parse as a separate if expression
                let condition = if self.match_token(&Token::LeftParen) {
                    let cond = Box::new(self.parse_expression()?);
                    self.consume(&Token::RightParen, "Expected ')' after if condition")?;
                    cond
                } else {
                    Box::new(self.parse_expression()?)
                };
                
                let then_branch = if self.match_token(&Token::LeftBrace) {
                    // Block expression
                    let mut statements = Vec::new();
                    while !self.check(&Token::RightBrace) && !self.is_at_end() {
                        if let Some(statement) = self.parse_statement()? {
                            statements.push(statement);
                        } else {
                            if !self.is_at_end() {
                                self.advance();
                            }
                        }
                    }
                    self.consume(&Token::RightBrace, "Expected '}' after if block")?;
                    Box::new(Expression::Block(statements))
                } else {
                    // Single expression
                    Box::new(self.parse_expression()?)
                };
                
                // Parse the else branch of the else-if
                let else_branch = if self.match_token(&Token::Else) {
                    Some(if self.match_token(&Token::LeftBrace) {
                        // Block expression
                        let mut statements = Vec::new();
                        while !self.check(&Token::RightBrace) && !self.is_at_end() {
                            if let Some(statement) = self.parse_statement()? {
                                statements.push(statement);
                            } else {
                                if !self.is_at_end() {
                                    self.advance();
                                }
                            }
                        }
                        self.consume(&Token::RightBrace, "Expected '}' after else block")?;
                        Box::new(Expression::Block(statements))
                    } else {
                        // Single expression
                        Box::new(self.parse_expression()?)
                    })
                } else {
                    None
                };
                
                Box::new(Expression::If(IfExpression {
                    condition,
                    then_branch,
                    else_branch,
                }))
            } else if self.match_token(&Token::LeftBrace) {
                // Block expression
                let mut statements = Vec::new();
                while !self.check(&Token::RightBrace) && !self.is_at_end() {
                    if let Some(statement) = self.parse_statement()? {
                        statements.push(statement);
                    } else {
                        if !self.is_at_end() {
                            self.advance();
                        }
                    }
                }
                self.consume(&Token::RightBrace, "Expected '}' after else block")?;
                Box::new(Expression::Block(statements))
            } else {
                // Single expression - but else is not a valid expression
                return Err("Expected '{' or 'if' after 'else'".to_string());
            })
        } else {
            None
        };
        
        Ok(Expression::If(IfExpression {
            condition,
            then_branch,
            else_branch,
        }))
    }
    
    fn parse_match_expression(&mut self) -> Result<Expression, String> {
        let value = Box::new(self.parse_or()?);
        
        self.consume(&Token::LeftBrace, "Expected '{' after match value")?;
        
        let mut arms = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);
            
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after match arms")?;
        
        Ok(*value)
    }
    
    fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        let pattern = self.parse_pattern()?;
        
        self.consume(&Token::Arrow, "Expected '=>' in match arm")?;
        
        let expression = Box::new(self.parse_expression()?);
        
        Ok(MatchArm { 
            pattern, 
            expression: expression.clone(),
            body: expression,
            guard: None,
            location: 0,
        })
    }
    
    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        // Check for wildcard pattern
        if let Token::Identifier(name) = &self.peek() {
            if name == "_" {
                self.advance();
                return Ok(Pattern::Wildcard);
            }
        }
        
        // Check for identifier pattern
        if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            return Ok(Pattern::Identifier(name));
        }
        
        // Check for literal patterns
        match &self.peek() {
            Token::Number(value) => {
                let value = *value;
                self.advance();
                return Ok(Pattern::Literal(Literal::Int(value)));
            }
            Token::Float(value) => {
                let value = *value;
                self.advance();
                return Ok(Pattern::Literal(Literal::Float(value)));
            }
            Token::True => {
                self.advance();
                return Ok(Pattern::Literal(Literal::Bool(true)));
            }
            Token::False => {
                self.advance();
                return Ok(Pattern::Literal(Literal::Bool(false)));
            }
            Token::String(value) => {
                let value = value.clone();
                self.advance();
                return Ok(Pattern::Literal(Literal::String(value)));
            }
            Token::Char(value) => {
                let value = *value;
                self.advance();
                return Ok(Pattern::Literal(Literal::Char(value)));
            }
            _ => {}
        }
        
        // TODO: Implement more complex patterns (tuple, struct, enum)
        Err("Pattern parsing not yet implemented for this pattern type".to_string())
    }
    
    fn parse_loop_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::LeftBrace, "Expected '{' after 'loop'")?;
        
        let mut body_statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Some(statement) = self.parse_statement()? {
                body_statements.push(statement);
            }
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after loop body")?;
        
        Ok(Expression::Loop(LoopExpression {
            body: Box::new(Expression::Block(body_statements)),
            label: None,
        }))
    }
    
    fn parse_while_expression(&mut self) -> Result<Expression, String> {
        // Optional parentheses around condition
        let condition = if self.match_token(&Token::LeftParen) {
            let expr = Box::new(self.parse_expression()?);
            self.consume(&Token::RightParen, "Expected ')' after while condition")?;
            expr
        } else {
            Box::new(self.parse_expression()?)
        };
        
        let body = if self.match_token(&Token::LeftBrace) {
            // Block expression
            let mut statements = Vec::new();
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                if let Some(statement) = self.parse_statement()? {
                    statements.push(statement);
                } else {
                    if !self.is_at_end() {
                        self.advance();
                    }
                }
            }
            self.consume(&Token::RightBrace, "Expected '}' after while block")?;
            Box::new(Expression::Block(statements))
        } else {
            // Single expression
            Box::new(self.parse_expression()?)
        };
        
        Ok(Expression::While(WhileExpression {
            condition,
            body,
        }))
    }
    
    fn parse_for_expression(&mut self) -> Result<Expression, String> {
        let iterator = Box::new(self.parse_expression()?);
        
        self.consume(&Token::In, "Expected 'in' in for loop")?;
        
        let body = Box::new(self.parse_expression()?);
        
        Ok(*iterator)
    }

    fn parse_box_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Box, "Expected 'box'")?;
        self.consume(&Token::LeftAngle, "Expected '<' after 'box'")?;
        
        let _type_annotation = Some(self.parse_type()?);
        
        self.consume(&Token::RightAngle, "Expected '>' after box type")?;
        self.consume(&Token::LeftParen, "Expected '(' after box type")?;
        
        let inner_value = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after box value")?;
        
        Ok(Expression::Box(BoxExpression {
            value: Box::new(inner_value),
        }))
    }

    fn parse_rc_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Rc, "Expected 'rc'")?;
        self.consume(&Token::LeftAngle, "Expected '<' after 'rc'")?;
        
        let _type_annotation = Some(self.parse_type()?);
        
        self.consume(&Token::RightAngle, "Expected '>' after rc type")?;
        self.consume(&Token::LeftParen, "Expected '(' after rc type")?;
        
        let inner_value = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after rc value")?;
        
        Ok(Expression::Rc(RcExpression {
            value: Box::new(inner_value),
        }))
    }

    fn parse_arc_expression(&mut self) -> Result<Expression, String> {
        // Arc token has already been consumed by match_token in parse_expression
        
        // Check if we have angle brackets for type annotation
        let _type_annotation = if self.match_token(&Token::LeftAngle) {
            let type_annotation = Some(self.parse_type()?);
            self.consume(&Token::RightAngle, "Expected '>' after arc type")?;
            type_annotation
        } else {
            None
        };
        
        // Check if we have parentheses around the value
        let inner_value = if self.match_token(&Token::LeftParen) {
            let value = self.parse_expression()?;
            self.consume(&Token::RightParen, "Expected ')' after arc value")?;
            value
        } else {
            // No parentheses, just parse the expression directly
            self.parse_expression()?
        };
        
        Ok(Expression::Arc(ArcExpression {
            value: Box::new(inner_value),
        }))
    }

    fn parse_cell_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Cell, "Expected 'cell'")?;
        self.consume(&Token::LeftAngle, "Expected '<' after 'cell'")?;
        
        let _type_annotation = Some(self.parse_type()?);
        
        self.consume(&Token::RightAngle, "Expected '>' after cell type")?;
        self.consume(&Token::LeftParen, "Expected '(' after cell type")?;
        
        let inner_value = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after cell value")?;
        
        Ok(Expression::Cell(CellExpression {
            value: Box::new(inner_value),
        }))
    }

    fn parse_refcell_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::RefCell, "Expected 'refcell'")?;
        self.consume(&Token::LeftAngle, "Expected '<' after 'refcell'")?;
        
        let _type_annotation = Some(self.parse_type()?);
        
        self.consume(&Token::RightAngle, "Expected '>' after refcell type")?;
        self.consume(&Token::LeftParen, "Expected '(' after refcell type")?;
        
        let inner_value = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after refcell value")?;
        
        Ok(Expression::RefCell(RefCellExpression {
            value: Box::new(inner_value),
        }))
    }

    fn parse_malloc_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Malloc, "Expected 'malloc'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'malloc'")?;
        
        let size = self.parse_expression()?;
        let type_annotation = if self.match_token(&Token::Comma) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(&Token::RightParen, "Expected ')' after malloc arguments")?;
        
        Ok(Expression::Malloc(MallocExpression {
            size: Box::new(size),
            type_annotation,
        }))
    }

    fn parse_free_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Free, "Expected 'free'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'free'")?;
        
        let pointer = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after free argument")?;
        
        Ok(Expression::Free(FreeExpression {
            pointer: Box::new(pointer),
        }))
    }

    fn parse_realloc_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Realloc, "Expected 'realloc'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'realloc'")?;
        
        let pointer = self.parse_expression()?;
        self.consume(&Token::Comma, "Expected ',' after pointer")?;
        
        let new_size = self.parse_expression()?;
        let type_annotation = if self.match_token(&Token::Comma) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(&Token::RightParen, "Expected ')' after realloc arguments")?;
        
        Ok(Expression::Realloc(ReallocExpression {
            pointer: Box::new(pointer),
            new_size: Box::new(new_size),
        }))
    }

    pub fn parse_member_access(&mut self, mut expr: Expression) -> Result<Expression, String> {
        while self.match_token(&Token::Dot) {
            let member = if let Token::Identifier(name) = &self.peek() {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err("Expected member name after '.'".to_string());
            };

            expr = Expression::MemberAccess(MemberAccessExpression {
                object: Box::new(expr),
                member,
            });
        }
        Ok(expr)
    }

    pub fn parse_function_call(&mut self, mut expr: Expression) -> Result<Expression, String> {
        while self.match_token(&Token::LeftParen) {
            let mut arguments = Vec::new();
            
            if !self.check(&Token::RightParen) {
                loop {
                    arguments.push(CallArgument { name: None, value: self.parse_expression()? });
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(&Token::RightParen, "Expected ')' after function arguments")?;
            
            expr = Expression::FunctionCall(Box::new(expr), arguments);
        }
        Ok(expr)
    }
    
    // Missing parse methods
    fn parse_try_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Try, "Expected 'try'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'try'")?;
        let expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after try expression")?;
        Ok(Expression::Try(TryExpression {
            expression: Box::new(expr),
        }))
    }
    
    fn parse_spawn_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Spawn, "Expected 'spawn'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'spawn'")?;
        let expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after spawn expression")?;
        Ok(Expression::Spawn(SpawnExpression {
            expression: Box::new(expr),
        }))
    }
    
    fn parse_join_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Join, "Expected 'join'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'join'")?;
        let handle = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after join handle")?;
        Ok(Expression::Join(JoinExpression {
            handle: Box::new(handle),
        }))
    }
    
    fn parse_channel_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Channel, "Expected 'channel'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'channel'")?;
        let capacity = if !self.check(&Token::RightParen) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.consume(&Token::RightParen, "Expected ')' after channel arguments")?;
        Ok(Expression::Channel(ChannelExpression {
            channel_type: crate::ast::ChannelType::Unbounded,
            capacity,
        }))
    }
    
    fn parse_pipeline_expression(&mut self) -> Result<Expression, String> {
        let mut stages = Vec::new();
        stages.push(self.parse_expression()?);
        
        while self.match_token(&Token::Pipeline) {
            stages.push(self.parse_expression()?);
        }
        
        Ok(Expression::Pipeline(PipelineExpression { stages }))
    }
    
    fn parse_clone_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Clone, "Expected 'clone'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'clone'")?;
        let expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after clone expression")?;
        Ok(Expression::Clone(CloneExpression {
            expression: Box::new(expr),
        }))
    }
    
    fn parse_move_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Move, "Expected 'move'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'move'")?;
        let _expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after move expression")?;
        Ok(Expression::Move(MoveStatement {
            from: "".to_string(),
            to: "".to_string(),
            ownership_transfer: true,
        }))
    }
    
    fn parse_drop_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Drop, "Expected 'drop'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'drop'")?;
        let _expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after drop expression")?;
        Ok(Expression::Drop(DropStatement {
            variable: "".to_string(),
            explicit: true,
        }))
    }
    
    fn parse_borrow_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Borrow, "Expected 'borrow'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'borrow'")?;
        let expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after borrow expression")?;
        Ok(Expression::Borrow(BorrowExpression {
            expression: Box::new(expr),
            borrow_type: BorrowType::ImmutableBorrow,
            lifetime: None,
        }))
    }
    
    fn parse_borrow_mut_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::BorrowMut, "Expected 'borrow_mut'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'borrow_mut'")?;
        let expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after borrow_mut expression")?;
        Ok(Expression::BorrowMut(BorrowMutExpression {
            expression: Box::new(expr),
            lifetime: None,
        }))
    }
    
    fn parse_lifetime_expression(&mut self) -> Result<Expression, String> {
        self.consume(&Token::Lifetime, "Expected 'lifetime'")?;
        self.consume(&Token::LeftParen, "Expected '(' after 'lifetime'")?;
        let lifetime = self.parse_lifetime()?;
        self.consume(&Token::Comma, "Expected ',' after lifetime")?;
        let expr = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')' after lifetime expression")?;
        Ok(Expression::Lifetime(LifetimeExpression {
            lifetime,
            expression: Box::new(expr),
        }))
    }
    
    fn parse_array_literal(&mut self) -> Result<Expression, String> {
        self.parse_list_literal()
    }

    fn parse_class_statement(&mut self) -> Result<ClassStatement, String> {
        let name = if let Token::Identifier(name) = &self.peek() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected class name".to_string());
        };
        let superclass = if self.match_token(&Token::Extends) {
            if let Token::Identifier(super_name) = &self.peek() {
                let super_name = super_name.clone();
                self.advance();
                Some(super_name)
            } else {
                return Err("Expected superclass name after 'extends'".to_string());
            }
        } else {
            None
        };
        self.consume(&Token::LeftBrace, "Expected '{' after class name")?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Fn) {
                methods.push(self.parse_function_statement()?);
            } else {
                fields.push(self.parse_struct_field()?);
            }
            if !self.match_token(&Token::Comma) {
                // Allow optional commas between fields/methods
            }
        }
        self.consume(&Token::RightBrace, "Expected '}' after class body")?;
        Ok(ClassStatement {
            name,
            superclass,
            fields,
            methods,
            annotations: Vec::new(),
        })
    }
} 