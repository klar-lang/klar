---
title: "Compiler Contributing"
version: "0.1.0"
category: "guide"
keywords: ["contributing", "compiler", "rust", "cargo", "workflow"]
ai_context: true
last_updated: "2026-03-30"
---

# Compiler Contributing

## Prerequisites

- Rust + Cargo
- Node.js (for default JS run/test/bench paths)
- LLVM toolchain (for native and LLVM IR paths)

## Local Build and Test

```bash
cd compiler
cargo build
cargo test
```

Run CLI:

```bash
cargo build -p klar-compiler
./target/debug/klar --help
./target/debug/klar check examples/demo.klar
```

## Recommended Change Order

When changing language behavior:

1. Lexer (if token changes are needed)
2. AST (if model changes)
3. Parser
4. Type checker
5. Codegen backend(s)
6. CLI integration
7. Docs updates

## Related Docs

- [Compiler Overview](/docs/compiler-overview)
- [Backends](/docs/backends)
- [CLI Reference](/docs/cli-reference)
