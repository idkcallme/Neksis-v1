// Modern Lexer for Neksis 2025
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    
    // Identifiers and keywords
    Identifier(String),
    
    // Keywords - Core language
    Let,
    Mut,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Loop,
    Break,
    Continue,
    Match,
    
    // Keywords - Type system
    Struct,
    Enum,
    Class,
    Trait,
    Impl,
    Type,
    
    // Keywords - Module system
    Module,
    Use,
    Import,
    Export,
    From,
    As,
    Pub,
    
    // Keywords - Async/concurrency
    Async,
    Await,
    Spawn,
    Join,
    Send,
    Sync,
    
    // Keywords - Error handling
    Try,
    Catch,
    Finally,
    Throw,
    Result,
    Option,
    Some,
    None,
    Ok,
    Err,
    
    // Keywords - Memory management
    Box,
    Rc,
    Arc,
    Ref,
    MutRef,
    Move,
    Copy,
    Clone,
    Drop,
    
    // Keywords - Generics
    Where,
    Self_,
    Super,
    
    // Operators - Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,     // **
    
    // Operators - Comparison
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    
    // Operators - Logical
    And,          // &&
    Or,           // ||
    Not,          // !
    
    // Operators - Bitwise
    Ampersand,    // &
    Pipe,         // |
    Caret,        // ^
    Tilde,        // ~
    LeftShift,    // <<
    RightShift,   // >>
    
    // Operators - Assignment
    Assign,       // =
    PlusAssign,   // +=
    MinusAssign,  // -=
    StarAssign,   // *=
    SlashAssign,  // /=
    PercentAssign,// %=
    
    // Operators - Misc
    Arrow,        // ->
    FatArrow,     // =>
    Question,     // ?
    Dot,          // .
    DotDot,       // ..
    DotDotEqual,  // ..=
    DoubleColon,  // ::
    
    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    LeftAngle,    // <
    RightAngle,   // >
    
    // Punctuation
    Comma,        // ,
    Semicolon,    // ;
    Colon,        // :
    At,           // @
    Hash,         // #
    Dollar,       // $
    
    // Special
    Newline,
    Eof,
    
    // String interpolation
    InterpolationStart,  // ${
    InterpolationEnd,    // }
}

