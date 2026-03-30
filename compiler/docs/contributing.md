# Contributing

Practical contributor workflow for the Klar compiler workspace.

## Prerequisites

- Rust toolchain (edition 2024 project)
- `cargo`
- Node.js (required for default JS execution paths: `run`, `test`, `bench`)
- LLVM toolchain support for native/IR flows (`klar-codegen-llvm`)

## Repository Workflow

From `compiler/`:

```bash
# build everything in workspace
cargo build

# run tests
cargo test
```

Targeted loops:

```bash
# compile CLI binary
cargo build -p klar-compiler

# run lexer tests only
cargo test -p klar-lexer

# run parser tests only
cargo test -p klar-parser

# run type checker tests only
cargo test -p klar-typeck
```

## Running the CLI Locally

```bash
./target/debug/klar --help
./target/debug/klar check examples/demo.klar
./target/debug/klar run examples/hello.klar
```

## Running the LSP Locally

```bash
cargo run -p klar-lsp
```

The server communicates over stdio (LSP protocol).

## Suggested Development Order

When implementing language/compiler changes:

1. Update `klar-lexer` (if token-level changes are needed)
2. Update `klar-ast` (if syntax model changes)
3. Update `klar-parser`
4. Update `klar-typeck`
5. Update backends (`klar-codegen-js`, optionally LLVM)
6. Validate via `klar-compiler` commands on `examples/*.klar`
7. Update docs in `docs/` and language-facing docs as needed

## Practical Validation Checklist

- `klar parse` succeeds for updated syntax
- `klar check` catches intended type errors
- `klar run` works in JS mode
- `klar build --target native` works if native path changed
- `klar fmt --check` behavior is known and acceptable
- tests for touched crate(s) are updated and passing

## Documentation Expectations

When behavior changes, update:

- `docs/cli.md` for command semantics/flags
- `docs/architecture.md` for pipeline changes
- `docs/diagnostics.md` for machine-readable `check` output changes
