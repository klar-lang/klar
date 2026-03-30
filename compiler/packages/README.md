# Official Klar Packages

These are the official seed packages for the Klar ecosystem. Each package demonstrates idiomatic Klar patterns and serves as a template for community package authors.

## Available Packages

| Package | Description | Status |
|---------|-------------|--------|
| `klar-uuid` | UUID v4 generation and parsing | Built-in (`std.crypto.uuid`) |
| `klar-dotenv` | Load `.env` files into environment | Built-in (`std.env`) |
| `klar-cors` | CORS middleware for HTTP server | Built-in (`std.http.cors`) |
| `klar-jwt` | JSON Web Token encode/decode | Built-in (`std.auth`) |
| `klar-rate-limit` | Rate limiting middleware | Planned |
| `klar-retry` | Retry with exponential backoff | Planned |
| `klar-csv` | CSV parsing and generation | Planned |
| `klar-argparse` | CLI argument parsing | Planned |
| `klar-validate` | Schema validation utilities | Built-in (`@schema`) |
| `klar-websocket` | WebSocket server and client | Built-in (`std.ws`) |

## Note

Many common utilities are already built into Klar's standard library. The packages listed as "Built-in" are available without installation. External packages will be published to `packages.klar.dev` when the registry launches.
