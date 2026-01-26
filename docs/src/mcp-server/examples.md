# MCP Server Examples

Common interactions with the ankit MCP server through Claude.

## Basic Operations

### Check Connection

> "Is Anki running? Can you check the AnkiConnect version?"

Claude will call the `version` tool to verify connectivity.

### List Decks

> "What decks do I have in Anki?"

Uses `list_decks` to show all available decks.

### Find Cards

> "How many cards are due in my Japanese deck?"

Uses `find_cards` with query `deck:Japanese is:due`.

## Study Analysis

### Study Summary

> "How has my studying been going this week in my Japanese deck?"

Uses `study_summary` with the deck name and 7 days.

### Find Problem Cards

> "Which cards in my Japanese deck am I struggling with?"

Uses `find_problems` to identify leeches (cards with high lapse counts).

### Deck Health

> "Give me a health report for my vocabulary deck"

Uses `deck_health_report` for comprehensive deck statistics including:
- Card counts by state (new, learning, review, suspended)
- Average ease and interval
- Leech count
- Total lapses

## Card Management

### Suspend Problem Cards

> "Suspend all cards in Japanese that have more than 8 lapses"

Uses `suspend_by_criteria` with min_lapses set to 8.

### Tag Performance

> "Tag my struggling and mastered cards in the Japanese deck"

Uses `tag_by_performance` to automatically tag cards based on:
- Struggling: Low ease factor or high lapse count
- Mastered: High ease with many successful reviews

### Reset Progress

> "Reset all progress in my test deck so I can start fresh"

Uses `reset_deck_progress` to convert all cards back to new state.

## Organization

### Clone a Deck

> "Make a copy of my Japanese Vocabulary deck called Japanese Review"

Uses `clone_deck` to duplicate the deck with all notes.

### Merge Decks

> "Merge my JLPT N5 and JLPT N4 decks into a single JLPT deck"

Uses `merge_decks` with source decks and destination.

### Move by Tag

> "Move all notes tagged 'difficult' to my Review deck"

Uses `move_by_tag` to relocate notes based on tags.

## Media Management

### Audit Media

> "Are there any orphaned media files in my Anki collection?"

Uses `audit_media` to find:
- Orphaned files (in media folder but not referenced)
- Missing references (referenced in notes but file missing)

### Cleanup Media

> "Delete the orphaned media files (show me what will be deleted first)"

First uses `cleanup_media` with `dry_run=true` to preview, then with `dry_run=false` to delete.

## Bulk Operations

### Add Tags

> "Add the tag 'review-2024' to all cards I've reviewed this month"

Uses `find_notes` with `rated:30` then `add_tags`.

### Import Notes

> "Import these vocabulary words into my Spanish deck"

Uses `import_notes` with duplicate handling options (skip, update, or allow).

### Export Deck

> "Export my Japanese deck so I can back it up"

Uses `export_deck` to get all notes as JSON.
