# ankit Project Context

## Overview

Rust toolkit for Anki deck management via AnkiConnect. Four crates in a workspace:

- **ankit**: Core 1:1 AnkiConnect API client (106 methods, fully tested)
- **ankit-engine**: High-level workflow operations (import, export, analyze, organize, media, migrate, progress, enrich, deduplicate)
- **ankit-mcp**: MCP server exposing engine + raw API as tools
- **ankit-builder**: TOML-based deck builder with .apkg generation and AnkiConnect import

## Current Status

All crates complete and passing tests (129 integration + 88 doctests = 217 total).

## MCP Server Tools (46 total)

### Raw API - Notes (5)
| Tool | Description | Write |
|------|-------------|-------|
| `add_note` | Add single flashcard | Yes |
| `find_notes` | Search notes by query | No |
| `get_notes_info` | Get note details | No |
| `update_note` | Update note fields | Yes |
| `delete_notes` | Delete notes | Yes |

### Raw API - Cards (6)
| Tool | Description | Write |
|------|-------------|-------|
| `find_cards` | Search cards by query | No |
| `get_cards_info` | Full card details (reps, lapses, ease, interval) | No |
| `suspend_cards` | Suspend cards from reviews | Yes |
| `unsuspend_cards` | Unsuspend cards | Yes |
| `forget_cards` | Reset cards to new state | Yes |
| `set_ease` | Adjust ease factors | Yes |

### Raw API - Tags (4)
| Tool | Description | Write |
|------|-------------|-------|
| `add_tags` | Add tags to notes | Yes |
| `remove_tags` | Remove tags from notes | Yes |
| `replace_tags_all` | Rename tag globally | Yes |
| `clear_unused_tags` | Cleanup orphaned tags | Yes |

### Raw API - Decks/Models/Misc (7)
| Tool | Description | Write |
|------|-------------|-------|
| `list_decks` | List all decks | No |
| `create_deck` | Create new deck | Yes |
| `delete_deck` | Delete deck (optionally with cards) | Yes |
| `list_models` | List note types | No |
| `get_model_fields` | Get model field names | No |
| `sync` | Sync with AnkiWeb | Yes |
| `version` | Get AnkiConnect version | No |

### Engine Workflows - Import/Export (4)
| Tool | Description | Write |
|------|-------------|-------|
| `import_notes` | Bulk import with duplicate handling | Yes |
| `validate_notes` | Validate notes before import | No |
| `export_deck` | Export deck as JSON | No |
| `export_reviews` | Export review history | No |

### Engine Workflows - Organize (3)
| Tool | Description | Write |
|------|-------------|-------|
| `clone_deck` | Clone deck with notes | Yes |
| `merge_decks` | Merge multiple decks | Yes |
| `move_by_tag` | Move notes by tag | Yes |

### Engine Workflows - Analyze (3)
| Tool | Description | Write |
|------|-------------|-------|
| `study_summary` | Study statistics | No |
| `find_problems` | Find leech cards | No |
| `retention_stats` | Retention statistics | No |

### Engine Workflows - Media (2)
| Tool | Description | Write |
|------|-------------|-------|
| `audit_media` | Find orphaned media | No |
| `cleanup_media` | Delete orphaned media | Yes (unless dry_run) |

### Engine Workflows - Progress (5)
| Tool | Description | Write |
|------|-------------|-------|
| `reset_deck_progress` | Reset all cards to new state | Yes |
| `tag_by_performance` | Auto-tag struggling/mastered cards | Yes |
| `suspend_by_criteria` | Suspend cards matching criteria | Yes |
| `deck_health_report` | Comprehensive deck analysis | No |
| `bulk_tag_operation` | Bulk add/remove/replace tags | Yes |

### Engine Workflows - Enrich (3)
| Tool | Description | Write |
|------|-------------|-------|
| `find_enrich_candidates` | Find notes with empty fields | No |
| `enrich_note` | Update single note with content | Yes |
| `enrich_notes` | Update multiple notes with content | Yes |

