# CLI Reference (`klar`)

This reference documents the current behavior of `klar` from `crates/klar-compiler/src/main.rs`.

## Install / Build Locally

From `compiler/`:

```bash
cargo build -p klar-compiler
./target/debug/klar --help
```

## Command Overview

| Command | Purpose |
|---|---|
| `klar lex <file>` | Tokenize a file and print token stream |
| `klar parse <file>` | Parse and print AST summary counts |
| `klar build <file> [target args]` | Build to JS (default), native, or LLVM IR |
| `klar run <file> [target args]` | Compile and execute via JS+Node (default) or native |
| `klar test <file>` | Run `test` blocks through JS backend + Node |
| `klar check <file> [--format json]` | Parse/type-check only, optional machine JSON diagnostics |
| `klar fmt <file> [--check]` | AST-based formatter |
| `klar lint <file>` | Style + simple static warnings |
| `klar bench <file>` | Run `test bench_*` style benchmarks |
| `klar doc <file>` | Print markdown API docs to stdout |
| `klar init [name]` | Create `klar.toml` in current directory |
| `klar add <pkg>[@version]` | Add dependency to `klar.toml` and regenerate `klar.lock` |
| `klar remove <pkg>` | Remove dependency and regenerate lockfile |

## Build Targets

Default:

```bash
klar build examples/hello.klar
# writes examples/hello.js
```

Native:

```bash
klar build examples/native_hello.klar --target native
# writes examples/native_hello (platform binary)
```

LLVM IR:

```bash
klar build examples/hello.klar --target llvm-ir
# writes examples/hello.ll
```

Notes:

- Native and LLVM IR flows require LLVM toolchain support in your environment.
- CLI target parsing is string-based; practical usage is `--target native` or `--target llvm-ir`.

## `run` Modes

Default JS mode:

```bash
klar run examples/hello.klar
```

- Generates JS to a temp file (`klar_output.js`)
- Executes with `node`
- Fails with a clear message if Node is not installed

Native mode:

```bash
klar run examples/native_hello.klar --target native
```

- Compiles temporary native binary (`klar_native`) and executes it

## `check` for AI / tooling

Human-readable mode:

```bash
klar check examples/demo.klar
```

JSON diagnostics mode:

```bash
klar check examples/demo.klar --format json
```

On success:

```json
[]
```

On failure: JSON array of diagnostics with fields including `code`, `severity`, `message`, `location`, `actions`, `fixes`.

## `fmt` behavior

Write-in-place:

```bash
klar fmt examples/demo.klar
```

Check mode:

```bash
klar fmt examples/demo.klar --check
```

See [Formatting](./formatting.md) for formatter limitations and current behavior.

## Package commands

Initialize:

```bash
klar init my-app
```

Add dependency:

```bash
klar add http-server@1.0.0
```

Remove dependency:

```bash
klar remove http-server
```

See [Package Manager](./package-manager.md).

## Exit behavior

- Parse/type/check failures exit with non-zero status.
- `fmt --check` exits non-zero when formatting changes are needed.
- `check --format json` still exits non-zero on diagnostics (useful for CI and tool integration).
