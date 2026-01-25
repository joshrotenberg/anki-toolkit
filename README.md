# yanki

A Rust toolkit for programmatic Anki deck management via AnkiConnect.

[![License](https://img.shields.io/crates/l/yanki.svg)](https://github.com/joshrotenberg/yanki#license)

## Crates

| Crate | Description | Crates.io |
|-------|-------------|-----------|
| [yanki](crates/yanki) | Complete async AnkiConnect API client | [![Crates.io](https://img.shields.io/crates/v/yanki.svg)](https://crates.io/crates/yanki) |
| [yanki-engine](crates/yanki-engine) | High-level workflow operations | [![Crates.io](https://img.shields.io/crates/v/yanki-engine.svg)](https://crates.io/crates/yanki-engine) |

## Quick Start

For direct API access, use `yanki`:

```rust
use yanki::{AnkiClient, NoteBuilder};

#[tokio::main]
async fn main() -> yanki::Result<()> {
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

For high-level workflows, use `yanki-engine`:

```rust
use yanki_engine::Engine;
use yanki_engine::import::OnDuplicate;

#[tokio::main]
async fn main() -> yanki_engine::Result<()> {
    let engine = Engine::new();

    // Bulk import with duplicate handling
    let notes = vec![/* ... */];
    let report = engine.import().notes(&notes, OnDuplicate::Update).await?;

    // Analyze study patterns
    let stats = engine.analyze().study_summary("Japanese", 30).await?;

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
