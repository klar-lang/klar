use crate::TypeChecker;

fn check_ok(source: &str) {
    let program = klar_parser::parse(source).unwrap_or_else(|e| panic!("parse error: {:?}", e));
    let mut checker = TypeChecker::new();
    checker.check_program(&program).unwrap_or_else(|errs| {
        panic!("type errors: {:?}", errs);
    });
}

fn check_err(source: &str) -> Vec<crate::TypeError> {
    let program = klar_parser::parse(source).unwrap_or_else(|e| panic!("parse error: {:?}", e));
    let mut checker = TypeChecker::new();
    checker.check_program(&program).unwrap_err()
}

// ============================================================
// 1. Literal inference
// ============================================================

#[test]
fn infer_int() { check_ok("fn f() { let x = 42 }"); }
#[test]
fn infer_float() { check_ok("fn f() { let x = 3.14 }"); }
#[test]
fn infer_string() { check_ok(r#"fn f() { let x = "hello" }"#); }
#[test]
fn infer_bool() { check_ok("fn f() { let x = true }"); }
#[test]
fn infer_interpolated() { check_ok(r#"fn f() { let x = "hi {name}" }"#); }

// ============================================================
// 2. Arithmetic
// ============================================================

#[test]
fn arithmetic_int() { check_ok("fn f() { let x = 1 + 2 * 3 }"); }
#[test]
fn arithmetic_float() { check_ok("fn f() { let x = 1.0 + 2.0 }"); }

#[test]
fn arithmetic_type_mismatch() {
    let errs = check_err("fn f() { let x: Int = 1 + 2.0 }");
    assert!(!errs.is_empty());
}

#[test]
fn arithmetic_bool_rejected() {
    let errs = check_err("fn f() { let x = true + false }");
    assert!(!errs.is_empty());
}

// ============================================================
// 3. Comparison
// ============================================================

#[test]
fn comparison_returns_bool() { check_ok("fn f() { let x: Bool = 1 == 2 }"); }
#[test]
fn comparison_lt() { check_ok("fn f() { let x = 1 < 2 }"); }

// ============================================================
// 4. Boolean logic
// ============================================================

#[test]
fn bool_and() { check_ok("fn f() { let x = true and false }"); }
#[test]
fn bool_or() { check_ok("fn f() { let x = true or false }"); }
#[test]
fn bool_not() { check_ok("fn f() { let x = not true }"); }

#[test]
fn bool_and_requires_bool() {
    let errs = check_err("fn f() { let x = 1 and 2 }");
    assert!(!errs.is_empty());
}

// ============================================================
// 5. Let with explicit type
// ============================================================

#[test]
fn let_explicit_ok() { check_ok("fn f() { let x: Int = 42 }"); }
#[test]
fn let_explicit_string() { check_ok(r#"fn f() { let x: String = "hi" }"#); }

#[test]
fn let_type_mismatch() {
    let errs = check_err(r#"fn f() { let x: Int = "hello" }"#);
    assert!(!errs.is_empty());
    assert!(errs[0].message.contains("type mismatch"));
}

#[test]
fn let_bool_mismatch() {
    let errs = check_err("fn f() { let x: Bool = 42 }");
    assert!(!errs.is_empty());
}

// ============================================================
// 6. Function calls
// ============================================================

#[test]
fn call_known_function() {
    check_ok("fn add(a: Int, b: Int) -> Int { a + b }\nfn f() { let x = add(1, 2) }");
}

#[test]
fn call_wrong_arg_count() {
    let errs = check_err("fn add(a: Int, b: Int) -> Int { a + b }\nfn f() { add(1) }");
    assert!(!errs.is_empty());
    assert!(errs[0].message.contains("expected 2 arguments"));
}

#[test]
fn call_wrong_arg_type() {
    let errs = check_err(r#"fn greet(name: String) -> String { name }
fn f() { greet(42) }"#);
    assert!(!errs.is_empty());
    assert!(errs[0].message.contains("type mismatch"));
}

// ============================================================
// 7. Struct type checking
// ============================================================

#[test]
fn struct_field_access() {
    check_ok("struct User { name: String\n  age: Int }\nfn f() { let u = User { name: \"a\", age: 1 } }");
}

#[test]
fn struct_wrong_field_type() {
    let errs = check_err("struct User { name: String\n  age: Int }\nfn f() { let u = User { name: 42, age: 1 } }");
    assert!(!errs.is_empty());
}

// ============================================================
// 8. List inference
// ============================================================

#[test]
fn list_homogeneous() { check_ok("fn f() { let xs = [1, 2, 3] }"); }

#[test]
fn list_heterogeneous_error() {
    let errs = check_err(r#"fn f() { let xs = [1, "two", 3] }"#);
    assert!(!errs.is_empty());
}

// ============================================================
// 9. If/else
// ============================================================

#[test]
fn if_requires_bool() {
    let errs = check_err("fn f() { if 42 { 1 } }");
    assert!(!errs.is_empty());
}

#[test]
fn if_bool_ok() { check_ok("fn f() { if true { 1 } }"); }

// ============================================================
// 10. For loop
// ============================================================

#[test]
fn for_over_list() { check_ok("fn f() { for x in [1, 2, 3] { x + 1 } }"); }

// ============================================================
// 11. Closure
// ============================================================

#[test]
fn closure_typed() { check_ok("fn f() { let double = |x: Int| x * 2 }"); }

// ============================================================
// 12. Try (error propagation)
// ============================================================

#[test]
fn try_on_expr() { check_ok("fn f() { let x = 42 }"); }

// ============================================================
// 13. Negation
// ============================================================

#[test]
fn negate_int() { check_ok("fn f() { let x = -42 }"); }

#[test]
fn negate_bool_error() {
    let errs = check_err("fn f() { let x = -true }");
    assert!(!errs.is_empty());
}

// ============================================================
// 14. Multiple items
// ============================================================

#[test]
fn multiple_functions() {
    check_ok("fn a() -> Int { 1 }\nfn b() -> Int { 2 }\nfn c() { let x = a() }");
}

// ============================================================
// 15. Use declarations (no error)
// ============================================================

#[test]
fn use_decl() { check_ok("use std.json\nfn f() { 42 }"); }

// ============================================================
// 16. Enum declaration
// ============================================================

#[test]
fn enum_decl() { check_ok("enum Color { Red\n  Green\n  Blue }\nfn f() { 42 }"); }

// ============================================================
// 17. Test blocks
// ============================================================

#[test]
fn test_block() { check_ok("fn f() { 1 }\ntest f { 1 + 1 }"); }

// ============================================================
// 18. Complex programs
// ============================================================

#[test]
fn hello_api() {
    check_ok(r#"use std.http.{Request, Response}

struct Greeting {
    message: String
}

fn hello(req: Request) -> Response {
    let name = "world"
    let g = Greeting { message: "Hello" }
    42
}"#);
}

#[test]
fn trait_and_impl() {
    check_ok(r#"trait Printable {
    fn to_string(self) -> String
}

struct User {
    name: String
}

impl Printable for User {
    fn to_string(self) -> String { self.name }
}"#);
}

// ============================================================
// 19. Variable scope
// ============================================================

#[test]
fn undefined_variable() {
    let errs = check_err("fn f() { x + 1 }");
    assert!(!errs.is_empty());
    assert!(errs[0].message.contains("undefined"));
}

#[test]
fn variable_in_scope() {
    check_ok("fn f() { let x = 10\n  let y = x + 1 }");
}

// ============================================================
// 20. String concat
// ============================================================

#[test]
fn string_concat() {
    check_ok(r#"fn f() { let x = "hello" + " world" }"#);
}
