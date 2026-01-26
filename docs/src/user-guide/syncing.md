# Syncing with Anki

Keep your TOML deck definitions in sync with your Anki collection.

## Sync Strategies

### Push Only (TOML to Anki)

Send changes from your TOML file to Anki:

> "Sync my vocabulary.toml to Anki using push-only strategy"

- New notes in TOML are added to Anki
- Modified notes in TOML update Anki
- Notes only in Anki are left unchanged

### Pull Only (Anki to TOML)

Export changes from Anki to your TOML file:

> "Export my Japanese deck to japanese.toml"

- Fetches all notes from the deck
- Converts HTML to Markdown (for markdown_fields)
- Preserves note IDs for future syncing

### Bidirectional

Sync changes in both directions:

> "Do a bidirectional sync between my deck.toml and Anki"

- Pushes TOML changes to Anki
- Pulls Anki changes to TOML
- Handles conflicts based on your chosen resolution

## Previewing Changes

Always preview before syncing:

> "Show me what would change if I sync vocabulary.toml with Anki"

This shows:
- Notes only in TOML (would be added to Anki)
- Notes only in Anki (would be pulled to TOML)
- Modified notes (differences in both)
- Unchanged notes

## Comparing (Diff)

See detailed differences:

> "Compare my deck.toml against what's in Anki"

Shows:
- Field-by-field changes
- Tag differences
- New and deleted notes

## Note ID Tracking

After syncing, note IDs are recorded in your TOML:

```toml
[[notes]]
deck = "Spanish"
model = "Basic"
note_id = 1234567890  # Assigned after first sync

[notes.fields]
Front = "hola"
Back = "hello"
```

This enables:
- Updating existing notes (instead of creating duplicates)
- Tracking which notes came from Anki
- Round-trip syncing

## Conflict Resolution

When a note differs in both TOML and Anki:

| Strategy | Behavior |
|----------|----------|
| `prefer_toml` | TOML version wins |
| `prefer_anki` | Anki version wins |
| `skip` | Leave unchanged (default) |
| `fail` | Stop and report error |

## Workflow Example

1. **Export from Anki**:
   > "Export my Spanish deck to spanish.toml"

2. **Edit the TOML** in your text editor

3. **Preview changes**:
   > "Show what would change if I sync spanish.toml"

4. **Sync**:
   > "Sync spanish.toml to Anki"

5. **Commit to git**:
   ```bash
   git add spanish.toml
   git commit -m "Update Spanish vocabulary"
   ```
