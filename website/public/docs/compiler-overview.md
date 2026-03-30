---
title: "Compiler Overview"
version: "0.1.0"
category: "concepts"
keywords: ["compiler", "architecture", "pipeline", "crates"]
ai_context: true
last_updated: "2026-03-30"
---

# Compiler Overview

This page explains how the Klar compiler workspace is organized.

## Workspace Crates

- `klar-lexer`: tokenization and spans
- `klar-ast`: syntax tree data structures
- `klar-parser`: source to AST parsing
- `klar-typeck`: semantic/type checking
- `klar-codegen-js`: JavaScript backend
- `klar-codegen-llvm`: LLVM backend (native + IR)
- `klar-pkg`: manifest and lockfile support
- `klar-compiler`: `klar` CLI binary
- `klar-lsp`: language server binary

## Pipeline

For most compiler commands:

1. Lex source (`Lexer::tokenize`)
2. Parse tokens into AST (`klar_parser::parse`)
3. Type-check (`TypeChecker::check_program`)
4. Emit target code (JS or LLVM)

## Binaries

- `klar`: CLI for compile/run/check/test/tooling tasks
- `klar-lsp`: LSP server for editor integration

## Related Docs

- [CLI Reference](/docs/cli-reference)
- [Backends](/docs/backends)
- [Compiler Contributing](/docs/compiler-contributing)
