# Installation

## Installing the MCP Server

### From crates.io (Recommended)

```bash
cargo install ankit-mcp
```

### From Source

```bash
git clone https://github.com/joshrotenberg/anki-toolkit
cd anki-toolkit
cargo install --path crates/ankit-mcp
```

## Installing AnkiConnect

AnkiConnect is required for the MCP server to communicate with Anki.

1. Open Anki
2. Go to **Tools > Add-ons > Get Add-ons**
3. Enter the code: `2055492159`
4. Click OK and restart Anki

### Verifying AnkiConnect

With Anki running, open a browser and go to:

```
http://localhost:8765
```

You should see: `AnkiConnect v.6`

## Command Line Options

```
ankit-mcp [OPTIONS]

Options:
    --host <HOST>       AnkiConnect host [default: 127.0.0.1]
    --port <PORT>       AnkiConnect port [default: 8765]
    --transport <TYPE>  Transport: stdio or http [default: stdio]
    --http-port <PORT>  HTTP server port [default: 3000]
    --http-host <HOST>  HTTP server host [default: 127.0.0.1]
    --read-only         Disable write operations
    -v, --verbose       Logging level (-v=info, -vv=debug, -vvv=trace)
```

### Read-Only Mode

Use `--read-only` to prevent any modifications to your Anki collection:

```bash
ankit-mcp --read-only
```

This is useful for:
- Exploring decks safely
- Demo environments
- Shared systems where write access should be restricted

### HTTP Transport

For clients that prefer HTTP over stdio:

```bash
ankit-mcp --transport http --http-port 3000
```
