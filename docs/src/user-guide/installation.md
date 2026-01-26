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

### Using Docker

If you don't have Rust installed, you can use the Docker image:

```bash
# Pull the image
docker pull ghcr.io/joshrotenberg/ankit-mcp:latest

# Run with stdio transport (for Claude Desktop)
docker run -it --rm ghcr.io/joshrotenberg/ankit-mcp

# Run with HTTP transport
docker run -it --rm -p 3000:3000 ghcr.io/joshrotenberg/ankit-mcp --transport http --http-host 0.0.0.0
```

#### Connecting to Anki on the Host

The MCP server needs to reach AnkiConnect running on your host machine:

**Linux:**
```bash
docker run -it --rm --network host ghcr.io/joshrotenberg/ankit-mcp
```

**macOS/Windows:**
```bash
docker run -it --rm ghcr.io/joshrotenberg/ankit-mcp --host host.docker.internal
```

See [Claude Desktop Setup](claude-desktop-setup.md) for Docker configuration examples.

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

### Read-Only Mode (Recommended for New Users)

> **Important**: Without `--read-only`, the MCP server has full write access to your Anki collection. This means it can delete notes, reset learning progress, and make permanent changes.

Use `--read-only` to prevent any modifications:

```bash
ankit-mcp --read-only
```

**We recommend starting with read-only mode** until you're familiar with the tools. This lets you safely:
- Search and explore your decks
- View statistics and health reports
- Find duplicates and problem cards
- Export data

When ready for write access, remove the flag. Always maintain backups of your collection (File > Export in Anki).

### HTTP Transport

For clients that prefer HTTP over stdio:

```bash
ankit-mcp --transport http --http-port 3000
```
