# Developer Guide Overview

The ankit workspace provides Rust crates for building Anki integrations.

## Crates

| Crate | Purpose | Docs |
|-------|---------|------|
| `ankit` | AnkiConnect API client | [docs.rs/ankit](https://docs.rs/ankit) |
| `ankit-engine` | High-level workflows | [docs.rs/ankit-engine](https://docs.rs/ankit-engine) |
| `ankit-builder` | TOML deck builder | [docs.rs/ankit-builder](https://docs.rs/ankit-builder) |
| `ankit-mcp` | MCP server | [docs.rs/ankit-mcp](https://docs.rs/ankit-mcp) |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ankit = "0.1"
```

For high-level workflows:

```toml
[dependencies]
ankit-engine = "0.1"
```

For TOML deck building:

```toml
[dependencies]
ankit-builder = "0.1"
```

## Architecture

```
ankit-mcp (MCP Server)
    |
    +-- ankit-engine (Workflows)
    |       |
    |       +-- ankit (API Client)
    |
    +-- ankit-builder (TOML Builder)
            |
            +-- ankit (for AnkiConnect import)
```

- **ankit** provides 1:1 AnkiConnect API bindings
- **ankit-engine** composes multiple API calls into workflows
- **ankit-builder** handles TOML parsing and .apkg generation
- **ankit-mcp** exposes everything as MCP tools

## Requirements

- Rust 1.85+ (Edition 2024)
- Anki with AnkiConnect add-on (for runtime)

## Next Steps

- [Crates Overview](crates.md)
- [ankit (API Client)](ankit.md)
- [ankit-engine (Workflows)](ankit-engine.md)
- [ankit-builder (Deck Builder)](ankit-builder.md)
- [Full API Documentation](api-docs.md)
