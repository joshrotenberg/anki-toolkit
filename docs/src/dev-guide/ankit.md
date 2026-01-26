# ankit (API Client)

Complete async AnkiConnect API client.

## Installation

```toml
[dependencies]
ankit = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use ankit::AnkiClient;

#[tokio::main]
async fn main() -> ankit::Result<()> {
    let client = AnkiClient::new();

    // Check connection
    let version = client.misc().version().await?;
    println!("AnkiConnect v{}", version);

    // List decks
    let decks = client.decks().names().await?;

    // Find notes
    let notes = client.notes().find("deck:Japanese").await?;

    Ok(())
}
```

## Action Groups

| Group | Methods |
|-------|---------|
| `client.cards()` | find, info, suspend, unsuspend, forget, ease |
| `client.decks()` | names, create, delete, config, stats |
| `client.notes()` | add, find, info, update, delete, tags |
| `client.models()` | names, fields, templates, create |
| `client.media()` | store, retrieve, list, delete |
| `client.statistics()` | reviewed_today, reviewed_by_day |
| `client.misc()` | version, sync, profiles |

## Configuration

```rust
use std::time::Duration;

let client = AnkiClient::builder()
    .url("http://localhost:8765")
    .api_key("your-api-key")
    .timeout(Duration::from_secs(60))
    .build();
```

## Full Documentation

See [docs.rs/ankit](https://docs.rs/ankit) for complete API documentation.
