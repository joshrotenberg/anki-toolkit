# ankit-builder (Deck Builder)

TOML-based deck builder with .apkg generation and AnkiConnect import.

## Installation

```toml
[dependencies]
ankit-builder = "0.1"
```

For async AnkiConnect import:

```toml
[dependencies]
ankit-builder = { version = "0.1", features = ["connect"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

### Generate .apkg File

```rust
use ankit_builder::DeckBuilder;

fn main() -> ankit_builder::Result<()> {
    let builder = DeckBuilder::from_file("deck.toml")?;
    builder.write_apkg("deck.apkg")?;
    Ok(())
}
```

### Import via AnkiConnect

```rust
use ankit_builder::DeckBuilder;

#[tokio::main]
async fn main() -> ankit_builder::Result<()> {
    let builder = DeckBuilder::from_file("deck.toml")?;
    let result = builder.import_connect().await?;
    println!("Created {} notes", result.notes_created);
    Ok(())
}
```

### Bidirectional Sync

```rust
use ankit_builder::{DeckBuilder, SyncStrategy};

let builder = DeckBuilder::from_file("deck.toml")?;
let result = builder.sync(SyncStrategy::bidirectional()).await?;

if let Some(updated) = result.updated_definition {
    updated.write_toml("deck_updated.toml")?;
}
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `apkg` | Yes | .apkg file generation |
| `connect` | Yes | AnkiConnect import/sync |

## Full Documentation

See [docs.rs/ankit-builder](https://docs.rs/ankit-builder) for complete API documentation.
