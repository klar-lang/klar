# Klar Compiler Workspace

Klar is an AI-first programming language and toolchain.

This directory contains the Rust compiler workspace (CLI, parser, checker, codegen, package manager, and LSP).

## Quick Start (Local Dev)

```bash
cd compiler
cargo build -p klar-compiler
./target/debug/klar --help
```

Run a sample:

```bash
./target/debug/klar run examples/hello.klar
```

## Practical Requirements

- Rust + Cargo
- Node.js (required for default JS execution paths: `run`, `test`, `bench`)
- LLVM toolchain support (for native and LLVM IR paths)

## CLI Commands

Core:

- `klar lex <file>`
- `klar parse <file>`
- `klar build <file> [--target native|llvm-ir]`
- `klar run <file> [--target native]`
- `klar check <file> [--format json]`
- `klar test <file>`
- `klar fmt <file> [--check]`
- `klar lint <file>`
- `klar bench <file>`
- `klar doc <file>`

Package manager:

- `klar init [name]`
- `klar add <pkg>[@version]`
- `klar remove <pkg>`

Full command details: [`docs/cli.md`](./docs/cli.md)

## Documentation Map

- Start here: [`docs/README.md`](./docs/README.md)
- Architecture: [`docs/architecture.md`](./docs/architecture.md)
- Contributing workflow: [`docs/contributing.md`](./docs/contributing.md)
- Backends: [`docs/backends.md`](./docs/backends.md)
- Package manager: [`docs/package-manager.md`](./docs/package-manager.md)
- Diagnostics (`klar check --format json`): [`docs/diagnostics.md`](./docs/diagnostics.md)
- Formatting caveats: [`docs/formatting.md`](./docs/formatting.md)
- LSP server: [`docs/lsp.md`](./docs/lsp.md)

## Language Spec

Language reference for AI and humans:

- [`LANGUAGE_SPEC.md`](./LANGUAGE_SPEC.md)

## Benchmark

Benchmark docs and structure:

- [`benchmark/README.md`](./benchmark/README.md)

## License

Apache 2.0 (declared in workspace metadata and crate manifests).
