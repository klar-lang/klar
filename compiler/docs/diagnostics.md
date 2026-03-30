# Diagnostics

`klar check` provides two output modes:

- Human-readable CLI diagnostics (default)
- Machine-readable JSON diagnostics (`--format json`)

## Commands

```bash
# human output
klar check path/to/file.klar

# machine output
klar check path/to/file.klar --format json
```

## JSON Contract (Current)

Each diagnostic includes:

- `code` (for example `E001`, `E020`)
- `severity` (currently `error`)
- `message`
- `location` object: `file`, `line`, `col`
- `actions` array
- `fixes` array (action + description)

Success output in JSON mode:

```json
[]
```

## Current Diagnostic Codes

These are inferred from the current `categorize_type_error` logic:

- `E001` parse/syntax errors
- `E010` undefined name/import shape
- `E020` type mismatch
- `E030` argument count/signature mismatch
- `E040` invalid operator operand types
- `E050` non-iterable used in iteration context
- `E099` fallback catch-all

## Practical AI Integration

Recommended flow:

1. Run `klar check file.klar --format json`
2. Parse diagnostics array
3. Apply one or more listed `actions`
4. Re-run `check` until output is `[]`

## Caveats

- Diagnostic categorization is message-pattern based today.
- If type error wording changes, code classification may shift.
- Keep tooling resilient to new/unknown codes.
