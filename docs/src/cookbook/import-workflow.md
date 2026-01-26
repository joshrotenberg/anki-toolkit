# Import Workflow

Patterns for importing notes into Anki programmatically.

## Simple Note Import

```rust,ignore
use ankit::{AnkiClient, NoteBuilder};

let client = AnkiClient::new();

let note = NoteBuilder::new("Vocabulary", "Basic")
    .field("Front", "hello")
    .field("Back", "world")
    .tag("imported")
    .build();

let note_id = client.notes().add(note).await?;
```

## Bulk Import with Engine

```rust,ignore
use ankit_engine::Engine;
use ankit_engine::import::OnDuplicate;

let engine = Engine::new();

let notes = vec![
    NoteBuilder::new("Vocabulary", "Basic")
        .field("Front", "one").field("Back", "1").build(),
    NoteBuilder::new("Vocabulary", "Basic")
        .field("Front", "two").field("Back", "2").build(),
    NoteBuilder::new("Vocabulary", "Basic")
        .field("Front", "three").field("Back", "3").build(),
];

let report = engine.import()
    .notes(&notes, OnDuplicate::Skip)
    .await?;

println!("Created: {}", report.created);
println!("Skipped: {}", report.skipped);
println!("Updated: {}", report.updated);
```

## Duplicate Handling Options

```rust,ignore
use ankit_engine::import::OnDuplicate;

// Skip duplicates (default)
OnDuplicate::Skip

// Update existing notes with new field values
OnDuplicate::Update

// Allow duplicates (creates new notes)
OnDuplicate::Allow
```

## Import from CSV

```rust,ignore
use std::fs::File;
use csv::Reader;

let mut reader = Reader::from_path("vocabulary.csv")?;

let mut notes = Vec::new();
for result in reader.records() {
    let record = result?;
    let note = NoteBuilder::new("Vocabulary", "Basic")
        .field("Front", &record[0])
        .field("Back", &record[1])
        .build();
    notes.push(note);
}

let report = engine.import()
    .notes(&notes, OnDuplicate::Skip)
    .await?;
```

## Import from JSON

```rust,ignore
use serde::Deserialize;

#[derive(Deserialize)]
struct VocabEntry {
    word: String,
    definition: String,
    tags: Vec<String>,
}

let json_data = std::fs::read_to_string("vocabulary.json")?;
let entries: Vec<VocabEntry> = serde_json::from_str(&json_data)?;

let notes: Vec<_> = entries.iter().map(|e| {
    let mut builder = NoteBuilder::new("Vocabulary", "Basic")
        .field("Front", &e.word)
        .field("Back", &e.definition);
    for tag in &e.tags {
        builder = builder.tag(tag);
    }
    builder.build()
}).collect();

let report = engine.import()
    .notes(&notes, OnDuplicate::Skip)
    .await?;
```

## Validate Before Import

```rust,ignore
// Check if all notes can be added
let can_add = client.notes().can_add(&notes).await?;

let valid_notes: Vec<_> = notes.into_iter()
    .zip(can_add.iter())
    .filter(|(_, &can_add)| can_add)
    .map(|(note, _)| note)
    .collect();

println!("{} of {} notes are valid", valid_notes.len(), can_add.len());
```

## Error Handling for Large Imports

```rust,ignore
let mut created = 0;
let mut errors = Vec::new();

for (i, note) in notes.into_iter().enumerate() {
    match client.notes().add(note).await {
        Ok(_) => created += 1,
        Err(e) => errors.push((i, e.to_string())),
    }
}

println!("Created {} notes", created);
if !errors.is_empty() {
    println!("Errors:");
    for (i, err) in &errors {
        println!("  Note {}: {}", i, err);
    }
}
```

## Import with Media

```rust,ignore
use ankit::{NoteBuilder, MediaAttachment};

let note = NoteBuilder::new("Vocabulary", "Basic")
    .field("Front", "pronunciation")
    .field("Back", "the way a word is spoken")
    .audio(MediaAttachment {
        url: Some("https://example.com/pronunciation.mp3".to_string()),
        data: None,
        path: None,
        filename: "pronunciation.mp3".to_string(),
        fields: vec!["Front".to_string()],
        skip_hash: None,
    })
    .build();

client.notes().add(note).await?;
```
