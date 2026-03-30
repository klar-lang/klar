---
title: "Diagnostics"
version: "0.1.0"
category: "reference"
keywords: ["diagnostics", "errors", "json", "check", "ai"]
ai_context: true
last_updated: "2026-03-30"
---

# Diagnostics

`klar check` supports human-readable and machine-readable diagnostics.

## Human Mode

```bash
klar check examples/demo.klar
```

Prints formatted errors with location and fix suggestions.

## JSON Mode

```bash
klar check examples/demo.klar --format json
```

Success output:

```json
[]
```

Error output includes:

- `code`
- `severity`
- `message`
- `location` (`file`, `line`, `col`)
- `actions`
- `fixes`

## Current Code Families

- `E001`: parse errors
- `E010`: undefined name/import issues
- `E020`: type mismatch
- `E030`: argument mismatch
- `E040`: invalid operator types
- `E050`: iteration type issues
- `E099`: fallback category

## Related Docs

- [CLI Reference](/docs/cli-reference)