#[allow(dead_code)]
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::str::Chars<'a>,
    current_char: Option<char>,
    position: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            chars: input.chars(),
            current_char: None,
            position: 0,
            line: 1,
            column: 1,
            keywords: HashMap::new(),
        };
        
        lexer.init_keywords();
        lexer.advance();
        lexer
    }
    
    fn init_keywords(&mut self) {
        let keywords = [
            // Core language
            ("let", Token::Let),
            ("mut", Token::Mut),
            ("fn", Token::Fn),
            ("return", Token::Return),
            ("if", Token::If),
            ("else", Token::Else),
            ("while", Token::While),
            ("for", Token::For),
            ("in", Token::In),
            ("loop", Token::Loop),
            ("break", Token::Break),
            ("continue", Token::Continue),
            ("match", Token::Match),
            
            // Type system
            ("struct", Token::Struct),
            ("enum", Token::Enum),
            ("class", Token::Class),
            ("trait", Token::Trait),
            ("impl", Token::Impl),
            ("type", Token::Type),
            
            // Module system
            ("module", Token::Module),
            ("use", Token::Use),
            ("import", Token::Import),
            ("export", Token::Export),
            ("from", Token::From),
            ("as", Token::As),
            ("pub", Token::Pub),
            
            // Async/concurrency
            ("async", Token::Async),
            ("await", Token::Await),
            ("spawn", Token::Spawn),
            ("join", Token::Join),
            ("send", Token::Send),
            ("sync", Token::Sync),
            
            // Error handling
            ("try", Token::Try),
            ("catch", Token::Catch),
            ("finally", Token::Finally),
            ("throw", Token::Throw),
            ("Result", Token::Result),
            ("Option", Token::Option),
            ("Some", Token::Some),
            ("None", Token::None),
            ("Ok", Token::Ok),
            ("Err", Token::Err),
            
            // Memory management
            ("Box", Token::Box),
            ("Rc", Token::Rc),
            ("Arc", Token::Arc),
            ("ref", Token::Ref),
            ("mut_ref", Token::MutRef),
            ("move", Token::Move),
            ("copy", Token::Copy),
            ("clone", Token::Clone),
            ("drop", Token::Drop),
            
            // Generics
            ("where", Token::Where),
            ("self", Token::Self_),
            ("super", Token::Super),
            
            // Literals
            ("true", Token::Boolean(true)),
            ("false", Token::Boolean(false)),
            ("null", Token::Null),
        ];
        
        for (keyword, token) in keywords.iter() {
            self.keywords.insert(keyword.to_string(), token.clone());
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<TokenInfo> {
        let mut tokens = Vec::new();
        
        while let Some(token_info) = self.next_token() {
            if token_info.token == Token::Eof {
                tokens.push(token_info);
                break;
            }
            tokens.push(token_info);
        }
        
        tokens
    }
    
    pub fn next_token(&mut self) -> Option<TokenInfo> {
        self.skip_whitespace();
        
        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.position;
        
        let token = match self.current_char {
            None => Token::Eof,
            Some('\n') => {
                self.advance();
                Token::Newline
            },
            Some(ch) if ch.is_ascii_digit() => self.read_number(),
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_identifier_or_keyword(),
            Some('"') => self.read_string(),
            Some('\'') => self.read_char(),
            Some('+') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::PlusAssign
                } else {
                    Token::Plus
                }
            },
            Some('-') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::MinusAssign
                } else if self.current_char == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            },
            Some('*') => {
                self.advance();
                if self.current_char == Some('*') {
                    self.advance();
                    Token::StarStar
                } else if self.current_char == Some('=') {
                    self.advance();
                    Token::StarAssign
                } else {
                    Token::Star
                }
            },
            Some('/') => {
                self.advance();
                if self.current_char == Some('/') {
                    self.skip_line_comment();
                    return self.next_token();
                } else if self.current_char == Some('*') {
                    self.skip_block_comment();
                    return self.next_token();
                } else if self.current_char == Some('=') {
                    self.advance();
                    Token::SlashAssign
                } else {
                    Token::Slash
                }
            },
            Some('%') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::PercentAssign
                } else {
                    Token::Percent
                }
            },
            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::Equal
                } else if self.current_char == Some('>') {
                    self.advance();
                    Token::FatArrow
                } else {
                    Token::Assign
                }
            },
            Some('!') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    Token::Not
                }
            },
            Some('<') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::LessEqual
                } else if self.current_char == Some('<') {
                    self.advance();
                    Token::LeftShift
                } else {
                    Token::LeftAngle
                }
            },
            Some('>') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else if self.current_char == Some('>') {
                    self.advance();
                    Token::RightShift
                } else {
                    Token::RightAngle
                }
            },
            Some('&') => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    Token::And
                } else {
                    Token::Ampersand
                }
            },
            Some('|') => {
                self.advance();
                if self.current_char == Some('|') {
                    self.advance();
                    Token::Or
                } else {
                    Token::Pipe
                }
            },
            Some('^') => {
                self.advance();
                Token::Caret
            },
            Some('~') => {
                self.advance();
                Token::Tilde
            },
            Some('(') => {
                self.advance();
                Token::LeftParen
            },
            Some(')') => {
                self.advance();
                Token::RightParen
            },
            Some('{') => {
                self.advance();
                Token::LeftBrace
            },
            Some('}') => {
                self.advance();
                Token::RightBrace
            },
            Some('[') => {
                self.advance();
                Token::LeftBracket
            },
            Some(']') => {
                self.advance();
                Token::RightBracket
            },
            Some(',') => {
                self.advance();
                Token::Comma
            },
            Some(';') => {
                self.advance();
                Token::Semicolon
            },
            Some(':') => {
                self.advance();
                if self.current_char == Some(':') {
                    self.advance();
                    Token::DoubleColon
                } else {
                    Token::Colon
                }
            },
            Some('.') => {
                self.advance();
                if self.current_char == Some('.') {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::DotDotEqual
                    } else {
                        Token::DotDot
                    }
                } else {
                    Token::Dot
                }
            },
            Some('?') => {
                self.advance();
                Token::Question
            },
            Some('@') => {
                self.advance();
                Token::At
            },
            Some('#') => {
                self.advance();
                Token::Hash
            },
            Some('$') => {
                self.advance();
                if self.current_char == Some('{') {
                    self.advance();
                    Token::InterpolationStart
                } else {
                    Token::Dollar
                }
            },
            Some(ch) => {
                self.advance();
                return Some(TokenInfo {
                    token: Token::Identifier(ch.to_string()),
                    line: start_line,
                    column: start_column,
                    span: Span { start: start_pos, end: self.position },
                });
            }
        };
        
        Some(TokenInfo {
            token,
            line: start_line,
            column: start_column,
            span: Span { start: start_pos, end: self.position },
        })
    }
    
    fn advance(&mut self) {
        if let Some('\n') = self.current_char {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        
        self.position += self.current_char.map_or(0, |c| c.len_utf8());
        self.current_char = self.chars.next();
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.current_char {
            self.advance();
            if ch == '\n' {
                break;
            }
        }
    }
    
    fn skip_block_comment(&mut self) {
        self.advance(); // skip '*'
        
        while let Some(ch) = self.current_char {
            if ch == '*' {
                self.advance();
                if self.current_char == Some('/') {
                    self.advance();
                    break;
                }
            } else {
                self.advance();
            }
        }
    }
    
    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        let mut is_float = false;
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                // Check if it's a decimal point or method call
                let mut chars_copy = self.chars.clone();
                if let Some(next_ch) = chars_copy.next() {
                    if next_ch.is_ascii_digit() {
                        is_float = true;
                        number.push(ch);
                        self.advance();
                    } else {
                        break; // It's a method call like 42.to_string()
                    }
                } else {
                    break;
                }
            } else if ch == '_' {
                // Skip underscores in numbers (like 1_000_000)
                self.advance();
            } else {
                break;
            }
        }
        
        if is_float {
            Token::Float(number.parse().unwrap_or(0.0))
        } else {
            Token::Integer(number.parse().unwrap_or(0))
        }
    }
    
    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut identifier = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check if it's a keyword
        self.keywords.get(&identifier)
            .cloned()
            .unwrap_or_else(|| Token::Identifier(identifier))
    }
    
    fn read_string(&mut self) -> Token {
        let mut string = String::new();
        self.advance(); // skip opening quote
        
        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some('0') => string.push('\0'),
                    Some(c) => string.push(c),
                    None => break,
                }
                self.advance();
            } else {
                string.push(ch);
                self.advance();
            }
        }
        
        Token::String(string)
    }
    
    fn read_char(&mut self) -> Token {
        self.advance(); // skip opening quote
        
        let ch = match self.current_char {
            Some('\\') => {
                self.advance();
                match self.current_char {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('0') => '\0',
                    Some(c) => c,
                    None => '\0',
                }
            },
            Some(c) => c,
            None => '\0',
        };
        
        self.advance();
        
        if self.current_char == Some('\'') {
            self.advance(); // skip closing quote
        }
        
        Token::String(ch.to_string()) // For now, treat chars as single-character strings
    }
}
