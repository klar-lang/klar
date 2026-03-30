use crate::Span;

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// All possible token kinds in Klar.
///
/// Klar has exactly 29 keywords. Operators and delimiters are minimal
/// to enforce the one-way principle.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === Keywords (29 total) ===
    Let,
    Mut,
    Fn,
    Struct,
    Enum,
    Trait,
    Impl,
    Use,
    If,
    Else,
    Match,
    For,
    In,
    Loop,
    Break,
    Return,
    True,
    False,
    And,
    Or,
    Not,
    Pub,
    Priv,
    Test,
    Spawn,
    Parallel,
    Catch,
    Async,
    Unsafe,

    // === Literals ===
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    /// String with interpolation segments: alternating literal parts and expression parts.
    /// e.g. "Hello {name}!" => ["Hello ", "name", "!"]
    /// Odd indices are expressions, even indices are literal text.
    InterpolatedString(Vec<StringPart>),

    // === Identifiers ===
    Ident(String),

    // === Operators ===
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Eq,          // =
    EqEq,        // ==
    NotEq,       // !=
    Lt,          // <
    Gt,          // >
    LtEq,        // <=
    GtEq,        // >=
    Arrow,       // ->
    FatArrow,    // =>
    Pipe,        // |
    PipeArrow,   // |>
    Question,    // ?
    Bang,        // !
    Dot,         // .
    DotDot,      // ..
    Colon,       // :
    ColonColon,  // ::
    Comma,       // ,
    Semicolon,   // ;
    At,          // @
    Ampersand,   // &

    // === Delimiters ===
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]

    // === Special ===
    Newline,
    Comment(String),
    Eof,
    Error(String),
}

/// A part of an interpolated string.
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Expr(String),
}

impl TokenKind {
    /// Try to match an identifier string to a keyword.
    pub fn keyword(ident: &str) -> Option<TokenKind> {
        match ident {
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "fn" => Some(TokenKind::Fn),
            "struct" => Some(TokenKind::Struct),
            "enum" => Some(TokenKind::Enum),
            "trait" => Some(TokenKind::Trait),
            "impl" => Some(TokenKind::Impl),
            "use" => Some(TokenKind::Use),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "match" => Some(TokenKind::Match),
            "for" => Some(TokenKind::For),
            "in" => Some(TokenKind::In),
            "loop" => Some(TokenKind::Loop),
            "break" => Some(TokenKind::Break),
            "return" => Some(TokenKind::Return),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "and" => Some(TokenKind::And),
            "or" => Some(TokenKind::Or),
            "not" => Some(TokenKind::Not),
            "pub" => Some(TokenKind::Pub),
            "priv" => Some(TokenKind::Priv),
            "test" => Some(TokenKind::Test),
            "spawn" => Some(TokenKind::Spawn),
            "parallel" => Some(TokenKind::Parallel),
            "catch" => Some(TokenKind::Catch),
            "async" => Some(TokenKind::Async),
            "unsafe" => Some(TokenKind::Unsafe),
            _ => None,
        }
    }
}
