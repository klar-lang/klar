---
title: "CLI Reference"
version: "0.1.0"
category: "reference"
keywords: ["cli", "build", "run", "check", "test", "lint"]
ai_context: true
last_updated: "2026-03-30"
---

# CLI Reference

## Common Commands

| Command | Purpose |
|---|---|
| `klar lex <file>` | Print token stream |
| `klar parse <file>` | Parse and summarize AST |
| `klar build <file>` | Compile file (default JS) |
| `klar run <file>` | Compile and execute |
| `klar test <file>` | Run test blocks |
| `klar check <file>` | Parse + type-check |
| `klar fmt <file>` | Format source |
| `klar lint <file>` | Emit lint warnings |
| `klar bench <file>` | Run benchmark-style tests |
| `klar doc <file>` | Generate markdown API docs to stdout |

## Build Targets

Default JS output:

```bash
klar build examples/hello.klar
```

Native binary:

```bash
klar build examples/native_hello.klar --target native
```

LLVM IR:

```bash
klar build examples/hello.klar --target llvm-ir
```

## JSON Diagnostics for Tools

```bash
klar check examples/demo.klar --format json
```

Returns `[]` on success or a JSON array of diagnostics on errors.

## Package Commands

```bash
klar init my-app
klar add http-server@1.0.0
klar remove http-server
```

## Related Docs

- [Diagnostics](/docs/diagnostics)
- [Package Manager](/docs/package-manager)
