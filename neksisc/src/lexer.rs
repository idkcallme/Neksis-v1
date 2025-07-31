use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Let,
    Mut,
    Struct,
    Enum,
    Fn,
    Return,
    If,
    Else,
    Match,
    Loop,
    While,
    For,
    In,
    Break,
    Continue,
    Use,
    From,
    Mod,
    Module,
    Pub,
    Borrow,
    Copy,
    Comptime,
    Try,
    Catch,
    And,
    Or,
    Not,
    Throw,
    Class,
    Extends,
    Import,
    As,
    
    // Ownership and borrowing keywords
    Move,
    Drop,
    Clone,
    Ref,
    MutRef,
    Owned,
    Shared,
    Lifetime,
    Where,
    Impl,
    Trait,
    Box,
    Rc,
    Arc,
    Cell,
    RefCell,
    Mutex,
    RwLock,
    Malloc,
    Free,
    Realloc,
    
    // Identifiers and literals
    Identifier(String),
    Number(i64), // Changed from Integer to Number to match parser expectations
    Float(f64),
    True, // Added missing boolean literals
    False,
    Char(char), // Changed from Character to Char
    String(String),
    Null, // Added null literal
    InterpolatedString(Vec<InterpolatedPart>),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Ampersand,
    AmpersandAmpersand,
    Pipe,
    PipePipe,
    Pipeline, // |>
    DotProduct, // .*
    MatrixMultiply, // @
    Dot, // . for member access
    Range, // .. operator
    
    // Concurrency tokens
    Spawn,
    Join,
    Channel,
    
    // Borrowing tokens
    BorrowMut,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftAngle, // < for generics
    RightAngle, // > for generics
    Comma,
    Semicolon,
    Colon,
    ColonColon, // :: for enum variant access
    Arrow, // ->
    
    // Special
    At, // @ for annotations
    Hash, // # for comments
    Dollar, // $ for string interpolation
    
    // End of file
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedPart {
    String(String),
    Expr(String), // Raw expression as string for now; parser will parse it
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub lexeme: String,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    current_file: String,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, file: String) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
            current_file: file,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<TokenInfo>, String> {
        let mut tokens = Vec::new();
        let mut last_position = 0;
        
        while let Some(token_info) = self.next_token()? {
            tokens.push(token_info);
            
            // Check if we're making progress
            let current_position = self.input.clone().count();
            if current_position == last_position {
                // No progress made, forcibly advance
                if let Some(c) = self.input.next() {
                    self.column += 1;
                    // Add an error token to indicate the issue
                    tokens.push(TokenInfo {
                        token: Token::Eof,
                        line: self.line,
                        column: self.column,
                        lexeme: format!("Unexpected character: {}", c),
                    });
                } else {
                    break; // End of input
                }
            }
            last_position = self.input.clone().count();
        }
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Option<TokenInfo>, String> {
        self.skip_whitespace();
        
        if self.input.peek().is_none() {
            return Ok(None);
        }
        
        let start_line = self.line;
        let start_column = self.column;
        
        let token = match self.input.next().unwrap() {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            ':' => {
                if self.input.peek() == Some(&':') {
                    self.input.next();
                    Token::ColonColon
                } else {
                    Token::Colon
                }
            },
            '.' => {
                if self.input.peek() == Some(&'.') {
                    self.input.next();
                    self.column += 1;
                    if self.input.peek() == Some(&'.') {
                        self.input.next();
                        self.column += 1;
                        Token::Range // '...' (triple dot) if you want to support it
                    } else {
                        Token::Range // '..' (double dot)
                    }
                } else {
                    Token::Dot
                }
            }
            '@' => Token::At,

            '+' => Token::Plus,
            '-' => {
                if self.input.peek() == Some(&'>') {
                    self.input.next();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => {
                if self.input.peek() == Some(&'.') {
                    self.input.next();
                    Token::DotProduct
                } else {
                    Token::Star
                }
            }
            '/' => {
                if self.input.peek() == Some(&'/') {
                    // Skip single-line comments
                    self.input.next(); // consume the second '/'
                    while let Some(c) = self.input.next() {
                        if c == '\n' {
                            self.line += 1;
                            self.column = 1;
                            break;
                        }
                    }
                    // Recursively call next_token to get the next real token
                    return self.next_token();
                } else {
                    Token::Slash
                }
            },
            '%' => Token::Percent,
            '=' => {
                if self.input.peek() == Some(&'=') {
                    self.input.next();
                    Token::EqualEqual
                } else if self.input.peek() == Some(&'>') {
                    self.input.next();
                    Token::Arrow
                } else {
                    Token::Equal
                }
            }
            '!' => {
                if self.input.peek() == Some(&'=') {
                    self.input.next();
                    Token::BangEqual
                } else {
                    Token::Bang
                }
            }
            '<' => {
                if self.input.peek() == Some(&'=') {
                    self.input.next();
                    Token::LessEqual
                } else {
                    Token::LeftAngle
                }
            }
            '>' => {
                if self.input.peek() == Some(&'=') {
                    self.input.next();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '&' => {
                if self.input.peek() == Some(&'&') {
                    self.input.next();
                    Token::AmpersandAmpersand
                } else {
                    Token::Ampersand
                }
            }
            '|' => {
                if self.input.peek() == Some(&'>') {
                    self.input.next();
                    Token::Pipeline
                } else if self.input.peek() == Some(&'|') {
                    self.input.next();
                    Token::PipePipe
                } else {
                    Token::Pipe
                }
            }
            '?' => Token::Try,
            '$' => Token::Dollar, // Add support for $ character
            '"' => self.read_string()?,
            '\'' => self.read_character()?,
            c if c.is_alphabetic() || c == '_' => self.read_identifier_or_keyword(c)?,
            c if c.is_digit(10) => self.read_number(c)?,
            c => return Err(format!("Unexpected character: {}", c)),
        };
        
        let lexeme = self.get_lexeme(start_line, start_column);
        
        Ok(Some(TokenInfo {
            token,
            line: start_line,
            column: start_column,
            lexeme,
        }))
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.input.peek() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.input.next();
                    self.column += 1;
                }
                '\n' => {
                    self.input.next();
                    self.line += 1;
                    self.column = 1;
                }

                _ => break,
            }
        }
    }
    
    fn read_string(&mut self) -> Result<Token, String> {
        let mut string = String::new();
        let mut parts = Vec::new();
        let _in_interpolation = false;
        while let Some(c) = self.input.next() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.input.next().ok_or("Unexpected end of string")?;
                    string.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                    });
                }
                '{' => {
                    // Start of interpolation
                    if !string.is_empty() {
                        parts.push(InterpolatedPart::String(string.clone()));
                        string.clear();
                    }
                    let mut expr = String::new();
                    let mut brace_count = 1;
                    while let Some(ec) = self.input.next() {
                        if ec == '{' {
                            brace_count += 1;
                        } else if ec == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                break;
                            }
                        }
                        expr.push(ec);
                    }
                    parts.push(InterpolatedPart::Expr(expr));
                }
                _ => string.push(c),
            }
        }
        if !string.is_empty() {
            parts.push(InterpolatedPart::String(string));
        }
        if parts.len() == 1 {
            if let InterpolatedPart::String(s) = &parts[0] {
                return Ok(Token::String(s.clone()));
            }
        }
        Ok(Token::InterpolatedString(parts))
    }
    
    fn read_character(&mut self) -> Result<Token, String> {
        let c = self.input.next().ok_or("Unexpected end of character literal")?;
        
        if c == '\\' {
            let escaped = self.input.next().ok_or("Unexpected end of character literal")?;
            let char_value = match escaped {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
            };
            
            if self.input.next() != Some('\'') {
                return Err("Character literal not properly closed".to_string());
            }
            
            Ok(Token::Char(char_value))
        } else {
            if self.input.next() != Some('\'') {
                return Err("Character literal not properly closed".to_string());
            }
            
            Ok(Token::Char(c))
        }
    }
    
    fn read_identifier_or_keyword(&mut self, first: char) -> Result<Token, String> {
        let mut identifier = String::from(first);
        
        while let Some(&c) = self.input.peek() {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(self.input.next().unwrap());
            } else {
                break;
            }
        }
        
        Ok(match identifier.as_str() {
            "let" => Token::Let,
            "mut" => Token::Mut,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "fn" => Token::Fn,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "match" => Token::Match,
            "loop" => Token::Loop,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "use" => Token::Use,
            "from" => Token::From,
            "mod" => Token::Mod,
            "module" => Token::Module,
            "pub" => Token::Pub,
            "borrow" => Token::Borrow,
            "copy" => Token::Copy,
            "comptime" => Token::Comptime,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "throw" => Token::Throw,
            
            // Ownership and borrowing keywords
            "move" => Token::Move,
            "drop" => Token::Drop,
            "clone" => Token::Clone,
            "ref" => Token::Ref,
            "mutref" => Token::MutRef,
            "owned" => Token::Owned,
            "shared" => Token::Shared,
            "lifetime" => Token::Lifetime,
            "where" => Token::Where,
            "impl" => Token::Impl,
            "trait" => Token::Trait,
            "box" => Token::Box,
            "rc" => Token::Rc,
            "arc" => Token::Arc,
            "cell" => Token::Cell,
            "refcell" => Token::RefCell,
            "mutex" => Token::Mutex,
            "rwlock" => Token::RwLock,
            "malloc" => Token::Malloc,
            "free" => Token::Free,
            "realloc" => Token::Realloc,
            
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "spawn" => Token::Spawn,
            "join" => Token::Join,
            "channel" => Token::Channel,
            "class" => Token::Class,
            "extends" => Token::Extends,
            "import" => Token::Import,
            "as" => Token::As,
            _ => Token::Identifier(identifier),
        })
    }
    
    fn read_number(&mut self, first: char) -> Result<Token, String> {
        let mut number = String::from(first);
        let mut has_decimal = false;
        let mut has_exponent = false;
        
        while let Some(&c) = self.input.peek() {
            match c {
                '0'..='9' => {
                    number.push(self.input.next().unwrap());
                }
                '.' => {
                    // Peek ahead to see if the next character is a digit
                    let mut iter = self.input.clone();
                    iter.next(); // consume the current '.'
                    if let Some(&next_c) = iter.peek() {
                        if next_c.is_digit(10) {
                            if has_decimal {
                                return Err("Invalid number: multiple decimal points".to_string());
                            }
                            has_decimal = true;
                            number.push(self.input.next().unwrap());
                        } else {
                            // Not a digit, so break and let the main lexer handle the '.' or '..'
                            break;
                        }
                    } else {
                        // End of input after '.', treat as end of number
                        break;
                    }
                }
                'e' | 'E' => {
                    if has_exponent {
                        return Err("Invalid number: multiple exponents".to_string());
                    }
                    has_exponent = true;
                    number.push(self.input.next().unwrap());
                    
                    if let Some(&sign) = self.input.peek() {
                        if sign == '+' || sign == '-' {
                            number.push(self.input.next().unwrap());
                        }
                    }
                }
                '_' => {
                    self.input.next(); // Skip underscore separators
                }
                _ => break,
            }
        }
        
        if has_decimal || has_exponent {
            number.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| format!("Invalid float: {}", number))
        } else {
            number.parse::<i64>()
                .map(Token::Number)
                .map_err(|_| format!("Invalid integer: {}", number))
        }
    }
    
    fn get_lexeme(&self, start_line: usize, start_column: usize) -> String {
        // This is a simplified version - in a real implementation,
        // you'd want to track the actual lexeme more precisely
        format!("line {}:{}", start_line, start_column)
    }
}