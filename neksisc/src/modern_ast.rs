// Modern AST for Neksis 2025
use std::fmt;
use std::collections::HashMap;

// Core Types
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub modules: HashMap<String, Module>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub statements: Vec<Statement>,
    pub exports: Vec<String>,
    pub imports: Vec<Import>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
    pub items: Vec<String>,
    pub alias: Option<String>,
}

// Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(LetStatement),
    Function(FunctionStatement),
    Struct(StructStatement),
    Enum(EnumStatement),
    Class(ClassStatement),
    Module(ModuleStatement),
    Use(UseStatement),
    Expression(Expression),
    Return(ReturnStatement),
    Break,
    Continue,
    Throw(ThrowStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetStatement {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub value: Box<Expression>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Box<Expression>,
    pub generic_params: Vec<String>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Type,
    pub default_value: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructStatement {
    pub name: String,
    pub fields: Vec<StructField>,
    pub generic_params: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumStatement {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub generic_params: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassStatement {
    pub name: String,
    pub fields: Vec<StructField>,
    pub methods: Vec<FunctionStatement>,
    pub superclass: Option<String>,
    pub generic_params: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleStatement {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    pub path: String,
    pub items: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub value: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrowStatement {
    pub value: Box<Expression>,
}

// Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    Literal(Literal),
    Identifier(String),
    
    // Binary operations
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    
    // Unary operations
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    
    // Function calls
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    
    // Control flow
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    },
    
    While {
        condition: Box<Expression>,
        body: Box<Expression>,
    },
    
    For {
        variable: String,
        iterable: Box<Expression>,
        body: Box<Expression>,
    },
    
    Loop {
        body: Box<Expression>,
    },
    
    Match {
        expression: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    
    // Block expressions
    Block {
        statements: Vec<Statement>,
        expression: Option<Box<Expression>>,
    },
    
    // Collections
    Array {
        elements: Vec<Expression>,
    },
    
    HashMap {
        pairs: Vec<(Expression, Expression)>,
    },
    
    HashSet {
        elements: Vec<Expression>,
    },
    
    // Object access
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    
    // Struct literal
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    
    // Assignment
    Assignment {
        target: String,
        value: Box<Expression>,
    },
    
    // Async/await
    Async {
        body: Box<Expression>,
    },
    
    Await {
        expression: Box<Expression>,
    },
    
    // Try/catch
    Try {
        body: Box<Expression>,
        catch_clauses: Vec<CatchClause>,
        finally_clause: Option<Box<Expression>>,
    },
    
    // Closures/lambdas
    Lambda {
        parameters: Vec<Parameter>,
        body: Box<Expression>,
    },
    
    // String interpolation
    InterpolatedString {
        parts: Vec<InterpolatedPart>,
    },
    
    // Range expressions
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expression>>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    Enum {
        variant: String,
        fields: Vec<Pattern>,
    },
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub exception_type: Option<Type>,
    pub variable: Option<String>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedPart {
    String(String),
    Expression(Expression),
}

// Literals
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Literal>),
    HashMap(Vec<(Literal, Literal)>),
}

// Operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    
    // Logical
    And,
    Or,
    
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    
    // String
    Concat,
    
    // Assignment
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    
    // Range
    Range,
    RangeInclusive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
    BitwiseNot,
    Dereference,
    Reference,
    MutableReference,
}

// Types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitives
    Int,
    Float,
    String,
    Boolean,
    Void,
    
    // Collections
    Array(Box<Type>),
    Vec(Box<Type>),
    HashMap(Box<Type>, Box<Type>),
    HashSet(Box<Type>),
    
    // User-defined
    Struct(String),
    Enum(String),
    Class(String),
    
    // Function types
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    
    // Generic types
    Generic(String, Vec<Type>),
    
    // Optional and Result types
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    
    // Reference types
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    
    // Async types
    Future(Box<Type>),
    
    // Any type for dynamic typing
    Any,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Boolean => write!(f, "Bool"),
            Type::Void => write!(f, "Void"),
            Type::Array(t) => write!(f, "[{}]", t),
            Type::Vec(t) => write!(f, "Vec<{}>", t),
            Type::HashMap(k, v) => write!(f, "HashMap<{}, {}>", k, v),
            Type::HashSet(t) => write!(f, "HashSet<{}>", t),
            Type::Struct(name) => write!(f, "{}", name),
            Type::Enum(name) => write!(f, "{}", name),
            Type::Class(name) => write!(f, "{}", name),
            Type::Function { parameters, return_type } => {
                write!(f, "fn(")?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            },
            Type::Generic(name, args) => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            },
            Type::Option(t) => write!(f, "Option<{}>", t),
            Type::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            Type::Reference(t) => write!(f, "&{}", t),
            Type::MutableReference(t) => write!(f, "&mut {}", t),
            Type::Future(t) => write!(f, "Future<{}>", t),
            Type::Any => write!(f, "Any"),
        }
    }
}
