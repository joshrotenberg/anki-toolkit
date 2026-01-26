# ankit

A Rust toolkit for programmatic Anki deck management via AnkiConnect.

[![License](https://img.shields.io/crates/l/ankit.svg)](https://github.com/joshrotenberg/ankit#license)

## Crates

| Crate | Description | Crates.io |
|-------|-------------|-----------|
| [ankit](crates/ankit) | Complete async AnkiConnect API client | [![Crates.io](https://img.shields.io/crates/v/ankit.svg)](https://crates.io/crates/ankit) |
| [ankit-engine](crates/ankit-engine) | High-level workflow operations | [![Crates.io](https://img.shields.io/crates/v/ankit-engine.svg)](https://crates.io/crates/ankit-engine) |
| [ankit-builder](crates/ankit-builder) | TOML-based deck builder with .apkg generation | [![Crates.io](https://img.shields.io/crates/v/ankit-builder.svg)](https://crates.io/crates/ankit-builder) |
| [ankit-mcp](crates/ankit-mcp) | MCP server for AI assistant integration | [![Crates.io](https://img.shields.io/crates/v/ankit-mcp.svg)](https://crates.io/crates/ankit-mcp) |

## Quick Start

For direct API access, use `ankit`:

```rust
use ankit::{AnkiClient, NoteBuilder};

#[tokio::main]
async fn main() -> ankit::Result<()> {
    let client = AnkiClient::new();

    // Add a note
    let note = NoteBuilder::new("Default", "Basic")
        .field("Front", "Question")
        .field("Back", "Answer")
        .build();
    client.notes().add(note).await?;

    Ok(())
}
```

For high-level workflows, use `ankit-engine`:

```rust
use ankit_engine::Engine;
use ankit_engine::import::OnDuplicate;

#[tokio::main]
async fn main() -> ankit_engine::Result<()> {
    let engine = Engine::new();

    // Bulk import with duplicate handling
    let notes = vec![/* ... */];
    let report = engine.import().notes(&notes, OnDuplicate::Update).await?;

    // Analyze study patterns
    let stats = engine.analyze().study_summary("Japanese", 30).await?;

    Ok(())
}
```

For TOML-based deck creation, use `ankit-builder`:

```rust
use ankit_builder::DeckBuilder;

fn main() -> ankit_builder::Result<()> {
    // Load deck definition and generate .apkg
    let builder = DeckBuilder::from_file("vocabulary.toml")?;
    builder.write_apkg("vocabulary.apkg")?;
    Ok(())
}
```

## Requirements

- [Anki](https://apps.ankiweb.net/) with [AnkiConnect](https://ankiweb.net/shared/info/2055492159) add-on
- Rust 1.85+ (Edition 2024)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
