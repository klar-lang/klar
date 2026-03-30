---
title: "Getting Started with Klar"
version: "0.1.0"
category: "guide"
keywords: ["install", "setup", "hello world", "tutorial", "quickstart"]
ai_context: true
last_updated: "2026-03-30"
---

# Getting Started with Klar

## Install

```bash
# macOS / Linux
curl -fsSL https://klar.dev/install | sh

# Verify installation
klar --version
```

## Create a Project

```bash
klar new my-app
cd my-app
```

This creates:

```
my-app/
├── klar.toml          # Project manifest
├── src/
│   └── main.klar      # Entry point
└── test/
    └── main_test.klar  # Tests
```

## Hello World

```klar
// src/main.klar
fn main() {
    println("Hello, world!")
}
```

```bash
klar run
# Output: Hello, world!
```

## Your First API

```bash
klar new my-api --template api
cd my-api
```

```klar
// src/main.klar
use std.http.{Router, Request, Response, serve}

@schema
struct Greeting {
    message: String
    timestamp: DateTime
}

fn main() ! ServerError {
    let router = Router.new()
        |> Router.get("/hello/{name}", hello)
    serve(router, port: 3000)?
}

fn hello(req: Request) -> Response ! AppError {
    let name = req.param("name")
    let greeting = Greeting {
        message: "Hello, {name}!",
        timestamp: time.now(),
    }
    Response.json(greeting)
}
```

```bash
klar run
# Server running on :3000

curl localhost:3000/hello/world
# {"message":"Hello, world!","timestamp":"2026-03-30T12:00:00Z"}
```

## AI Setup

To enable AI code generation for Klar, add the language spec to your AI tool:

### Claude Code / Claude Projects
Copy `LANGUAGE_SPEC.md` (shipped with the compiler at `~/.klar/LANGUAGE_SPEC.md`) into your Claude Project knowledge.

### Cursor
Copy to `.cursorrules` in your project root:
```bash
cp ~/.klar/LANGUAGE_SPEC.md .cursorrules
```

### VS Code
Install the Klar extension from the VS Code Marketplace. It includes the LSP for real-time diagnostics, completions, and formatting.

## Toolchain Commands

| Command | Description |
|---------|-------------|
| `klar new <name>` | Create a new project |
| `klar build` | Compile the project |
| `klar run` | Build and execute |
| `klar test` | Run all tests |
| `klar fmt` | Format source code |
| `klar lint` | Run static analysis |
| `klar add <pkg>` | Add a dependency |
| `klar check` | Type-check without building |
| `klar doc --open` | Generate and view docs |
| `klar repl` | Interactive REPL |

## Next Steps

- [Language Specification](/docs/language-spec) — Full syntax and type system reference
- [Standard Library](/docs/stdlib) — Module-by-module API reference
- [Examples](/docs/examples) — Real-world code samples
- [Benchmark Results](/docs/benchmark) — AI correctness data
