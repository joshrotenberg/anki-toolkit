# Using the MCP Server

The ankit MCP server exposes 50 tools that let AI assistants manage your Anki decks.

## How It Works

1. You ask Claude (or another AI assistant) to do something with your Anki decks
2. Claude chooses the appropriate tool(s) and executes them
3. The MCP server communicates with Anki via AnkiConnect
4. Claude reports the results back to you

## Tool Categories

### Notes
Create, search, update, and delete flashcard notes.

### Cards
Manage individual cards - suspend, unsuspend, reset progress, adjust ease factors.

### Tags
Add, remove, or rename tags across your collection.

### Decks
List, create, delete, clone, and merge decks.

### Analysis
Get study statistics, find problem cards (leeches), check retention rates.

### Import/Export
Bulk import notes, export decks to JSON or TOML format.

### Deduplication
Find and remove duplicate notes.

### Media
Audit your media files and clean up orphaned files.

### TOML Builder
Work with TOML deck definitions - import, export, diff, and sync.

## Example Conversation

**You:** "Show me my study stats for the Japanese deck over the last 30 days"

**Claude:** Uses `study_summary` tool, then responds:
> "In the last 30 days, you reviewed 1,247 cards in your Japanese deck with an 87% retention rate. You've studied an average of 42 cards per day."

**You:** "Are there any cards I'm struggling with?"

**Claude:** Uses `find_problems` tool, then responds:
> "I found 12 leech cards (cards with 5+ lapses). Would you like me to list them or suggest what to do with them?"

## Best Practices

### Be Specific
Instead of "add some vocabulary", say "add a note to my Spanish deck with front 'hola' and back 'hello'".

### Use Deck Names
Always specify which deck you're working with to avoid confusion.

### Preview Before Bulk Operations
For operations like deduplication, ask Claude to preview first:
> "Find duplicates in my vocabulary deck but don't remove them yet"

### Use Read-Only Mode for Exploration
If you're just exploring, configure the server with `--read-only` to prevent accidental changes.
