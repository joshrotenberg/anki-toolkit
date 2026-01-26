# Claude Desktop Setup

Configure Claude Desktop to use the ankit MCP server.

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

### Read-Only Mode

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
