# yanki

A complete, async-first Rust client for the [AnkiConnect](https://foosoft.net/projects/anki-connect/) API.

[![Crates.io](https://img.shields.io/crates/v/yanki.svg)](https://crates.io/crates/yanki)
[![Documentation](https://docs.rs/yanki/badge.svg)](https://docs.rs/yanki)
[![License](https://img.shields.io/crates/l/yanki.svg)](https://github.com/joshrotenberg/yanki#license)

## Features

- **Complete API coverage** - All 100+ AnkiConnect actions implemented
- **Async-first** - Built on `reqwest` and `tokio`
- **Type-safe** - Strongly typed request/response types with `serde`
- **Ergonomic** - Fluent API with action groups (`client.notes()`, `client.decks()`, etc.)
- **Well-documented** - Doc comments on all public items

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
yanki = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use yanki::{AnkiClient, NoteBuilder};

#[tokio::main]
async fn main() -> yanki::Result<()> {
    let client = AnkiClient::new();

    // Check connection
    let version = client.misc().version().await?;
    println!("AnkiConnect version: {}", version);

    // List decks
    let decks = client.decks().names().await?;
    println!("Decks: {:?}", decks);

    // Add a note
    let note = NoteBuilder::new("Default", "Basic")
        .field("Front", "What is the capital of France?")
        .field("Back", "Paris")
        .tag("geography")
        .build();

    let note_id = client.notes().add(note).await?;
    println!("Created note: {}", note_id);

    Ok(())
}
```

## Action Groups

Operations are organized into logical groups:

| Group | Description | Examples |
|-------|-------------|----------|
| `client.cards()` | Card operations | find, info, suspend, answer |
| `client.decks()` | Deck management | create, delete, stats, config |
| `client.gui()` | GUI control | browse, add_cards, current_card |
| `client.media()` | Media files | store, retrieve, list, delete |
| `client.models()` | Note types | names, fields, templates, create |
| `client.notes()` | Note operations | add, find, update, delete, tags |
| `client.statistics()` | Study stats | cards_reviewed_today, reviews_since |
| `client.misc()` | Utilities | version, sync, profiles, multi |

## Examples

### Find and inspect cards

```rust
// Find due cards
let due = client.cards().find("is:due").await?;

// Get card details
let info = client.cards().info(&due[..10]).await?;
for card in info {
    println!("{}: {} reps, {} lapses", card.card_id, card.reps, card.lapses);
}
```

### Work with media

```rust
use yanki::StoreMediaParams;

// Store from base64
let params = StoreMediaParams::from_base64("audio.mp3", base64_data);
client.media().store(params).await?;

// Store from URL
let params = StoreMediaParams::from_url("image.png", "https://example.com/image.png");
client.media().store(params).await?;

// List media files
let files = client.media().list("*.mp3").await?;
```

### Batch operations

```rust
use yanki::actions::MultiAction;

let actions = vec![
    MultiAction::new("deckNames"),
    MultiAction::new("modelNames"),
    MultiAction::with_params("findNotes", serde_json::json!({"query": "deck:Default"})),
];

let results = client.misc().multi(&actions).await?;
```

## Client Configuration

```rust
use std::time::Duration;
use yanki::AnkiClient;

let client = AnkiClient::builder()
    .url("http://localhost:8765")  // Custom URL
    .api_key("your-api-key")       // If AnkiConnect requires auth
    .timeout(Duration::from_secs(60))
    .build();
```

## Requirements

- [Anki](https://apps.ankiweb.net/) with [AnkiConnect](https://ankiweb.net/shared/info/2055492159) add-on installed
- AnkiConnect running (Anki must be open)
- Rust 1.85+ (Edition 2024)

## Comparison

| Feature | yanki | anki_bridge | ankiconnect-rs |
|---------|-------|-------------|----------------|
| API Coverage | ~100% | ~73% | WIP |
| Async | Yes | Optional | Yes |
| API Style | Fluent groups | Trait-based | Domain-driven |
| Rust Edition | 2024 | 2021 | 2021 |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
