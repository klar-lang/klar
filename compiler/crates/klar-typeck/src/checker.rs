use crate::env::TypeEnv;
use crate::types::Type;
use klar_ast::*;
use klar_lexer::Span;

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type error at {}:{}: {}", self.span.start, self.span.end, self.message)
    }
}

pub struct TypeChecker {
    env: TypeEnv,
    errors: Vec<TypeError>,
    next_var: usize,
    /// Substitution map for type variables
    substitutions: Vec<Option<Type>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            env: TypeEnv::new(),
            errors: Vec::new(),
            next_var: 0,
            substitutions: Vec::new(),
        };
        checker.register_builtins();
        checker
    }

    fn register_builtins(&mut self) {
        // IO — accept any type
        let any = self.fresh_var();
        self.env.define("println".into(), Type::Fn(vec![any.clone()], Box::new(Type::Unit)));
        self.env.define("print".into(), Type::Fn(vec![any], Box::new(Type::Unit)));
        self.env.define("read_line".into(), Type::Fn(vec![], Box::new(Type::String)));

        // Assertions
        self.env.define("assert".into(), Type::Fn(vec![Type::Bool], Box::new(Type::Unit)));
        self.env.define("assert_eq".into(), Type::Fn(vec![Type::Var(0), Type::Var(0)], Box::new(Type::Unit)));

        // Type conversions
        self.env.define("to_string".into(), Type::Fn(vec![Type::Var(0)], Box::new(Type::String)));
        self.env.define("to_int".into(), Type::Fn(vec![Type::String], Box::new(Type::Int)));

        // Standard library modules — registered as opaque Named types
        // The type checker allows field access on Named types (returns fresh vars)
        // Async/concurrency builtins
        let async_var = self.fresh_var();
        self.env.define("spawn".into(), Type::Fn(vec![async_var.clone()], Box::new(async_var.clone())));
        self.env.define("parallel".into(), Type::Fn(vec![async_var], Box::new(Type::Unit)));

        for module in ["string", "list", "map", "set", "json", "math", "io", "env", "time", "crypto", "log", "http", "sql", "channel", "ws", "redis", "auth", "std"] {
            self.env.define(module.into(), Type::Named(format!("std.{}", module)));
        }
    }

    /// Look up the type of an identifier at a given source offset.
    /// Returns the display name of the type if found.
    pub fn type_at_offset(&self, _offset: usize) -> Option<String> {
        // Simplified: would need position-to-name mapping
        // For now, return None (hover will fall through to keyword/builtin checks)
        None
    }

    /// Look up the type of a name in the current environment.
    pub fn type_of(&self, name: &str) -> Option<String> {
        self.env.lookup(name).map(|ty| ty.display_name())
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        // First pass: register all top-level declarations
        for item in &program.items {
            self.register_item(item);
        }

        // Second pass: type check bodies
        for item in &program.items {
            self.check_item(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn fresh_var(&mut self) -> Type {
        let id = self.next_var;
        self.next_var += 1;
        self.substitutions.push(None);
        Type::Var(id)
    }

    // ============================================================
    // Registration (first pass)
    // ============================================================

    fn register_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                let param_types: Vec<Type> = f.params.iter().map(|p| self.resolve_type_expr(&p.ty)).collect();
                let ret = f.return_type.as_ref().map_or(Type::Unit, |t| self.resolve_type_expr(t));
                let fn_type = Type::Fn(param_types, Box::new(ret));
                self.env.define(f.name.name.clone(), fn_type);
            }
            Item::Struct(s) => {
                let fields: Vec<(String, Type)> = s.fields.iter()
                    .map(|f| (f.name.name.clone(), self.resolve_type_expr(&f.ty)))
                    .collect();
                self.env.define(s.name.name.clone(), Type::Struct(s.name.name.clone(), fields));
            }
            Item::Enum(e) => {
                let variants: Vec<(String, Vec<Type>)> = e.variants.iter()
                    .map(|v| {
                        let tys = v.fields.iter().map(|f| self.resolve_type_expr(&f.ty)).collect();
                        (v.name.name.clone(), tys)
                    })
                    .collect();
                self.env.define(e.name.name.clone(), Type::Enum(e.name.name.clone(), variants));
            }
            Item::Use(u) => {
                // Register imported names from std modules as opaque types
                // e.g., `use std.http.{Router, Request, Response}` defines those names
                if let Some(items) = &u.items {
                    for item_name in items {
                        let fresh = self.fresh_var();
                        self.env.define(item_name.name.clone(), fresh);
                    }
                } else if let Some(last) = u.path.last() {
                    // `use std.http` defines `http` as a module
                    let module_name = last.name.clone();
                    self.env.define(module_name.clone(), Type::Named(
                        u.path.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(".")
                    ));
                }
            }
            _ => {}
        }
    }

    // ============================================================
    // Type checking (second pass)
    // ============================================================

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.check_fn(f),
            Item::Test(t) => self.check_block(&t.body),
            _ => {}
        }
    }

    fn check_fn(&mut self, f: &FnDecl) {
        self.env.push_scope();

        // Bind parameters
        for param in &f.params {
            let ty = self.resolve_type_expr(&param.ty);
            self.env.define(param.name.name.clone(), ty);
        }

        // Check body
        self.check_block(&f.body);

        self.env.pop_scope();
    }

    fn check_block(&mut self, block: &Block) {
        self.env.push_scope();
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        self.env.pop_scope();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(l) => {
                let value_ty = self.infer_expr(&l.value);
                if let Some(declared) = &l.ty {
                    let expected = self.resolve_type_expr(declared);
                    self.unify(&expected, &value_ty, l.span);
                }
                self.env.define(l.name.name.clone(), value_ty);
            }
            Stmt::Expr(expr) => {
                self.infer_expr(expr);
            }
            Stmt::For(f) => {
                let iter_ty = self.infer_expr(&f.iterable);
                let elem_ty = match &iter_ty {
                    Type::List(t) => *t.clone(),
                    Type::String => Type::String,
                    _ => {
                        self.error(format!("cannot iterate over {}", iter_ty.display_name()), f.span);
                        Type::Error
                    }
                };
                self.env.push_scope();
                self.env.define(f.binding.name.clone(), elem_ty);
                if let Some(idx) = &f.index {
                    self.env.define(idx.name.clone(), Type::Int);
                }
                self.check_block(&f.body);
                self.env.pop_scope();
            }
            Stmt::Loop(block, _) => self.check_block(block),
            Stmt::Return(expr, _) => {
                if let Some(e) = expr {
                    self.infer_expr(e);
                }
            }
            Stmt::Assign(target, value, span) => {
                let target_ty = self.infer_expr(target);
                let value_ty = self.infer_expr(value);
                self.unify(&target_ty, &value_ty, *span);
            }
            Stmt::Break(_) => {}
            Stmt::Item(item) => self.check_item(item),
        }
    }

    // ============================================================
    // Type inference
    // ============================================================

    fn infer_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::IntLit(_, _) => Type::Int,
            Expr::FloatLit(_, _) => Type::Float,
            Expr::StringLit(_, _) => Type::String,
            Expr::InterpolatedString(_, _) => Type::String,
            Expr::BoolLit(_, _) => Type::Bool,

            Expr::Ident(id) => {
                match self.env.lookup(&id.name) {
                    Some(ty) => ty.clone(),
                    None => {
                        self.error(format!("undefined: '{}'", id.name), id.span);
                        Type::Error
                    }
                }
            }

            Expr::Binary(lhs, op, rhs, span) => {
                let lt = self.infer_expr(lhs);
                let rt = self.infer_expr(rhs);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if lt == Type::String && *op == BinOp::Add {
                            return Type::String; // string concatenation
                        }
                        if !lt.is_numeric() && lt != Type::Error {
                            self.error(format!("operator requires numeric types, got {}", lt.display_name()), *span);
                        }
                        self.unify(&lt, &rt, *span);
                        lt
                    }
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                        self.unify(&lt, &rt, *span);
                        Type::Bool
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(&lt, &Type::Bool, *span);
                        self.unify(&rt, &Type::Bool, *span);
                        Type::Bool
                    }
                }
            }

            Expr::Unary(op, inner, span) => {
                let t = self.infer_expr(inner);
                match op {
                    UnaryOp::Neg => {
                        if !t.is_numeric() && t != Type::Error {
                            self.error(format!("negation requires numeric type, got {}", t.display_name()), *span);
                        }
                        t
                    }
                    UnaryOp::Not => {
                        self.unify(&t, &Type::Bool, *span);
                        Type::Bool
                    }
                }
            }

            Expr::FieldAccess(obj, field, span) => {
                let obj_ty = self.infer_expr(obj);
                match &obj_ty {
                    Type::Struct(_, fields) => {
                        for (name, ty) in fields {
                            if name == &field.name {
                                return ty.clone();
                            }
                        }
                        // Allow static methods on structs (fromJSON, schema, etc.)
                        self.fresh_var()
                    }
                    Type::Error => Type::Error,
                    // For now, allow field access on unknown types (method calls etc.)
                    _ => self.fresh_var(),
                }
            }

            Expr::Call(callee, args, span) => {
                let callee_ty = self.infer_expr(callee);
                match callee_ty {
                    Type::Fn(params, ret) => {
                        if args.len() != params.len() {
                            self.error(
                                format!("expected {} arguments, got {}", params.len(), args.len()),
                                *span,
                            );
                        } else {
                            for (arg, expected) in args.iter().zip(params.iter()) {
                                let actual = self.infer_expr(&arg.value);
                                self.unify(expected, &actual, arg.span);
                            }
                        }
                        *ret
                    }
                    Type::Error => Type::Error,
                    _ => {
                        // Could be a method call or constructor — infer args and return fresh
                        for arg in args {
                            self.infer_expr(&arg.value);
                        }
                        self.fresh_var()
                    }
                }
            }

            Expr::Index(obj, idx, span) => {
                let obj_ty = self.infer_expr(obj);
                let idx_ty = self.infer_expr(idx);
                match &obj_ty {
                    Type::List(elem) => {
                        self.unify(&idx_ty, &Type::Int, *span);
                        *elem.clone()
                    }
                    Type::Map(k, v) => {
                        self.unify(&idx_ty, k, *span);
                        *v.clone()
                    }
                    Type::String => {
                        self.unify(&idx_ty, &Type::Int, *span);
                        Type::String
                    }
                    _ => self.fresh_var(),
                }
            }

            Expr::StructInit(name, fields, _span) => {
                match self.env.lookup(&name.name).cloned() {
                    Some(Type::Struct(sname, expected_fields)) => {
                        for fi in fields {
                            if fi.name.name == ".." { continue; }
                            let actual = self.infer_expr(&fi.value);
                            if let Some((_, expected)) = expected_fields.iter().find(|(n, _)| n == &fi.name.name) {
                                self.unify(expected, &actual, fi.span);
                            } else {
                                self.error(format!("unknown field '{}' on {}", fi.name.name, sname), fi.span);
                            }
                        }
                        Type::Struct(sname, expected_fields)
                    }
                    _ => {
                        for fi in fields {
                            self.infer_expr(&fi.value);
                        }
                        Type::Named(name.name.clone())
                    }
                }
            }

            Expr::ListLit(items, _) => {
                if items.is_empty() {
                    Type::List(Box::new(self.fresh_var()))
                } else {
                    let first_ty = self.infer_expr(&items[0]);
                    for item in &items[1..] {
                        let t = self.infer_expr(item);
                        self.unify(&first_ty, &t, item.span());
                    }
                    Type::List(Box::new(first_ty))
                }
            }

            Expr::MapLit(entries, _) => {
                if entries.is_empty() {
                    Type::Map(Box::new(self.fresh_var()), Box::new(self.fresh_var()))
                } else {
                    let key_ty = self.infer_expr(&entries[0].0);
                    let val_ty = self.infer_expr(&entries[0].1);
                    for (k, v) in &entries[1..] {
                        let kt = self.infer_expr(k);
                        let vt = self.infer_expr(v);
                        self.unify(&key_ty, &kt, k.span());
                        self.unify(&val_ty, &vt, v.span());
                    }
                    Type::Map(Box::new(key_ty), Box::new(val_ty))
                }
            }

            Expr::If(cond, then_block, else_branch, span) => {
                let cond_ty = self.infer_expr(cond);
                self.unify(&cond_ty, &Type::Bool, *span);
                self.check_block(then_block);
                if let Some(else_expr) = else_branch {
                    self.infer_expr(else_expr);
                }
                // For now, if/else returns Unit (proper return type tracking is a later enhancement)
                Type::Unit
            }

            Expr::Match(scrutinee, arms, _) => {
                let _scrut_ty = self.infer_expr(scrutinee);
                for arm in arms {
                    self.infer_expr(&arm.body);
                }
                if arms.is_empty() { Type::Unit } else { self.fresh_var() }
            }

            Expr::Block(block) => {
                self.check_block(block);
                Type::Unit
            }

            Expr::Closure(params, body, _) => {
                self.env.push_scope();
                let param_types: Vec<Type> = params.iter().map(|p| {
                    let ty = match &p.ty {
                        Some(t) => self.resolve_type_expr(t),
                        None => self.fresh_var(),
                    };
                    self.env.define(p.name.name.clone(), ty.clone());
                    ty
                }).collect();
                let ret_ty = self.infer_expr(body);
                self.env.pop_scope();
                Type::Fn(param_types, Box::new(ret_ty))
            }

            Expr::Pipe(lhs, rhs, _) => {
                let _input = self.infer_expr(lhs);
                self.infer_expr(rhs)
            }

            Expr::Try(inner, _) => {
                let inner_ty = self.infer_expr(inner);
                // Try unwraps Result[T, E] to T
                match inner_ty {
                    Type::Result(t, _) => *t,
                    _ => inner_ty,
                }
            }

            Expr::Catch(inner, err_name, body, _) => {
                let inner_ty = self.infer_expr(inner);
                self.env.push_scope();
                let err_ty = match &inner_ty {
                    Type::Result(_, e) => *e.clone(),
                    _ => Type::Error,
                };
                self.env.define(err_name.name.clone(), err_ty);
                self.check_block(body);
                self.env.pop_scope();
                match inner_ty {
                    Type::Result(t, _) => *t,
                    _ => inner_ty,
                }
            }

            Expr::ElseUnwrap(inner, body, _) => {
                let inner_ty = self.infer_expr(inner);
                self.check_block(body);
                match inner_ty {
                    Type::Option(t) => *t,
                    _ => inner_ty,
                }
            }

            Expr::Spread(inner, _) => self.infer_expr(inner),
            Expr::Range(start, end, span) => {
                let st = self.infer_expr(start);
                let et = self.infer_expr(end);
                self.unify(&st, &et, *span);
                Type::List(Box::new(st))
            }

            Expr::Error(_) => Type::Error,
        }
    }

    // ============================================================
    // Type resolution & unification
    // ============================================================

    fn resolve_type_expr(&mut self, ty: &TypeExpr) -> Type {
        match ty {
            TypeExpr::Named(ident) => match ident.name.as_str() {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Bool" => Type::Bool,
                "String" => Type::String,
                "Byte" => Type::Byte,
                "Unit" => Type::Unit,
                name => {
                    // Check if it's a known struct/enum
                    if let Some(t) = self.env.lookup(name) {
                        t.clone()
                    } else {
                        Type::Named(name.to_string())
                    }
                }
            },
            TypeExpr::Generic(name, args) => {
                let resolved_args: Vec<Type> = args.iter().map(|a| self.resolve_type_expr(a)).collect();
                match name.name.as_str() {
                    "List" => Type::List(Box::new(resolved_args.into_iter().next().unwrap_or(Type::Error))),
                    "Map" => {
                        let mut it = resolved_args.into_iter();
                        Type::Map(
                            Box::new(it.next().unwrap_or(Type::Error)),
                            Box::new(it.next().unwrap_or(Type::Error)),
                        )
                    }
                    "Set" => Type::Set(Box::new(resolved_args.into_iter().next().unwrap_or(Type::Error))),
                    "Option" => Type::Option(Box::new(resolved_args.into_iter().next().unwrap_or(Type::Error))),
                    "Result" => {
                        let mut it = resolved_args.into_iter();
                        Type::Result(
                            Box::new(it.next().unwrap_or(Type::Error)),
                            Box::new(it.next().unwrap_or(Type::Error)),
                        )
                    }
                    _ => Type::Named(name.name.clone()),
                }
            },
            TypeExpr::Option(inner) => Type::Option(Box::new(self.resolve_type_expr(inner))),
            TypeExpr::Unit => Type::Unit,
        }
    }

    fn unify(&mut self, expected: &Type, actual: &Type, span: Span) {
        if *expected == Type::Error || *actual == Type::Error {
            return; // Don't cascade errors
        }
        match (expected, actual) {
            (Type::Var(_), _) | (_, Type::Var(_)) => {
                // For now, type variables unify with anything
            }
            (Type::Named(_), _) | (_, Type::Named(_)) => {
                // Named types unify permissively until we have full resolution
            }
            (a, b) if a == b => {}
            (Type::List(a), Type::List(b)) => self.unify(a, b, span),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.unify(k1, k2, span);
                self.unify(v1, v2, span);
            }
            (Type::Option(a), Type::Option(b)) => self.unify(a, b, span),
            (Type::Result(a1, e1), Type::Result(a2, e2)) => {
                self.unify(a1, a2, span);
                self.unify(e1, e2, span);
            }
            _ => {
                self.error(
                    format!("type mismatch: expected {}, got {}", expected.display_name(), actual.display_name()),
                    span,
                );
            }
        }
    }

    fn error(&mut self, message: String, span: Span) {
        self.errors.push(TypeError { message, span });
    }
}
