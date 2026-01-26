# AnkiConnect Import

Import deck definitions directly into a running Anki instance.

## Basic Usage

```rust,ignore
use ankit_builder::DeckBuilder;

let builder = DeckBuilder::from_file("deck.toml")?;
let result = builder.import_connect().await?;

println!("Decks created: {}", result.decks_created);
println!("Notes created: {}", result.notes_created);
println!("Notes skipped: {}", result.notes_skipped);
```

## Requirements

Unlike APKG generation, AnkiConnect import requires:

1. **Anki running** with AnkiConnect add-on
2. **Models must already exist** in Anki

The builder will:
- Create missing decks automatically
- Fail if a referenced model doesn't exist

## Validate Before Import

Check prerequisites before attempting import:

```rust,ignore
use ankit_builder::{ConnectImporter, DeckDefinition};

let definition = DeckDefinition::from_file("deck.toml")?;
let importer = ConnectImporter::new(definition);

// Check for missing models
let missing_models = importer.validate_models().await?;
if !missing_models.is_empty() {
    eprintln!("Missing models in Anki:");
    for model in &missing_models {
        eprintln!("  - {}", model);
    }
    eprintln!("Please create these note types manually first.");
    return Ok(());
}

// Check which decks will be created
let missing_decks = importer.validate_decks().await?;
if !missing_decks.is_empty() {
    println!("Will create {} decks", missing_decks.len());
}

// Proceed with import
let result = importer.import().await?;
```

## Batch Import

For large decks, batch import is more efficient:

```rust,ignore
let result = builder.import_connect_batch().await?;
```

This uses a single API call to add all notes, versus one call per note.

Trade-offs:
- **Single import**: Better error tracking (know exactly which note failed)
- **Batch import**: Faster for large decks (one API call)

## Handling Duplicates

By default, AnkiConnect rejects duplicate notes. The import result tracks skipped notes:

```rust,ignore
let result = builder.import_connect().await?;

if !result.errors.is_empty() {
    println!("Some notes were skipped:");
    for (index, error) in &result.errors {
        println!("  Note {}: {}", index, error);
    }
}
```

## Custom Client Configuration

Use a custom AnkiConnect client:

```rust,ignore
use ankit_builder::{ConnectImporter, DeckDefinition};
use ankit::AnkiClient;

let client = AnkiClient::builder()
    .url("http://localhost:8765")
    .api_key("your-api-key")
    .build();

let definition = DeckDefinition::from_file("deck.toml")?;
let importer = ConnectImporter::with_client(definition, client);
let result = importer.import().await?;
```

## When to Use Each Method

| Scenario | Method |
|----------|--------|
| Share deck with others | APKG |
| Backup deck | APKG |
| Quick iteration during development | AnkiConnect |
| CI/CD pipeline | AnkiConnect |
| No Anki running | APKG |
| Preserve models exactly | APKG |
| Use existing models | AnkiConnect |

## Error Handling

```rust,ignore
use ankit_builder::{DeckBuilder, Error};

match builder.import_connect().await {
    Ok(result) => {
        println!("Success: {} notes created", result.notes_created);
    }
    Err(Error::ModelNotFound(name)) => {
        eprintln!("Model '{}' not found in Anki", name);
    }
    Err(Error::Ankit(ankit::Error::ConnectionRefused)) => {
        eprintln!("Cannot connect to Anki. Is it running?");
    }
    Err(e) => {
        eprintln!("Import failed: {}", e);
    }
}
```
