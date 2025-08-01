use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Array(Vec<Literal>),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    FunctionCall(Box<Expression>, Vec<CallArgument>),
    If(IfExpression),
    While(WhileExpression),
    Block(Vec<Statement>),
    Return(Option<Box<Expression>>),
    Let(LetStatement),
    Assignment(AssignmentStatement),
    Malloc(MallocExpression),
    Free(FreeExpression),
    Realloc(ReallocExpression),
    TryCatch(TryCatchExpression),
    Move(MoveStatement),
    Drop(DropStatement),
    Borrow(BorrowExpression),
    BorrowMut(BorrowMutExpression),
    Clone(CloneExpression),
    BinaryOperation { left: Box<Expression>, operator: BinaryOperator, right: Box<Expression> },
    // Add missing variants
    Loop(LoopExpression),
    StructLiteral(StructLiteralExpression),
    MemberAccess(MemberAccessExpression),
    EnumVariantAccess { enum_name: String, variant_name: String },
    BuiltinFunction { name: String, arguments: Vec<Expression> },
    ArrayAccess(ArrayAccessExpression),
    Box(BoxExpression),
    Rc(RcExpression),
    Arc(ArcExpression),
    Cell(CellExpression),
    RefCell(RefCellExpression),
    Lifetime(LifetimeExpression),
    Match(MatchExpression),
    Spawn(SpawnExpression),
    Join(JoinExpression),
    Channel(ChannelExpression),
    Try(TryExpression),
    Pipeline(PipelineExpression),
    Throw(ThrowExpression),
    Lambda(LambdaExpression),
    DictLiteral(DictLiteralExpression),
    SetLiteral(SetLiteralExpression),
    InterpolatedString(InterpolatedStringExpression),
    ListComprehension(ListComprehensionExpression),
    Slice(SliceExpression),
    // Add missing variants for type inference and borrow checker
    BinaryExpression { left: Box<Expression>, operator: BinaryOperator, right: Box<Expression> },
    UnaryExpression { operator: UnaryOperator, operand: Box<Expression> },
    CallExpression { function: String, arguments: Vec<Expression> },
    IfExpression { condition: Box<Expression>, then_branch: Box<Expression>, else_branch: Option<Box<Expression>> },
    BlockExpression { statements: Vec<Statement> },
    ReferenceExpression { target: Box<Expression>, borrow_type: BorrowType },
    DereferenceExpression { target: Box<Expression> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOp {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOp {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub else_branch: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileExpression {
    pub condition: Box<Expression>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetStatement {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub value: Box<Expression>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStatement {
    pub target: String,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MallocExpression {
    pub size: Box<Expression>,
    pub type_annotation: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeExpression {
    pub pointer: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReallocExpression {
    pub pointer: Box<Expression>,
    pub new_size: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryCatchExpression {
    pub try_block: Box<Expression>,
    pub catch_block: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveStatement {
    pub from: String,
    pub to: String,
    pub ownership_transfer: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropStatement {
    pub variable: String,
    pub explicit: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorrowExpression {
    pub expression: Box<Expression>,
    pub borrow_type: BorrowType,
    pub lifetime: Option<Lifetime>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorrowMutExpression {
    pub expression: Box<Expression>,
    pub lifetime: Option<Lifetime>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CloneExpression {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    // Add missing variants for type inference
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
    Dereference,
    Reference,
    ReferenceMut,
    // Add missing variants
    Copy,
    Borrow,
    BorrowMut,
    Move,
    Drop,
    // Add missing variant for type inference
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowType {
    ImmutableBorrow,
    MutableBorrow,
    Move,
    Copy,
    // Add missing variants
    Borrowed,
    MutableBorrowed,
    // Add missing variants for borrow checker
    Immutable,
    Mutable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub name: String,
    pub is_inferred: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Let(LetStatement),
    Return(ReturnStatement),
    Function(FunctionStatement),
    Struct(StructStatement),
    Enum(EnumStatement),
    Trait(TraitStatement),
    Impl(ImplStatement),
    Use(UseStatement),
    Module(ModuleStatement),
    Move(MoveStatement),
    Drop(DropStatement),
    // Add missing variant
    GenericFunction(GenericFunctionStatement),
    Class(ClassStatement),
    // Add missing variants for type inference and borrow checker
    LetStatement { name: String, value: Box<Expression>, var_type: Option<Type> },
    AssignmentStatement { name: String, value: Box<Expression> },
    FunctionStatement { name: String, parameters: Vec<Parameter>, return_type: Option<Type>, body: Box<Expression> },
    ReturnStatement { value: Option<Box<Expression>> },
    ExpressionStatement { expression: Box<Expression> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub value: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Box<Expression>,
    pub annotations: Vec<Annotation>,
    pub signature: FunctionSignature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Type,
    pub borrow_type: Option<BorrowType>,
    pub lifetime: Option<Lifetime>,
    pub ownership: Option<String>,
    pub default_value: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructStatement {
    pub name: String,
    pub fields: Vec<StructField>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumStatement {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<StructField>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitStatement {
    pub name: String,
    pub methods: Vec<FunctionStatement>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplStatement {
    pub trait_name: Option<String>,
    pub type_name: String,
    pub methods: Vec<FunctionStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleStatement {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub attached_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Char,
    Void,
    Never,
    Array(Box<Type>, usize),
    Pointer(Box<Type>),
    Reference(Box<Type>, BorrowType, Option<Lifetime>),
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Enum(String),
    Trait(String),
    Generic(String, Vec<Type>),
    // Add missing variants
    Unknown,
    GenericType(String, Vec<Type>),
    Owned(Box<Type>),
    Shared(Box<Type>),
    Weak(Box<Type>),
    Unique(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Slice(Box<Type>),
    Tuple(Vec<Type>),
    Union(Vec<Type>),
    // Add missing variants for type inference
    Any,
    Null,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Char => write!(f, "char"),
            Type::Void => write!(f, "void"),
            Type::Never => write!(f, "never"),
            Type::Array(element_type, size) => write!(f, "[{}; {}]", element_type, size),
            Type::Pointer(pointee_type) => write!(f, "*{}", pointee_type),
            Type::Reference(referent_type, borrow_type, lifetime) => {
                let borrow_str = match borrow_type {
                    BorrowType::ImmutableBorrow => "&",
                    BorrowType::MutableBorrow => "&mut ",
                    BorrowType::Move => "move ",
                    BorrowType::Copy => "copy ",
                    BorrowType::Borrowed => "&",
                    BorrowType::MutableBorrowed => "&mut ",
                    BorrowType::Immutable => "&",
                    BorrowType::Mutable => "&mut ",
                };
                let lifetime_str = lifetime.as_ref().map(|l| format!("'{} ", l.name)).unwrap_or_default();
                write!(f, "{}{}{}", lifetime_str, borrow_str, referent_type)
            }
            Type::Function(params, return_type) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Struct(name) => write!(f, "struct {}", name),
            Type::Enum(name) => write!(f, "enum {}", name),
            Type::Trait(name) => write!(f, "trait {}", name),
            Type::Generic(name, type_args) => {
                write!(f, "{}", name)?;
                if !type_args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in type_args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Unknown => write!(f, "unknown"),
            Type::GenericType(name, type_args) => {
                write!(f, "{}", name)?;
                if !type_args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in type_args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Owned(inner_type) => write!(f, "owned {}", inner_type),
            Type::Shared(inner_type) => write!(f, "shared {}", inner_type),
            Type::Weak(inner_type) => write!(f, "weak {}", inner_type),
            Type::Unique(inner_type) => write!(f, "unique {}", inner_type),
            Type::Result(ok_type, err_type) => write!(f, "Result<{}, {}>", ok_type, err_type),
            Type::Option(inner_type) => write!(f, "Option<{}>", inner_type),
            Type::Slice(inner_type) => write!(f, "[{}]", inner_type),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Union(types) => {
                write!(f, "union(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Any => write!(f, "any"),
            Type::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Struct(String, Vec<Pattern>),
    Tuple(Vec<Pattern>),
    Wildcard,
    Or(Vec<Pattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expression: Box<Expression>,
    pub body: Box<Expression>,
    pub guard: Option<Box<Expression>>,
    pub location: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub annotations: Vec<Annotation>,
} 

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpression {
    pub body: Box<Expression>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructLiteralExpression {
    pub struct_name: String,
    pub fields: Vec<(String, Expression)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberAccessExpression {
    pub object: Box<Expression>,
    pub member: String,
}

// Alias for backward compatibility
pub type MemberAccess = MemberAccessExpression;

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayAccessExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxExpression {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RcExpression {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArcExpression {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellExpression {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefCellExpression {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeExpression {
    pub lifetime: Lifetime,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpression {
    pub expression: Box<Expression>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnExpression {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinExpression {
    pub handle: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelExpression {
    pub channel_type: ChannelType,
    pub capacity: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelType {
    Sender,
    Receiver,
    Bounded,
    Unbounded,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpression {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    pub stages: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericFunctionStatement {
    pub name: String,
    pub type_parameters: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Box<Expression>,
    pub annotations: Vec<Annotation>,
} 

#[derive(Debug, Clone, PartialEq)]
pub struct ThrowExpression {
    pub value: Box<Expression>,
} 

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpression {
    pub parameters: Vec<Parameter>,
    pub body: Box<Expression>,
} 

#[derive(Debug, Clone, PartialEq)]
pub struct ClassStatement {
    pub name: String,
    pub superclass: Option<String>,
    pub fields: Vec<StructField>,
    pub methods: Vec<FunctionStatement>,
    pub annotations: Vec<Annotation>,
} 

#[derive(Debug, Clone, PartialEq)]
pub struct DictLiteralExpression {
    pub entries: Vec<(Expression, Expression)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetLiteralExpression {
    pub elements: Vec<Expression>,
} 

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedPart {
    String(String),
    Expr(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpolatedStringExpression {
    pub parts: Vec<InterpolatedPart>,
} 

#[derive(Debug, Clone, PartialEq)]
pub struct ListComprehensionExpression {
    pub element: Box<Expression>,
    pub iterator: String,
    pub iterable: Box<Expression>,
    pub condition: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SliceExpression {
    pub collection: Box<Expression>,
    pub start: Option<Box<Expression>>,
    pub end: Option<Box<Expression>>,
    pub step: Option<Box<Expression>>,
} 

#[derive(Debug, Clone, PartialEq)]
pub struct CallArgument {
    pub name: Option<String>,
    pub value: Expression,
} 