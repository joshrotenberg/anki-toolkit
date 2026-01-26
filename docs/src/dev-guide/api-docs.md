# API Documentation

Full API documentation is available on docs.rs:

- [ankit](https://docs.rs/ankit) - AnkiConnect API client
- [ankit-engine](https://docs.rs/ankit-engine) - Workflow operations
- [ankit-builder](https://docs.rs/ankit-builder) - TOML deck builder
- [ankit-mcp](https://docs.rs/ankit-mcp) - MCP server

## Building Docs Locally

```bash
cargo doc --all-features --open
```

## Query Syntax Reference

Many methods accept Anki search queries:

```
deck:Name           # Deck filter
tag:name            # Tag filter
is:due              # Due cards
is:new              # New cards
is:suspended        # Suspended cards
rated:N             # Reviewed in last N days
prop:lapses>=N      # Property comparisons
"exact phrase"      # Phrase search
field:value         # Field search
-filter             # NOT
filter1 filter2     # AND
filter1 OR filter2  # OR
```

## Common Types

### NoteBuilder

```rust
let note = NoteBuilder::new(deck, model)
    .field(name, value)
    .tag(tag)
    .tags(vec![...])
    .allow_duplicate(true)
    .build();
```

### QueryBuilder

```rust
use ankit::QueryBuilder;

let query = QueryBuilder::new()
    .deck("Japanese")
    .tag("verb")
    .is_due()
    .build();
```
