# ankit-engine

High-level workflow operations for Anki via AnkiConnect.

[![Crates.io](https://img.shields.io/crates/v/ankit-engine.svg)](https://crates.io/crates/ankit-engine)
[![Documentation](https://docs.rs/ankit-engine/badge.svg)](https://docs.rs/ankit-engine)

## Overview

While [`ankit`](https://crates.io/crates/ankit) provides 1:1 API bindings for AnkiConnect,
`ankit-engine` builds higher-level workflows that combine multiple API calls into
cohesive operations.

## Features

- **Import** - Bulk import with duplicate detection and conflict resolution
- **Export** - Deck and review history export
- **Organize** - Deck cloning, merging, and tag-based reorganization
- **Analyze** - Study statistics, retention rates, and problem card detection
- **Progress** - Deck health reports, performance tagging, bulk operations
- **Migrate** - Note type migration with field mapping
- **Media** - Media file audit and cleanup
- **Enrich** - Find and fill empty fields
- **Deduplicate** - Find and remove duplicate notes

All features are enabled by default but can be individually disabled.

## Quick Start

```toml
[dependencies]
ankit-engine = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use ankit_engine::Engine;

#[tokio::main]
async fn main() -> ankit_engine::Result<()> {
    let engine = Engine::new();

    // Get study statistics
    let stats = engine.analyze().study_summary("Japanese", 30).await?;
    println!("Reviews in last 30 days: {}", stats.total_reviews);

    // Find problem cards (leeches)
    use ankit_engine::analyze::ProblemCriteria;
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
use ankit_engine::{Engine, NoteBuilder};
use ankit_engine::import::OnDuplicate;

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
use ankit_engine::Engine;

let engine = Engine::new();
let report = engine.organize()
    .clone_deck("Japanese", "Japanese Backup")
    .await?;
println!("Cloned {} notes", report.notes_cloned);
```

### Deck Health Report

```rust
use ankit_engine::Engine;

let engine = Engine::new();
let health = engine.progress().deck_health("Japanese").await?;
println!("Total cards: {}", health.total_cards);
println!("Average ease: {:.1}%", health.avg_ease);
println!("Leeches: {}", health.leeches);
```

### Find and Remove Duplicates

```rust
use ankit_engine::Engine;
use ankit_engine::deduplicate::KeepStrategy;

let engine = Engine::new();

// Preview what would be removed
let duplicates = engine.deduplicate()
    .find("deck:Vocabulary", "Front")
    .await?;

// Remove duplicates, keeping the note with most content
let result = engine.deduplicate()
    .remove("deck:Vocabulary", "Front", KeepStrategy::MostContent)
    .await?;
println!("Removed {} duplicate notes", result.removed);
```

### Media Audit

```rust
use ankit_engine::Engine;

let engine = Engine::new();
let audit = engine.media().audit().await?;
println!("Total files: {}", audit.total_files);
println!("Orphaned: {}", audit.orphaned.len());
println!("Missing: {}", audit.missing.len());

// Clean up orphaned files (dry run first)
let preview = engine.media().cleanup_orphaned(true).await?;
println!("Would delete {} files", preview.deleted.len());
```

## Feature Flags

All workflow modules are enabled by default. To use only specific features:

```toml
[dependencies]
ankit-engine = { version = "0.1", default-features = false, features = ["analyze", "import"] }
```

Available features: `import`, `export`, `organize`, `analyze`, `migrate`, `media`, `progress`, `enrich`, `deduplicate`

## Related Crates

- [`ankit`](https://crates.io/crates/ankit) - Core AnkiConnect client
- [`ankit-builder`](https://crates.io/crates/ankit-builder) - TOML-based deck builder
- [`ankit-mcp`](https://crates.io/crates/ankit-mcp) - MCP server for AI assistant integration

## Requirements

- [Anki](https://apps.ankiweb.net/) with [AnkiConnect](https://ankiweb.net/shared/info/2055492159) add-on
- Rust 1.85+ (Edition 2024)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
