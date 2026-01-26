# TOML Deck Builder

Define Anki decks in simple TOML files and sync them with your Anki collection.

## Why TOML?

- **Version control**: Track deck changes in git
- **Collaboration**: Share deck definitions as text files
- **Automation**: Generate decks from scripts or data sources
- **Readable**: Human-friendly format you can edit in any text editor

## Quick Start

### 1. Create a TOML File

```toml
[package]
name = "Spanish Vocabulary"
version = "1.0.0"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Spanish"

[[notes]]
deck = "Spanish"
model = "Basic"
tags = ["food"]

[notes.fields]
Front = "el gato"
Back = "the cat"
```

### 2. Import to Anki

Ask Claude:
> "Import this TOML file to Anki: [paste contents or provide path]"

Or use the `import_deck_toml` tool directly.

## Features

- **Bidirectional sync**: Push changes to Anki or pull from Anki
- **Markdown support**: Write content in Markdown, converts to HTML
- **Diff/preview**: See what would change before syncing
- **Note tracking**: Track note IDs for updates

## Next Steps

- [TOML Format Reference](toml-format.md) - Complete format documentation
- [Markdown Fields](markdown-fields.md) - Using Markdown in your notes
- [Syncing with Anki](syncing.md) - Push, pull, and bidirectional sync