### Engine Workflows - Deduplicate (4)
| Tool | Description | Write |
|------|-------------|-------|
| `find_duplicates` | Find duplicate notes by key field | No |
| `preview_deduplicate` | Preview deduplication results | No |
| `remove_duplicates` | Remove duplicate notes | Yes |

## CLI Options

```
--host <HOST>     AnkiConnect host [default: 127.0.0.1]
--port <PORT>     AnkiConnect port [default: 8765]
--read-only       Disable write operations
-v, --verbose     Logging level (-v=info, -vv=debug, -vvv=trace)
```

## Testing the MCP Server

After restart, test these scenarios:

1. **Connection check**: `version` - verify Anki is running
2. **Read operations**: `list_decks`, `list_models`, `find_notes`, `find_cards`
3. **Card info**: `get_cards_info` with card IDs
4. **Analysis**: `study_summary`, `find_problems`, `retention_stats`, `deck_health_report`
5. **Export**: `export_deck`, `export_reviews`
6. **Write operations**: `add_note`, `create_deck`, `delete_deck`, `import_notes`
7. **Card management**: `suspend_cards`, `unsuspend_cards`, `forget_cards`, `set_ease`
8. **Tag operations**: `add_tags`, `remove_tags`, `replace_tags_all`, `clear_unused_tags`
9. **Progress workflows**: `reset_deck_progress`, `tag_by_performance`, `suspend_by_criteria`, `bulk_tag_operation`
10. **Organize**: `clone_deck`, `merge_decks`, `move_by_tag`
11. **Media**: `audit_media`, `cleanup_media` (with dry_run=true first)
12. **Sync**: `sync` - sync with AnkiWeb

## Key Files

- `crates/ankit-mcp/src/main.rs` - MCP server implementation
- `crates/ankit-engine/src/lib.rs` - Engine with workflow modules
- `crates/ankit/src/client.rs` - AnkiConnect client
- `crates/ankit-builder/src/lib.rs` - TOML deck builder
- `crates/ankit-builder/src/schema.rs` - TOML schema types
- `crates/ankit-builder/src/apkg.rs` - .apkg file generation
- `.mcp.json` - Local MCP config for testing

## Architecture Patterns

### ankit (Core Client)

Action modules in `src/actions/`: cards.rs, decks.rs, notes.rs, models.rs, media.rs, statistics.rs, etc.

```rust
// Each action module follows this pattern:
pub struct Cards<'a> { client: &'a AnkiClient }
impl<'a> Cards<'a> {
    pub async fn find(&self, query: &str) -> Result<Vec<i64>> { ... }
    pub async fn info(&self, card_ids: &[i64]) -> Result<Vec<CardInfo>> { ... }
}

// Client provides factory methods:
impl AnkiClient {
    pub fn cards(&self) -> Cards<'_> { Cards::new(self) }
    pub fn notes(&self) -> Notes<'_> { Notes::new(self) }
}
```

### ankit-engine (Workflows)

Each workflow module has a dedicated engine struct with lifetime borrowing the client:

```rust
// crates/ankit-engine/src/{module}.rs
pub struct ImportEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> ImportEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self { Self { client } }
    pub async fn notes(&self, notes: &[Note], on_dup: OnDuplicate) -> Result<ImportReport> {
        // Compose multiple client calls
    }
}

// lib.rs exposes via Engine:
impl Engine {
    pub fn import(&self) -> ImportEngine<'_> { ImportEngine::new(&self.client) }
}
```

Features: import, export, organize, analyze, media, migrate, progress (all default-enabled).

### ankit-mcp (MCP Server)

Uses rmcp macros for tool registration:

```rust
#[derive(Clone)]
struct AnkiServer {
    engine: Arc<Engine>,
    tool_router: ToolRouter<AnkiServer>,
    read_only: bool,
}

// Parameter structs for each tool:
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AddNoteParams {
    deck: String,
    model: String,
    fields: HashMap<String, String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[tool_router]
impl AnkiServer {
    #[tool(description = "Add a single flashcard note to Anki.")]
    async fn add_note(&self, Parameters(params): Parameters<AddNoteParams>) -> Result<CallToolResult, McpError> {
        self.check_write("add_note")?;  // Guard for write ops
        // ... implementation
        Ok(CallToolResult::success(vec![Content::text(...)]))
    }
}

#[tool_handler]
impl ServerHandler for AnkiServer { ... }
```

