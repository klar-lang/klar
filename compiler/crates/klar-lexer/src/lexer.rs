use crate::span::Span;
use crate::token::{StringPart, Token, TokenKind};

/// The Klar lexer. Converts source text into a stream of tokens.
///
/// Designed for error recovery: malformed input produces `TokenKind::Error`
/// tokens instead of panicking, allowing the parser to continue.
pub struct Lexer<'src> {
    source: &'src str,
    bytes: &'src [u8],
    pos: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    /// Tokenize the entire source into a Vec<Token>.
    pub fn tokenize(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.is_at_end() {
            return Token::new(TokenKind::Eof, Span::new(self.pos, self.pos));
        }

        let start = self.pos;
        let ch = self.advance();

        match ch {
            b'\n' => Token::new(TokenKind::Newline, Span::new(start, self.pos)),

            // Single-char delimiters
            b'(' => Token::new(TokenKind::LParen, Span::new(start, self.pos)),
            b')' => Token::new(TokenKind::RParen, Span::new(start, self.pos)),
            b'{' => Token::new(TokenKind::LBrace, Span::new(start, self.pos)),
            b'}' => Token::new(TokenKind::RBrace, Span::new(start, self.pos)),
            b'[' => Token::new(TokenKind::LBracket, Span::new(start, self.pos)),
            b']' => Token::new(TokenKind::RBracket, Span::new(start, self.pos)),
            b',' => Token::new(TokenKind::Comma, Span::new(start, self.pos)),
            b';' => Token::new(TokenKind::Semicolon, Span::new(start, self.pos)),
            b'@' => Token::new(TokenKind::At, Span::new(start, self.pos)),
            b'&' => Token::new(TokenKind::Ampersand, Span::new(start, self.pos)),
            b'+' => Token::new(TokenKind::Plus, Span::new(start, self.pos)),
            b'*' => Token::new(TokenKind::Star, Span::new(start, self.pos)),
            b'%' => Token::new(TokenKind::Percent, Span::new(start, self.pos)),
            b'?' => Token::new(TokenKind::Question, Span::new(start, self.pos)),

            // Two-char operators
            b'-' => {
                if self.peek() == Some(b'>') {
                    self.advance();
                    Token::new(TokenKind::Arrow, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Minus, Span::new(start, self.pos))
                }
            }
            b'=' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    Token::new(TokenKind::EqEq, Span::new(start, self.pos))
                } else if self.peek() == Some(b'>') {
                    self.advance();
                    Token::new(TokenKind::FatArrow, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Eq, Span::new(start, self.pos))
                }
            }
            b'!' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    Token::new(TokenKind::NotEq, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Bang, Span::new(start, self.pos))
                }
            }
            b'<' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    Token::new(TokenKind::LtEq, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Lt, Span::new(start, self.pos))
                }
            }
            b'>' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    Token::new(TokenKind::GtEq, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Gt, Span::new(start, self.pos))
                }
            }
            b'|' => {
                if self.peek() == Some(b'>') {
                    self.advance();
                    Token::new(TokenKind::PipeArrow, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Pipe, Span::new(start, self.pos))
                }
            }
            b'.' => {
                if self.peek() == Some(b'.') {
                    self.advance();
                    Token::new(TokenKind::DotDot, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Dot, Span::new(start, self.pos))
                }
            }
            b':' => {
                if self.peek() == Some(b':') {
                    self.advance();
                    Token::new(TokenKind::ColonColon, Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Colon, Span::new(start, self.pos))
                }
            }

            // Comments
            b'/' => {
                if self.peek() == Some(b'/') {
                    self.advance(); // consume second /
                    let comment_start = self.pos;
                    while !self.is_at_end() && self.peek() != Some(b'\n') {
                        self.advance();
                    }
                    let text = self.source[comment_start..self.pos].to_string();
                    Token::new(TokenKind::Comment(text), Span::new(start, self.pos))
                } else {
                    Token::new(TokenKind::Slash, Span::new(start, self.pos))
                }
            }

            // String literals
            b'"' => self.lex_string(start),

            // Number literals
            b'0'..=b'9' => self.lex_number(start),

            // Identifiers and keywords
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_ident(start),

            // Error recovery: unknown character
            _ => {
                let ch_str = self.source[start..self.pos].to_string();
                Token::new(
                    TokenKind::Error(format!("unexpected character: '{}'", ch_str)),
                    Span::new(start, self.pos),
                )
            }
        }
    }

    fn lex_string(&mut self, start: usize) -> Token {
        let mut parts: Vec<StringPart> = Vec::new();
        let mut current_literal = String::new();
        let mut has_interpolation = false;

        while !self.is_at_end() {
            match self.bytes[self.pos] {
                b'"' => {
                    self.advance(); // consume closing quote
                    if has_interpolation {
                        if !current_literal.is_empty() {
                            parts.push(StringPart::Literal(current_literal));
                        }
                        return Token::new(
                            TokenKind::InterpolatedString(parts),
                            Span::new(start, self.pos),
                        );
                    } else {
                        return Token::new(
                            TokenKind::StringLit(current_literal),
                            Span::new(start, self.pos),
                        );
                    }
                }
                b'{' => {
                    has_interpolation = true;
                    if !current_literal.is_empty() {
                        parts.push(StringPart::Literal(std::mem::take(&mut current_literal)));
                    }
                    self.advance(); // consume {
                    let expr_start = self.pos;
                    let mut depth = 1;
                    while !self.is_at_end() && depth > 0 {
                        match self.bytes[self.pos] {
                            b'{' => depth += 1,
                            b'}' => depth -= 1,
                            _ => {}
                        }
                        if depth > 0 {
                            self.advance();
                        }
                    }
                    let expr = self.source[expr_start..self.pos].to_string();
                    parts.push(StringPart::Expr(expr));
                    if !self.is_at_end() {
                        self.advance(); // consume closing }
                    }
                }
                b'\\' => {
                    self.advance(); // consume backslash
                    if !self.is_at_end() {
                        let escaped = match self.bytes[self.pos] {
                            b'n' => '\n',
                            b't' => '\t',
                            b'r' => '\r',
                            b'\\' => '\\',
                            b'"' => '"',
                            b'{' => '{',
                            b'}' => '}',
                            _ => {
                                let ch = self.source[self.pos..].chars().next().unwrap_or('?');
                                self.advance();
                                return Token::new(
                                    TokenKind::Error(format!("invalid escape: \\{}", ch)),
                                    Span::new(start, self.pos),
                                );
                            }
                        };
                        current_literal.push(escaped);
                        self.advance();
                    }
                }
                b'\n' => {
                    // Unterminated string
                    return Token::new(
                        TokenKind::Error("unterminated string literal".to_string()),
                        Span::new(start, self.pos),
                    );
                }
                _ => {
                    let ch = self.source[self.pos..].chars().next().unwrap_or('?');
                    current_literal.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }

        Token::new(
            TokenKind::Error("unterminated string literal".to_string()),
            Span::new(start, self.pos),
        )
    }

    fn lex_number(&mut self, start: usize) -> Token {
        while !self.is_at_end() && self.bytes[self.pos].is_ascii_digit() {
            self.advance();
        }

        // Check for float
        if self.peek() == Some(b'.') && self.peek_at(1).is_some_and(|b| b.is_ascii_digit()) {
            self.advance(); // consume .
            while !self.is_at_end() && self.bytes[self.pos].is_ascii_digit() {
                self.advance();
            }
            let text = &self.source[start..self.pos];
            match text.parse::<f64>() {
                Ok(val) => Token::new(TokenKind::FloatLit(val), Span::new(start, self.pos)),
                Err(_) => Token::new(
                    TokenKind::Error(format!("invalid float: {}", text)),
                    Span::new(start, self.pos),
                ),
            }
        } else {
            let text = &self.source[start..self.pos];
            match text.parse::<i64>() {
                Ok(val) => Token::new(TokenKind::IntLit(val), Span::new(start, self.pos)),
                Err(_) => Token::new(
                    TokenKind::Error(format!("invalid integer: {}", text)),
                    Span::new(start, self.pos),
                ),
            }
        }
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while !self.is_at_end()
            && (self.bytes[self.pos].is_ascii_alphanumeric() || self.bytes[self.pos] == b'_')
        {
            self.advance();
        }

        let text = &self.source[start..self.pos];

        if let Some(keyword) = TokenKind::keyword(text) {
            Token::new(keyword, Span::new(start, self.pos))
        } else {
            Token::new(TokenKind::Ident(text.to_string()), Span::new(start, self.pos))
        }
    }

    // === Helpers ===

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\r' => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> u8 {
        let b = self.bytes[self.pos];
        self.pos += 1;
        b
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }
}
