---
title: "Package Manager"
version: "0.1.0"
category: "guide"
keywords: ["package", "klar.toml", "klar.lock", "dependencies"]
ai_context: true
last_updated: "2026-03-30"
---

# Package Manager

Klar package commands manage `klar.toml` and `klar.lock`.

## Files

- `klar.toml`: project metadata and dependencies
- `klar.lock`: resolved dependency snapshot

## Commands

Initialize manifest:

```bash
klar init my-app
```

Add dependency:

```bash
klar add http-server@1.0.0
```

Remove dependency:

```bash
klar remove http-server
```

## Dependency Format

Simple form:

```toml
[dependencies]
http-server = "^1.0"
```

Detailed form:

```toml
[dependencies]
json-utils = { version = "2.1.0", git = "https://github.com/example/json-utils" }
```

## Related Docs

- [CLI Reference](/docs/cli-reference)
