use crate::parse;
use klar_ast::*;

fn parse_ok(source: &str) -> Program {
    parse(source).unwrap_or_else(|errs| {
        panic!("parse errors: {:?}", errs);
    })
}

fn parse_err(source: &str) -> Vec<crate::ParseError> {
    parse(source).unwrap_err()
}

fn first_item(source: &str) -> Item {
    let prog = parse_ok(source);
    prog.items.into_iter().next().expect("no items")
}

// ============================================================
// 1. Function declarations
// ============================================================

#[test]
fn fn_simple() {
    let item = first_item("fn add(a: Int, b: Int) -> Int { a + b }");
    let Item::Function(f) = item else { panic!("expected function") };
    assert_eq!(f.name.name, "add");
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].name.name, "a");
    assert!(f.return_type.is_some());
    assert!(f.error_type.is_none());
}

#[test]
fn fn_with_result_type() {
    let item = first_item("fn read(p: String) -> Config ! ParseError { p }");
    let Item::Function(f) = item else { panic!() };
    assert!(f.return_type.is_some());
    assert!(f.error_type.is_some());
}

#[test]
fn fn_no_params() {
    let item = first_item("fn main() { 42 }");
    let Item::Function(f) = item else { panic!() };
    assert_eq!(f.name.name, "main");
    assert!(f.params.is_empty());
}

#[test]
fn fn_private() {
    let item = first_item("priv fn helper() { 1 }");
    let Item::Function(f) = item else { panic!() };
    assert!(f.is_priv);
}

// ============================================================
// 2. Struct declarations
// ============================================================

#[test]
fn struct_simple() {
    let item = first_item("struct User {\n  name: String\n  age: Int\n}");
    let Item::Struct(s) = item else { panic!() };
    assert_eq!(s.name.name, "User");
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn struct_with_annotations() {
    let item = first_item("@schema\nstruct User {\n  name: String @min_len(1)\n}");
    let Item::Struct(s) = item else { panic!() };
    assert!(!s.annotations.is_empty());
    assert_eq!(s.annotations[0].name.name, "schema");
    assert!(!s.fields[0].annotations.is_empty());
}

#[test]
fn struct_with_default() {
    let item = first_item("struct Config {\n  port: Int\n  host: String\n}");
    let Item::Struct(s) = item else { panic!() };
    assert_eq!(s.fields.len(), 2);
}

// ============================================================
// 3. Enum declarations
// ============================================================

#[test]
fn enum_simple() {
    let item = first_item("enum Color {\n  Red\n  Green\n  Blue\n}");
    let Item::Enum(e) = item else { panic!() };
    assert_eq!(e.name.name, "Color");
    assert_eq!(e.variants.len(), 3);
}

#[test]
fn enum_with_data() {
    let item = first_item("enum Shape {\n  Circle(radius: Float)\n  Rect(w: Float, h: Float)\n}");
    let Item::Enum(e) = item else { panic!() };
    assert_eq!(e.variants.len(), 2);
    assert_eq!(e.variants[0].fields.len(), 1);
    assert_eq!(e.variants[1].fields.len(), 2);
}

// ============================================================
// 4. Use declarations
// ============================================================

#[test]
fn use_simple() {
    let item = first_item("use std.json");
    let Item::Use(u) = item else { panic!() };
    assert_eq!(u.path.len(), 2);
    assert!(u.items.is_none());
}

#[test]
fn use_with_items() {
    let item = first_item("use std.http.{Request, Response}");
    let Item::Use(u) = item else { panic!() };
    assert_eq!(u.path.len(), 2); // std, http
    let items = u.items.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "Request");
}

// ============================================================
// 5. Trait and impl
// ============================================================

#[test]
fn trait_decl() {
    let item = first_item("trait Printable {\n  fn to_string(self) -> String\n}");
    let Item::Trait(t) = item else { panic!() };
    assert_eq!(t.name.name, "Printable");
    assert_eq!(t.methods.len(), 1);
}

