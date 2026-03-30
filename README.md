<p align="center">
  <img src="website/public/brand/klar-mascot.svg" width="120" alt="Klar mascot" />
</p>

<h1 align="center">Klar</h1>

<p align="center">
  <strong>The AI-first programming language.</strong><br>
  Designed for machines to write. Built for humans to trust.
</p>

<p align="center">
  <a href="https://klar.run">Website</a> &middot;
  <a href="https://klar.run/docs">Docs</a> &middot;
  <a href="https://discord.gg/klar">Discord</a> &middot;
  <a href="https://klar.run/docs/language-spec">Language Spec</a>
</p>

---

## Why Klar?

AI generates code in existing languages — and gets it wrong **35-40% of the time**. Not because the AI is bad, but because the languages are ambiguous. Multiple ways to do the same thing. Null. Exceptions. Implicit behavior.

Klar eliminates these problems by design:

- **No null** — `Option` type, always explicit
- **No exceptions** — `Result` type, must be handled
- **One way to do everything** — zero ambiguity for AI and humans
- **`@schema` auto-generation** — validation, serialization, OpenAPI from struct annotations
- **AI-readable errors** — compiler errors include machine-parseable fix suggestions

The result: **95% first-pass correctness** on our 500-task benchmark.

## Quick Start

```bash
curl -fsSL klar.run/install | sh
klar new my-app --template api
klar run
```

## Code Example

```
@schema
struct User {
    name: String       @min_len(1) @max_len(100)
    email: String      @format(email)
    age: Int           @range(0, 150)
    role: Role = Role.Member
}

fn create_user(req: Request) -> Response ! AppError {
    let input = req.json[User]()?
    let user = db.insert(input)?
    Response.json(user, status: 201)
}
```

14 lines. Validation, serialization, error handling — all generated from `@schema`. The equivalent TypeScript + Zod is 28 lines and ~700 tokens.

## Performance

Klar compiles to **native binaries via LLVM** and **JavaScript via transpilation**.

| Benchmark | Klar (native) | Node.js | Speedup |
|-----------|---------------|---------|---------|
| Fibonacci(40) | 0.20s | 0.59s | **~3x** |
| Fibonacci(35→40) | 0.51s | 1.33s | **~2.6x** |
| Ackermann(3,9)+(3,10) | 0.21s | 0.26s | **~1.2x** |

*Apple M-series, best of 5 runs, wall-clock time.*

## Project Structure

```
klar/
├── crates/               # Rust compiler source
│   ├── klar-compiler/    # Main compiler binary
│   ├── klar-lexer/       # Tokenizer
│   ├── klar-parser/      # Parser → AST
│   ├── klar-ast/         # AST types
│   ├── klar-typeck/      # Type checker + inference
│   ├── klar-codegen-js/  # JavaScript backend
│   ├── klar-codegen-llvm/# LLVM native backend
│   ├── klar-lsp/         # Language server
│   ├── klar-pkg/         # Package manager
│   └── klar-runtime/     # Runtime support
├── examples/             # Example programs
├── benchmark/            # AI correctness benchmark
├── editors/              # Editor integrations (VS Code)
├── website/              # klar.run source (Next.js)
└── LANGUAGE_SPEC.md      # AI instruction file (~3000 tokens)
```

## Language Spec for AI

Paste [`LANGUAGE_SPEC.md`](LANGUAGE_SPEC.md) into your AI system prompt, `.cursorrules`, or Claude Project. It gives any model everything it needs to generate correct Klar code — in ~3,000 tokens.

## Building from Source

```bash
# Prerequisites: Rust 1.78+, LLVM 18 (for native target)
git clone https://github.com/klar-lang/klar.git
cd klar
cargo build --release

# Run a .klar file (JS target)
cargo run -- run examples/hello.klar

# Run with native target (requires LLVM 18)
cargo run -- run --target native examples/bench_fib.klar
```

## Roadmap

| Phase | Timeline | Focus |
|-------|----------|-------|
| **01 — Proof of Concept** | Months 1-6 | Core language, JS target, benchmark (active) |
| 02 — Usable Language | Months 6-12 | LLVM backend, async, HTTP server, LSP |
| 03 — Production Backend | Months 12-18 | ORM, migrations, WebSocket, deploy |
| 04 — Ecosystem | Months 18-24 | Package registry, AI integrations, 1.0 |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines. We use conventional commits and squash merges.

## License

Apache 2.0 — see [LICENSE](LICENSE).

---

<p align="center">
  <sub>Less code. Fewer errors. Greener compute.</sub>
</p>
