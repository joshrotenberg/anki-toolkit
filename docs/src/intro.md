# Introduction

**ankit** is a comprehensive Rust toolkit for Anki deck management via AnkiConnect.

## What is Anki?

[Anki](https://apps.ankiweb.net/) is a free, open-source flashcard application that uses spaced repetition to help you remember things efficiently. It's widely used for:

- Language learning
- Medical education
- Programming concepts
- Any subject requiring memorization

Anki shows you flashcards just before you're about to forget them, making your study time more effective.

## What is AnkiConnect?

[AnkiConnect](https://ankiweb.net/shared/info/2055492159) is an Anki add-on that exposes a REST API, allowing external programs to interact with your Anki collection. ankit uses this API to manage your decks programmatically.

## Who Is This For?

### Users

If you want to manage your Anki decks through AI assistants like Claude, the **MCP Server** is for you. No programming required - just install, configure, and start asking Claude to manage your flashcards.

**What you can do:**
- Create and edit flashcards through natural conversation
- Analyze your study patterns and find problem cards
- Bulk import/export decks
- Find and remove duplicates
- Manage media files
- Define decks in simple TOML files

[Get started with the User Guide](user-guide/getting-started.md)

### Developers

If you're building Anki integrations in Rust, the **library crates** provide complete programmatic access to Anki:

- **ankit**: Complete async AnkiConnect API client
- **ankit-engine**: High-level workflow operations
- **ankit-builder**: TOML deck builder with .apkg generation

[Get started with the Developer Guide](dev-guide/overview.md)

## Components

| Component | Description | Audience |
|-----------|-------------|----------|
| [ankit-mcp](user-guide/mcp-server.md) | MCP server with 50 tools for AI assistants | Users |
| [TOML Builder](user-guide/toml-builder.md) | Define decks in TOML, generate .apkg files | Users / Developers |
| [ankit](dev-guide/ankit.md) | Complete AnkiConnect API client | Developers |
| [ankit-engine](dev-guide/ankit-engine.md) | High-level workflow operations | Developers |
| [ankit-builder](dev-guide/ankit-builder.md) | Deck builder library | Developers |

## Requirements

- [Anki](https://apps.ankiweb.net/) desktop application
- [AnkiConnect](https://ankiweb.net/shared/info/2055492159) add-on installed
- For users: Claude Desktop or another MCP client
- For developers: Rust 1.85+ (Edition 2024)

## Links

- [GitHub Repository](https://github.com/joshrotenberg/anki-toolkit)
- [API Documentation](https://docs.rs/ankit)
- [AnkiConnect Documentation](https://foosoft.net/projects/anki-connect/)
