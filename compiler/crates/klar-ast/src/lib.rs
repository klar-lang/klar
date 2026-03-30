use klar_lexer::Span;

/// A complete Klar source file.
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// A top-level item in a Klar program.
#[derive(Debug, Clone)]
pub enum Item {
    Function(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    Use(UseDecl),
    Test(TestDecl),
}

// ============================================================
// Declarations
// ============================================================

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub error_type: Option<TypeExpr>,
    pub body: Block,
    pub is_priv: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub ty: TypeExpr,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: Ident,
    pub annotations: Vec<Annotation>,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub ty: TypeExpr,
    pub annotations: Vec<Annotation>,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: Ident,
    pub variants: Vec<Variant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: Ident,
    pub fields: Vec<Param>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub name: Ident,
    pub methods: Vec<FnSig>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub error_type: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplDecl {
    pub trait_name: Ident,
    pub target: Ident,
    pub methods: Vec<FnDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Vec<Ident>,
    pub items: Option<Vec<Ident>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TestDecl {
    pub name: Ident,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

// ============================================================
// Types
// ============================================================

#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// Simple named type: `Int`, `String`, `User`
    Named(Ident),
    /// Generic type: `List[Int]`, `Map[String, Int]`
    Generic(Ident, Vec<TypeExpr>),
    /// Option type: `User?` (sugar for `Option[User]`)
    Option(Box<TypeExpr>),
    /// Unit type: `()`
    Unit,
}

// ============================================================
// Expressions
// ============================================================

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    IntLit(i64, Span),
    FloatLit(f64, Span),
    StringLit(String, Span),
    InterpolatedString(Vec<StringPart>, Span),
    BoolLit(bool, Span),

    // Identifiers
    Ident(Ident),

    // Operations
    Binary(Box<Expr>, BinOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),

    // Access
    FieldAccess(Box<Expr>, Ident, Span),
    Index(Box<Expr>, Box<Expr>, Span),

    // Calls
    Call(Box<Expr>, Vec<CallArg>, Span),

    // Constructors
    StructInit(Ident, Vec<FieldInit>, Span),
    ListLit(Vec<Expr>, Span),
    MapLit(Vec<(Expr, Expr)>, Span),

    // Control flow (as expressions)
    If(Box<Expr>, Block, Option<Box<Expr>>, Span),
    Match(Box<Expr>, Vec<MatchArm>, Span),
    Block(Block),

    // Closures
    Closure(Vec<ClosureParam>, Box<Expr>, Span),

    // Pipe
    Pipe(Box<Expr>, Box<Expr>, Span),

    // Error handling
    Try(Box<Expr>, Span),          // expr?
    Catch(Box<Expr>, Ident, Block, Span), // expr catch err { }
    ElseUnwrap(Box<Expr>, Block, Span),   // expr else { }

    // Struct spread
    Spread(Box<Expr>, Span),       // ..expr

    // Range
    Range(Box<Expr>, Box<Expr>, Span), // a..b

    // Error placeholder for recovery
    Error(Span),
}

#[derive(Debug, Clone)]
pub struct StringPart {
    pub kind: StringPartKind,
}

#[derive(Debug, Clone)]
pub enum StringPartKind {
    Literal(String),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct CallArg {
    pub name: Option<Ident>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: Ident,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub name: Ident,
    pub ty: Option<TypeExpr>,
}

// ============================================================
// Patterns
// ============================================================

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Bind to a name: `x`
    Binding(Ident),
    /// Literal: `42`, `"hello"`, `true`
    Literal(Expr),
    /// Enum variant: `Circle(r)`, `Some(val)`
    Variant(Ident, Vec<Pattern>),
    /// Wildcard: `_`
    Wildcard(Span),
    /// Struct destructure: `User { name, age }`
    Struct(Ident, Vec<(Ident, Option<Pattern>)>, Span),
}

// ============================================================
// Statements
// ============================================================

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    For(ForStmt),
    Loop(Block, Span),
    Break(Span),
    Return(Option<Expr>, Span),
    Assign(Expr, Expr, Span),
    Item(Item),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub binding: Ident,
    pub index: Option<Ident>,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

// ============================================================
// Operators
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, Not,
}

// ============================================================
// Common
// ============================================================

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self { name: name.into(), span }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLit(_, s) | Expr::FloatLit(_, s) | Expr::StringLit(_, s)
            | Expr::InterpolatedString(_, s) | Expr::BoolLit(_, s) => *s,
            Expr::Ident(id) => id.span,
            Expr::Binary(_, _, _, s) | Expr::Unary(_, _, s) => *s,
            Expr::FieldAccess(_, _, s) | Expr::Index(_, _, s) | Expr::Call(_, _, s) => *s,
            Expr::StructInit(_, _, s) | Expr::ListLit(_, s) | Expr::MapLit(_, s) => *s,
            Expr::If(_, _, _, s) | Expr::Match(_, _, s) => *s,
            Expr::Block(b) => b.span,
            Expr::Closure(_, _, s) | Expr::Pipe(_, _, s) => *s,
            Expr::Try(_, s) | Expr::Catch(_, _, _, s) | Expr::ElseUnwrap(_, _, s) => *s,
            Expr::Spread(_, s) | Expr::Range(_, _, s) => *s,
            Expr::Error(s) => *s,
        }
    }
}
