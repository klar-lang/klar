---
title: "Language Server (LSP)"
version: "0.1.0"
category: "guide"
keywords: ["lsp", "editor", "diagnostics", "hover", "completion"]
ai_context: true
last_updated: "2026-03-30"
---

# Language Server (LSP)

Klar ships an LSP server binary: `klar-lsp`.

## Features

- Parse and type-check diagnostics
- Hover info
- Completions
- Go-to-definition (file-local)

## Build and Run

```bash
cd compiler
cargo build -p klar-lsp
cargo run -p klar-lsp
```

The server uses stdio transport (standard LSP setup).

## Editor Integration

Configure your editor client to launch:

```bash
<repo>/compiler/target/debug/klar-lsp
```

for Klar files.

## Related Docs

- [Compiler Overview](/docs/compiler-overview)
