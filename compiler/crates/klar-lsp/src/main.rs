//! Klar Language Server Protocol (LSP) implementation.
//!
//! Provides real-time diagnostics, hover information, go-to-definition,
//! and basic completions for Klar source files.

use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use klar_lexer::{Lexer, TokenKind, Span};

struct KlarLanguageServer {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
}

impl KlarLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
        }
    }

    async fn analyze(&self, uri: &Url, text: &str) {
        let mut diagnostics = Vec::new();

        // Parse
        match klar_parser::parse(text) {
            Ok(program) => {
                // Type check
                let mut checker = klar_typeck::TypeChecker::new();
                if let Err(errors) = checker.check_program(&program) {
                    for err in errors {
                        let (line, col) = offset_to_position(text, err.span.start);
                        let (end_line, end_col) = offset_to_position(text, err.span.end);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position::new(line, col),
                                end: Position::new(end_line, end_col),
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("klar".into()),
                            message: err.message,
                            ..Default::default()
                        });
                    }
                }
            }
            Err(errors) => {
                for err in errors {
                    let (line, col) = offset_to_position(text, err.span.start);
                    let (end_line, end_col) = offset_to_position(text, err.span.end);
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position::new(line, col),
                            end: Position::new(end_line, end_col),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("klar".into()),
                        message: err.message,
                        ..Default::default()
                    });
                }
            }
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    fn get_hover_info(&self, text: &str, position: Position) -> Option<String> {
        let offset = position_to_offset(text, position.line, position.character)?;

        // Find the token at this position
        let tokens = Lexer::tokenize(text);
        let mut token_at_pos = None;

        for tok in &tokens {
            if tok.span.start <= offset && offset < tok.span.end {
                token_at_pos = Some(tok);
                break;
            }
        }

        let tok = token_at_pos?;
        let token_text = &text[tok.span.start..tok.span.end];

        // Try to find type information
        if let Ok(program) = klar_parser::parse(text) {
            let mut checker = klar_typeck::TypeChecker::new();
            let _ = checker.check_program(&program);

            // Look up the type of the identifier at this position
            if let Some(ty) = checker.type_at_offset(offset) {
                return Some(format!("```klar\n{}: {}\n```", token_text, ty));
            }

            // Check if it's a keyword
            match token_text {
                "fn" => return Some("**fn** — Function declaration".into()),
                "let" => return Some("**let** — Immutable variable binding".into()),
                "mut" => return Some("**mut** — Mutable variable modifier".into()),
                "struct" => return Some("**struct** — Struct type declaration".into()),
                "enum" => return Some("**enum** — Enum type declaration".into()),
                "match" => return Some("**match** — Pattern matching expression".into()),
                "if" => return Some("**if** — Conditional expression".into()),
                "for" => return Some("**for** — Loop over iterable".into()),
                "return" => return Some("**return** — Early return from function".into()),
                "use" => return Some("**use** — Import declaration".into()),
                "test" => return Some("**test** — Test block declaration".into()),
                "trait" => return Some("**trait** — Trait declaration".into()),
                "impl" => return Some("**impl** — Trait implementation".into()),
                _ => {}
            }

            // Check built-in types
            match token_text {
                "Int" => return Some("**Int** — 64-bit signed integer".into()),
                "Float" => return Some("**Float** — 64-bit IEEE floating point".into()),
                "Bool" => return Some("**Bool** — Boolean (true/false)".into()),
                "String" => return Some("**String** — UTF-8 immutable string".into()),
                "Byte" => return Some("**Byte** — 8-bit unsigned integer".into()),
                _ => {}
            }

            // Check built-in functions
            match token_text {
                "println" => return Some("```klar\nfn println(value: Any) -> ()\n```\nPrint value followed by newline".into()),
                "print" => return Some("```klar\nfn print(value: Any) -> ()\n```\nPrint value without newline".into()),
                "assert" => return Some("```klar\nfn assert(condition: Bool) -> ()\n```\nAssert condition is true, abort if false".into()),
                "assert_eq" => return Some("```klar\nfn assert_eq(a: T, b: T) -> ()\n```\nAssert two values are equal".into()),
                _ => {}
            }
        }

        Some(format!("`{}`", token_text))
    }

    fn get_completions(&self, text: &str, position: Position) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Keywords
        let keywords = [
            ("fn", "Function declaration", CompletionItemKind::KEYWORD),
            ("let", "Variable binding", CompletionItemKind::KEYWORD),
            ("let mut", "Mutable variable binding", CompletionItemKind::KEYWORD),
            ("struct", "Struct declaration", CompletionItemKind::KEYWORD),
            ("enum", "Enum declaration", CompletionItemKind::KEYWORD),
            ("match", "Pattern matching", CompletionItemKind::KEYWORD),
            ("if", "Conditional expression", CompletionItemKind::KEYWORD),
            ("else", "Else branch", CompletionItemKind::KEYWORD),
            ("for", "For loop", CompletionItemKind::KEYWORD),
            ("in", "In operator", CompletionItemKind::KEYWORD),
            ("return", "Return from function", CompletionItemKind::KEYWORD),
            ("use", "Import declaration", CompletionItemKind::KEYWORD),
            ("test", "Test block", CompletionItemKind::KEYWORD),
            ("trait", "Trait declaration", CompletionItemKind::KEYWORD),
            ("impl", "Trait implementation", CompletionItemKind::KEYWORD),
            ("true", "Boolean true", CompletionItemKind::KEYWORD),
            ("false", "Boolean false", CompletionItemKind::KEYWORD),
            ("not", "Logical NOT", CompletionItemKind::KEYWORD),
            ("and", "Logical AND", CompletionItemKind::KEYWORD),
            ("or", "Logical OR", CompletionItemKind::KEYWORD),
        ];

        for (label, detail, kind) in &keywords {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(*kind),
                detail: Some(detail.to_string()),
                ..Default::default()
            });
        }

        // Types
        for ty in &["Int", "Float", "Bool", "String", "Byte", "List", "Map", "Set", "Option", "Result"] {
            items.push(CompletionItem {
                label: ty.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                ..Default::default()
            });
        }

        // Built-in functions
        for func in &["println", "print", "assert", "assert_eq", "read_line", "to_string", "to_int"] {
            items.push(CompletionItem {
                label: func.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            });
        }

        // Std modules
        for module in &["std", "string", "list", "map", "set", "json", "math", "io", "env", "time", "crypto", "log", "http", "sql"] {
            items.push(CompletionItem {
                label: module.to_string(),
                kind: Some(CompletionItemKind::MODULE),
                ..Default::default()
            });
        }

        // Parse current file for local definitions
        if let Ok(program) = klar_parser::parse(text) {
            for item in &program.items {
                match item {
                    klar_ast::Item::Function(f) => {
                        items.push(CompletionItem {
                            label: f.name.name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!("fn {}(...)", f.name.name)),
                            ..Default::default()
                        });
                    }
                    klar_ast::Item::Struct(s) => {
                        items.push(CompletionItem {
                            label: s.name.name.clone(),
                            kind: Some(CompletionItemKind::STRUCT),
                            detail: Some(format!("struct {}", s.name.name)),
                            ..Default::default()
                        });
                    }
                    klar_ast::Item::Enum(e) => {
                        items.push(CompletionItem {
                            label: e.name.name.clone(),
                            kind: Some(CompletionItemKind::ENUM),
                            detail: Some(format!("enum {}", e.name.name)),
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
        }

        items
    }

    fn get_definitions(&self, text: &str, position: Position) -> Vec<Location> {
        let offset = match position_to_offset(text, position.line, position.character) {
            Some(o) => o,
            None => return vec![],
        };

        let tokens = Lexer::tokenize(text);
        let mut target_name = None;

        for tok in &tokens {
            if tok.span.start <= offset && offset < tok.span.end {
                if let TokenKind::Ident(_) = &tok.kind {
                    target_name = Some(text[tok.span.start..tok.span.end].to_string());
                }
                break;
            }
        }

        let name = match target_name {
            Some(n) => n,
            None => return vec![],
        };

        let mut locations = Vec::new();

        if let Ok(program) = klar_parser::parse(text) {
            for item in &program.items {
                let (item_name, span) = match item {
                    klar_ast::Item::Function(f) => (&f.name.name, f.name.span),
                    klar_ast::Item::Struct(s) => (&s.name.name, s.name.span),
                    klar_ast::Item::Enum(e) => (&e.name.name, e.name.span),
                    klar_ast::Item::Trait(t) => (&t.name.name, t.name.span),
                    _ => continue,
                };

                if item_name == &name {
                    let (line, col) = offset_to_position(text, span.start);
                    let (end_line, end_col) = offset_to_position(text, span.end);
                    locations.push(Location {
                        uri: Url::parse("file:///").unwrap(), // Will be replaced with actual URI
                        range: Range {
                            start: Position::new(line, col),
                            end: Position::new(end_line, end_col),
                        },
                    });
                }
            }
        }

        locations
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for KlarLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), ":".into()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "klar-lsp".into(),
                version: Some("0.1.0".into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Klar Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        self.documents.lock().unwrap().insert(uri.clone(), text.clone());
        self.analyze(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text.clone();
            self.documents.lock().unwrap().insert(uri.clone(), text.clone());
            self.analyze(&uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.lock().unwrap().remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.lock().unwrap();
        let text = match docs.get(uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        drop(docs);

        if let Some(info) = self.get_hover_info(&text, position) {
            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: info,
                }),
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.lock().unwrap();
        let text = match docs.get(uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let items = self.get_completions(&text, position);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.lock().unwrap();
        let text = match docs.get(uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let mut locations = self.get_definitions(&text, position);
        // Replace dummy URI with actual URI
        for loc in &mut locations {
            loc.uri = uri.clone();
        }

        if locations.is_empty() {
            Ok(None)
        } else if locations.len() == 1 {
            Ok(Some(GotoDefinitionResponse::Scalar(locations.remove(0))))
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }
}

fn offset_to_position(text: &str, offset: usize) -> (u32, u32) {
    let mut line: u32 = 0;
    let mut col: u32 = 0;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn position_to_offset(text: &str, line: u32, col: u32) -> Option<usize> {
    let mut current_line: u32 = 0;
    let mut current_col: u32 = 0;
    for (i, ch) in text.char_indices() {
        if current_line == line && current_col == col {
            return Some(i);
        }
        if ch == '\n' {
            if current_line == line {
                return Some(i);
            }
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
    }
    if current_line == line && current_col == col {
        Some(text.len())
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| KlarLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
