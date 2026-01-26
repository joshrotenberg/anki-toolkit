# TOML Format Reference

Complete reference for the deck definition TOML format.

## Package Section

Required metadata about the deck package.

```toml
[package]
name = "My Deck"           # Required: package name
version = "1.0.0"          # Optional: version (default: "1.0.0")
author = "Your Name"       # Optional: author
description = "A deck"     # Optional: description
```

## Models Section

Define note types (models) with their fields and templates.

```toml
[[models]]
name = "Basic"                    # Required: unique model name
fields = ["Front", "Back"]        # Required: field names in order
sort_field = "Front"              # Optional: field to sort by (default: first)
markdown_fields = ["Back"]        # Optional: fields using Markdown
css = ".card { font-size: 20px }" # Optional: custom CSS

[[models.templates]]
name = "Card 1"                   # Required: template name
front = "{{Front}}"               # Required: front template
back = "{{FrontSide}}<hr>{{Back}}" # Required: back template
```

### Multiple Templates

For reverse cards:

```toml
[[models]]
name = "Basic (and reversed)"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[models.templates]]
name = "Card 2 (reversed)"
front = "{{Back}}"
back = "{{Front}}"
```

### Cloze Models

For cloze deletion cards, set `model_type = "cloze"`:

```toml
[[models]]
name = "My Cloze"
model_type = "cloze"
fields = ["Text", "Extra"]

[[models.templates]]
name = "Cloze"
front = "{{cloze:Text}}"
back = "{{cloze:Text}}<br>{{Extra}}"
```

Use cloze syntax in your notes:

```toml
[[notes]]
deck = "Vocabulary"
model = "My Cloze"

[notes.fields]
Text = "The {{c1::mitochondria}} is the powerhouse of the {{c2::cell}}."
Extra = "Biology 101"
```

This creates two cards:
- Card 1: "The [...] is the powerhouse of the cell."
- Card 2: "The mitochondria is the powerhouse of the [...]."

## Decks Section

Define decks (can use `::` for hierarchy).

```toml
[[decks]]
name = "Spanish::Vocabulary"      # Required: deck name
description = "Spanish vocab"     # Optional: description
```

## Notes Section

Define individual flashcard notes.

```toml
[[notes]]
deck = "Spanish::Vocabulary"      # Required: target deck
model = "Basic"                   # Required: model name
tags = ["food", "chapter1"]       # Optional: tags
guid = "unique-id-123"            # Optional: custom GUID
note_id = 1234567890             # Optional: Anki note ID (for updates)

[notes.fields]
Front = "el gato"
Back = "the cat"
```

### Multiline Content

Use triple quotes for long content:

```toml
[[notes]]
deck = "Spanish"
model = "Basic"

[notes.fields]
Front = "Translate this sentence"
Back = """
This is a longer explanation
that spans multiple lines.

It can include:
- Lists
- **Markdown** (if markdown_fields is set)
"""
```

## Media Section

Reference media files to include.

```toml
[[media]]
name = "audio.mp3"               # Filename in Anki
path = "./media/audio.mp3"       # Source file path
```

## Complete Example

```toml
[package]
name = "Spanish Vocabulary"
version = "1.0.0"
author = "Language Learner"

[[models]]
name = "Vocab"
fields = ["Spanish", "English", "Example"]
markdown_fields = ["Example"]

[[models.templates]]
name = "Spanish -> English"
front = "{{Spanish}}"
back = "{{English}}<br><br><i>{{Example}}</i>"

[[decks]]
name = "Spanish::Vocabulary"
description = "Core vocabulary"

[[notes]]
deck = "Spanish::Vocabulary"
model = "Vocab"
tags = ["food", "common"]

[notes.fields]
Spanish = "el gato"
English = "the cat"
Example = "**El gato** es negro."

[[notes]]
deck = "Spanish::Vocabulary"
model = "Vocab"
tags = ["food"]

[notes.fields]
Spanish = "la manzana"
English = "the apple"
Example = "Me gusta **la manzana** roja."
```
