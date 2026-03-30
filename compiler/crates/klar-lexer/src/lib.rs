mod token;
mod lexer;
mod span;

pub use token::{Token, TokenKind, StringPart as LexerStringPart};
pub use lexer::Lexer;
pub use span::Span;

#[cfg(test)]
mod tests;
