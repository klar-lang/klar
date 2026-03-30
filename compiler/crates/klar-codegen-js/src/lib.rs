use klar_ast::*;

/// Generate JavaScript source from a Klar AST.
pub fn generate(program: &Program) -> String {
    let mut g = JsGen::new();
    g.emit_program(program);
    g.output
}

struct JsGen {
    output: String,
    indent: usize,
}

impl JsGen {
    fn new() -> Self {
        Self { output: String::new(), indent: 0 }
    }

    fn emit_program(&mut self, program: &Program) {
        // Emit runtime helpers first
        self.emit_runtime();
        self.line("");

        for item in &program.items {
            self.emit_item(item);
            self.line("");
        }

        // Auto-call main if it exists
        if program.items.iter().any(|i| matches!(i, Item::Function(f) if f.name.name == "main")) {
            self.line("// Entry point");
            self.line("main();");
        }
    }

    fn emit_runtime(&mut self) {
        self.output.push_str(include_str!("runtime/stdlib.js"));
        self.line("");
    }

    // ============================================================
    // Items
    // ============================================================

    fn emit_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.emit_fn(f),
            Item::Struct(s) => self.emit_struct(s),
            Item::Enum(e) => self.emit_enum(e),
            Item::Test(t) => self.emit_test(t),
            Item::Use(u) => self.emit_use(u),
            Item::Trait(_) | Item::Impl(_) => {
                // Traits and impls are handled structurally (no JS output needed for now)
            }
        }
    }

    fn emit_fn(&mut self, f: &FnDecl) {
        let params: Vec<&str> = f.params.iter().map(|p| p.name.name.as_str()).collect();
        let has_return = f.return_type.is_some();
        self.line(&format!("function {}({}) {{", f.name.name, params.join(", ")));
        self.indent += 1;
        self.emit_stmts(&f.body.stmts, has_return);
        self.indent -= 1;
        self.line("}");
    }

    fn emit_struct(&mut self, s: &StructDecl) {
        let is_schema = s.annotations.iter().any(|a| a.name.name == "schema");
        let fields: Vec<&str> = s.fields.iter().map(|f| f.name.name.as_str()).collect();
        self.line(&format!("class {} {{", s.name.name));
        self.indent += 1;
        self.line(&format!("constructor({}) {{", fields.join(", ")));
        self.indent += 1;
        for field in &s.fields {
            self.line(&format!("this.{0} = {0};", field.name.name));
        }
        self.indent -= 1;
        self.line("}");

        if is_schema {
            // Generate toJSON
            self.line("toJSON() {");
            self.indent += 1;
            self.line(&format!("return {{ {} }};",
                fields.iter().map(|f| format!("{0}: this.{0}", f)).collect::<Vec<_>>().join(", ")));
            self.indent -= 1;
            self.line("}");

            // Generate static fromJSON with validation
            self.line(&format!("static fromJSON(data) {{"));
            self.indent += 1;
            self.line("const errors = [];");
            for field in &s.fields {
                let field_name = &field.name.name;
                // Check required
                self.line(&format!(
                    "if (data.{0} === undefined || data.{0} === null) errors.push({{ field: '{0}', error: 'required' }});",
                    field_name
                ));

                // Generate validation for field annotations
                for ann in &field.annotations {
                    match ann.name.name.as_str() {
                        "min_len" => {
                            if let Some(arg) = ann.args.first() {
                                let val = self.expr_to_js(arg);
                                self.line(&format!(
                                    "if (data.{0} && data.{0}.length < {1}) errors.push({{ field: '{0}', error: 'min_len({1})' }});",
                                    field_name, val
                                ));
                            }
                        }
                        "max_len" => {
                            if let Some(arg) = ann.args.first() {
                                let val = self.expr_to_js(arg);
                                self.line(&format!(
                                    "if (data.{0} && data.{0}.length > {1}) errors.push({{ field: '{0}', error: 'max_len({1})' }});",
                                    field_name, val
                                ));
                            }
                        }
                        "range" => {
                            if ann.args.len() >= 2 {
                                let min = self.expr_to_js(&ann.args[0]);
                                let max = self.expr_to_js(&ann.args[1]);
                                self.line(&format!(
                                    "if (data.{0} !== undefined && (data.{0} < {1} || data.{0} > {2})) errors.push({{ field: '{0}', error: 'range({1}, {2})' }});",
                                    field_name, min, max
                                ));
                            }
                        }
                        "format" => {
                            if let Some(arg) = ann.args.first() {
                                let fmt = self.expr_to_js(arg);
                                self.line(&format!(
                                    "if (data.{0} && !{1}.test(data.{0})) errors.push({{ field: '{0}', error: 'format' }});",
                                    field_name, fmt
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            self.line("if (errors.length > 0) return { ok: false, errors };");
            self.line(&format!(
                "return {{ ok: true, value: new {}({}) }};",
                s.name.name,
                fields.iter().map(|f| format!("data.{}", f)).collect::<Vec<_>>().join(", ")
            ));
            self.indent -= 1;
            self.line("}");

            // Generate static schema() for introspection
            self.line("static schema() {");
            self.indent += 1;
            self.line(&format!("return {{"));
            self.indent += 1;
            self.line(&format!("name: '{}',", s.name.name));
            self.line("fields: {");
            self.indent += 1;
            for field in &s.fields {
                let ty_name = format_type_name(&field.ty);
                let validations: Vec<String> = field.annotations.iter().map(|a| {
                    let args: Vec<String> = a.args.iter().map(|arg| {
                        let mut g = JsGen::new();
                        g.expr_to_js(arg)
                    }).collect();
                    if args.is_empty() {
                        format!("'{}'", a.name.name)
                    } else {
                        format!("'{}({})'", a.name.name, args.join(", "))
                    }
                }).collect();
                self.line(&format!("{}: {{ type: '{}', validations: [{}] }},",
                    field.name.name, ty_name, validations.join(", ")));
            }
            self.indent -= 1;
            self.line("},");
            self.indent -= 1;
            self.line("};");
            self.indent -= 1;
            self.line("}");
        }

        self.indent -= 1;
        self.line("}");
    }

    fn emit_enum(&mut self, e: &EnumDecl) {
        self.line(&format!("const {} = Object.freeze({{", e.name.name));
        self.indent += 1;
        for variant in &e.variants {
            if variant.fields.is_empty() {
                self.line(&format!("{}: Symbol('{}'),", variant.name.name, variant.name.name));
            } else {
                let params: Vec<&str> = variant.fields.iter().map(|f| f.name.name.as_str()).collect();
                self.line(&format!(
                    "{}: ({}) => ({{ _tag: '{}', {} }}),",
                    variant.name.name,
                    params.join(", "),
                    variant.name.name,
                    params.join(", "),
                ));
            }
        }
        self.indent -= 1;
        self.line("});");
    }

    fn emit_use(&mut self, u: &UseDecl) {
        let path: Vec<&str> = u.path.iter().map(|id| id.name.as_str()).collect();

        // std.* modules are already in the runtime — only emit destructured imports
        if path.first() == Some(&"std") {
            if let Some(items) = &u.items {
                let js_path = path.join(".");
                for item in items {
                    self.line(&format!("const {} = {}.{};", item.name, js_path, item.name));
                }
            }
            // `use std.json` — already available as `json` from runtime, skip
            return;
        }

        // Non-std imports: generate const bindings
        let js_path = path.join(".");
        if let Some(items) = &u.items {
            for item in items {
                self.line(&format!("const {} = {}.{};", item.name, js_path, item.name));
            }
        } else if let Some(last) = path.last() {
            self.line(&format!("const {} = {};", last, js_path));
        }
    }

    fn emit_test(&mut self, t: &TestDecl) {
        self.line(&format!("// test: {}", t.name.name));
        self.line(&format!("(function test_{}() {{", t.name.name));
        self.indent += 1;
        self.emit_block_body(&t.body);
        self.line(&format!("console.log('  \\x1b[32m✓\\x1b[0m {}');", t.name.name));
        self.indent -= 1;
        self.line("})();");
    }

    // ============================================================
    // Blocks & Statements
    // ============================================================

    fn emit_block_body(&mut self, block: &Block) {
        self.emit_stmts(&block.stmts, true);
    }

    fn emit_stmts(&mut self, stmts: &[Stmt], auto_return: bool) {
        for (i, stmt) in stmts.iter().enumerate() {
            let is_last = auto_return && i == stmts.len() - 1;
            self.emit_stmt(stmt, is_last);
        }
    }

    fn emit_stmts_no_return(&mut self, stmts: &[Stmt]) {
        self.emit_stmts(stmts, false);
    }

    fn emit_stmt(&mut self, stmt: &Stmt, is_last: bool) {
        match stmt {
            Stmt::Let(l) => {
                let kw = if l.mutable { "let" } else { "const" };
                let val = self.expr_to_js(&l.value);
                self.line(&format!("{} {} = {};", kw, l.name.name, val));
            }
            Stmt::Expr(expr) => {
                let js = self.expr_to_js(expr);
                if is_last {
                    self.line(&format!("return {};", js));
                } else {
                    self.line(&format!("{};", js));
                }
            }
            Stmt::For(f) => {
                let iter = self.expr_to_js(&f.iterable);
                if let Some(idx) = &f.index {
                    self.line(&format!("for (let [{}, {}] of {}.entries()) {{", idx.name, f.binding.name, iter));
                } else {
                    self.line(&format!("for (const {} of {}) {{", f.binding.name, iter));
                }
                self.indent += 1;
                self.emit_stmts_no_return(&f.body.stmts);
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Loop(block, _) => {
                self.line("while (true) {");
                self.indent += 1;
                self.emit_stmts_no_return(&block.stmts);
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Break(_) => self.line("break;"),
            Stmt::Return(expr, _) => {
                if let Some(e) = expr {
                    let js = self.expr_to_js(e);
                    self.line(&format!("return {};", js));
                } else {
                    self.line("return;");
                }
            }
            Stmt::Assign(target, value, _) => {
                let t = self.expr_to_js(target);
                let v = self.expr_to_js(value);
                self.line(&format!("{} = {};", t, v));
            }
            Stmt::Item(item) => self.emit_item(item),
        }
    }

    // ============================================================
    // Expressions
    // ============================================================

    fn expr_to_js(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(n, _) => n.to_string(),
            Expr::FloatLit(n, _) => n.to_string(),
            Expr::StringLit(s, _) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Expr::InterpolatedString(parts, _) => {
                let mut js_parts = Vec::new();
                for part in parts {
                    match &part.kind {
                        StringPartKind::Literal(s) => js_parts.push(s.clone()),
                        StringPartKind::Expr(e) => {
                            let js = self.expr_to_js(e);
                            js_parts.push(format!("${{{}}}", js));
                        }
                    }
                }
                format!("`{}`", js_parts.join(""))
            }
            Expr::BoolLit(b, _) => b.to_string(),
            Expr::Ident(id) => id.name.clone(),

            Expr::Binary(lhs, op, rhs, _) => {
                let l = self.expr_to_js(lhs);
                let r = self.expr_to_js(rhs);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq => "===",
                    BinOp::NotEq => "!==",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }

            Expr::Unary(op, inner, _) => {
                let js = self.expr_to_js(inner);
                match op {
                    UnaryOp::Neg => format!("(-{})", js),
                    UnaryOp::Not => format!("(!{})", js),
                }
            }

            Expr::FieldAccess(obj, field, _) => {
                let o = self.expr_to_js(obj);
                format!("{}.{}", o, field.name)
            }

            Expr::Index(obj, idx, _) => {
                let o = self.expr_to_js(obj);
                let i = self.expr_to_js(idx);
                format!("{}[{}]", o, i)
            }

            Expr::Call(callee, args, _) => {
                let c = self.expr_to_js(callee);
                let js_args: Vec<String> = args.iter().map(|a| self.expr_to_js(&a.value)).collect();
                format!("{}({})", c, js_args.join(", "))
            }

            Expr::StructInit(name, fields, _) => {
                let args: Vec<String> = fields.iter()
                    .filter(|f| f.name.name != "..")
                    .map(|f| self.expr_to_js(&f.value))
                    .collect();
                format!("new {}({})", name.name, args.join(", "))
            }

            Expr::ListLit(items, _) => {
                let js_items: Vec<String> = items.iter().map(|i| self.expr_to_js(i)).collect();
                format!("[{}]", js_items.join(", "))
            }

            Expr::MapLit(entries, _) => {
                let js_entries: Vec<String> = entries.iter()
                    .map(|(k, v)| format!("[{}, {}]", self.expr_to_js(k), self.expr_to_js(v)))
                    .collect();
                format!("new Map([{}])", js_entries.join(", "))
            }

            Expr::If(cond, then_block, else_branch, _) => {
                let c = self.expr_to_js(cond);
                // For simple if/else used as expression, emit ternary
                if let Some(else_expr) = else_branch {
                    let e = self.expr_to_js(else_expr);
                    format!("({} ? (() => {{ {} }})() : {})", c,
                        then_block.stmts.iter().map(|s| {
                            let mut g = JsGen::new();
                            g.emit_stmt(s, true);
                            g.output.trim().to_string()
                        }).collect::<Vec<_>>().join(" "),
                        e
                    )
                } else {
                    format!("({} ? (() => {{ {} }})() : undefined)", c,
                        then_block.stmts.iter().map(|s| {
                            let mut g = JsGen::new();
                            g.emit_stmt(s, true);
                            g.output.trim().to_string()
                        }).collect::<Vec<_>>().join(" "),
                    )
                }
            }

            Expr::Match(scrutinee, arms, _) => {
                let s = self.expr_to_js(scrutinee);
                // Emit as IIFE with switch
                let mut cases = Vec::new();
                for arm in arms {
                    let body = self.expr_to_js(&arm.body);
                    match &arm.pattern {
                        Pattern::Wildcard(_) => cases.push(format!("default: return {};", body)),
                        Pattern::Binding(id) => cases.push(format!("default: const {} = _s; return {};", id.name, body)),
                        Pattern::Literal(lit) => {
                            let val = self.expr_to_js(lit);
                            cases.push(format!("case {}: return {};", val, body));
                        }
                        Pattern::Variant(name, _) => {
                            cases.push(format!("case '{}': return {};", name.name, body));
                        }
                        Pattern::Struct(_, _, _) => cases.push(format!("default: return {};", body)),
                    }
                }
                format!("((_s) => {{ switch(_s._tag || _s) {{ {} }} }})({})", cases.join(" "), s)
            }

            Expr::Block(block) => {
                let mut g = JsGen::new();
                g.emit_block_body(block);
                format!("(() => {{ {} }})()", g.output.trim())
            }

            Expr::Closure(params, body, _) => {
                let ps: Vec<&str> = params.iter().map(|p| p.name.name.as_str()).collect();
                let b = self.expr_to_js(body);
                format!("({}) => {}", ps.join(", "), b)
            }

            Expr::Pipe(lhs, rhs, _) => {
                let l = self.expr_to_js(lhs);
                let r = self.expr_to_js(rhs);
                // Pipe: x |> f  =>  f(x)
                format!("{}({})", r, l)
            }

            Expr::Try(inner, _) => self.expr_to_js(inner),
            Expr::Catch(inner, _err, _body, _) => self.expr_to_js(inner),
            Expr::ElseUnwrap(inner, _body, _) => self.expr_to_js(inner),
            Expr::Spread(inner, _) => format!("...{}", self.expr_to_js(inner)),
            Expr::Range(start, end, _) => {
                let s = self.expr_to_js(start);
                let e = self.expr_to_js(end);
                format!("Array.from({{length: {} - {}}}, (_, i) => {} + i)", e, s, s)
            }
            Expr::Error(_) => "undefined /* error */".into(),
        }
    }

    // ============================================================
    // Helpers
    // ============================================================

    fn line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
        self.output.push_str(text);
        self.output.push('\n');
    }
}

fn format_type_name(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(id) => id.name.clone(),
        TypeExpr::Generic(id, args) => {
            let a: Vec<String> = args.iter().map(|t| format_type_name(t)).collect();
            format!("{}[{}]", id.name, a.join(", "))
        }
        TypeExpr::Option(inner) => format!("{}?", format_type_name(inner)),
        TypeExpr::Unit => "()".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codegen(source: &str) -> String {
        let program = klar_parser::parse(source).unwrap();
        generate(&program)
    }

    #[test]
    fn gen_hello_world() {
        let js = codegen(r#"fn main() { println("Hello, world!") }"#);
        assert!(js.contains("function main()"));
        assert!(js.contains("console.log"));
        assert!(js.contains("main()"));
    }

    #[test]
    fn gen_struct() {
        let js = codegen("struct User { name: String\n  age: Int }");
        assert!(js.contains("class User"));
        assert!(js.contains("this.name = name"));
    }

    #[test]
    fn gen_enum() {
        let js = codegen("enum Color { Red\n  Green\n  Blue }");
        assert!(js.contains("const Color"));
        assert!(js.contains("Symbol"));
    }

    #[test]
    fn gen_enum_with_data() {
        let js = codegen("enum Shape { Circle(radius: Float)\n  Rect(w: Float, h: Float) }");
        assert!(js.contains("_tag: 'Circle'"));
    }

    #[test]
    fn gen_let_binding() {
        let js = codegen("fn f() { let x = 42 }");
        assert!(js.contains("const x = 42"));
    }

    #[test]
    fn gen_let_mut() {
        let js = codegen("fn f() { let mut x = 0 }");
        assert!(js.contains("let x = 0"));
    }

    #[test]
    fn gen_arithmetic() {
        let js = codegen("fn f() { let x = 1 + 2 * 3 }");
        assert!(js.contains("(1 + (2 * 3))"));
    }

    #[test]
    fn gen_string_interpolation() {
        let js = codegen(r#"fn f() { let s = "Hello {name}" }"#);
        assert!(js.contains("`Hello ${name}`"));
    }

    #[test]
    fn gen_for_loop() {
        let js = codegen("fn f() { for x in [1, 2, 3] { println(x) } }");
        assert!(js.contains("for (const x of"));
    }

    #[test]
    fn gen_closure() {
        let js = codegen("fn f() { let double = |x| x * 2 }");
        assert!(js.contains("(x) => (x * 2)"));
    }

    #[test]
    fn gen_test_block() {
        let js = codegen("test add { assert 1 + 1 == 2 }");
        assert!(js.contains("test_add"));
        assert!(js.contains("assert"));
    }

    #[test]
    fn gen_main_auto_called() {
        let js = codegen("fn main() { println(42) }");
        assert!(js.ends_with("main();\n"));
    }

    #[test]
    fn gen_if_else() {
        let js = codegen("fn f() { if true { 1 } else { 0 } }");
        assert!(js.contains("true ?"));
    }

    #[test]
    fn gen_list_literal() {
        let js = codegen("fn f() { let xs = [1, 2, 3] }");
        assert!(js.contains("[1, 2, 3]"));
    }
}
