# Klar — AI-First Programming Language

## Language Spec

**READ THIS FIRST**: The complete language specification is at `LANGUAGE_SPEC.md` in the repo root. You MUST read it before writing any Klar code.

## Project Structure

- `compiler/` — Rust compiler (lexer, parser, typechecker, JS + LLVM codegen)
- `compiler/crates/` — Individual compiler crates
- `compiler/examples/` — Example .klar programs
- `compiler/benchmark/` — AI correctness benchmark suite
- `website/` — kler.run website (Next.js)
- `LANGUAGE_SPEC.md` — Complete language spec for AI code generation

## Building the Compiler

```bash
cd compiler
cargo build --release
```

## Running Klar Code

```bash
# JS target (default)
cargo run -- run examples/hello.klar

# Native target (requires LLVM 18)
LIBRARY_PATH="$(brew --prefix zstd)/lib" cargo run -- build --target native examples/bench_fib.klar
```

## Key Language Rules

- NO null — use `Option` type (`T?`)
- NO exceptions — use `Result` type (`T ! E`)
- ONE loop for collections: `for item in collection { }`
- ONE infinite loop: `loop { break }`
- NO while loops
- Pattern matching must be exhaustive
- Last expression is the return value
- String interpolation: `"Hello {name}"`
- Pipe operator: `value |> fn1 |> fn2`
- Error propagation: `expr?`
