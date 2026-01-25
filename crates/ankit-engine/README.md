# yanki-engine

High-level workflow operations for Anki via AnkiConnect.

[![Crates.io](https://img.shields.io/crates/v/yanki-engine.svg)](https://crates.io/crates/yanki-engine)
[![Documentation](https://docs.rs/yanki-engine/badge.svg)](https://docs.rs/yanki-engine)

## Overview

While [`yanki`](https://crates.io/crates/yanki) provides 1:1 API bindings for AnkiConnect,
`yanki-engine` builds higher-level workflows that combine multiple API calls into
cohesive operations.

## Features

- **Import** - Bulk import with duplicate detection and conflict resolution
- **Export** - Deck and review history export
- **Organize** - Deck cloning, merging, and tag-based reorganization
- **Analyze** - Study statistics and problem card (leech) detection
- **Migrate** - Note type migration with field mapping
- **Media** - Media file audit and cleanup

All features are enabled by default but can be individually disabled.

## Quick Start

```toml
[dependencies]
yanki-engine = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use yanki_engine::Engine;

#[tokio::main]
async fn main() -> yanki_engine::Result<()> {
    let engine = Engine::new();

    // Get study statistics
    let stats = engine.analyze().study_summary("Japanese", 30).await?;
    println!("Reviews in last 30 days: {}", stats.total_reviews);

    // Find problem cards (leeches)
    use yanki_engine::analyze::ProblemCriteria;
    let problems = engine.analyze()
        .find_problems("deck:Japanese", ProblemCriteria::default())
        .await?;
    println!("Found {} problem cards", problems.len());

    // Direct API access when needed
    let version = engine.client().misc().version().await?;
    println!("AnkiConnect version: {}", version);

    Ok(())
}
```

## Workflow Examples

### Bulk Import with Duplicate Handling

```rust
use yanki_engine::{Engine, NoteBuilder};
use yanki_engine::import::OnDuplicate;

let engine = Engine::new();

let notes = vec![
    NoteBuilder::new("Japanese", "Basic")
        .field("Front", "hello")
        .field("Back", "world")
        .build(),
];

// Skip duplicates
let report = engine.import().notes(&notes, OnDuplicate::Skip).await?;
println!("Added: {}, Skipped: {}", report.added, report.skipped);

// Or update existing notes
let report = engine.import().notes(&notes, OnDuplicate::Update).await?;
```

### Clone a Deck

```rust
use yanki_engine::Engine;

let engine = Engine::new();
let report = engine.organize()
    .clone_deck("Japanese", "Japanese Backup")
    .await?;
println!("Cloned {} notes", report.notes_cloned);
```

### Media Audit

```rust
use yanki_engine::Engine;

let engine = Engine::new();
let audit = engine.media().audit().await?;
println!("Total files: {}", audit.total_files);
println!("Orphaned: {}", audit.orphaned.len());
println!("Missing: {}", audit.missing.len());

// Clean up orphaned files
let cleanup = engine.media().cleanup_orphaned(false).await?;
```

## Feature Flags

All workflow modules are enabled by default. To use only specific features:

```toml
[dependencies]
yanki-engine = { version = "0.1", default-features = false, features = ["analyze", "import"] }
```

Available features: `import`, `export`, `organize`, `analyze`, `migrate`, `media`

## Requirements

- [Anki](https://apps.ankiweb.net/) with [AnkiConnect](https://ankiweb.net/shared/info/2055492159) add-on
- Rust 1.85+ (Edition 2024)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
