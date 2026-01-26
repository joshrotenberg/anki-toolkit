# MCP Server Setup

The ankit-mcp server exposes Anki operations as MCP tools for AI assistant integration.

## Installation

```bash
cargo install ankit-mcp
```

## Configuration

### Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "anki": {
      "command": "ankit-mcp",
      "args": []
    }
  }
}
```

### With Custom Settings

```json
{
  "mcpServers": {
    "anki": {
      "command": "ankit-mcp",
      "args": ["--port", "8765", "--read-only"]
    }
  }
}
```

## Command Line Options

```
ankit-mcp [OPTIONS]

Options:
    --host <HOST>     AnkiConnect host [default: 127.0.0.1]
    --port <PORT>     AnkiConnect port [default: 8765]
    --read-only       Disable write operations
    -v, --verbose     Logging level (-v=info, -vv=debug, -vvv=trace)
```

### Read-Only Mode

Use `--read-only` to disable all write operations. This is useful for:
- Exploring decks without risk of modification
- Shared environments where write access should be restricted
- Demo and presentation scenarios

## Verification

After restarting Claude Desktop:

1. Open a conversation
2. Ask Claude to check the Anki connection:
   ```
   Can you check if Anki is running by getting the AnkiConnect version?
   ```

Claude will use the `version` tool to verify connectivity.

## Troubleshooting

### "Connection refused" errors

1. Ensure Anki is running
2. Verify AnkiConnect add-on is installed (code: 2055492159)
3. Check if another application is using port 8765
4. Try specifying the host explicitly: `--host 127.0.0.1`

### "Permission denied" errors

1. Check AnkiConnect's allowed origins in Anki's add-on settings
2. If using an API key, ensure it's configured correctly

### Tools not appearing

1. Restart Claude Desktop after configuration changes
2. Check the MCP server logs with `-vv` for debug output
3. Verify the binary path is correct and executable