Pattern: Raw API tools call `self.engine.client().{module}()`, workflow tools call `self.engine.{module}()`.

## Future Considerations

### Workspace Structure (Complete)

The project has been renamed from `yanki` to `ankit`:
- `ankit` - Core AnkiConnect client
- `ankit-engine` - High-level workflows
- `ankit-mcp` - MCP server
- `ankit-builder` - TOML deck builder (implemented)
- `ankit-cli` - CLI tool (planned)

### ankit-builder (Implemented)

TOML-based deck builder with dual output: .apkg file generation and AnkiConnect import.

**Features:**
- `apkg` - Generate .apkg files using native SQLite (no genanki dependency)
- `connect` - Import via AnkiConnect

**Architecture:**
```
deck.toml
    |
    v
ankit-builder (parses TOML, validates)
    |
    +---> .apkg file (rusqlite + zip, schema v11)
    |
    +---> AnkiConnect (live import via ankit client)
```

**Example TOML format:**
```toml
[package]
name = "Italian::CILS B1"
version = "1.0.0"
author = "Your Name"

[[models]]
name = "CILS Vocabulary"
fields = ["Italiano", "English", "Example"]

[[models.templates]]
name = "Italian -> English"
front = "{{Italiano}}"
back = "{{FrontSide}}<hr>{{English}}<br><i>{{Example}}</i>"

[[decks]]
name = "Italian::CILS B1"
description = "CILS B1 exam vocabulary"

[[notes]]
deck = "Italian::CILS B1"
model = "CILS Vocabulary"
tags = ["lavoro", "b1"]

[notes.fields]
Italiano = "il colloquio"
English = "interview"
Example = "Ho un colloquio domani."
```

**Usage:**
```rust
use ankit_builder::{DeckBuilder, DeckDefinition};

// Generate .apkg
let builder = DeckBuilder::from_file("deck.toml")?;
builder.write_apkg("deck.apkg")?;

// Import via AnkiConnect
let result = builder.import_connect().await?;
```

**Key types:**
- `DeckDefinition` - Parsed TOML structure
- `DeckBuilder` - Unified builder for both outputs
- `ApkgBuilder` - Direct .apkg generation
- `ConnectImporter` - AnkiConnect import

**Implementation notes:**
- Uses schema v11 (legacy, maximum compatibility)
- Native .apkg generation (rusqlite + zip)
- Stable ID generation from names (deterministic)
- Media file support with manifest
- Full validation before generation

### Potential Raw API Additions

**Card operations** (from yanki cards.rs):
- `get_ease` - Get ease factors for cards
- `relearn_cards` - Put cards back into learning queue
- `answer_cards` - Answer cards programmatically
- `are_suspended` / `are_due` - Check card states

**Additional tag operations**:
- `get_tags` - Get tags for a note
- `replace_tags` - Replace tag on specific notes (not global)

### Potential Workflow Ideas

**Implemented:**
1. `convert_note_type` - Via `migrate` module: convert notes between models with field mapping, preserve tags and deck location
2. `enrich_notes` - Via `enrich` module: find notes with empty fields, update with new content
3. `deduplicate_notes` - Via `deduplicate` module: find and remove duplicates based on key field, with keep strategies (first, last, most_content, most_tags)

**Backup Workflows (planned - would use genanki-rs):**
- `backup_deck` - Export deck to .apkg file with media
- `restore_deck` - Import deck from .apkg backup
- `backup_collection` - Full collection snapshot
- `list_backups` - List available backups with metadata

**Other ideas:**
- `smart_suspend` - AI-assisted suspension based on content similarity
- `deck_comparison` - Compare two decks for overlap/differences
- `study_plan` - Generate study recommendations based on due cards

### Other
- Tool registry with read/write metadata
- CLI tool (yanki-cli)
- Web UI for workflow management
