# ankit

A comprehensive Rust toolkit for Anki deck management via AnkiConnect.

[![CI](https://github.com/joshrotenberg/ankit/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/ankit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ankit.svg)](https://crates.io/crates/ankit)
[![docs.rs](https://docs.rs/ankit/badge.svg)](https://docs.rs/ankit)
[![License](https://img.shields.io/crates/l/ankit.svg)](https://github.com/joshrotenberg/ankit#license)

## Overview

ankit provides tools for managing Anki flashcard decks programmatically. Whether you're an **end user** wanting to manage decks through AI assistants, or a **developer** building Anki integrations, this toolkit has you covered.

### Key Features

- **55 MCP tools** for AI assistant integration (Claude, etc.)
- **Complete AnkiConnect API** coverage with async Rust client
- **TOML-based deck definitions** with .apkg generation
- **High-level workflows**: bulk import, deduplication, analysis, media management
- **Bidirectional sync** between TOML files and Anki

## For Users: MCP Server

The `ankit-mcp` server lets AI assistants (like Claude) manage your Anki decks directly.

### Installation

```bash
# From crates.io
cargo install ankit-mcp

# Or using Docker
docker pull ghcr.io/joshrotenberg/ankit-mcp:latest
```

### Important: Data Safety

> **Warning**: By default, ankit-mcp runs in **write mode** with full access to modify your Anki collection. This includes the ability to delete notes, reset learning progress, and modify scheduling data.

**To protect your data:**

1. **Use read-only mode** when exploring or if unsure: `ankit-mcp --read-only`
2. **Back up your collection** regularly via Anki's File > Export (include scheduling info)
3. **Test with a copy** of your collection first if making bulk changes
4. **Review operations** before confirming - ask Claude to preview changes first

The authors are not responsible for any data loss. **Use at your own risk.**

### Setup with Claude Desktop

Add to your Claude Desktop config (`~/.config/claude/claude_desktop_config.json` on Linux/macOS):

```json
{
  "mcpServers": {
    "anki": {
      "command": "ankit-mcp"
    }
  }
}
```

### What You Can Do

Once configured, ask Claude to:

- **Create flashcards**: "Add a note to my Japanese deck with front 'hello' and back 'konnichiwa'"
- **Search and analyze**: "Show me cards I'm struggling with in my Spanish deck"
- **Bulk operations**: "Find and remove duplicate notes in my vocabulary deck"
- **Import/export**: "Export my Japanese deck to a TOML file"
- **Deck health**: "Give me a health report on my medical terminology deck"
- **Media management**: "Find orphaned media files in my collection"

### Available Tools (55 total)

| Category | Tools |
|----------|-------|
| Notes | add, find, get info, update, delete |
| Cards | find, get info, suspend, unsuspend, forget, set ease, set due date |
| Tags | add, remove, replace all, clear unused |
| Decks | list, create, delete, clone, merge |
| Models | list models, get model fields |
| Analysis | study summary, retention stats, find problems |
| Progress | reset deck, tag by performance, suspend by criteria, deck health, bulk tag |
| Import/Export | import notes, validate notes, export deck, export reviews |
| Deduplication | find duplicates, preview, remove |
| Enrichment | find candidates, enrich note, enrich notes |
| Media | audit, cleanup |
| Backup | backup deck, backup collection, restore deck, list backups |
| Organization | move by tag |
| TOML Sync | export, diff, plan sync, sync, import |
| Misc | version, sync with AnkiWeb |

## For Developers: Rust Crates

| Crate | Description | Crates.io |
|-------|-------------|-----------|
| [ankit](crates/ankit) | Complete async AnkiConnect API client | [![Crates.io](https://img.shields.io/crates/v/ankit.svg)](https://crates.io/crates/ankit) |
| [ankit-engine](crates/ankit-engine) | High-level workflow operations | [![Crates.io](https://img.shields.io/crates/v/ankit-engine.svg)](https://crates.io/crates/ankit-engine) |
| [ankit-builder](crates/ankit-builder) | TOML deck builder with .apkg generation | [![Crates.io](https://img.shields.io/crates/v/ankit-builder.svg)](https://crates.io/crates/ankit-builder) |
| [ankit-mcp](crates/ankit-mcp) | MCP server for AI assistants | [![Crates.io](https://img.shields.io/crates/v/ankit-mcp.svg)](https://crates.io/crates/ankit-mcp) |

### Quick Start: API Client

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

    // Search for notes
    let notes = client.notes().find("deck:Default").await?;

    Ok(())
}
```

### Quick Start: High-Level Workflows

```rust
use ankit_engine::Engine;

#[tokio::main]
async fn main() -> ankit_engine::Result<()> {
    let engine = Engine::new();

    // Analyze study patterns
    let stats = engine.analyze().study_summary("Japanese", 30).await?;
    println!("Reviews: {}, Retention: {:.1}%",
        stats.total_reviews, stats.retention_rate * 100.0);

    // Find and remove duplicates
    let report = engine.deduplicate().remove_duplicates(&query).await?;
    println!("Removed {} duplicates", report.deleted);

    Ok(())
}
```

### Quick Start: TOML Deck Builder

Define decks in TOML:

```toml
[package]
name = "Spanish Vocabulary"
version = "1.0.0"

[[models]]
name = "Basic Spanish"
fields = ["Spanish", "English"]
markdown_fields = ["English"]  # Supports Markdown

[[models.templates]]
name = "Card 1"
front = "{{Spanish}}"
back = "{{English}}"

[[decks]]
name = "Spanish::Vocabulary"

[[notes]]
deck = "Spanish::Vocabulary"
model = "Basic Spanish"
tags = ["food"]

[notes.fields]
Spanish = "el gato"
English = "the **cat**"
```

Generate .apkg or import via AnkiConnect:

```rust
use ankit_builder::DeckBuilder;

fn main() -> ankit_builder::Result<()> {
    let builder = DeckBuilder::from_file("vocabulary.toml")?;

    // Generate .apkg file
    builder.write_apkg("vocabulary.apkg")?;

    // Or import directly via AnkiConnect
    // builder.import_connect().await?;

    Ok(())
}
```

## Requirements

- [Anki](https://apps.ankiweb.net/) with [AnkiConnect](https://ankiweb.net/shared/info/2055492159) add-on installed
- Rust 1.85+ (for building from source)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
