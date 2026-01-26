# Installation

## Prerequisites

1. **Anki Desktop**: Download from [apps.ankiweb.net](https://apps.ankiweb.net/)

2. **AnkiConnect Add-on**: Install from within Anki:
   - Tools > Add-ons > Get Add-ons
   - Enter code: `2055492159`
   - Restart Anki

## Installing Ankit Crates

Add the crates you need to your `Cargo.toml`:

### Core Client Only

```toml
[dependencies]
ankit = "0.1"
```

### With Engine Workflows

```toml
[dependencies]
ankit-engine = "0.1"
```

### Deck Builder

```toml
[dependencies]
ankit-builder = "0.1"
```

## Feature Flags

### ankit-engine

All features are enabled by default. To use only specific workflows:

```toml
[dependencies]
ankit-engine = { version = "0.1", default-features = false, features = ["analyze", "import"] }
```

Available features:
- `import` - Bulk import with duplicate handling
- `export` - Deck and review history export
- `organize` - Deck cloning, merging, reorganization
- `analyze` - Study statistics and problem detection
- `migrate` - Note type migration
- `media` - Media audit and cleanup
- `progress` - Card state management
- `enrich` - Note enrichment workflows
- `deduplicate` - Duplicate detection and removal

### ankit-builder

```toml
[dependencies]
ankit-builder = { version = "0.1", default-features = false, features = ["apkg"] }
```

Available features:
- `apkg` (default) - Generate `.apkg` files
- `connect` (default) - AnkiConnect import

## MCP Server Installation

The MCP server is distributed as a binary. Install with Cargo:

```bash
cargo install ankit-mcp
```

Or build from source:

```bash
git clone https://github.com/joshrotenberg/ankit
cd ankit
cargo build --release -p ankit-mcp
```

## Verifying Installation

Ensure Anki is running with AnkiConnect, then:

```rust,ignore
use ankit::AnkiClient;

#[tokio::main]
async fn main() -> ankit::Result<()> {
    let client = AnkiClient::new();
    let version = client.misc().version().await?;
    println!("AnkiConnect version: {}", version);
    Ok(())
}
```
