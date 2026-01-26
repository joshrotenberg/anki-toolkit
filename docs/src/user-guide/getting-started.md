# Getting Started

This guide will help you set up the ankit MCP server to manage your Anki decks through AI assistants like Claude.

## Prerequisites

Before you begin, make sure you have:

1. **Anki** installed and running
2. **AnkiConnect** add-on installed in Anki
3. **Claude Desktop** (or another MCP client)

## Important: Data Safety

Before you start, please understand:

**Write mode (default) can permanently modify your Anki collection.** The MCP server has full access to:

- Delete notes and entire decks
- Modify card content, tags, and fields
- Reset learning progress and statistics
- Change card scheduling

**Recommendations:**

1. **Back up your collection first** - In Anki: File > Export > select "Include scheduling information"
2. **Start with read-only mode** - Add `--read-only` flag to test safely
3. **Review before bulk operations** - Ask Claude to preview changes first
4. **Keep regular backups** - Anki stores backups in your profile folder

The authors are not responsible for any data loss. Use at your own risk.

## Quick Setup

### 1. Install AnkiConnect

In Anki, go to **Tools > Add-ons > Get Add-ons** and enter code: `2055492159`

Restart Anki after installation.

### 2. Install the MCP Server

```bash
cargo install ankit-mcp
```

### 3. Configure Claude Desktop

See [Claude Desktop Setup](claude-desktop-setup.md) for detailed configuration instructions.

### 4. Test the Connection

After restarting Claude Desktop, ask:

> "Can you check if Anki is running?"

Claude will use the `version` tool to verify the connection.

## What's Next?

- [Available Tools](tools-reference.md) - See all 50 tools you can use
- [Example Prompts](example-prompts.md) - Learn what you can ask Claude to do
- [TOML Deck Builder](toml-builder.md) - Define decks in simple text files
