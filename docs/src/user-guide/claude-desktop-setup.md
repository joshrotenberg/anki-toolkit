# Claude Desktop Setup

Configure Claude Desktop to use the ankit MCP server.

## Before You Start

> **Warning**: The MCP server has full access to modify your Anki collection. This means Claude can delete notes, reset progress, and change your data permanently.
>
> **We strongly recommend starting with read-only mode** until you're comfortable with how the tools work. See [Read-Only Mode](#read-only-mode-recommended) below.

## Configuration File Location

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Linux | `~/.config/claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |

## Basic Configuration

Add the following to your config file:

```json
{
  "mcpServers": {
    "anki": {
      "command": "ankit-mcp"
    }
  }
}
```

## With Custom Settings

### Custom AnkiConnect Port

```json
{
  "mcpServers": {
    "anki": {
      "command": "ankit-mcp",
      "args": ["--port", "8766"]
    }
  }
}
```

### Read-Only Mode (Recommended)

**Start here!** Read-only mode lets you explore all the analysis and query tools without any risk of modifying your data:

```json
{
  "mcpServers": {
    "anki": {
      "command": "ankit-mcp",
      "args": ["--read-only"]
    }
  }
}
```

In read-only mode, you can:
- Search and browse notes and cards
- View deck statistics and health reports
- Find duplicates and problem cards
- Export data to JSON or TOML

Write operations will be blocked with a clear error message. Once you're comfortable, remove the `--read-only` flag to enable full access.

### With Verbose Logging

```json
{
  "mcpServers": {
    "anki": {
      "command": "ankit-mcp",
      "args": ["-vv"]
    }
  }
}
```

## Using Docker

If you prefer Docker over installing the binary directly:

### macOS/Windows

```json
{
  "mcpServers": {
    "anki": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "ghcr.io/joshrotenberg/ankit-mcp",
        "--host", "host.docker.internal"
      ]
    }
  }
}
```

### Linux

On Linux, use `--network host` to access AnkiConnect:

```json
{
  "mcpServers": {
    "anki": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "--network", "host",
        "ghcr.io/joshrotenberg/ankit-mcp"
      ]
    }
  }
}
```

### Docker with Read-Only Mode

```json
{
  "mcpServers": {
    "anki": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "ghcr.io/joshrotenberg/ankit-mcp",
        "--host", "host.docker.internal",
        "--read-only"
      ]
    }
  }
}
```

## Applying Changes

After editing the configuration:

1. Save the file
2. Quit Claude Desktop completely
3. Restart Claude Desktop
4. Open a new conversation

## Verifying the Setup

Ask Claude:

> "Can you check if Anki is running by getting the AnkiConnect version?"

If configured correctly, Claude will respond with the version number.

## Troubleshooting

### "Connection refused" errors

1. Ensure Anki is running
2. Verify AnkiConnect is installed (Tools > Add-ons should show AnkiConnect)
3. Check if port 8765 is blocked or in use

### Tools not appearing

1. Restart Claude Desktop after configuration changes
2. Verify the `ankit-mcp` binary is in your PATH
3. Try running `ankit-mcp -vv` in a terminal to check for errors

### "Permission denied" errors

1. Check AnkiConnect's allowed origins in Anki's add-on config
2. Add `http://localhost` to the allowed origins list
