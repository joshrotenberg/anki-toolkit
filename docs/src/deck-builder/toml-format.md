# TOML Format

The deck builder uses TOML to define Anki decks declaratively.

## Basic Structure

```toml
[package]
name = "My Deck"
version = "1.0.0"
author = "Your Name"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{FrontSide}}<hr>{{Back}}"

[[decks]]
name = "My Deck"
description = "Optional description"

[[notes]]
deck = "My Deck"
model = "Basic"
tags = ["optional", "tags"]

[notes.fields]
Front = "Question"
Back = "Answer"
```

## Package Section

Required metadata about the deck package.

```toml
[package]
name = "Spanish Vocabulary"  # Required: Package name
version = "1.0.0"            # Required: Semantic version
author = "Your Name"         # Optional: Author name
description = "..."          # Optional: Package description
```

## Models Section

Define note types (models) with fields and card templates.

```toml
[[models]]
name = "Vocabulary"          # Required: Model name
fields = ["Word", "Definition", "Example"]  # Required: Field names
css = ".card { font-size: 20px; }"  # Optional: Styling

[[models.templates]]
name = "Word -> Definition"  # Required: Template name
front = "{{Word}}"           # Required: Front template
back = "{{FrontSide}}<hr>{{Definition}}<br><i>{{Example}}</i>"  # Required: Back template
```

### Multiple Templates

A model can have multiple card templates:

```toml
[[models]]
name = "Bidirectional"
fields = ["Front", "Back"]

[[models.templates]]
name = "Forward"
front = "{{Front}}"
back = "{{Back}}"

[[models.templates]]
name = "Reverse"
front = "{{Back}}"
back = "{{Front}}"
```

### Cloze Deletion

```toml
[[models]]
name = "Cloze"
fields = ["Text", "Extra"]
cloze = true

[[models.templates]]
name = "Cloze"
front = "{{cloze:Text}}"
back = "{{cloze:Text}}<br>{{Extra}}"
```

## Decks Section

Define deck hierarchy.

```toml
[[decks]]
name = "Languages::Spanish"  # Use :: for nested decks
description = "Spanish vocabulary cards"
```

## Notes Section

Define individual flashcards.

```toml
[[notes]]
deck = "Languages::Spanish"  # Must match a deck name
model = "Vocabulary"         # Must match a model name
tags = ["beginner", "food"]  # Optional tags

[notes.fields]
Word = "el gato"
Definition = "the cat"
Example = "El gato es negro."
```

### Multiple Notes

```toml
[[notes]]
deck = "Spanish"
model = "Basic"
[notes.fields]
Front = "hola"
Back = "hello"

[[notes]]
deck = "Spanish"
model = "Basic"
[notes.fields]
Front = "adios"
Back = "goodbye"
```

## Media Section

Reference media files for audio, images, and video.

```toml
[[media]]
filename = "audio_hola.mp3"
path = "media/hola.mp3"  # Relative to TOML file or media_base_path
```

Then reference in note fields:

```toml
[notes.fields]
Front = "hola [sound:audio_hola.mp3]"
```

## Complete Example

```toml
[package]
name = "Spanish::DELE B1"
version = "1.0.0"
author = "Language Learner"

[[models]]
name = "DELE Vocabulary"
fields = ["Spanish", "English", "Example", "Audio"]
css = """
.card {
    font-family: Arial, sans-serif;
    font-size: 22px;
    text-align: center;
}
.example {
    font-style: italic;
    color: #666;
}
"""

[[models.templates]]
name = "Spanish -> English"
front = "{{Spanish}}{{Audio}}"
back = """{{FrontSide}}
<hr>
{{English}}
<div class="example">{{Example}}</div>"""

[[decks]]
name = "Spanish::DELE B1"
description = "DELE B1 exam preparation vocabulary"

[[decks]]
name = "Spanish::DELE B1::Verbs"

[[notes]]
deck = "Spanish::DELE B1::Verbs"
model = "DELE Vocabulary"
tags = ["verb", "present-tense"]

[notes.fields]
Spanish = "trabajar"
English = "to work"
Example = "Trabajo en una oficina."
Audio = "[sound:trabajar.mp3]"

[[notes]]
deck = "Spanish::DELE B1"
model = "DELE Vocabulary"
tags = ["noun", "food"]

[notes.fields]
Spanish = "el pan"
English = "bread"
Example = "Compre pan en la panaderia."
```
