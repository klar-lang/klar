use klar_ast::*;
use klar_lexer::{LexerStringPart, Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at {}:{}: {}", self.span.start, self.span.end, self.message)
    }
}

pub struct Parser<'src> {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    source: &'src str,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: Vec<Token>, source: &'src str) -> Self {
        Self { tokens, pos: 0, errors: Vec::new(), source }
    }

    pub fn parse_program(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.at_eof() {
            match self.parse_item() {
                Some(item) => items.push(item),
                None => {
                    // Error recovery: skip to next item boundary
                    self.advance();
                    self.skip_newlines();
                }
            }
            self.skip_newlines();
        }
        if self.errors.is_empty() {
            Ok(Program { items })
        } else {
            Err(self.errors.clone())
        }
    }

    // ============================================================
    // Items
    // ============================================================

    fn parse_item(&mut self) -> Option<Item> {
        // Collect annotations
        let mut annotations = Vec::new();
        while self.check(&TokenKind::At) {
            if let Some(ann) = self.parse_annotation() {
                annotations.push(ann);
            }
            self.skip_newlines();
        }

        let is_priv = self.eat(&TokenKind::Priv);

        match self.peek_kind() {
            Some(TokenKind::Fn) => self.parse_fn_decl(is_priv).map(Item::Function),
            Some(TokenKind::Struct) => self.parse_struct_decl(annotations).map(Item::Struct),
            Some(TokenKind::Enum) => self.parse_enum_decl().map(Item::Enum),
            Some(TokenKind::Trait) => self.parse_trait_decl().map(Item::Trait),
            Some(TokenKind::Impl) => self.parse_impl_decl().map(Item::Impl),
            Some(TokenKind::Use) => self.parse_use_decl().map(Item::Use),
            Some(TokenKind::Test) => self.parse_test_decl().map(Item::Test),
            _ => {
                if !annotations.is_empty() || is_priv {
                    self.error("expected declaration after annotation or visibility modifier");
                }
                None
            }
        }
    }

    fn parse_annotation(&mut self) -> Option<Annotation> {
        let start = self.current_span();
        self.expect(&TokenKind::At)?;
        let name = self.parse_ident()?;
        let mut args = Vec::new();
        if self.eat(&TokenKind::LParen) {
            while !self.check(&TokenKind::RParen) && !self.at_eof() {
                if let Some(expr) = self.parse_expr() {
                    args.push(expr);
                }
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RParen);
        }
        Some(Annotation { name, args, span: self.span_from(start) })
    }

    fn parse_fn_decl(&mut self, is_priv: bool) -> Option<FnDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Fn)?;
        let name = self.parse_ident()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list();
        self.expect(&TokenKind::RParen);

        let mut return_type = None;
        let mut error_type = None;
        if self.eat(&TokenKind::Arrow) {
            return_type = Some(self.parse_type()?);
            if self.eat(&TokenKind::Bang) {
                error_type = Some(self.parse_type()?);
            }
        } else if self.eat(&TokenKind::Bang) {
            // fn main() ! AppError — no return type, just error type
            error_type = Some(self.parse_type()?);
        }

        self.skip_newlines();
        let body = self.parse_block()?;

        Some(FnDecl {
            name, params, return_type, error_type, body, is_priv,
            span: self.span_from(start),
        })
    }

    fn parse_param_list(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.at_eof() {
            let start = self.current_span();
            if let Some(name) = self.parse_ident() {
                if self.eat(&TokenKind::Colon) {
                    if let Some(ty) = self.parse_type() {
                        let default = if self.eat(&TokenKind::Eq) {
                            self.parse_expr()
                        } else {
                            None
                        };
                        params.push(Param { name, ty, default, span: self.span_from(start) });
                    }
                } else {
                    // Handle `self` parameter
                    params.push(Param {
                        ty: TypeExpr::Named(name.clone()),
                        name,
                        default: None,
                        span: self.span_from(start),
                    });
                }
            }
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        params
    }

    fn parse_struct_decl(&mut self, annotations: Vec<Annotation>) -> Option<StructDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Struct)?;
        let name = self.parse_ident()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let fstart = self.current_span();
            let mut field_annotations = Vec::new();

            // Field-level annotations come after the type
            let fname = self.parse_ident()?;
            self.expect(&TokenKind::Colon);
            let fty = self.parse_type()?;

            // Parse field annotations: @min_len(1) @format(email) etc.
            while self.check(&TokenKind::At) {
                if let Some(ann) = self.parse_annotation() {
                    field_annotations.push(ann);
                }
            }

            let default = if self.eat(&TokenKind::Eq) {
                self.parse_expr()
            } else {
                None
            };

            fields.push(Field {
                name: fname, ty: fty, annotations: field_annotations,
                default, span: self.span_from(fstart),
            });
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace);
        Some(StructDecl { name, annotations, fields, span: self.span_from(start) })
    }

    fn parse_enum_decl(&mut self) -> Option<EnumDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Enum)?;
        let name = self.parse_ident()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut variants = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let vstart = self.current_span();
            let vname = self.parse_ident()?;
            let fields = if self.eat(&TokenKind::LParen) {
                let params = self.parse_param_list();
                self.expect(&TokenKind::RParen);
                params
            } else {
                Vec::new()
            };
            variants.push(Variant { name: vname, fields, span: self.span_from(vstart) });
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace);
        Some(EnumDecl { name, variants, span: self.span_from(start) })
    }

    fn parse_trait_decl(&mut self) -> Option<TraitDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Trait)?;
        let name = self.parse_ident()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let mstart = self.current_span();
            self.expect(&TokenKind::Fn);
            let mname = self.parse_ident()?;
            self.expect(&TokenKind::LParen);
            let params = self.parse_param_list();
            self.expect(&TokenKind::RParen);

            let mut return_type = None;
            let mut error_type = None;
            if self.eat(&TokenKind::Arrow) {
                return_type = self.parse_type();
                if self.eat(&TokenKind::Bang) {
                    error_type = self.parse_type();
                }
            }

            methods.push(FnSig {
                name: mname, params, return_type, error_type,
                span: self.span_from(mstart),
            });
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace);
        Some(TraitDecl { name, methods, span: self.span_from(start) })
    }

    fn parse_impl_decl(&mut self) -> Option<ImplDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Impl)?;
        let trait_name = self.parse_ident()?;
        self.expect(&TokenKind::For);
        let target = self.parse_ident()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            self.skip_newlines();
            if let Some(f) = self.parse_fn_decl(false) {
                methods.push(f);
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace);
        Some(ImplDecl { trait_name, target, methods, span: self.span_from(start) })
    }

    fn parse_use_decl(&mut self) -> Option<UseDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Use)?;
        let mut path = vec![self.parse_ident()?];
        while self.eat(&TokenKind::Dot) {
            if self.check(&TokenKind::LBrace) {
                break;
            }
            path.push(self.parse_ident()?);
        }
        let items = if self.eat(&TokenKind::LBrace) {
            let mut names = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                names.push(self.parse_ident()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RBrace);
            Some(names)
        } else {
            None
        };
        Some(UseDecl { path, items, span: self.span_from(start) })
    }

    fn parse_test_decl(&mut self) -> Option<TestDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Test)?;
        let name = self.parse_ident()?;
        self.skip_newlines();
        let body = self.parse_block()?;
        Some(TestDecl { name, body, span: self.span_from(start) })
    }

    // ============================================================
    // Types
    // ============================================================

    fn parse_type(&mut self) -> Option<TypeExpr> {
        if self.eat(&TokenKind::LParen) {
            self.expect(&TokenKind::RParen);
            return Some(TypeExpr::Unit);
        }

        let name = self.parse_ident()?;

        let base = if self.eat(&TokenKind::LBracket) {
            let mut args = vec![self.parse_type()?];
            while self.eat(&TokenKind::Comma) {
                args.push(self.parse_type()?);
            }
            self.expect(&TokenKind::RBracket);
            TypeExpr::Generic(name, args)
        } else {
            TypeExpr::Named(name)
        };

        if self.eat(&TokenKind::Question) {
            Some(TypeExpr::Option(Box::new(base)))
        } else {
            Some(base)
        }
    }

    // ============================================================
    // Blocks & Statements
    // ============================================================

    fn parse_block(&mut self) -> Option<Block> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                self.advance();
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace);
        Some(Block { stmts, span: self.span_from(start) })
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        self.skip_newlines();
        match self.peek_kind() {
            Some(TokenKind::Let) => self.parse_let_stmt().map(Stmt::Let),
            Some(TokenKind::For) => self.parse_for_stmt().map(Stmt::For),
            Some(TokenKind::Loop) => {
                let start = self.current_span();
                self.advance();
                self.skip_newlines();
                let body = self.parse_block()?;
                Some(Stmt::Loop(body, self.span_from(start)))
            }
            Some(TokenKind::Break) => {
                let span = self.current_span();
                self.advance();
                Some(Stmt::Break(span))
            }
            Some(TokenKind::Return) => {
                let start = self.current_span();
                self.advance();
                let value = if !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Newline) && !self.at_eof() {
                    self.parse_expr()
                } else {
                    None
                };
                Some(Stmt::Return(value, self.span_from(start)))
            }
            _ => {
                let expr = self.parse_expr()?;
                if self.eat(&TokenKind::Eq) {
                    let value = self.parse_expr()?;
                    let span = Span::new(expr.span().start, value.span().end);
                    Some(Stmt::Assign(expr, value, span))
                } else {
                    Some(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Option<LetStmt> {
        let start = self.current_span();
        self.expect(&TokenKind::Let)?;
        let mutable = self.eat(&TokenKind::Mut);
        let name = self.parse_ident()?;

        let ty = if self.eat(&TokenKind::Colon) {
            self.parse_type()
        } else {
            None
        };

        self.expect(&TokenKind::Eq);
        let value = self.parse_expr()?;
        Some(LetStmt { name, ty, value, mutable, span: self.span_from(start) })
    }

    fn parse_for_stmt(&mut self) -> Option<ForStmt> {
        let start = self.current_span();
        self.expect(&TokenKind::For)?;
        let first = self.parse_ident()?;

        let (binding, index) = if self.eat(&TokenKind::Comma) {
            let second = self.parse_ident()?;
            (second, Some(first))
        } else {
            (first, None)
        };

        self.expect(&TokenKind::In);
        let iterable = self.parse_expr()?;
        self.skip_newlines();
        let body = self.parse_block()?;
        Some(ForStmt { binding, index, iterable, body, span: self.span_from(start) })
    }

    // ============================================================
    // Expressions (Pratt parser)
    // ============================================================

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Option<Expr> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Postfix: ?, field access, call, index
            loop {
                if self.eat(&TokenKind::Question) {
                    let span = Span::new(lhs.span().start, self.prev_span().end);
                    lhs = Expr::Try(Box::new(lhs), span);
                } else if self.check(&TokenKind::Dot) && !self.check_at(1, &TokenKind::Dot) {
                    self.advance();
                    let field = self.parse_ident()?;
                    let span = Span::new(lhs.span().start, field.span.end);
                    lhs = Expr::FieldAccess(Box::new(lhs), field, span);
                } else if self.check(&TokenKind::LParen) {
                    let args = self.parse_call_args();
                    let span = Span::new(lhs.span().start, self.prev_span().end);
                    lhs = Expr::Call(Box::new(lhs), args, span);
                } else if self.check(&TokenKind::LBracket) {
                    self.advance();
                    let idx = self.parse_expr()?;
                    if self.eat(&TokenKind::DotDot) {
                        let end = self.parse_expr()?;
                        let span = Span::new(lhs.span().start, end.span().end);
                        self.expect(&TokenKind::RBracket);
                        lhs = Expr::Range(Box::new(idx), Box::new(end), span);
                    } else {
                        let span = Span::new(lhs.span().start, self.current_span().end);
                        self.expect(&TokenKind::RBracket);
                        lhs = Expr::Index(Box::new(lhs), Box::new(idx), span);
                    }
                } else {
                    break;
                }
            }

            // Catch / else (postfix keywords)
            if self.check(&TokenKind::Catch) {
                self.advance();
                let err_name = self.parse_ident()?;
                self.skip_newlines();
                let body = self.parse_block()?;
                let span = Span::new(lhs.span().start, body.span.end);
                lhs = Expr::Catch(Box::new(lhs), err_name, body, span);
                continue;
            }
            if self.check(&TokenKind::Else) {
                self.advance();
                self.skip_newlines();
                let body = self.parse_block()?;
                let span = Span::new(lhs.span().start, body.span.end);
                lhs = Expr::ElseUnwrap(Box::new(lhs), body, span);
                continue;
            }

            // Pipe operator |> (can continue on next line)
            self.skip_newlines();
            if self.check(&TokenKind::PipeArrow) {
                let (l_bp, r_bp) = (1, 2);
                if l_bp < min_bp { break; }
                self.advance();
                self.skip_newlines();
                let rhs = self.parse_expr_bp(r_bp)?;
                let span = Span::new(lhs.span().start, rhs.span().end);
                lhs = Expr::Pipe(Box::new(lhs), Box::new(rhs), span);
                continue;
            }

            // Binary operators
            let Some(op) = self.peek_binop() else { break };
            let (l_bp, r_bp) = binop_bp(op);
            if l_bp < min_bp { break; }
            self.advance();
            self.skip_newlines();
            let rhs = self.parse_expr_bp(r_bp)?;
            let span = Span::new(lhs.span().start, rhs.span().end);
            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs), span);
        }

        Some(lhs)
    }

    fn parse_prefix(&mut self) -> Option<Expr> {
        match self.peek_kind() {
            Some(TokenKind::IntLit(_)) => {
                let tok = self.advance_tok();
                if let TokenKind::IntLit(n) = tok.kind {
                    Some(Expr::IntLit(n, tok.span))
                } else { None }
            }
            Some(TokenKind::FloatLit(_)) => {
                let tok = self.advance_tok();
                if let TokenKind::FloatLit(n) = tok.kind {
                    Some(Expr::FloatLit(n, tok.span))
                } else { None }
            }
            Some(TokenKind::StringLit(_)) => {
                let tok = self.advance_tok();
                if let TokenKind::StringLit(s) = tok.kind {
                    Some(Expr::StringLit(s, tok.span))
                } else { None }
            }
            Some(TokenKind::InterpolatedString(_)) => {
                let tok = self.advance_tok();
                if let TokenKind::InterpolatedString(parts) = tok.kind {
                    let ast_parts = parts.into_iter().map(|p| match p {
                        LexerStringPart::Literal(s) => StringPart {
                            kind: StringPartKind::Literal(s),
                        },
                        LexerStringPart::Expr(s) => StringPart {
                            kind: StringPartKind::Expr(Box::new(Expr::Ident(
                                Ident::new(s, tok.span),
                            ))),
                        },
                    }).collect();
                    Some(Expr::InterpolatedString(ast_parts, tok.span))
                } else { None }
            }
            Some(TokenKind::True) => {
                let tok = self.advance_tok();
                Some(Expr::BoolLit(true, tok.span))
            }
            Some(TokenKind::False) => {
                let tok = self.advance_tok();
                Some(Expr::BoolLit(false, tok.span))
            }
            Some(TokenKind::Minus) => {
                let start = self.current_span();
                self.advance();
                let expr = self.parse_expr_bp(9)?;
                let span = self.span_from(start);
                Some(Expr::Unary(UnaryOp::Neg, Box::new(expr), span))
            }
            Some(TokenKind::Not) => {
                let start = self.current_span();
                self.advance();
                let expr = self.parse_expr_bp(9)?;
                let span = self.span_from(start);
                Some(Expr::Unary(UnaryOp::Not, Box::new(expr), span))
            }
            Some(TokenKind::If) => self.parse_if_expr(),
            Some(TokenKind::Match) => self.parse_match_expr(),
            Some(TokenKind::LParen) => {
                self.advance();
                if self.eat(&TokenKind::RParen) {
                    let span = self.prev_span();
                    return Some(Expr::IntLit(0, span)); // Unit value
                }
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen);
                Some(expr)
            }
            Some(TokenKind::LBracket) => self.parse_list_or_map(),
            Some(TokenKind::LBrace) => {
                let block = self.parse_block()?;
                Some(Expr::Block(block))
            }
            Some(TokenKind::Pipe) => self.parse_closure(),
            Some(TokenKind::DotDot) => {
                let start = self.current_span();
                self.advance();
                let expr = self.parse_expr_bp(9)?;
                let span = self.span_from(start);
                Some(Expr::Spread(Box::new(expr), span))
            }
            Some(TokenKind::Ident(_)) => {
                let ident = self.parse_ident()?;
                // Check for struct initialization: `Name { ... }`
                if self.check(&TokenKind::LBrace) && is_type_name(&ident.name) {
                    return self.parse_struct_init(ident);
                }
                Some(Expr::Ident(ident))
            }
            _ => {
                self.error("expected expression");
                None
            }
        }
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.expect(&TokenKind::If)?;
        let cond = self.parse_expr()?;
        self.skip_newlines();
        let then_block = self.parse_block()?;
        let else_branch = if self.eat(&TokenKind::Else) {
            self.skip_newlines();
            if self.check(&TokenKind::If) {
                Some(Box::new(self.parse_if_expr()?))
            } else {
                let block = self.parse_block()?;
                Some(Box::new(Expr::Block(block)))
            }
        } else {
            None
        };
        let span = self.span_from(start);
        Some(Expr::If(Box::new(cond), then_block, else_branch, span))
    }

    fn parse_match_expr(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.expect(&TokenKind::Match)?;
        let scrutinee = self.parse_expr()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let astart = self.current_span();
            let pattern = self.parse_pattern()?;
            self.expect(&TokenKind::FatArrow);
            let body = self.parse_expr()?;
            arms.push(MatchArm { pattern, body, span: self.span_from(astart) });
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace);
        Some(Expr::Match(Box::new(scrutinee), arms, self.span_from(start)))
    }

    fn parse_pattern(&mut self) -> Option<Pattern> {
        match self.peek_kind() {
            Some(TokenKind::Ident(name)) if name == "_" => {
                let span = self.current_span();
                self.advance();
                Some(Pattern::Wildcard(span))
            }
            Some(TokenKind::Ident(_)) => {
                let ident = self.parse_ident()?;
                if self.eat(&TokenKind::LParen) {
                    let mut sub = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.at_eof() {
                        sub.push(self.parse_pattern()?);
                        if !self.eat(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen);
                    Some(Pattern::Variant(ident, sub))
                } else {
                    Some(Pattern::Binding(ident))
                }
            }
            Some(TokenKind::IntLit(_) | TokenKind::FloatLit(_) | TokenKind::StringLit(_) | TokenKind::True | TokenKind::False) => {
                let expr = self.parse_prefix()?;
                Some(Pattern::Literal(expr))
            }
            _ => {
                self.error("expected pattern");
                None
            }
        }
    }

    fn parse_closure(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.expect(&TokenKind::Pipe)?;
        let mut params = Vec::new();
        while !self.check(&TokenKind::Pipe) && !self.at_eof() {
            let name = self.parse_ident()?;
            let ty = if self.eat(&TokenKind::Colon) {
                self.parse_type()
            } else {
                None
            };
            params.push(ClosureParam { name, ty });
            if !self.eat(&TokenKind::Comma) { break; }
        }
        self.expect(&TokenKind::Pipe);
        let body = self.parse_expr()?;
        let span = self.span_from(start);
        Some(Expr::Closure(params, Box::new(body), span))
    }

    fn parse_list_or_map(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.advance(); // consume [
        if self.eat(&TokenKind::RBracket) {
            return Some(Expr::ListLit(Vec::new(), self.span_from(start)));
        }
        let first = self.parse_expr()?;
        if self.eat(&TokenKind::Colon) {
            // Map literal
            let val = self.parse_expr()?;
            let mut entries = vec![(first, val)];
            while self.eat(&TokenKind::Comma) {
                if self.check(&TokenKind::RBracket) { break; }
                let k = self.parse_expr()?;
                self.expect(&TokenKind::Colon);
                let v = self.parse_expr()?;
                entries.push((k, v));
            }
            self.expect(&TokenKind::RBracket);
            Some(Expr::MapLit(entries, self.span_from(start)))
        } else {
            let mut items = vec![first];
            while self.eat(&TokenKind::Comma) {
                if self.check(&TokenKind::RBracket) { break; }
                items.push(self.parse_expr()?);
            }
            self.expect(&TokenKind::RBracket);
            Some(Expr::ListLit(items, self.span_from(start)))
        }
    }

    fn parse_struct_init(&mut self, name: Ident) -> Option<Expr> {
        let start = name.span;
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::DotDot) {
                let spread_expr = self.parse_prefix()?;
                fields.push(FieldInit {
                    name: Ident::new("..", spread_expr.span()),
                    value: spread_expr.clone(),
                    span: spread_expr.span(),
                });
            } else {
                let fstart = self.current_span();
                let fname = self.parse_ident()?;
                self.expect(&TokenKind::Colon);
                let fval = self.parse_expr()?;
                fields.push(FieldInit { name: fname, value: fval, span: self.span_from(fstart) });
            }
            self.eat(&TokenKind::Comma);
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace);
        Some(Expr::StructInit(name, fields, self.span_from(start)))
    }

    fn parse_call_args(&mut self) -> Vec<CallArg> {
        self.advance(); // consume (
        let mut args = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.at_eof() {
            let start = self.current_span();
            // Check for named arg: `name: value`
            if self.peek_is_ident() && self.check_at(1, &TokenKind::Colon) {
                let name = self.parse_ident();
                self.advance(); // consume :
                let value = self.parse_expr().unwrap_or(Expr::Error(self.current_span()));
                args.push(CallArg { name, value, span: self.span_from(start) });
            } else {
                let value = self.parse_expr().unwrap_or(Expr::Error(self.current_span()));
                args.push(CallArg { name: None, value, span: self.span_from(start) });
            }
            if !self.eat(&TokenKind::Comma) { break; }
        }
        self.expect(&TokenKind::RParen);
        args
    }

    // ============================================================
    // Operator helpers
    // ============================================================

    fn peek_binop(&self) -> Option<BinOp> {
        match self.peek_kind() {
            Some(TokenKind::Plus) => Some(BinOp::Add),
            Some(TokenKind::Minus) => Some(BinOp::Sub),
            Some(TokenKind::Star) => Some(BinOp::Mul),
            Some(TokenKind::Slash) => Some(BinOp::Div),
            Some(TokenKind::Percent) => Some(BinOp::Mod),
            Some(TokenKind::EqEq) => Some(BinOp::Eq),
            Some(TokenKind::NotEq) => Some(BinOp::NotEq),
            Some(TokenKind::Lt) => Some(BinOp::Lt),
            Some(TokenKind::Gt) => Some(BinOp::Gt),
            Some(TokenKind::LtEq) => Some(BinOp::LtEq),
            Some(TokenKind::GtEq) => Some(BinOp::GtEq),
            Some(TokenKind::And) => Some(BinOp::And),
            Some(TokenKind::Or) => Some(BinOp::Or),
            _ => None,
        }
    }

    // ============================================================
    // Token helpers
    // ============================================================

    fn parse_ident(&mut self) -> Option<Ident> {
        if let Some(TokenKind::Ident(_)) = self.peek_kind() {
            let tok = self.advance_tok();
            if let TokenKind::Ident(name) = tok.kind {
                return Some(Ident::new(name, tok.span));
            }
        }
        self.error("expected identifier");
        None
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn peek_is_ident(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Ident(_)))
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind().is_some_and(|k| std::mem::discriminant(k) == std::mem::discriminant(kind))
    }

    fn check_at(&self, offset: usize, kind: &TokenKind) -> bool {
        self.tokens.get(self.pos + offset)
            .is_some_and(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind))
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Option<()> {
        if self.eat(kind) {
            Some(())
        } else {
            self.error(&format!("expected {:?}", kind));
            None
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn advance_tok(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn at_eof(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.peek_kind(), Some(TokenKind::Eof))
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), Some(TokenKind::Newline | TokenKind::Comment(_))) {
            self.pos += 1;
        }
    }

    fn current_span(&self) -> Span {
        self.tokens.get(self.pos).map_or(Span::new(self.source.len(), self.source.len()), |t| t.span)
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }

    fn span_from(&self, start: Span) -> Span {
        Span::new(start.start, self.prev_span().end)
    }

    fn error(&mut self, message: &str) {
        self.errors.push(ParseError {
            message: message.to_string(),
            span: self.current_span(),
        });
    }
}

fn binop_bp(op: BinOp) -> (u8, u8) {
    match op {
        BinOp::Or => (1, 2),
        BinOp::And => (3, 4),
        BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => (5, 6),
        BinOp::Add | BinOp::Sub => (7, 8),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (9, 10),
    }
}

fn is_type_name(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_uppercase())
}

