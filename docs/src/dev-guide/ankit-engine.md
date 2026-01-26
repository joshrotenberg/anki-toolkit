# ankit-engine (Workflows)

High-level workflow operations built on the ankit client.

## Installation

```toml
[dependencies]
ankit-engine = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use ankit_engine::Engine;

#[tokio::main]
async fn main() -> ankit_engine::Result<()> {
    let engine = Engine::new();

    // Analysis
    let stats = engine.analyze().study_summary("Japanese", 30).await?;
    println!("Reviews: {}", stats.total_reviews);

    // Find problem cards
    let problems = engine.analyze().find_problems("deck:Japanese", 5).await?;

    Ok(())
}
```

## Workflow Modules

| Module | Purpose |
|--------|---------|
| `engine.analyze()` | Study statistics, retention, leeches |
| `engine.import()` | Bulk import with duplicate handling |
| `engine.export()` | Deck and review history export |
| `engine.organize()` | Clone, merge, reorganize decks |
| `engine.progress()` | Reset, tag by performance, suspend |
| `engine.media()` | Audit and cleanup media files |
| `engine.enrich()` | Find and update notes with empty fields |
| `engine.deduplicate()` | Find and remove duplicates |
| `engine.backup()` | Backup decks to .apkg files |

## Feature Flags

All modules are enabled by default. Disable with:

```toml
[dependencies]
ankit-engine = { version = "0.1", default-features = false, features = ["analyze"] }
```

## Full Documentation

See [docs.rs/ankit-engine](https://docs.rs/ankit-engine) for complete API documentation.
