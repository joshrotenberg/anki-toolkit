# Quick Start

This guide walks through common operations with the ankit library.

## Connect to Anki

```rust,ignore
use ankit::AnkiClient;

#[tokio::main]
async fn main() -> ankit::Result<()> {
    // Create client with default settings (localhost:8765)
    let client = AnkiClient::new();

    // Verify connection
    let version = client.misc().version().await?;
    println!("Connected to AnkiConnect v{}", version);

    Ok(())
}
```

## List Decks

```rust,ignore
let decks = client.decks().names().await?;
for deck in decks {
    println!("- {}", deck);
}
```

## Add a Note

```rust,ignore
use ankit::NoteBuilder;

let note = NoteBuilder::new("My Deck", "Basic")
    .field("Front", "What is the capital of France?")
    .field("Back", "Paris")
    .tag("geography")
    .tag("europe")
    .build();

let note_id = client.notes().add(note).await?;
println!("Created note: {}", note_id);
```

## Find and Update Notes

```rust,ignore
use std::collections::HashMap;

// Find notes with a specific tag
let note_ids = client.notes().find("tag:geography").await?;

// Get note details
let notes = client.notes().info(&note_ids).await?;

// Update a note's fields
let mut fields = HashMap::new();
fields.insert("Back".to_string(), "Paris, France".to_string());
client.notes().update_fields(note_ids[0], &fields).await?;
```

## Using the Engine for Workflows

```rust,ignore
use ankit_engine::Engine;

let engine = Engine::new();

// Get study summary for the last 30 days
let stats = engine.analyze().study_summary("Japanese", 30).await?;
println!("Total reviews: {}", stats.total_reviews);
println!("Average per day: {:.1}", stats.avg_reviews_per_day);

// Find problem cards (leeches)
use ankit_engine::analyze::ProblemCriteria;

let criteria = ProblemCriteria {
    min_lapses: 5,
    ..Default::default()
};
let problems = engine.analyze().find_problems("deck:Japanese", criteria).await?;

for card in problems {
    println!("Leech: {} ({} lapses)", card.front, card.lapses);
}
```

## Build a Deck from TOML

```rust,ignore
use ankit_builder::DeckBuilder;

let toml = r#"
[package]
name = "Vocabulary"
version = "1.0.0"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{FrontSide}}<hr>{{Back}}"

[[decks]]
name = "Vocabulary"

[[notes]]
deck = "Vocabulary"
model = "Basic"

[notes.fields]
Front = "Hello"
Back = "World"
"#;

let builder = DeckBuilder::parse(toml)?;

// Generate .apkg file
builder.write_apkg("vocabulary.apkg")?;

// Or import directly to Anki
let result = builder.import_connect().await?;
println!("Created {} notes", result.notes_created);
```

## Error Handling

```rust,ignore
use ankit::{AnkiClient, Error};

let client = AnkiClient::new();

match client.decks().names().await {
    Ok(decks) => println!("Found {} decks", decks.len()),
    Err(Error::ConnectionRefused) => {
        eprintln!("Is Anki running with AnkiConnect?");
    }
    Err(Error::PermissionDenied) => {
        eprintln!("Check your API key configuration");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```
