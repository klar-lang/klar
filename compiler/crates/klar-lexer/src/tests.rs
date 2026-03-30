use crate::token::{StringPart, TokenKind};
use crate::Lexer;

fn kinds(source: &str) -> Vec<TokenKind> {
    Lexer::tokenize(source)
        .into_iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof | TokenKind::Comment(_)))
        .map(|t| t.kind)
        .collect()
}

fn kind(source: &str) -> TokenKind {
    let mut k = kinds(source);
    assert_eq!(k.len(), 1, "expected 1 token, got {}: {:?}", k.len(), k);
    k.remove(0)
}

// ============================================================
// 1. Keywords (all 29)
// ============================================================

#[test]
fn keyword_let() { assert_eq!(kind("let"), TokenKind::Let); }
#[test]
fn keyword_mut() { assert_eq!(kind("mut"), TokenKind::Mut); }
#[test]
fn keyword_fn_() { assert_eq!(kind("fn"), TokenKind::Fn); }
#[test]
fn keyword_struct() { assert_eq!(kind("struct"), TokenKind::Struct); }
#[test]
fn keyword_enum() { assert_eq!(kind("enum"), TokenKind::Enum); }
#[test]
fn keyword_trait() { assert_eq!(kind("trait"), TokenKind::Trait); }
#[test]
fn keyword_impl() { assert_eq!(kind("impl"), TokenKind::Impl); }
#[test]
fn keyword_use() { assert_eq!(kind("use"), TokenKind::Use); }
#[test]
fn keyword_if_() { assert_eq!(kind("if"), TokenKind::If); }
#[test]
fn keyword_else_() { assert_eq!(kind("else"), TokenKind::Else); }
#[test]
fn keyword_match() { assert_eq!(kind("match"), TokenKind::Match); }
#[test]
fn keyword_for_() { assert_eq!(kind("for"), TokenKind::For); }
#[test]
fn keyword_in_() { assert_eq!(kind("in"), TokenKind::In); }
#[test]
fn keyword_loop_() { assert_eq!(kind("loop"), TokenKind::Loop); }
#[test]
fn keyword_break_() { assert_eq!(kind("break"), TokenKind::Break); }
#[test]
fn keyword_return_() { assert_eq!(kind("return"), TokenKind::Return); }
#[test]
fn keyword_true_() { assert_eq!(kind("true"), TokenKind::True); }
#[test]
fn keyword_false_() { assert_eq!(kind("false"), TokenKind::False); }
#[test]
fn keyword_and() { assert_eq!(kind("and"), TokenKind::And); }
#[test]
fn keyword_or() { assert_eq!(kind("or"), TokenKind::Or); }
#[test]
fn keyword_not() { assert_eq!(kind("not"), TokenKind::Not); }
#[test]
fn keyword_pub() { assert_eq!(kind("pub"), TokenKind::Pub); }
#[test]
fn keyword_priv() { assert_eq!(kind("priv"), TokenKind::Priv); }
#[test]
fn keyword_test() { assert_eq!(kind("test"), TokenKind::Test); }
#[test]
fn keyword_spawn() { assert_eq!(kind("spawn"), TokenKind::Spawn); }
#[test]
fn keyword_parallel() { assert_eq!(kind("parallel"), TokenKind::Parallel); }
#[test]
fn keyword_catch() { assert_eq!(kind("catch"), TokenKind::Catch); }
#[test]
fn keyword_async_() { assert_eq!(kind("async"), TokenKind::Async); }
#[test]
fn keyword_unsafe_() { assert_eq!(kind("unsafe"), TokenKind::Unsafe); }

// ============================================================
// 2. Identifiers
// ============================================================

#[test]
fn ident_simple() { assert_eq!(kind("foo"), TokenKind::Ident("foo".into())); }
#[test]
fn ident_underscore() { assert_eq!(kind("_bar"), TokenKind::Ident("_bar".into())); }
#[test]
fn ident_with_numbers() { assert_eq!(kind("x42"), TokenKind::Ident("x42".into())); }
#[test]
fn ident_snake_case() { assert_eq!(kind("my_var"), TokenKind::Ident("my_var".into())); }

// ============================================================
// 3. Integer literals
// ============================================================

