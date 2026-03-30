---
title: "Backends"
version: "0.1.0"
category: "reference"
keywords: ["backend", "javascript", "llvm", "native", "wasm"]
ai_context: true
last_updated: "2026-03-30"
---

# Backends

Klar currently supports multiple code generation paths.

## JavaScript Backend

- Crate: `klar-codegen-js`
- Used by default for:
  - `klar build`
  - `klar run`
  - `klar test`
  - `klar bench`

## LLVM Backend

- Crate: `klar-codegen-llvm`
- Used for:
  - `klar build --target native`
  - `klar build --target llvm-ir`
  - `klar run --target native`

## WASM Backend

- Crate: `klar-codegen-wasm`
- WASM APIs exist in crate code.
- Not exposed as a top-level `klar` CLI target yet.

## Related Docs

- [CLI Reference](/docs/cli-reference)
- [Compiler Overview](/docs/compiler-overview)
