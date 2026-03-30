# Formatting

Formatter command:

```bash
klar fmt <file.klar>
klar fmt <file.klar> --check
```

## Current Implementation

`klar fmt` is currently an AST re-emitter:

1. Parse source into AST
2. Reconstruct text from AST nodes
3. Write output back (or compare in `--check`)

## Important Caveat

Function and test bodies are currently emitted as placeholders:

```klar
// ...
```

That means this formatter is **not yet lossless** for statement-level details.

## Practical Usage Guidance

Use `klar fmt` today when:

- validating parser/AST shape in development
- testing formatting pipeline behavior

Avoid using it as a final source formatter for production code until body-preserving formatting is implemented.

## `--check` in CI

`klar fmt --check` exits non-zero when formatted output differs from the source file.

Given current caveats, enable this gate only if your workflow expects current formatter behavior.
