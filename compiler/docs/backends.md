# Backends

Klar has two backend crates for code generation.

## JS Backend (`klar-codegen-js`)

- Public API: `generate(program: &Program) -> String`
- Used by CLI commands:
  - `klar build` (default output: `.js`)
  - `klar run` (default execution path via Node)
  - `klar test`
  - `klar bench`

Practical implication:

- Node.js is required for execution-oriented JS paths.

## LLVM Backend (`klar-codegen-llvm`)

Key APIs:

- `generate_ir`
- `compile_to_object`
- `compile_to_native`

Used by CLI commands:

- `klar build --target native`
- `klar build --target llvm-ir`
- `klar run --target native`

Practical implication:

- Native and LLVM IR workflows depend on LLVM/inkwell toolchain setup.

## Runtime (`klar-runtime`)

- Runtime helper crate (`staticlib`) for native code support
- Part of workspace and relevant for native backend evolution

## Choosing a Backend During Development

- Fast inner loop: JS backend (`klar run`, `klar test`)
- Native correctness/performance checks: LLVM native path

## Known Integration Gap

If you add a new backend feature:

1. Add crate-level API/tests
2. Wire CLI options in `crates/klar-compiler/src/main.rs`
3. Document the feature in `docs/cli.md` and this file
