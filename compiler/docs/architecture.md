# Compiler Architecture

This document maps the current Klar compiler workspace and how source flows through it.

## Workspace Layout

Root workspace file: `Cargo.toml`

Workspace members:

- `crates/klar-lexer`
- `crates/klar-ast`
- `crates/klar-parser`
- `crates/klar-typeck`
- `crates/klar-codegen-js`
- `crates/klar-codegen-llvm`
- `crates/klar-runtime`
- `crates/klar-pkg`
- `crates/klar-compiler` (binary: `klar`)
- `crates/klar-lsp` (binary: `klar-lsp`)

## Compilation Pipeline

For the `klar` CLI, the common pipeline is:

1. **Lexing** via `klar_lexer::Lexer::tokenize`
2. **Parsing** via `klar_parser::parse` into `klar_ast::Program`
3. **Type checking** via `klar_typeck::TypeChecker::check_program`
4. **Code generation**:
   - JS via `klar_codegen_js::generate`
   - LLVM IR / native binary via `klar_codegen_llvm`

## Crate Roles

### `klar-lexer`

- Tokenization and spans
- Source offsets used by parser/checker diagnostics

### `klar-ast`

- Core AST types (`Program`, `Item`, `Expr`, `Stmt`, etc.)
- Shared by parser, checker, and codegen crates

### `klar-parser`

- Public entrypoint: `parse(source: &str) -> Result<Program, Vec<ParseError>>`
- Converts token stream to typed AST

### `klar-typeck`

- Semantic checks and type inference/validation
- Reports `TypeError` values with spans
- Used both by CLI and LSP diagnostics

### `klar-codegen-js`

- JS backend used by default for `build`, `run`, `test`, `bench`

### `klar-codegen-llvm`

- LLVM backend:
  - `generate_ir`
  - `compile_to_object`
  - `compile_to_native`

### `klar-runtime`

- Runtime helpers for native backend (`staticlib`)

### `klar-pkg`

- Manifest model for `klar.toml`
- Lockfile model for `klar.lock`
- Commands used by CLI: `init`, `add_dependency`, `remove_dependency`

### `klar-lsp`

- LSP server over stdio
- Publishes parse/type diagnostics
- Hover, completion, and go-to-definition support

### `klar-compiler`

- End-user `klar` binary
- Command dispatch and user-facing diagnostics

## CLI and LSP Relationship

- `klar` and `klar-lsp` are separate binaries.
- `klar` is task-oriented (build/run/check/test/etc.).
- `klar-lsp` is editor-oriented (incremental diagnostics, hover, completion, goto definition).

## Backends at a Glance

- **Default backend in CLI**: JS
- **Native backend in CLI**: LLVM (`native` target)
- **LLVM IR in CLI**: `llvm-ir` target
