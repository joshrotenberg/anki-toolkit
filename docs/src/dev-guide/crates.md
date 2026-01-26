# Crates Overview

## ankit

Complete async AnkiConnect API client.

```rust
use ankit::{AnkiClient, NoteBuilder};

let client = AnkiClient::new();
let note = NoteBuilder::new("Default", "Basic")
    .field("Front", "Question")
    .field("Back", "Answer")
    .build();
client.notes().add(note).await?;
```

[Full documentation](https://docs.rs/ankit)

## ankit-engine

High-level workflow operations built on ankit.

```rust
use ankit_engine::Engine;

let engine = Engine::new();
let stats = engine.analyze().study_summary("Japanese", 30).await?;
```

Features: import, export, organize, analyze, media, progress, enrich, deduplicate, backup.

[Full documentation](https://docs.rs/ankit-engine)

## ankit-builder

TOML-based deck builder with .apkg generation.

```rust
use ankit_builder::DeckBuilder;

let builder = DeckBuilder::from_file("deck.toml")?;
builder.write_apkg("deck.apkg")?;
// Or: builder.import_connect().await?;
```

Features: apkg generation, AnkiConnect import, bidirectional sync, markdown fields.

[Full documentation](https://docs.rs/ankit-builder)

## ankit-mcp

MCP server exposing 50 tools for AI assistants.

```bash
cargo install ankit-mcp
ankit-mcp --help
```

[Full documentation](https://docs.rs/ankit-mcp)
