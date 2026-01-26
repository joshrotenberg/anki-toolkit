# Getting Started

This guide will help you set up the ankit MCP server to manage your Anki decks through AI assistants like Claude.

## Prerequisites

Before you begin, make sure you have:

1. **Anki** installed and running
2. **AnkiConnect** add-on installed in Anki
3. **Claude Desktop** (or another MCP client)

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