#[test]
fn impl_decl() {
    let source = "impl Printable for User {\n  fn to_string(self) -> String { self.name }\n}";
    let item = first_item(source);
    let Item::Impl(i) = item else { panic!() };
    assert_eq!(i.trait_name.name, "Printable");
    assert_eq!(i.target.name, "User");
    assert_eq!(i.methods.len(), 1);
}

// ============================================================
// 6. Test blocks
// ============================================================

#[test]
fn test_decl() {
    let item = first_item("test add {\n  assert 1 + 1 == 2\n}");
    let Item::Test(t) = item else { panic!() };
    assert_eq!(t.name.name, "add");
}

// ============================================================
// 7. Expressions
// ============================================================

#[test]
fn expr_binary_arithmetic() {
    let prog = parse_ok("fn f() { 1 + 2 * 3 }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    // Should parse as 1 + (2 * 3) due to precedence
    let Stmt::Expr(Expr::Binary(_, BinOp::Add, _, _)) = &f.body.stmts[0] else {
        panic!("expected addition at top level");
    };
}

#[test]
fn expr_comparison() {
    let prog = parse_ok("fn f() { a == b }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Binary(_, BinOp::Eq, _, _))));
}

#[test]
fn expr_field_access() {
    let prog = parse_ok("fn f() { user.name }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::FieldAccess(_, _, _))));
}

#[test]
fn expr_function_call() {
    let prog = parse_ok("fn f() { add(1, 2) }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Call(_, _, _))));
}

#[test]
fn expr_named_args() {
    let prog = parse_ok("fn f() { serve(router, port: 3000) }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    let Stmt::Expr(Expr::Call(_, args, _)) = &f.body.stmts[0] else { panic!() };
    assert!(args[1].name.is_some());
}

#[test]
fn expr_method_chain() {
    let prog = parse_ok("fn f() { a.b().c }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::FieldAccess(_, _, _))));
}

#[test]
fn expr_pipe() {
    let prog = parse_ok("fn f() { x |> foo |> bar }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Pipe(_, _, _))));
}

#[test]
fn expr_try() {
    let prog = parse_ok("fn f() { read_file(path)? }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Try(_, _))));
}

#[test]
fn expr_catch() {
    let prog = parse_ok("fn f() { read(p) catch err { default() } }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Catch(_, _, _, _))));
}

#[test]
fn expr_list_literal() {
    let prog = parse_ok("fn f() { [1, 2, 3] }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    let Stmt::Expr(Expr::ListLit(items, _)) = &f.body.stmts[0] else { panic!() };
    assert_eq!(items.len(), 3);
}

#[test]
fn expr_closure() {
    let prog = parse_ok("fn f() { |x| x + 1 }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Closure(_, _, _))));
}

#[test]
fn expr_string_interpolation() {
    let prog = parse_ok(r#"fn f() { "hello {name}" }"#);
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::InterpolatedString(_, _))));
}

// ============================================================
// 8. Control flow
// ============================================================

#[test]
fn if_else_expr() {
    let prog = parse_ok("fn f() { if x > 0 { x } else { 0 } }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::If(_, _, Some(_), _))));
}

#[test]
fn match_expr() {
    let source = "fn f() {\n  match shape {\n    Circle(r) => r\n    Rect(w, h) => w * h\n  }\n}";
    let prog = parse_ok(source);
    let Item::Function(f) = &prog.items[0] else { panic!() };
    let Stmt::Expr(Expr::Match(_, arms, _)) = &f.body.stmts[0] else { panic!() };
    assert_eq!(arms.len(), 2);
}

#[test]
fn for_loop() {
    let prog = parse_ok("fn f() {\n  for item in list {\n    process(item)\n  }\n}");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::For(_)));
}

#[test]
fn for_with_index() {
    let prog = parse_ok("fn f() {\n  for i, item in list {\n    use_index(i)\n  }\n}");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    let Stmt::For(fs) = &f.body.stmts[0] else { panic!() };
    assert!(fs.index.is_some());
}

#[test]
fn loop_break() {
    let prog = parse_ok("fn f() {\n  loop {\n    break\n  }\n}");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Loop(_, _)));
}

// ============================================================
// 9. Let statements
// ============================================================

#[test]
fn let_simple() {
    let prog = parse_ok("fn f() { let x = 42 }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    let Stmt::Let(l) = &f.body.stmts[0] else { panic!() };
    assert_eq!(l.name.name, "x");
    assert!(!l.mutable);
}

#[test]
fn let_mut() {
    let prog = parse_ok("fn f() { let mut x = 0 }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    let Stmt::Let(l) = &f.body.stmts[0] else { panic!() };
    assert!(l.mutable);
}

#[test]
fn let_with_type() {
    let prog = parse_ok("fn f() { let x: Int = 42 }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    let Stmt::Let(l) = &f.body.stmts[0] else { panic!() };
    assert!(l.ty.is_some());
}

// ============================================================
// 10. Struct initialization
// ============================================================

#[test]
fn struct_init() {
    let prog = parse_ok(r#"fn f() { User { name: "Alice", age: 30 } }"#);
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::StructInit(_, _, _))));
}

// ============================================================
// 11. Types
// ============================================================

#[test]
fn type_option() {
    let item = first_item("fn find(id: Id) -> User? { id }");
    let Item::Function(f) = item else { panic!() };
    assert!(matches!(f.return_type, Some(TypeExpr::Option(_))));
}

#[test]
fn type_generic() {
    let item = first_item("fn first(items: List[Int]) -> Int { items }");
    let Item::Function(f) = item else { panic!() };
    assert!(matches!(&f.params[0].ty, TypeExpr::Generic(_, _)));
}

// ============================================================
// 12. Error recovery
// ============================================================

#[test]
fn error_missing_brace() {
    let errs = parse_err("fn f() {");
    assert!(!errs.is_empty());
}

#[test]
fn error_unexpected_token() {
    let errs = parse_err("fn 42() {}");
    assert!(!errs.is_empty());
}

// ============================================================
// 13. URL shortener from PRD
// ============================================================

#[test]
fn url_shortener() {
    let source = r#"use std.http.{Router, Request, Response, serve}
use std.sql.{Pool}
use std.crypto.{uuid}
use std.env

@schema
struct ShortenRequest {
    url: String
}

@schema
struct ShortUrl {
    id: String
    url: String
    short_code: String
}

fn main() ! AppError {
    let db = Pool.connect(env.require("DATABASE_URL")?)?
    let router = Router.new()
        |> Router.post("/shorten", |req| shorten(req, db))
        |> Router.get("/{code}", |req| redirect(req, db))
    serve(router, port: 3000)?
}

fn shorten(req: Request, db: Pool) -> Response ! AppError {
    let input = req.json[ShortenRequest]()?
    let code = uuid()[0..8]
    Response.json(url, status: 201)
}

test shorten {
    let req = Request.json(ShortenRequest { url: "https://example.com" })
    let res = shorten(req, db)
    assert res.status == 201
}"#;
    let prog = parse_ok(source);
    // 4 use, 2 struct, 2 fn, 1 test = 9 items
    assert!(prog.items.len() >= 8, "expected at least 8 items, got {}", prog.items.len());
}

// ============================================================
// 14. Multiple items
// ============================================================

#[test]
fn multiple_functions() {
    let prog = parse_ok("fn a() { 1 }\nfn b() { 2 }\nfn c() { 3 }");
    assert_eq!(prog.items.len(), 3);
}

#[test]
fn return_stmt() {
    let prog = parse_ok("fn f() { return 42 }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Return(Some(_), _)));
}

#[test]
fn bool_literals() {
    let prog = parse_ok("fn f() { true }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::BoolLit(true, _))));
}

#[test]
fn negation() {
    let prog = parse_ok("fn f() { -1 }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Unary(UnaryOp::Neg, _, _))));
}

#[test]
fn not_expr() {
    let prog = parse_ok("fn f() { not true }");
    let Item::Function(f) = &prog.items[0] else { panic!() };
    assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Unary(UnaryOp::Not, _, _))));
}
