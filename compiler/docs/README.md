# Compiler Documentation

Practical documentation for working with the Klar compiler workspace.

## Start Here

- New contributor: [Contributing](./contributing.md)
- CLI usage: [CLI Reference](./cli.md)
- Codebase internals: [Architecture](./architecture.md)

## Guides

- [Backends](./backends.md) (JS, LLVM native, LLVM IR)
- [Package Manager](./package-manager.md) (`klar.toml`, `klar.lock`, `init/add/remove`)
- [Diagnostics](./diagnostics.md) (`klar check` text and JSON output)
- [Formatting](./formatting.md) (`klar fmt` behavior and caveats)
- [LSP](./lsp.md) (`klar-lsp` capabilities and local setup)

## Scope

These docs reflect behavior in the current source tree under `crates/` and the `klar` CLI implemented in `crates/klar-compiler/src/main.rs`.
