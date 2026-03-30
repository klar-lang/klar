---
title: "Formatting"
version: "0.1.0"
category: "guide"
keywords: ["format", "fmt", "style", "ci"]
ai_context: true
last_updated: "2026-03-30"
---

# Formatting

Formatting command:

```bash
klar fmt <file.klar>
klar fmt <file.klar> --check
```

## Current Behavior

The formatter currently works by parsing into AST and re-emitting code.

## Important Caveat

Function and test bodies are currently emitted as placeholders in formatter output.

Use `fmt` with this behavior in mind until body-preserving formatting lands.

## CI Usage

`klar fmt --check` exits non-zero when formatting differs from source.

## Related Docs

- [CLI Reference](/docs/cli-reference)
