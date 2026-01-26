# APKG Generation

Generate `.apkg` files that can be imported directly into Anki.

## Basic Usage

```rust,ignore
use ankit_builder::DeckBuilder;

let builder = DeckBuilder::from_file("my_deck.toml")?;
builder.write_apkg("my_deck.apkg")?;
```

## From TOML String

```rust,ignore
let toml = r#"
[package]
name = "Test"
version = "1.0.0"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "Basic"
[notes.fields]
Front = "Hello"
Back = "World"
"#;

let builder = DeckBuilder::parse(toml)?;
builder.write_apkg("test.apkg")?;
```

## With Media Files

When your deck includes media (audio, images):

```rust,ignore
let builder = DeckBuilder::from_file("deck.toml")?
    .media_base_path("./media");  // Directory containing media files

builder.write_apkg("deck.apkg")?;
```

The TOML references media like:

```toml
[[media]]
filename = "pronunciation.mp3"
path = "audio/pronunciation.mp3"  # Relative to media_base_path
```

## How APKG Files Work

An `.apkg` file is a ZIP archive containing:

1. `collection.anki2` - SQLite database with:
   - Notes (the actual flashcard content)
   - Cards (generated from notes via templates)
   - Models (note types with field definitions)
   - Decks (organizational structure)

2. `media` - Directory of media files
3. `media` (JSON file) - Media file manifest

The builder generates Anki's schema v11 format for maximum compatibility.

## ID Generation

The builder generates deterministic IDs based on content:

- **Model IDs**: Hash of model name
- **Deck IDs**: Hash of deck name
- **Note IDs**: Hash of note content (fields + deck + model)
- **Card IDs**: Derived from note ID + template index

This means rebuilding the same TOML produces identical IDs, which helps with:
- Updating existing decks
- Version control friendliness
- Predictable behavior

## Importing the APKG

In Anki:

1. File > Import
2. Select the `.apkg` file
3. Choose import options:
   - Update existing notes if first field matches
   - Import as new notes
   - Skip duplicates

## Command Line (Planned)

Future CLI support:

```bash
# Build APKG from TOML
ankit build deck.toml -o deck.apkg

# Build with media directory
ankit build deck.toml --media ./assets -o deck.apkg
```

## Limitations

- Models must be fully defined in TOML (no inheritance)
- Media files must exist at build time
- Scheduling data (due dates, intervals) is not preserved
- Study progress starts fresh on import
