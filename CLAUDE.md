# ankit Project Context

## Overview

Rust toolkit for Anki deck management via AnkiConnect. Four crates in a workspace:

- **ankit**: Core 1:1 AnkiConnect API client (106 methods, fully tested)
- **ankit-engine**: High-level workflow operations (import, export, analyze, organize, media, migrate, progress, enrich, deduplicate)
- **ankit-mcp**: MCP server exposing engine + raw API as tools
- **ankit-builder**: TOML-based deck builder with .apkg generation and AnkiConnect import

## Current Status

All crates complete and passing tests (448 total tests).

## MCP Server Tools (50 total)

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

### Engine Workflows - Deduplicate (3)
| Tool | Description | Write |
|------|-------------|-------|
| `find_duplicates` | Find duplicate notes by key field | No |
| `preview_deduplicate` | Preview deduplication results | No |
| `remove_duplicates` | Remove duplicate notes | Yes |

### TOML Builder (5)
| Tool | Description | Write |
|------|-------------|-------|
| `export_deck_toml` | Export deck to TOML format | No |
| `diff_deck_toml` | Compare TOML against Anki state | No |
| `plan_sync_toml` | Preview sync without changes | No |
| `sync_deck_toml` | Sync TOML with Anki (push/pull/bidirectional) | Yes |
| `import_deck_toml` | Import TOML deck definition | Yes |

## CLI Options

```
--host <HOST>       AnkiConnect host [default: 127.0.0.1]
--port <PORT>       AnkiConnect port [default: 8765]
--transport <TYPE>  Transport type: stdio or http [default: stdio]
--http-port <PORT>  HTTP server port [default: 3000]
--http-host <HOST>  HTTP server host [default: 127.0.0.1]
--read-only         Disable write operations
-v, --verbose       Logging level (-v=info, -vv=debug, -vvv=trace)
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
13. **TOML builder**: `export_deck_toml`, `diff_deck_toml`, `plan_sync_toml`, `sync_deck_toml`, `import_deck_toml`

## Key Files

- `crates/ankit-mcp/src/main.rs` - MCP server implementation
- `crates/ankit-engine/src/lib.rs` - Engine with workflow modules
- `crates/ankit/src/client.rs` - AnkiConnect client
- `crates/ankit-builder/src/lib.rs` - TOML deck builder
- `crates/ankit-builder/src/schema.rs` - TOML schema types
- `crates/ankit-builder/src/apkg.rs` - .apkg file generation
- `crates/ankit-builder/src/markdown.rs` - Markdown/HTML conversion
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

The workspace contains these crates:
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
- Markdown field conversion (bidirectional)

### Markdown Fields

Models can specify `markdown_fields` to enable bidirectional Markdown/HTML conversion:

```toml
[[models]]
name = "Vocabulary"
fields = ["Word", "Definition", "Examples"]
markdown_fields = ["Definition", "Examples"]  # These fields use Markdown
```

**How it works:**
- **Push to Anki**: Markdown in `markdown_fields` is converted to HTML before sending
- **Pull from Anki**: HTML in `markdown_fields` is converted back to Markdown
- Non-markdown fields are left unchanged

**Supported Markdown:**
- **Bold** (`**text**`) and *italic* (`*text*`)
- Lists (ordered and unordered)
- Links (`[text](url)`)
- Code blocks and inline code
- Blockquotes
- Strikethrough (`~~text~~`)

**API:**
```rust
// Convert all markdown fields to HTML (before import)
definition.markdown_to_html();

// Convert HTML to markdown (after export)
definition.html_to_markdown();

// Get fields with markdown converted (for single note)
let html_fields = note.fields_as_html(&model.markdown_fields);

// Set markdown fields for a model
definition.set_markdown_fields("Vocabulary", &["Definition", "Examples"]);
```

**Dependencies:**
- `pulldown-cmark` for Markdown to HTML
- `html2md` for HTML to Markdown

## Roadmap

Work is tracked via GitHub issues: https://github.com/joshrotenberg/anki-toolkit/issues

### API Coverage (~85%)

**Implemented:** 79 actions

**Missing (tracked in issues #1-4):**
- Card: `setDueDate`, `setSpecificValueOfCard` (#1)
- Note: `updateNote`, `updateNoteModel`, `updateNoteTags`, `getTags` (#2)
- Model: `findModelsById`, `findModelsByName`, template manipulation (#3)
- GUI: `guiSelectCard`, `guiAddNoteSetData`, `guiPlayAudio`, `getActiveProfile` (#4)

### Planned Features

| Feature | Issue | Priority |
|---------|-------|----------|
| Backup workflows | #5 | High |
| CLI tool (ankit-cli) | #6 | Medium |
| smart_suspend workflow | #7 | Low |
| deck_comparison workflow | #8 | Low |
| study_plan workflow | #9 | Low |
