# LSP (`klar-lsp`)

`klar-lsp` is a standalone language server for Klar.

Binary crate:

- `crates/klar-lsp`

## Features (Current)

- Parse + type-check diagnostics on open/change
- Hover information (keywords, built-ins, inferred types when available)
- Completions:
  - language keywords
  - built-in types/functions
  - standard module names
  - local functions/structs/enums from current file
- Go-to-definition for symbols in current file

## Build and Run

From `compiler/`:

```bash
cargo build -p klar-lsp
cargo run -p klar-lsp
```

The server communicates over stdio using the LSP protocol.

## Editor Integration (Generic)

Configure your editor's Klar language client to launch:

```bash
<path-to-repo>/compiler/target/debug/klar-lsp
```

with stdio transport.

## Architecture Notes

- Full text sync mode (`TextDocumentSyncKind::FULL`)
- Diagnostics are published from parse/typecheck passes
- Definition resolution is currently file-local

## Troubleshooting

No diagnostics:

- Ensure file is recognized as Klar by the editor client
- Confirm client is actually spawning `klar-lsp`

No hover/completions:

- Verify text document change events are reaching server
- Ensure syntax parses (severe parse failures can degrade semantic features)
