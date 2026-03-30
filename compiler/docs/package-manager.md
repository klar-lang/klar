# Package Manager

The `klar-pkg` crate powers manifest and lockfile operations used by:

- `klar init`
- `klar add`
- `klar remove`

## Files

- `klar.toml` — project manifest
- `klar.lock` — generated lockfile

## `klar.toml` Shape

Top-level sections:

- `[project]`
- `[dependencies]`
- `[dev-dependencies]`

Example:

```toml
[project]
name = "my-app"
version = "0.1.0"
license = "Apache-2.0"
klar-version = "0.1.0"

[dependencies]
http-server = "^1.0"
json-utils = { version = "2.1.0", git = "https://github.com/example/json-utils" }
```

Supported dependency forms:

- Simple version string (for example `"1.0.0"`, `"^1.2"`, `"*"`).
- Detailed table with fields such as `version`, `git`, `branch`, `tag`, `path`.

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

Behavior details:

- `add` and `remove` require `klar.toml` in current directory.
- Both regenerate `klar.lock`.
- Default added version if omitted is `*`.

## Lockfile Notes

`klar.lock` currently stores:

- lockfile format version
- resolved package name/version/source/checksum
- dependency list per package (currently simple model)

Checksums are deterministic and based on package name/version data.

## Resolver Status

- Current resolver is intentionally simple (greedy).
- Future upgrade path is noted in source (`PubGrub`-style improvements).

## Practical Advice

- Commit `klar.lock` for reproducible builds.
- Prefer explicit versions over `*` in production projects.
- If dependency resolution semantics change, update this document with examples.
