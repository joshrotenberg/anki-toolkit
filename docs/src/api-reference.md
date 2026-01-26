# API Reference

Full API documentation is available on docs.rs:

- [ankit](https://docs.rs/ankit) - Core AnkiConnect client
- [ankit-engine](https://docs.rs/ankit-engine) - High-level workflows
- [ankit-builder](https://docs.rs/ankit-builder) - TOML deck builder

## Quick Reference

### AnkiClient Methods

```rust,ignore
let client = AnkiClient::new();

// Decks
client.decks().names()
client.decks().create(name)
client.decks().delete(names, cards_too)
client.decks().config(name)
client.decks().stats(names)

// Notes
client.notes().add(note)
client.notes().add_many(notes)
client.notes().find(query)
client.notes().info(note_ids)
client.notes().update_fields(note_id, fields)
client.notes().delete(note_ids)
client.notes().add_tags(note_ids, tags)
client.notes().remove_tags(note_ids, tags)

// Cards
client.cards().find(query)
client.cards().info(card_ids)
client.cards().suspend(card_ids)
client.cards().unsuspend(card_ids)
client.cards().forget(card_ids)
client.cards().get_ease(card_ids)
client.cards().set_ease(card_ids, ease_factors)

// Models
client.models().names()
client.models().field_names(model_name)
client.models().create(params)
client.models().templates(model_name)

// Media
client.media().store(params)
client.media().retrieve(filename)
client.media().list(pattern)
client.media().delete(filename)

// Statistics
client.statistics().cards_reviewed_today()
client.statistics().cards_reviewed_by_day()

// Misc
client.misc().version()
client.misc().sync()
```

### Engine Methods

```rust,ignore
let engine = Engine::new();

// Import
engine.import().notes(notes, on_duplicate)
engine.import().validate(notes)

// Export
engine.export().deck(deck_name)
engine.export().reviews(query)

// Organize
engine.organize().clone_deck(source, dest)
engine.organize().merge_decks(sources, dest)
engine.organize().move_by_tag(tag, dest)

// Analyze
engine.analyze().study_summary(deck, days)
engine.analyze().retention_stats(deck)
engine.analyze().find_problems(query, criteria)

// Progress
engine.progress().reset_deck(deck)
engine.progress().tag_by_performance(query, criteria, struggling_tag, mastered_tag)
engine.progress().suspend_by_criteria(query, criteria)
engine.progress().deck_health(deck)

// Media
engine.media().audit()
engine.media().cleanup(dry_run)

// Enrich
engine.enrich().find_candidates(query, field)
engine.enrich().update_note(note_id, field, value)

// Deduplicate
engine.deduplicate().find(query, key_field)
engine.deduplicate().remove(query, key_field, keep)
```

### DeckBuilder Methods

```rust,ignore
let builder = DeckBuilder::from_file(path)?;
let builder = DeckBuilder::parse(toml_str)?;

builder.definition()
builder.media_base_path(path)
builder.write_apkg(path)?
builder.import_connect().await?
builder.import_connect_batch().await?
```

## Common Types

### Note

```rust,ignore
let note = NoteBuilder::new(deck, model)
    .field(name, value)
    .tag(tag)
    .tags(vec![...])
    .audio(attachment)
    .picture(attachment)
    .video(attachment)
    .allow_duplicate(bool)
    .build();
```

### Query Syntax

```
deck:Name           # Deck filter
tag:name            # Tag filter
is:due              # Due cards
is:new              # New cards
is:review           # Review cards
is:suspended        # Suspended cards
rated:N             # Reviewed in last N days
prop:lapses>=N      # Property comparisons
prop:ease<N
"exact phrase"      # Phrase search
field:value         # Field search
-filter             # NOT
filter1 filter2     # AND
filter1 OR filter2  # OR
```