#[test]
fn int_zero() { assert_eq!(kind("0"), TokenKind::IntLit(0)); }
#[test]
fn int_positive() { assert_eq!(kind("42"), TokenKind::IntLit(42)); }
#[test]
fn int_large() { assert_eq!(kind("1000000"), TokenKind::IntLit(1_000_000)); }

// ============================================================
// 4. Float literals
// ============================================================

#[test]
fn float_simple() { assert_eq!(kind("3.14"), TokenKind::FloatLit(3.14)); }
#[test]
fn float_zero() { assert_eq!(kind("0.0"), TokenKind::FloatLit(0.0)); }
#[test]
fn float_one() { assert_eq!(kind("1.0"), TokenKind::FloatLit(1.0)); }

// ============================================================
// 5. String literals
// ============================================================

#[test]
fn string_empty() { assert_eq!(kind(r#""""#), TokenKind::StringLit("".into())); }
#[test]
fn string_simple() { assert_eq!(kind(r#""hello""#), TokenKind::StringLit("hello".into())); }
#[test]
fn string_escape_newline() { assert_eq!(kind(r#""a\nb""#), TokenKind::StringLit("a\nb".into())); }
#[test]
fn string_escape_tab() { assert_eq!(kind(r#""a\tb""#), TokenKind::StringLit("a\tb".into())); }
#[test]
fn string_escape_quote() { assert_eq!(kind(r#""say \"hi\"""#), TokenKind::StringLit("say \"hi\"".into())); }
#[test]
fn string_escape_backslash() { assert_eq!(kind(r#""a\\b""#), TokenKind::StringLit("a\\b".into())); }
#[test]
fn string_escape_brace() { assert_eq!(kind(r#""a\{b""#), TokenKind::StringLit("a{b".into())); }

// ============================================================
// 6. Interpolated strings
// ============================================================

#[test]
fn interpolated_simple() {
    assert_eq!(
        kind(r#""hello {name}""#),
        TokenKind::InterpolatedString(vec![
            StringPart::Literal("hello ".into()),
            StringPart::Expr("name".into()),
        ])
    );
}

#[test]
fn interpolated_multiple() {
    assert_eq!(
        kind(r#""{a} and {b}""#),
        TokenKind::InterpolatedString(vec![
            StringPart::Expr("a".into()),
            StringPart::Literal(" and ".into()),
            StringPart::Expr("b".into()),
        ])
    );
}

#[test]
fn interpolated_expr() {
    assert_eq!(
        kind(r#""total: {x + y}""#),
        TokenKind::InterpolatedString(vec![
            StringPart::Literal("total: ".into()),
            StringPart::Expr("x + y".into()),
        ])
    );
}

#[test]
fn interpolated_trailing_text() {
    assert_eq!(
        kind(r#""{name}!""#),
        TokenKind::InterpolatedString(vec![
            StringPart::Expr("name".into()),
            StringPart::Literal("!".into()),
        ])
    );
}

// ============================================================
// 7. Operators
// ============================================================

#[test]
fn op_plus() { assert_eq!(kind("+"), TokenKind::Plus); }
#[test]
fn op_minus() { assert_eq!(kind("-"), TokenKind::Minus); }
#[test]
fn op_star() { assert_eq!(kind("*"), TokenKind::Star); }
#[test]
fn op_slash() { assert_eq!(kind("/"), TokenKind::Slash); }
#[test]
fn op_percent() { assert_eq!(kind("%"), TokenKind::Percent); }
#[test]
fn op_eq() { assert_eq!(kind("="), TokenKind::Eq); }
#[test]
fn op_eq_eq() { assert_eq!(kind("=="), TokenKind::EqEq); }
#[test]
fn op_not_eq() { assert_eq!(kind("!="), TokenKind::NotEq); }
#[test]
fn op_lt() { assert_eq!(kind("<"), TokenKind::Lt); }
#[test]
fn op_gt() { assert_eq!(kind(">"), TokenKind::Gt); }
#[test]
fn op_lt_eq() { assert_eq!(kind("<="), TokenKind::LtEq); }
#[test]
fn op_gt_eq() { assert_eq!(kind(">="), TokenKind::GtEq); }
#[test]
fn op_arrow() { assert_eq!(kind("->"), TokenKind::Arrow); }
#[test]
fn op_fat_arrow() { assert_eq!(kind("=>"), TokenKind::FatArrow); }
#[test]
fn op_pipe() { assert_eq!(kind("|"), TokenKind::Pipe); }
#[test]
fn op_pipe_arrow() { assert_eq!(kind("|>"), TokenKind::PipeArrow); }
#[test]
fn op_question() { assert_eq!(kind("?"), TokenKind::Question); }
#[test]
fn op_bang() { assert_eq!(kind("!"), TokenKind::Bang); }
#[test]
fn op_dot() { assert_eq!(kind("."), TokenKind::Dot); }
#[test]
fn op_dot_dot() { assert_eq!(kind(".."), TokenKind::DotDot); }
#[test]
fn op_colon() { assert_eq!(kind(":"), TokenKind::Colon); }
#[test]
fn op_colon_colon() { assert_eq!(kind("::"), TokenKind::ColonColon); }
#[test]
fn op_at() { assert_eq!(kind("@"), TokenKind::At); }
#[test]
fn op_ampersand() { assert_eq!(kind("&"), TokenKind::Ampersand); }

// ============================================================
// 8. Delimiters
// ============================================================

#[test]
fn delim_parens() {
    let k = kinds("()");
    assert_eq!(k, vec![TokenKind::LParen, TokenKind::RParen]);
}
#[test]
fn delim_braces() {
    let k = kinds("{}");
    assert_eq!(k, vec![TokenKind::LBrace, TokenKind::RBrace]);
}
#[test]
fn delim_brackets() {
    let k = kinds("[]");
    assert_eq!(k, vec![TokenKind::LBracket, TokenKind::RBracket]);
}

// ============================================================
// 9. Comments
// ============================================================

#[test]
fn comment_line() {
    let tokens = Lexer::tokenize("// this is a comment");
    assert!(matches!(&tokens[0].kind, TokenKind::Comment(c) if c.contains("this is a comment")));
}

#[test]
fn comment_preserves_next_token() {
    let k = kinds("// comment\n42");
    assert_eq!(k, vec![TokenKind::IntLit(42)]);
}

// ============================================================
// 10. Whitespace handling
// ============================================================

#[test]
fn whitespace_spaces() {
    let k = kinds("a  b");
    assert_eq!(k, vec![TokenKind::Ident("a".into()), TokenKind::Ident("b".into())]);
}

#[test]
fn whitespace_tabs() {
    let k = kinds("a\tb");
    assert_eq!(k, vec![TokenKind::Ident("a".into()), TokenKind::Ident("b".into())]);
}

// ============================================================
// 11. Error recovery
// ============================================================

#[test]
fn error_unknown_char() {
    let k = kinds("$");
    assert!(matches!(&k[0], TokenKind::Error(_)));
}

#[test]
fn error_unterminated_string() {
    let tokens = Lexer::tokenize("\"hello");
    assert!(tokens.iter().any(|t| matches!(&t.kind, TokenKind::Error(msg) if msg.contains("unterminated"))));
}

#[test]
fn error_recovery_continues() {
    // After an error token, lexer should continue producing valid tokens
    let k = kinds("$ foo");
    assert!(matches!(&k[0], TokenKind::Error(_)));
    assert_eq!(k[1], TokenKind::Ident("foo".into()));
}

// ============================================================
// 12. Spans
// ============================================================

#[test]
fn span_keyword() {
    let tokens = Lexer::tokenize("let");
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 3);
}

#[test]
fn span_after_whitespace() {
    let tokens = Lexer::tokenize("  foo");
    assert_eq!(tokens[0].span.start, 2);
    assert_eq!(tokens[0].span.end, 5);
}

// ============================================================
// 13. Complex expressions
// ============================================================

#[test]
fn function_signature() {
    let k = kinds("fn add(a: Int, b: Int) -> Int");
    assert_eq!(k, vec![
        TokenKind::Fn,
        TokenKind::Ident("add".into()),
        TokenKind::LParen,
        TokenKind::Ident("a".into()),
        TokenKind::Colon,
        TokenKind::Ident("Int".into()),
        TokenKind::Comma,
        TokenKind::Ident("b".into()),
        TokenKind::Colon,
        TokenKind::Ident("Int".into()),
        TokenKind::RParen,
        TokenKind::Arrow,
        TokenKind::Ident("Int".into()),
    ]);
}

#[test]
fn result_type() {
    let k = kinds("fn read(p: String) -> Config ! ParseError");
    assert_eq!(k, vec![
        TokenKind::Fn,
        TokenKind::Ident("read".into()),
        TokenKind::LParen,
        TokenKind::Ident("p".into()),
        TokenKind::Colon,
        TokenKind::Ident("String".into()),
        TokenKind::RParen,
        TokenKind::Arrow,
        TokenKind::Ident("Config".into()),
        TokenKind::Bang,
        TokenKind::Ident("ParseError".into()),
    ]);
}

#[test]
fn option_type() {
    let k = kinds("fn find(id: Id) -> User?");
    assert!(k.contains(&TokenKind::Question));
}

#[test]
fn pipe_chain() {
    let k = kinds("x |> foo |> bar");
    assert_eq!(k, vec![
        TokenKind::Ident("x".into()),
        TokenKind::PipeArrow,
        TokenKind::Ident("foo".into()),
        TokenKind::PipeArrow,
        TokenKind::Ident("bar".into()),
    ]);
}

#[test]
fn struct_declaration() {
    let k = kinds("struct User { name: String }");
    assert_eq!(k[0], TokenKind::Struct);
    assert_eq!(k[1], TokenKind::Ident("User".into()));
    assert_eq!(k[2], TokenKind::LBrace);
}

#[test]
fn schema_annotation() {
    let k = kinds("@schema");
    assert_eq!(k, vec![TokenKind::At, TokenKind::Ident("schema".into())]);
}

#[test]
fn match_expression() {
    let k = kinds("match shape { Circle(r) => r }");
    assert_eq!(k[0], TokenKind::Match);
    assert!(k.contains(&TokenKind::FatArrow));
}

#[test]
fn closure_syntax() {
    let k = kinds("|x| x + 1");
    assert_eq!(k[0], TokenKind::Pipe);
    assert_eq!(k[1], TokenKind::Ident("x".into()));
    assert_eq!(k[2], TokenKind::Pipe);
}

#[test]
fn for_loop() {
    let k = kinds("for item in list { }");
    assert_eq!(k[0], TokenKind::For);
    assert_eq!(k[1], TokenKind::Ident("item".into()));
    assert_eq!(k[2], TokenKind::In);
}

#[test]
fn error_propagation() {
    let k = kinds("db.find(id)?");
    assert!(k.contains(&TokenKind::Question));
}

#[test]
fn catch_handling() {
    let k = kinds("expr catch err { }");
    assert!(k.contains(&TokenKind::Catch));
}

// ============================================================
// 14. URL shortener example (from PRD appendix)
// ============================================================

#[test]
fn url_shortener_snippet() {
    let source = r#"@schema
struct ShortenRequest {
    url: String
}

fn shorten(req: Request, db: Pool) -> Response ! AppError {
    let input = req.json[ShortenRequest]()?
    let code = uuid()[0..8]
    Response.json(url, status: 201)
}"#;
    let tokens = Lexer::tokenize(source);
    // Should not contain any Error tokens
    let errors: Vec<_> = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Error(_))).collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    // Should start with @schema
    let k = kinds(source);
    assert_eq!(k[0], TokenKind::At);
    assert_eq!(k[1], TokenKind::Ident("schema".into()));
    assert_eq!(k[2], TokenKind::Struct);
}

// ============================================================
// 15. Edge cases
// ============================================================

#[test]
fn empty_source() {
    let tokens = Lexer::tokenize("");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn only_whitespace() {
    let tokens = Lexer::tokenize("   \t  ");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn consecutive_operators() {
    let k = kinds("!= == >= <=");
    assert_eq!(k, vec![TokenKind::NotEq, TokenKind::EqEq, TokenKind::GtEq, TokenKind::LtEq]);
}

#[test]
fn keyword_prefix_is_ident() {
    // "letter" starts with "let" but is an identifier, not a keyword
    assert_eq!(kind("letter"), TokenKind::Ident("letter".into()));
    assert_eq!(kind("format"), TokenKind::Ident("format".into()));
    assert_eq!(kind("forEach"), TokenKind::Ident("forEach".into()));
}

#[test]
fn eof_token_present() {
    let tokens = Lexer::tokenize("42");
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}
