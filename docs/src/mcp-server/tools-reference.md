# Tools Reference

The MCP server exposes 46 tools organized into categories.

## Raw API - Notes

| Tool | Description | Write |
|------|-------------|-------|
| `add_note` | Add single flashcard | Yes |
| `find_notes` | Search notes by query | No |
| `get_notes_info` | Get note details | No |
| `update_note` | Update note fields | Yes |
| `delete_notes` | Delete notes | Yes |

## Raw API - Cards

| Tool | Description | Write |
|------|-------------|-------|
| `find_cards` | Search cards by query | No |
| `get_cards_info` | Full card details (reps, lapses, ease, interval) | No |
| `suspend_cards` | Suspend cards from reviews | Yes |
| `unsuspend_cards` | Unsuspend cards | Yes |
| `forget_cards` | Reset cards to new state | Yes |
| `set_ease` | Adjust ease factors | Yes |

## Raw API - Tags

| Tool | Description | Write |
|------|-------------|-------|
| `add_tags` | Add tags to notes | Yes |
| `remove_tags` | Remove tags from notes | Yes |
| `replace_tags_all` | Rename tag globally | Yes |
| `clear_unused_tags` | Cleanup orphaned tags | Yes |

## Raw API - Decks/Models/Misc

| Tool | Description | Write |
|------|-------------|-------|
| `list_decks` | List all decks | No |
| `create_deck` | Create new deck | Yes |
| `delete_deck` | Delete deck | Yes |
| `list_models` | List note types | No |
| `get_model_fields` | Get model field names | No |
| `sync` | Sync with AnkiWeb | Yes |
| `version` | Get AnkiConnect version | No |

## Engine Workflows - Import/Export

| Tool | Description | Write |
|------|-------------|-------|
| `import_notes` | Bulk import with duplicate handling | Yes |
| `validate_notes` | Validate notes before import | No |
| `export_deck` | Export deck as JSON | No |
| `export_reviews` | Export review history | No |

## Engine Workflows - Organize

| Tool | Description | Write |
|------|-------------|-------|
| `clone_deck` | Clone deck with notes | Yes |
| `merge_decks` | Merge multiple decks | Yes |
| `move_by_tag` | Move notes by tag | Yes |

## Engine Workflows - Analyze

| Tool | Description | Write |
|------|-------------|-------|
| `study_summary` | Study statistics | No |
| `find_problems` | Find leech cards | No |
| `retention_stats` | Retention statistics | No |

## Engine Workflows - Media

| Tool | Description | Write |
|------|-------------|-------|
| `audit_media` | Find orphaned media | No |
| `cleanup_media` | Delete orphaned media | Yes |

## Engine Workflows - Progress

| Tool | Description | Write |
|------|-------------|-------|
| `reset_deck_progress` | Reset all cards to new state | Yes |
| `tag_by_performance` | Auto-tag struggling/mastered cards | Yes |
| `suspend_by_criteria` | Suspend cards matching criteria | Yes |
| `deck_health_report` | Comprehensive deck analysis | No |
| `bulk_tag_operation` | Bulk add/remove/replace tags | Yes |

## Query Syntax

Many tools accept an Anki search query. Common patterns:

```
deck:Japanese           # Cards in Japanese deck
tag:verb                # Notes with verb tag
is:due                  # Due cards
is:new                  # New cards
is:review               # Review cards
is:suspended            # Suspended cards
rated:7                 # Reviewed in last 7 days
prop:lapses>=5          # Cards with 5+ lapses
prop:ease<2             # Cards with ease under 200%
"exact phrase"          # Exact phrase in any field
Front:hello             # Search in Front field
```

Combine with spaces (AND) or `OR`:

```
deck:Japanese tag:verb
deck:Japanese OR deck:Korean
-is:suspended           # NOT suspended
```
