mod parser;

pub use parser::{Parser, ParseError};

use klar_ast::Program;
use klar_lexer::Lexer;

/// Parse a Klar source string into an AST.
pub fn parse(source: &str) -> Result<Program, Vec<ParseError>> {
    let tokens = Lexer::tokenize(source);
    let mut parser = Parser::new(tokens, source);
    parser.parse_program()
}

#[cfg(test)]
mod tests;
