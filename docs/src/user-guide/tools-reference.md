# Available Tools

The MCP server provides 50 tools organized by category.

## Notes (5 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `add_note` | Add a single flashcard | Yes |
| `find_notes` | Search notes by query | No |
| `get_notes_info` | Get detailed note information | No |
| `update_note` | Update note fields | Yes |
| `delete_notes` | Delete notes | Yes |

## Cards (6 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `find_cards` | Search cards by query | No |
| `get_cards_info` | Get card details (reps, lapses, ease, interval) | No |
| `suspend_cards` | Suspend cards from reviews | Yes |
| `unsuspend_cards` | Unsuspend cards | Yes |
| `forget_cards` | Reset cards to new state | Yes |
| `set_ease` | Adjust ease factors | Yes |

## Tags (4 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `add_tags` | Add tags to notes | Yes |
| `remove_tags` | Remove tags from notes | Yes |
| `replace_tags_all` | Rename a tag globally | Yes |
| `clear_unused_tags` | Remove orphaned tags | Yes |

## Decks & Models (7 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `list_decks` | List all deck names | No |
| `create_deck` | Create a new deck | Yes |
| `delete_deck` | Delete a deck | Yes |
| `list_models` | List note types | No |
| `get_model_fields` | Get field names for a model | No |
| `sync` | Sync with AnkiWeb | Yes |
| `version` | Check AnkiConnect version | No |

## Import/Export (4 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `import_notes` | Bulk import with duplicate handling | Yes |
| `validate_notes` | Validate notes before import | No |
| `export_deck` | Export deck as JSON | No |
| `export_reviews` | Export review history | No |

## Organization (3 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `clone_deck` | Clone a deck with all notes | Yes |
| `merge_decks` | Merge multiple decks | Yes |
| `move_by_tag` | Move notes by tag to another deck | Yes |

## Analysis (4 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `study_summary` | Get study statistics | No |
| `find_problems` | Find leech cards | No |
| `retention_stats` | Get retention statistics | No |
| `deck_health_report` | Comprehensive deck analysis | No |

## Progress Management (4 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `reset_deck_progress` | Reset all cards to new | Yes |
| `tag_by_performance` | Auto-tag struggling/mastered cards | Yes |
| `suspend_by_criteria` | Suspend cards by ease/lapses | Yes |
| `bulk_tag_operation` | Bulk add/remove/replace tags | Yes |

## Media (2 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `audit_media` | Find orphaned media files | No |
| `cleanup_media` | Delete orphaned media | Yes |

## Enrichment (3 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `find_enrich_candidates` | Find notes with empty fields | No |
| `enrich_note` | Update a single note | Yes |
| `enrich_notes` | Update multiple notes | Yes |

## Deduplication (3 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `find_duplicates` | Find duplicate notes | No |
| `preview_deduplicate` | Preview what would be deleted | No |
| `remove_duplicates` | Remove duplicate notes | Yes |

## TOML Builder (5 tools)

| Tool | Description | Modifies Data |
|------|-------------|---------------|
| `export_deck_toml` | Export deck to TOML format | No |
| `diff_deck_toml` | Compare TOML against Anki | No |
| `plan_sync_toml` | Preview sync changes | No |
| `sync_deck_toml` | Sync TOML with Anki | Yes |
| `import_deck_toml` | Import TOML deck definition | Yes |

## Query Syntax

Many tools accept Anki search queries:

```
deck:Japanese          # Cards in a deck
tag:verb              # Cards with a tag
is:due                # Due cards
is:new                # New cards
is:suspended          # Suspended cards
rated:7               # Reviewed in last 7 days
prop:lapses>=5        # Cards with 5+ lapses (leeches)
prop:ease<2.0         # Low ease cards
"exact phrase"        # Phrase search
front:hello           # Field search
```

Combine with AND (space) or OR:

```
deck:Japanese tag:verb           # Japanese verbs
deck:Spanish OR deck:French      # Spanish or French cards
deck:Japanese -is:suspended      # Not suspended
```
