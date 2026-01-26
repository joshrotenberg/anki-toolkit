# Deduplication

Find and remove duplicate notes in your Anki collection.

## Finding Duplicates

Duplicates are detected by comparing a key field (usually the first field).

### Using MCP Tools

> "Find duplicate notes in my vocabulary deck based on the Front field"

### Using Rust

```rust
use ankit_engine::Engine;
use ankit_engine::deduplicate::{DedupeQuery, KeepStrategy};

let engine = Engine::new();

let query = DedupeQuery {
    search: "deck:Vocabulary".to_string(),
    key_field: "Front".to_string(),
    keep: KeepStrategy::MostContent,
};

let groups = engine.deduplicate().find_duplicates(&query).await?;
for group in &groups {
    println!("'{}': keep {}, delete {:?}",
        group.key_value,
        group.keep_note_id,
        group.duplicate_note_ids
    );
}
```

## Keep Strategies

| Strategy | Behavior |
|----------|----------|
| `First` | Keep the oldest note (lowest ID) |
| `Last` | Keep the newest note (highest ID) |
| `MostContent` | Keep the note with most non-empty fields |
| `MostTags` | Keep the note with most tags |

## Previewing Before Deletion

Always preview before removing duplicates:

```rust
let report = engine.deduplicate().preview(&query).await?;
println!("Would delete {} notes", report.deleted);
```

## Removing Duplicates

```rust
let report = engine.deduplicate().remove_duplicates(&query).await?;
println!("Deleted {} duplicates, kept {}", report.deleted, report.kept);
```

## Common Scenarios

### After Bulk Import

Imports can create duplicates. Clean up with:

```rust
let query = DedupeQuery {
    search: "tag:imported".to_string(),
    key_field: "Front".to_string(),
    keep: KeepStrategy::MostContent,
};
engine.deduplicate().remove_duplicates(&query).await?;
```

### Merged Decks

After merging decks, check for duplicates:

```rust
let query = DedupeQuery {
    search: "deck:\"Merged Deck\"".to_string(),
    key_field: "Word".to_string(),
    keep: KeepStrategy::First,
};
```
