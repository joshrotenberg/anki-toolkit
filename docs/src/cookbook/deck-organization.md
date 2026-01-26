# Deck Organization

Patterns for organizing and restructuring Anki decks.

## Clone a Deck

Create a copy of a deck for experimentation or backup:

```rust,ignore
use ankit_engine::Engine;

let engine = Engine::new();

let report = engine.organize()
    .clone_deck("Japanese::N5", "Japanese::N5-Copy")
    .await?;

println!("Cloned {} notes", report.notes_cloned);
```

## Merge Decks

Combine multiple decks into one:

```rust,ignore
let sources = vec![
    "Japanese::N5",
    "Japanese::N4",
    "Japanese::N3",
];

let report = engine.organize()
    .merge_decks(&sources, "Japanese::All")
    .await?;

println!("Merged {} notes from {} decks",
    report.notes_moved, report.decks_merged);
```

## Move Notes by Tag

Reorganize notes based on tags:

```rust,ignore
// Move all difficult cards to a review deck
let report = engine.organize()
    .move_by_tag("difficult", "Review::Difficult")
    .await?;

println!("Moved {} notes", report.notes_moved);
```

## Bulk Tag Operations

Add tags to multiple notes:

```rust,ignore
// Find all notes in a deck
let note_ids = engine.client().notes()
    .find("deck:Japanese")
    .await?;

// Add a tag to all of them
engine.client().notes()
    .add_tags(&note_ids, "reviewed-2024")
    .await?;
```

Remove tags:

```rust,ignore
engine.client().notes()
    .remove_tags(&note_ids, "old-tag")
    .await?;
```

Replace tags globally:

```rust,ignore
engine.client().notes()
    .replace_tags_all("typo-tag", "correct-tag")
    .await?;
```

## Clean Up Unused Tags

Remove tags that aren't used by any notes:

```rust,ignore
engine.client().notes()
    .clear_unused_tags()
    .await?;
```

## Create Deck Hierarchy

```rust,ignore
// Anki creates parent decks automatically
engine.client().decks()
    .create("Languages::Japanese::Vocabulary::N5")
    .await?;

// This creates:
// - Languages
// - Languages::Japanese
// - Languages::Japanese::Vocabulary
// - Languages::Japanese::Vocabulary::N5
```

## Delete Empty Decks

```rust,ignore
// Get all decks
let decks = engine.client().decks().names().await?;

for deck in decks {
    // Check if deck has cards
    let query = format!("deck:\"{}\"", deck);
    let cards = engine.client().cards().find(&query).await?;

    if cards.is_empty() {
        println!("Deleting empty deck: {}", deck);
        engine.client().decks()
            .delete(&[deck.as_str()], false)
            .await?;
    }
}
```

## Performance-Based Organization

Tag cards based on how well you know them:

```rust,ignore
use ankit_engine::progress::PerformanceCriteria;

let report = engine.progress().tag_by_performance(
    "deck:Japanese",
    PerformanceCriteria::default(),
    "struggling",
    "mastered",
).await?;

println!("Tagged {} struggling, {} mastered",
    report.struggling_count, report.mastered_count);
```

Then move them:

```rust,ignore
// Move struggling cards to a separate deck
engine.organize()
    .move_by_tag("struggling", "Japanese::Review")
    .await?;
```

## Deck Statistics

Get an overview of your decks:

```rust,ignore
let deck_names = engine.client().decks().names().await?;

for name in &deck_names {
    let health = engine.progress().deck_health(name).await?;

    println!("{}:", name);
    println!("  Total: {}", health.total_cards);
    println!("  New: {}", health.new_cards);
    println!("  Learning: {}", health.learning_cards);
    println!("  Review: {}", health.review_cards);
    println!("  Suspended: {}", health.suspended_cards);
    println!("  Leeches: {}", health.leech_count);
    println!();
}
```
