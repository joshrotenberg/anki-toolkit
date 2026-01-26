# Introduction

**Ankit** is a Rust toolkit for Anki deck management via AnkiConnect. It provides a comprehensive set of tools for working with Anki flashcards programmatically.

## Components

The project consists of four crates:

### ankit (Core Client)

A 1:1 mapping to the AnkiConnect API with 106 methods. Provides typed access to all AnkiConnect operations.

```rust,ignore
use ankit::AnkiClient;

let client = AnkiClient::new();
let decks = client.decks().names().await?;
```

### ankit-engine (Workflows)

High-level workflow operations that combine multiple API calls into cohesive actions:

- **Import/Export**: Bulk import with duplicate handling, deck export
- **Analyze**: Study statistics, problem card detection, retention analysis
- **Organize**: Deck cloning, merging, tag-based reorganization
- **Progress**: Card state management, performance tagging
- **Media**: Audit and cleanup orphaned files
- **Enrich**: Find and update notes with empty fields
- **Deduplicate**: Find and remove duplicate notes

### ankit-mcp (MCP Server)

A Model Context Protocol server exposing 46 tools for AI assistant integration. Works with Claude Desktop, VS Code, and other MCP clients.

### ankit-builder (Deck Builder)

TOML-based deck builder with dual output:

- Generate `.apkg` files for direct import into Anki
- Import directly via AnkiConnect to a running Anki instance

## Requirements

- Anki desktop application
- [AnkiConnect](https://foosoft.net/projects/anki-connect/) add-on installed
- Rust 2024 edition (for development)

## Quick Links

- [GitHub Repository](https://github.com/joshrotenberg/ankit)
- [API Documentation](https://docs.rs/ankit)
- [AnkiConnect Documentation](https://foosoft.net/projects/anki-connect/)
