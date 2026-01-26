//! MCP server for Anki deck management via AnkiConnect.
//!
//! This server exposes ankit-engine workflows and key raw API operations
//! as tools for LLM assistants like Claude.

use std::collections::HashMap;
use std::sync::Arc;

use ankit_engine::{
    ClientBuilder, Engine, NoteBuilder,
    analyze::ProblemCriteria,
    deduplicate::{DedupeQuery, KeepStrategy},
    enrich::EnrichQuery,
    import::OnDuplicate,
    progress::{PerformanceCriteria, SuspendCriteria, TagOperation},
};
use clap::Parser;
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
};
use tracing::{debug, info, warn};

// ============================================================================
// CLI Arguments
// ============================================================================

/// MCP server for Anki deck management via AnkiConnect.
#[derive(Parser, Debug)]
#[command(name = "ankit-mcp")]
#[command(version, about, long_about = None)]
struct Args {
    /// AnkiConnect host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// AnkiConnect port
    #[arg(long, default_value_t = 8765)]
    port: u16,

    /// Read-only mode (disables write operations)
    #[arg(long, default_value_t = false)]
    read_only: bool,

    /// Enable verbose logging (use multiple times for more verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Transport mode: stdio (default) or http
    #[arg(long, default_value = "stdio")]
    transport: Transport,

    /// HTTP server port (only used with --transport http)
    #[arg(long, default_value_t = 3000)]
    http_port: u16,

    /// HTTP server bind address (only used with --transport http)
    #[arg(long, default_value = "127.0.0.1")]
    http_host: String,
}

/// Transport mode for the MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Transport {
    /// Standard I/O transport (default, for CLI integration)
    #[default]
    Stdio,
    /// HTTP transport with SSE (for remote connections)
    Http,
}

impl std::str::FromStr for Transport {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stdio" => Ok(Transport::Stdio),
            "http" => Ok(Transport::Http),
            _ => Err(format!("Invalid transport: {}. Use 'stdio' or 'http'", s)),
        }
    }
}

// ============================================================================
// Parameter Types
// ============================================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AddNoteParams {
    /// Deck name to add the note to
    deck: String,
    /// Note type (model) name
    model: String,
    /// Field values (field_name -> value)
    fields: HashMap<String, String>,
    /// Optional tags
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct FindNotesParams {
    /// Anki search query (e.g., "deck:Japanese tag:verb")
    query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GetNotesInfoParams {
    /// Note IDs to get info for
    note_ids: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateNoteParams {
    /// Note ID to update
    note_id: i64,
    /// Field values to update (field_name -> value)
    fields: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteNotesParams {
    /// Note IDs to delete
    note_ids: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateDeckParams {
    /// Name of the deck to create
    name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteDeckParams {
    /// Name of the deck to delete
    name: String,
    /// If true, also delete all cards in the deck. If false, cards are moved to Default deck.
    #[serde(default)]
    cards_too: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GetModelFieldsParams {
    /// Model (note type) name
    model: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ImportNote {
    /// Deck name
    deck: String,
    /// Model (note type) name
    model: String,
    /// Field values
    fields: HashMap<String, String>,
    /// Tags
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ImportNotesParams {
    /// Notes to import
    notes: Vec<ImportNote>,
    /// How to handle duplicates: "skip", "update", or "allow"
    #[serde(default = "default_on_duplicate")]
    on_duplicate: String,
}

fn default_on_duplicate() -> String {
    "skip".to_string()
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ValidateNotesParams {
    /// Notes to validate
    notes: Vec<ImportNote>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ExportDeckParams {
    /// Deck name to export
    deck: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ExportReviewsParams {
    /// Anki search query to select cards
    query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CloneDeckParams {
    /// Source deck name
    source: String,
    /// Destination deck name
    destination: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MergeDecksParams {
    /// Source deck names to merge
    sources: Vec<String>,
    /// Destination deck name
    destination: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MoveByTagParams {
    /// Tag to search for
    tag: String,
    /// Destination deck name
    destination: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct StudySummaryParams {
    /// Deck name (use "*" for all decks)
    deck: String,
    /// Number of days to include
    days: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct FindProblemsParams {
    /// Anki search query
    query: String,
    /// Minimum lapse count to flag (default: 5)
    #[serde(default = "default_min_lapses")]
    min_lapses: i64,
}

fn default_min_lapses() -> i64 {
    5
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RetentionStatsParams {
    /// Deck name
    deck: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CleanupMediaParams {
    /// If true, only report what would be deleted
    dry_run: bool,
}

// Card operation params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct FindCardsParams {
    /// Anki search query (e.g., "deck:Japanese is:due")
    query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GetCardsInfoParams {
    /// Card IDs to get info for
    card_ids: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SuspendCardsParams {
    /// Card IDs to suspend
    card_ids: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UnsuspendCardsParams {
    /// Card IDs to unsuspend
    card_ids: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ForgetCardsParams {
    /// Card IDs to forget (reset to new state)
    card_ids: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SetEaseParams {
    /// Card IDs to set ease for
    card_ids: Vec<i64>,
    /// Ease factors as integers (e.g., 2500 = 250%)
    ease_factors: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SetDueDateParams {
    /// Card IDs to set due date for
    card_ids: Vec<i64>,
    /// Days specification: "0" (today), "1" (tomorrow), "-1" (yesterday), "1-7" (random range), "0!" (today and reset interval)
    days: String,
}

// Tag operation params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AddTagsParams {
    /// Note IDs to add tags to
    note_ids: Vec<i64>,
    /// Tags to add (space-separated string, e.g., "tag1 tag2")
    tags: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RemoveTagsParams {
    /// Note IDs to remove tags from
    note_ids: Vec<i64>,
    /// Tags to remove (space-separated string, e.g., "tag1 tag2")
    tags: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ReplaceTagsAllParams {
    /// Tag to replace
    old_tag: String,
    /// Replacement tag
    new_tag: String,
}

// Progress workflow params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ResetDeckProgressParams {
    /// Deck name to reset
    deck: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TagByPerformanceParams {
    /// Anki search query to filter cards
    query: String,
    /// Tag for struggling cards
    #[serde(default = "default_struggling_tag")]
    struggling_tag: String,
    /// Tag for mastered cards
    #[serde(default = "default_mastered_tag")]
    mastered_tag: String,
    /// Ease threshold for struggling (cards below this are struggling)
    #[serde(default = "default_struggling_ease")]
    struggling_ease: i64,
    /// Lapse threshold for struggling (cards above this are struggling)
    #[serde(default = "default_struggling_lapses")]
    struggling_lapses: i64,
    /// Ease threshold for mastered (cards above this are mastered)
    #[serde(default = "default_mastered_ease")]
    mastered_ease: i64,
    /// Minimum reps for mastered status
    #[serde(default = "default_mastered_min_reps")]
    mastered_min_reps: i64,
}

fn default_struggling_tag() -> String {
    "struggling".to_string()
}
fn default_mastered_tag() -> String {
    "mastered".to_string()
}
fn default_struggling_ease() -> i64 {
    2100
}
fn default_struggling_lapses() -> i64 {
    3
}
fn default_mastered_ease() -> i64 {
    2500
}
fn default_mastered_min_reps() -> i64 {
    5
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SuspendByCriteriaParams {
    /// Anki search query to filter cards
    query: String,
    /// Maximum ease factor (cards below this may be suspended)
    #[serde(default = "default_suspend_max_ease")]
    max_ease: i64,
    /// Minimum lapse count (cards above this may be suspended)
    #[serde(default = "default_suspend_min_lapses")]
    min_lapses: i64,
    /// Whether both conditions must be met (default: true)
    #[serde(default = "default_require_both")]
    require_both: bool,
}

fn default_suspend_max_ease() -> i64 {
    1800
}
fn default_suspend_min_lapses() -> i64 {
    5
}
fn default_require_both() -> bool {
    true
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeckHealthReportParams {
    /// Deck name to analyze
    deck: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct BulkTagOperationParams {
    /// Anki search query to filter notes
    query: String,
    /// Operation type: "add", "remove", or "replace"
    operation: String,
    /// Tags for add/remove, or old_tag for replace
    tags: String,
    /// New tag (only for replace operation)
    #[serde(default)]
    new_tag: Option<String>,
}

// Enrich workflow params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct FindEnrichCandidatesParams {
    /// Anki search query to filter notes
    query: String,
    /// Field names to check for empty values
    empty_fields: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct EnrichNoteParams {
    /// Note ID to update
    note_id: i64,
    /// Field values to set (field_name -> value)
    fields: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct EnrichNotesParams {
    /// Updates to apply: list of (note_id, fields) pairs
    updates: Vec<EnrichNoteUpdate>,
    /// Optional tag to add to enriched notes
    #[serde(default)]
    tag_enriched: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct EnrichNoteUpdate {
    /// Note ID to update
    note_id: i64,
    /// Field values to set
    fields: HashMap<String, String>,
}

// Deduplicate workflow params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct FindDuplicatesParams {
    /// Anki search query to filter notes
    query: String,
    /// Field name to use as the duplicate key
    key_field: String,
    /// Strategy for which duplicate to keep: "first", "last", "most_content", or "most_tags"
    #[serde(default = "default_keep_strategy")]
    keep: String,
}

fn default_keep_strategy() -> String {
    "first".to_string()
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RemoveDuplicatesParams {
    /// Anki search query to filter notes
    query: String,
    /// Field name to use as the duplicate key
    key_field: String,
    /// Strategy for which duplicate to keep: "first", "last", "most_content", or "most_tags"
    #[serde(default = "default_keep_strategy")]
    keep: String,
}

// TOML Builder params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ExportDeckTomlParams {
    /// Deck name to export
    deck: String,
    /// Optional path to write TOML file directly (if omitted, returns content)
    #[serde(default)]
    output_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DiffDeckTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    toml_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct PlanSyncTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    toml_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SyncDeckTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    toml_path: Option<String>,
    /// Sync strategy: "push_only", "pull_only", or "bidirectional"
    #[serde(default = "default_sync_strategy")]
    strategy: String,
    /// Conflict resolution: "prefer_toml", "prefer_anki", "fail", or "skip"
    #[serde(default = "default_conflict_resolution")]
    conflict_resolution: String,
}

fn default_sync_strategy() -> String {
    "push_only".to_string()
}

fn default_conflict_resolution() -> String {
    "skip".to_string()
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ImportDeckTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    toml_path: Option<String>,
}

/// Helper to resolve TOML content from either inline content or file path.
fn resolve_toml_content(
    toml_content: Option<String>,
    toml_path: Option<String>,
) -> Result<String, McpError> {
    match (toml_content, toml_path) {
        (Some(content), None) => Ok(content),
        (None, Some(path)) => std::fs::read_to_string(&path).map_err(|e| {
            McpError::invalid_params(format!("Failed to read TOML file '{}': {}", path, e), None)
        }),
        (Some(_), Some(_)) => Err(McpError::invalid_params(
            "Provide either toml_content or toml_path, not both",
            None,
        )),
        (None, None) => Err(McpError::invalid_params(
            "Must provide either toml_content or toml_path",
            None,
        )),
    }
}

// ============================================================================
// Server Implementation
// ============================================================================

#[derive(Clone)]
struct AnkiServer {
    engine: Arc<Engine>,
    tool_router: ToolRouter<AnkiServer>,
    read_only: bool,
}

impl AnkiServer {
    fn new(url: &str, read_only: bool) -> Self {
        let client = ClientBuilder::new().url(url).build();
        let engine = Engine::from_client(client);
        Self {
            engine: Arc::new(engine),
            tool_router: Self::tool_router(),
            read_only,
        }
    }

    fn check_write(&self, operation: &str) -> Result<(), McpError> {
        if self.read_only {
            warn!("Blocked write operation in read-only mode: {}", operation);
            Err(McpError::invalid_request(
                format!(
                    "Write operation '{}' is not allowed in read-only mode",
                    operation
                ),
                None,
            ))
        } else {
            Ok(())
        }
    }
}

#[tool_router]
impl AnkiServer {
    // ========================================================================
    // Raw API Tools - Notes
    // ========================================================================

    #[tool(description = "Add a single flashcard note to Anki. Returns the new note ID.")]
    async fn add_note(
        &self,
        Parameters(params): Parameters<AddNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("add_note")?;
        debug!(deck = %params.deck, model = %params.model, "Adding note");

        let mut builder = NoteBuilder::new(&params.deck, &params.model);
        for (field, value) in &params.fields {
            builder = builder.field(field, value);
        }
        builder = builder.tags(params.tags);

        let note_id = self
            .engine
            .client()
            .notes()
            .add(builder.build())
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(note_id, "Note created");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created note with ID: {}",
            note_id
        ))]))
    }

    #[tool(
        description = "Search for notes using Anki query syntax (e.g., 'deck:Japanese tag:verb'). Returns note IDs."
    )]
    async fn find_notes(
        &self,
        Parameters(params): Parameters<FindNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(query = %params.query, "Finding notes");

        let note_ids = self
            .engine
            .client()
            .notes()
            .find(&params.query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(count = note_ids.len(), "Found notes");
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&note_ids).unwrap(),
        )]))
    }

    #[tool(description = "Get detailed information about notes by their IDs.")]
    async fn get_notes_info(
        &self,
        Parameters(params): Parameters<GetNotesInfoParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(count = params.note_ids.len(), "Getting notes info");

        let notes = self
            .engine
            .client()
            .notes()
            .info(&params.note_ids)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&notes).unwrap(),
        )]))
    }

    #[tool(description = "Update a note's field values.")]
    async fn update_note(
        &self,
        Parameters(params): Parameters<UpdateNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("update_note")?;
        debug!(note_id = params.note_id, "Updating note");

        self.engine
            .client()
            .notes()
            .update_fields(params.note_id, &params.fields)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(note_id = params.note_id, "Note updated");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Updated note {}",
            params.note_id
        ))]))
    }

    #[tool(
        description = "Delete notes by their IDs. This also deletes all cards generated from the notes."
    )]
    async fn delete_notes(
        &self,
        Parameters(params): Parameters<DeleteNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("delete_notes")?;
        debug!(count = params.note_ids.len(), "Deleting notes");

        self.engine
            .client()
            .notes()
            .delete(&params.note_ids)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(count = params.note_ids.len(), "Notes deleted");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Deleted {} notes",
            params.note_ids.len()
        ))]))
    }

    // ========================================================================
    // Raw API Tools - Cards
    // ========================================================================

    #[tool(
        description = "Search for cards using Anki query syntax (e.g., 'deck:Japanese is:due'). Returns card IDs."
    )]
    async fn find_cards(
        &self,
        Parameters(params): Parameters<FindCardsParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(query = %params.query, "Finding cards");

        let card_ids = self
            .engine
            .client()
            .cards()
            .find(&params.query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(count = card_ids.len(), "Found cards");
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&card_ids).unwrap(),
        )]))
    }

    #[tool(
        description = "Get detailed information about cards including reps, lapses, ease factor, and interval."
    )]
    async fn get_cards_info(
        &self,
        Parameters(params): Parameters<GetCardsInfoParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(count = params.card_ids.len(), "Getting cards info");

        let cards = self
            .engine
            .client()
            .cards()
            .info(&params.card_ids)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&cards).unwrap(),
        )]))
    }

    #[tool(description = "Suspend cards to prevent them from appearing in reviews.")]
    async fn suspend_cards(
        &self,
        Parameters(params): Parameters<SuspendCardsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("suspend_cards")?;
        debug!(count = params.card_ids.len(), "Suspending cards");

        self.engine
            .client()
            .cards()
            .suspend(&params.card_ids)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(count = params.card_ids.len(), "Cards suspended");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Suspended {} cards",
            params.card_ids.len()
        ))]))
    }

    #[tool(description = "Unsuspend previously suspended cards.")]
    async fn unsuspend_cards(
        &self,
        Parameters(params): Parameters<UnsuspendCardsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("unsuspend_cards")?;
        debug!(count = params.card_ids.len(), "Unsuspending cards");

        self.engine
            .client()
            .cards()
            .unsuspend(&params.card_ids)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(count = params.card_ids.len(), "Cards unsuspended");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Unsuspended {} cards",
            params.card_ids.len()
        ))]))
    }

    #[tool(description = "Reset cards to new state, clearing all learning progress.")]
    async fn forget_cards(
        &self,
        Parameters(params): Parameters<ForgetCardsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("forget_cards")?;
        debug!(count = params.card_ids.len(), "Forgetting cards");

        self.engine
            .client()
            .cards()
            .forget(&params.card_ids)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(count = params.card_ids.len(), "Cards reset to new");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Reset {} cards to new state",
            params.card_ids.len()
        ))]))
    }

    #[tool(
        description = "Set ease factors for cards. Ease factors are integers (e.g., 2500 = 250%)."
    )]
    async fn set_ease(
        &self,
        Parameters(params): Parameters<SetEaseParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("set_ease")?;
        debug!(count = params.card_ids.len(), "Setting ease factors");

        let results = self
            .engine
            .client()
            .cards()
            .set_ease(&params.card_ids, &params.ease_factors)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let success_count = results.iter().filter(|&&r| r).count();
        info!(success_count, "Ease factors set");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set ease for {} of {} cards",
            success_count,
            params.card_ids.len()
        ))]))
    }

    #[tool(
        description = "Set due date for cards. Days can be: '0' (today), '1' (tomorrow), '-1' (yesterday), '1-7' (random range), '0!' (today and reset interval)."
    )]
    async fn set_due_date(
        &self,
        Parameters(params): Parameters<SetDueDateParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("set_due_date")?;
        debug!(count = params.card_ids.len(), days = %params.days, "Setting due date");

        self.engine
            .client()
            .cards()
            .set_due_date(&params.card_ids, &params.days)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(count = params.card_ids.len(), days = %params.days, "Due date set");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set due date to '{}' for {} cards",
            params.days,
            params.card_ids.len()
        ))]))
    }

    // ========================================================================
    // Raw API Tools - Tags
    // ========================================================================

    #[tool(description = "Add tags to notes. Tags are space-separated (e.g., 'tag1 tag2').")]
    async fn add_tags(
        &self,
        Parameters(params): Parameters<AddTagsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("add_tags")?;
        debug!(count = params.note_ids.len(), tags = %params.tags, "Adding tags");

        self.engine
            .client()
            .notes()
            .add_tags(&params.note_ids, &params.tags)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(count = params.note_ids.len(), tags = %params.tags, "Tags added");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added tags '{}' to {} notes",
            params.tags,
            params.note_ids.len()
        ))]))
    }

    #[tool(description = "Remove tags from notes. Tags are space-separated (e.g., 'tag1 tag2').")]
    async fn remove_tags(
        &self,
        Parameters(params): Parameters<RemoveTagsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("remove_tags")?;
        debug!(count = params.note_ids.len(), tags = %params.tags, "Removing tags");

        self.engine
            .client()
            .notes()
            .remove_tags(&params.note_ids, &params.tags)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(count = params.note_ids.len(), tags = %params.tags, "Tags removed");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Removed tags '{}' from {} notes",
            params.tags,
            params.note_ids.len()
        ))]))
    }

    #[tool(description = "Replace a tag with another across all notes in the collection.")]
    async fn replace_tags_all(
        &self,
        Parameters(params): Parameters<ReplaceTagsAllParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("replace_tags_all")?;
        debug!(old = %params.old_tag, new = %params.new_tag, "Replacing tag globally");

        self.engine
            .client()
            .notes()
            .replace_tags_all(&params.old_tag, &params.new_tag)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(old = %params.old_tag, new = %params.new_tag, "Tag replaced globally");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Replaced tag '{}' with '{}' across all notes",
            params.old_tag, params.new_tag
        ))]))
    }

    #[tool(description = "Remove all tags that are not used by any notes.")]
    async fn clear_unused_tags(&self) -> Result<CallToolResult, McpError> {
        self.check_write("clear_unused_tags")?;
        debug!("Clearing unused tags");

        self.engine
            .client()
            .notes()
            .clear_unused_tags()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!("Unused tags cleared");
        Ok(CallToolResult::success(vec![Content::text(
            "Cleared all unused tags",
        )]))
    }

    // ========================================================================
    // Raw API Tools - Decks
    // ========================================================================

    #[tool(description = "List all deck names in Anki.")]
    async fn list_decks(&self) -> Result<CallToolResult, McpError> {
        debug!("Listing decks");

        let decks = self
            .engine
            .client()
            .decks()
            .names()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(count = decks.len(), "Listed decks");
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&decks).unwrap(),
        )]))
    }

    #[tool(description = "Create a new deck. Returns the deck ID.")]
    async fn create_deck(
        &self,
        Parameters(params): Parameters<CreateDeckParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("create_deck")?;
        debug!(name = %params.name, "Creating deck");

        let deck_id = self
            .engine
            .client()
            .decks()
            .create(&params.name)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(deck_id, name = %params.name, "Deck created");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created deck '{}' with ID: {}",
            params.name, deck_id
        ))]))
    }

    #[tool(description = "Delete a deck. If cards_too is false, cards are moved to Default deck.")]
    async fn delete_deck(
        &self,
        Parameters(params): Parameters<DeleteDeckParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("delete_deck")?;
        debug!(name = %params.name, cards_too = params.cards_too, "Deleting deck");

        self.engine
            .client()
            .decks()
            .delete(&[params.name.as_str()], params.cards_too)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let action = if params.cards_too {
            "and its cards"
        } else {
            "(cards moved to Default)"
        };

        info!(name = %params.name, "Deck deleted");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Deleted deck '{}' {}",
            params.name, action
        ))]))
    }

    // ========================================================================
    // Raw API Tools - Models
    // ========================================================================

    #[tool(description = "List all note type (model) names in Anki.")]
    async fn list_models(&self) -> Result<CallToolResult, McpError> {
        debug!("Listing models");

        let models = self
            .engine
            .client()
            .models()
            .names()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(count = models.len(), "Listed models");
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&models).unwrap(),
        )]))
    }

    #[tool(description = "Get the field names for a note type (model).")]
    async fn get_model_fields(
        &self,
        Parameters(params): Parameters<GetModelFieldsParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(model = %params.model, "Getting model fields");

        let fields = self
            .engine
            .client()
            .models()
            .field_names(&params.model)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&fields).unwrap(),
        )]))
    }

    // ========================================================================
    // Raw API Tools - Misc
    // ========================================================================

    #[tool(description = "Sync the Anki collection with AnkiWeb.")]
    async fn sync(&self) -> Result<CallToolResult, McpError> {
        self.check_write("sync")?;
        debug!("Syncing with AnkiWeb");

        self.engine
            .client()
            .misc()
            .sync()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!("Sync completed");
        Ok(CallToolResult::success(vec![Content::text(
            "Sync completed successfully",
        )]))
    }

    #[tool(description = "Get the AnkiConnect version. Useful for checking if Anki is running.")]
    async fn version(&self) -> Result<CallToolResult, McpError> {
        debug!("Getting AnkiConnect version");

        let version = self
            .engine
            .client()
            .misc()
            .version()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "AnkiConnect version: {}",
            version
        ))]))
    }

    // ========================================================================
    // Engine Workflow Tools - Import
    // ========================================================================

    #[tool(
        description = "Import multiple notes with duplicate handling. on_duplicate can be 'skip', 'update', or 'allow'."
    )]
    async fn import_notes(
        &self,
        Parameters(params): Parameters<ImportNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("import_notes")?;
        debug!(
            count = params.notes.len(),
            on_duplicate = %params.on_duplicate,
            "Importing notes"
        );

        let on_duplicate = match params.on_duplicate.as_str() {
            "update" => OnDuplicate::Update,
            "allow" => OnDuplicate::Allow,
            _ => OnDuplicate::Skip,
        };

        let notes: Vec<_> = params
            .notes
            .iter()
            .map(|n| {
                let mut builder = NoteBuilder::new(&n.deck, &n.model);
                for (field, value) in &n.fields {
                    builder = builder.field(field, value);
                }
                builder.tags(n.tags.clone()).build()
            })
            .collect();

        let report = self
            .engine
            .import()
            .notes(&notes, on_duplicate)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            added = report.added,
            skipped = report.skipped,
            updated = report.updated,
            failed = report.failed,
            "Import completed"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Import complete: {} added, {} skipped, {} updated, {} failed",
            report.added, report.skipped, report.updated, report.failed
        ))]))
    }

    #[tool(description = "Validate notes before importing. Checks if decks and models exist.")]
    async fn validate_notes(
        &self,
        Parameters(params): Parameters<ValidateNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(count = params.notes.len(), "Validating notes");

        let notes: Vec<_> = params
            .notes
            .iter()
            .map(|n| {
                let mut builder = NoteBuilder::new(&n.deck, &n.model);
                for (field, value) in &n.fields {
                    builder = builder.field(field, value);
                }
                builder.tags(n.tags.clone()).build()
            })
            .collect();

        let results = self
            .engine
            .import()
            .validate(&notes)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let valid_count = results.iter().filter(|r| r.valid).count();
        let invalid: Vec<_> = results
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.valid)
            .map(|(i, r)| format!("Note {}: {}", i, r.errors.join(", ")))
            .collect();

        let message = if invalid.is_empty() {
            format!("All {} notes are valid", valid_count)
        } else {
            format!(
                "{} valid, {} invalid:\n{}",
                valid_count,
                invalid.len(),
                invalid.join("\n")
            )
        };

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    // ========================================================================
    // Engine Workflow Tools - Export
    // ========================================================================

    #[tool(description = "Export all notes and cards from a deck as JSON.")]
    async fn export_deck(
        &self,
        Parameters(params): Parameters<ExportDeckParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(deck = %params.deck, "Exporting deck");

        let export = self
            .engine
            .export()
            .deck(&params.deck)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&export).unwrap(),
        )]))
    }

    #[tool(description = "Export review history for cards matching an Anki query.")]
    async fn export_reviews(
        &self,
        Parameters(params): Parameters<ExportReviewsParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(query = %params.query, "Exporting reviews");

        let reviews = self
            .engine
            .export()
            .reviews(&params.query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&reviews).unwrap(),
        )]))
    }

    // ========================================================================
    // Engine Workflow Tools - Organize
    // ========================================================================

    #[tool(description = "Clone a deck with all its notes. Cards start as new.")]
    async fn clone_deck(
        &self,
        Parameters(params): Parameters<CloneDeckParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("clone_deck")?;
        debug!(source = %params.source, destination = %params.destination, "Cloning deck");

        let report = self
            .engine
            .organize()
            .clone_deck(&params.source, &params.destination)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            notes_cloned = report.notes_cloned,
            notes_failed = report.notes_failed,
            destination = %report.destination,
            "Deck cloned"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Cloned {} notes to '{}' ({} failed)",
            report.notes_cloned, report.destination, report.notes_failed
        ))]))
    }

    #[tool(description = "Merge multiple decks into one destination deck.")]
    async fn merge_decks(
        &self,
        Parameters(params): Parameters<MergeDecksParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("merge_decks")?;
        debug!(
            sources = ?params.sources,
            destination = %params.destination,
            "Merging decks"
        );

        let sources: Vec<&str> = params.sources.iter().map(|s| s.as_str()).collect();
        let report = self
            .engine
            .organize()
            .merge_decks(&sources, &params.destination)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            cards_moved = report.cards_moved,
            destination = %report.destination,
            "Decks merged"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Moved {} cards to '{}'",
            report.cards_moved, report.destination
        ))]))
    }

    #[tool(description = "Move all notes with a specific tag to a destination deck.")]
    async fn move_by_tag(
        &self,
        Parameters(params): Parameters<MoveByTagParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("move_by_tag")?;
        debug!(tag = %params.tag, destination = %params.destination, "Moving by tag");

        let count = self
            .engine
            .organize()
            .move_by_tag(&params.tag, &params.destination)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            count,
            tag = %params.tag,
            destination = %params.destination,
            "Cards moved"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Moved {} cards with tag '{}' to '{}'",
            count, params.tag, params.destination
        ))]))
    }

    // ========================================================================
    // Engine Workflow Tools - Analyze
    // ========================================================================

    #[tool(description = "Get study summary statistics for a deck over a number of days.")]
    async fn study_summary(
        &self,
        Parameters(params): Parameters<StudySummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(deck = %params.deck, days = params.days, "Getting study summary");

        let stats = self
            .engine
            .analyze()
            .study_summary(&params.deck, params.days)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&stats).unwrap(),
        )]))
    }

    #[tool(description = "Find problem cards (leeches) that may need attention.")]
    async fn find_problems(
        &self,
        Parameters(params): Parameters<FindProblemsParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            query = %params.query,
            min_lapses = params.min_lapses,
            "Finding problems"
        );

        let criteria = ProblemCriteria {
            min_lapses: params.min_lapses,
            ..Default::default()
        };

        let problems = self
            .engine
            .analyze()
            .find_problems(&params.query, criteria)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(count = problems.len(), "Found problem cards");
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&problems).unwrap(),
        )]))
    }

    #[tool(
        description = "Get retention statistics for a deck including average ease and retention rate."
    )]
    async fn retention_stats(
        &self,
        Parameters(params): Parameters<RetentionStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(deck = %params.deck, "Getting retention stats");

        let stats = self
            .engine
            .analyze()
            .retention_stats(&params.deck)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&stats).unwrap(),
        )]))
    }

    // ========================================================================
    // Engine Workflow Tools - Media
    // ========================================================================

    #[tool(description = "Audit media files to find orphaned files and missing references.")]
    async fn audit_media(&self) -> Result<CallToolResult, McpError> {
        debug!("Auditing media");

        let audit = self
            .engine
            .media()
            .audit()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&audit).unwrap(),
        )]))
    }

    #[tool(
        description = "Clean up orphaned media files. Set dry_run=true to preview without deleting."
    )]
    async fn cleanup_media(
        &self,
        Parameters(params): Parameters<CleanupMediaParams>,
    ) -> Result<CallToolResult, McpError> {
        if !params.dry_run {
            self.check_write("cleanup_media")?;
        }
        debug!(dry_run = params.dry_run, "Cleaning up media");

        let report = self
            .engine
            .media()
            .cleanup_orphaned(params.dry_run)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let action = if params.dry_run {
            "Would delete"
        } else {
            info!(count = report.files_deleted, "Media files deleted");
            "Deleted"
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{} {} files",
            action, report.files_deleted
        ))]))
    }

    // ========================================================================
    // Engine Workflow Tools - Progress
    // ========================================================================

    #[tool(description = "Reset all cards in a deck to new state, clearing learning progress.")]
    async fn reset_deck_progress(
        &self,
        Parameters(params): Parameters<ResetDeckProgressParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("reset_deck_progress")?;
        debug!(deck = %params.deck, "Resetting deck progress");

        let report = self
            .engine
            .progress()
            .reset_deck(&params.deck)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(cards_reset = report.cards_reset, deck = %report.deck, "Deck progress reset");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Reset {} cards in deck '{}'",
            report.cards_reset, report.deck
        ))]))
    }

    #[tool(
        description = "Tag cards based on performance. Adds 'struggling' tag to cards with low ease or high lapses, 'mastered' tag to cards with high ease and many reviews."
    )]
    async fn tag_by_performance(
        &self,
        Parameters(params): Parameters<TagByPerformanceParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("tag_by_performance")?;
        debug!(query = %params.query, "Tagging by performance");

        let criteria = PerformanceCriteria {
            struggling_ease: params.struggling_ease,
            struggling_lapses: params.struggling_lapses,
            mastered_ease: params.mastered_ease,
            mastered_min_reps: params.mastered_min_reps,
        };

        let report = self
            .engine
            .progress()
            .tag_by_performance(
                &params.query,
                criteria,
                &params.struggling_tag,
                &params.mastered_tag,
            )
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            struggling = report.struggling_count,
            mastered = report.mastered_count,
            "Cards tagged by performance"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Tagged {} as '{}', {} as '{}'",
            report.struggling_count,
            report.struggling_tag,
            report.mastered_count,
            report.mastered_tag
        ))]))
    }

    #[tool(
        description = "Suspend cards matching criteria (low ease and/or high lapses). By default requires both conditions."
    )]
    async fn suspend_by_criteria(
        &self,
        Parameters(params): Parameters<SuspendByCriteriaParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("suspend_by_criteria")?;
        debug!(query = %params.query, "Suspending by criteria");

        let criteria = SuspendCriteria {
            max_ease: params.max_ease,
            min_lapses: params.min_lapses,
            require_both: params.require_both,
        };

        let report = self
            .engine
            .progress()
            .suspend_by_criteria(&params.query, criteria)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(cards_suspended = report.cards_suspended, "Cards suspended");
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&report).unwrap(),
        )]))
    }

    #[tool(
        description = "Get comprehensive health report for a deck including card counts by state, average ease, leeches, and more."
    )]
    async fn deck_health_report(
        &self,
        Parameters(params): Parameters<DeckHealthReportParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(deck = %params.deck, "Getting deck health report");

        let report = self
            .engine
            .progress()
            .deck_health(&params.deck)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&report).unwrap(),
        )]))
    }

    #[tool(
        description = "Perform bulk tag operation on notes. Operation can be 'add', 'remove', or 'replace'."
    )]
    async fn bulk_tag_operation(
        &self,
        Parameters(params): Parameters<BulkTagOperationParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("bulk_tag_operation")?;
        debug!(query = %params.query, operation = %params.operation, "Bulk tag operation");

        let operation = match params.operation.as_str() {
            "add" => TagOperation::Add(params.tags.clone()),
            "remove" => TagOperation::Remove(params.tags.clone()),
            "replace" => {
                let new_tag = params.new_tag.clone().unwrap_or_default();
                TagOperation::Replace {
                    old: params.tags.clone(),
                    new: new_tag,
                }
            }
            _ => {
                return Err(McpError::invalid_params(
                    format!(
                        "Invalid operation '{}'. Use 'add', 'remove', or 'replace'",
                        params.operation
                    ),
                    None,
                ));
            }
        };

        let report = self
            .engine
            .progress()
            .bulk_tag(&params.query, operation)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            notes_affected = report.notes_affected,
            "Bulk tag operation complete"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{} on {} notes",
            report.operation, report.notes_affected
        ))]))
    }

    // ========================================================================
    // Engine Workflow Tools - Enrich
    // ========================================================================

    #[tool(
        description = "Find notes with empty fields that need enrichment. Returns candidates with their current field values and which fields are empty."
    )]
    async fn find_enrich_candidates(
        &self,
        Parameters(params): Parameters<FindEnrichCandidatesParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(query = %params.query, empty_fields = ?params.empty_fields, "Finding enrich candidates");

        let query = EnrichQuery {
            search: params.query,
            empty_fields: params.empty_fields,
        };

        let candidates = self
            .engine
            .enrich()
            .find_candidates(&query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(count = candidates.len(), "Found enrich candidates");
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&candidates).unwrap(),
        )]))
    }

    #[tool(description = "Update a single note with new field values for enrichment.")]
    async fn enrich_note(
        &self,
        Parameters(params): Parameters<EnrichNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("enrich_note")?;
        debug!(note_id = params.note_id, "Enriching note");

        self.engine
            .enrich()
            .update_note(params.note_id, &params.fields)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(note_id = params.note_id, "Note enriched");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Enriched note {}",
            params.note_id
        ))]))
    }

    #[tool(
        description = "Update multiple notes with enriched content. Optionally tag them as enriched."
    )]
    async fn enrich_notes(
        &self,
        Parameters(params): Parameters<EnrichNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("enrich_notes")?;
        debug!(count = params.updates.len(), "Enriching notes");

        let updates: Vec<(i64, HashMap<String, String>)> = params
            .updates
            .into_iter()
            .map(|u| (u.note_id, u.fields))
            .collect();

        let report = self
            .engine
            .enrich()
            .update_notes(&updates)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Tag enriched notes if requested
        if let Some(tag) = params.tag_enriched {
            let note_ids: Vec<i64> = updates.iter().map(|(id, _)| *id).collect();
            self.engine
                .enrich()
                .tag_enriched(&note_ids, &tag)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        info!(
            updated = report.updated,
            failed = report.failed,
            "Notes enriched"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Enriched {} notes ({} failed)",
            report.updated, report.failed
        ))]))
    }

    // ========================================================================
    // Engine Workflow Tools - Deduplicate
    // ========================================================================

    #[tool(
        description = "Find duplicate notes based on a key field. Returns groups of duplicates with which note would be kept."
    )]
    async fn find_duplicates(
        &self,
        Parameters(params): Parameters<FindDuplicatesParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            query = %params.query,
            key_field = %params.key_field,
            keep = %params.keep,
            "Finding duplicates"
        );

        let keep = match params.keep.as_str() {
            "last" => KeepStrategy::Last,
            "most_content" => KeepStrategy::MostContent,
            "most_tags" => KeepStrategy::MostTags,
            _ => KeepStrategy::First,
        };

        let query = DedupeQuery {
            search: params.query,
            key_field: params.key_field,
            keep,
        };

        let groups = self
            .engine
            .deduplicate()
            .find_duplicates(&query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let total_dups: usize = groups.iter().map(|g| g.duplicate_note_ids.len()).sum();
        debug!(
            groups = groups.len(),
            total_duplicates = total_dups,
            "Found duplicates"
        );

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&groups).unwrap(),
        )]))
    }

    #[tool(
        description = "Preview deduplication without making changes. Shows what would be deleted."
    )]
    async fn preview_deduplicate(
        &self,
        Parameters(params): Parameters<FindDuplicatesParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            query = %params.query,
            key_field = %params.key_field,
            "Previewing deduplication"
        );

        let keep = match params.keep.as_str() {
            "last" => KeepStrategy::Last,
            "most_content" => KeepStrategy::MostContent,
            "most_tags" => KeepStrategy::MostTags,
            _ => KeepStrategy::First,
        };

        let query = DedupeQuery {
            search: params.query,
            key_field: params.key_field,
            keep,
        };

        let report = self
            .engine
            .deduplicate()
            .preview(&query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&report).unwrap(),
        )]))
    }

    #[tool(
        description = "Remove duplicate notes. Keeps one note per duplicate group based on the keep strategy and deletes the rest."
    )]
    async fn remove_duplicates(
        &self,
        Parameters(params): Parameters<RemoveDuplicatesParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("remove_duplicates")?;
        debug!(
            query = %params.query,
            key_field = %params.key_field,
            keep = %params.keep,
            "Removing duplicates"
        );

        let keep = match params.keep.as_str() {
            "last" => KeepStrategy::Last,
            "most_content" => KeepStrategy::MostContent,
            "most_tags" => KeepStrategy::MostTags,
            _ => KeepStrategy::First,
        };

        let query = DedupeQuery {
            search: params.query,
            key_field: params.key_field,
            keep,
        };

        let report = self
            .engine
            .deduplicate()
            .remove_duplicates(&query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            deleted = report.deleted,
            kept = report.kept,
            "Duplicates removed"
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Removed {} duplicate notes (kept {} unique)",
            report.deleted, report.kept
        ))]))
    }

    // ========================================================================
    // TOML Builder Tools
    // ========================================================================

    #[tool(
        description = "Export a deck from Anki to TOML format. Returns the TOML content, or writes to output_path if provided."
    )]
    async fn export_deck_toml(
        &self,
        Parameters(params): Parameters<ExportDeckTomlParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(deck = %params.deck, output_path = ?params.output_path, "Exporting deck to TOML");

        let builder = ankit_builder::DeckBuilder::from_anki(self.engine.client(), &params.deck)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let toml = builder
            .definition()
            .to_toml()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Write to file if output_path provided, otherwise return content
        if let Some(path) = params.output_path {
            std::fs::write(&path, &toml).map_err(|e| {
                McpError::internal_error(format!("Failed to write to '{}': {}", path, e), None)
            })?;
            let note_count = builder.definition().notes.len();
            info!(deck = %params.deck, path = %path, notes = note_count, "Deck exported to file");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Exported {} notes to '{}'",
                note_count, path
            ))]))
        } else {
            info!(deck = %params.deck, "Deck exported to TOML");
            Ok(CallToolResult::success(vec![Content::text(toml)]))
        }
    }

    #[tool(
        description = "Compare a TOML deck definition against the current state in Anki. Shows notes only in TOML, only in Anki, and modified notes."
    )]
    async fn diff_deck_toml(
        &self,
        Parameters(params): Parameters<DiffDeckTomlParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Diffing TOML against Anki");

        let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
        let builder = ankit_builder::DeckBuilder::parse(&toml_content)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let diff = builder
            .diff_connect_with_client(self.engine.client())
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(
            toml_only = diff.toml_only.len(),
            anki_only = diff.anki_only.len(),
            modified = diff.modified.len(),
            unchanged = diff.unchanged,
            "Diff completed"
        );

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&diff).unwrap(),
        )]))
    }

    #[tool(
        description = "Preview what sync would do between a TOML definition and Anki without making changes."
    )]
    async fn plan_sync_toml(
        &self,
        Parameters(params): Parameters<PlanSyncTomlParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Planning TOML sync");

        let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
        let builder = ankit_builder::DeckBuilder::parse(&toml_content)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let plan = builder
            .plan_sync_with_client(self.engine.client())
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        debug!(
            to_push = plan.to_push.len(),
            to_pull = plan.to_pull.len(),
            conflicts = plan.conflicts.len(),
            "Sync plan generated"
        );

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&plan).unwrap(),
        )]))
    }

    #[tool(
        description = "Sync a TOML deck definition with Anki. Strategy can be 'push_only' (TOML -> Anki), 'pull_only' (Anki -> TOML), or 'bidirectional'. Returns sync results and optionally updated TOML."
    )]
    async fn sync_deck_toml(
        &self,
        Parameters(params): Parameters<SyncDeckTomlParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("sync_deck_toml")?;
        debug!(strategy = %params.strategy, "Syncing TOML with Anki");

        let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
        let builder = ankit_builder::DeckBuilder::parse(&toml_content)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let conflict_resolution = match params.conflict_resolution.as_str() {
            "prefer_toml" => ankit_builder::ConflictResolution::PreferToml,
            "prefer_anki" => ankit_builder::ConflictResolution::PreferAnki,
            "fail" => ankit_builder::ConflictResolution::Fail,
            _ => ankit_builder::ConflictResolution::Skip,
        };

        let strategy = match params.strategy.as_str() {
            "pull_only" => ankit_builder::SyncStrategy::pull_only(),
            "bidirectional" => ankit_builder::SyncStrategy {
                conflict_resolution,
                pull_new_notes: true,
                push_new_notes: true,
                update_tags: true,
            },
            _ => ankit_builder::SyncStrategy::push_only(),
        };

        let result = builder
            .sync_with_client(self.engine.client(), strategy)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            pushed = result.pushed.len(),
            pulled = result.pulled.len(),
            resolved = result.resolved_conflicts.len(),
            skipped = result.skipped_conflicts.len(),
            errors = result.errors.len(),
            "Sync completed"
        );

        // Build response with results and optionally updated TOML
        let mut response = serde_json::json!({
            "pushed": result.pushed.len(),
            "pulled": result.pulled.len(),
            "resolved_conflicts": result.resolved_conflicts.len(),
            "skipped_conflicts": result.skipped_conflicts.len(),
            "errors": result.errors,
        });

        if let Some(updated_def) = result.updated_definition {
            if let Ok(updated_toml) = updated_def.to_toml() {
                response["updated_toml"] = serde_json::Value::String(updated_toml);
            }
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Import a TOML deck definition into Anki. Creates decks and adds notes.")]
    async fn import_deck_toml(
        &self,
        Parameters(params): Parameters<ImportDeckTomlParams>,
    ) -> Result<CallToolResult, McpError> {
        self.check_write("import_deck_toml")?;
        debug!("Importing TOML to Anki");

        let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
        let builder = ankit_builder::DeckBuilder::parse(&toml_content)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let result = builder
            .import_connect_batch()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        info!(
            decks_created = result.decks_created,
            notes_created = result.notes_created,
            notes_skipped = result.notes_skipped,
            "TOML imported"
        );

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Imported: {} decks created, {} notes created, {} notes skipped",
            result.decks_created, result.notes_created, result.notes_skipped
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for AnkiServer {
    fn get_info(&self) -> ServerInfo {
        let mode = if self.read_only { " (read-only)" } else { "" };
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(format!(
                "Anki deck management via AnkiConnect{}. \
                 Requires Anki to be running with the AnkiConnect add-on installed. \
                 Tools: add_note, find_notes, list_decks, list_models, import_notes, \
                 study_summary, find_problems, clone_deck, and more.",
                mode
            )),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = match args.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .init();

    let url = format!("http://{}:{}", args.host, args.port);
    info!(
        anki_url = %url,
        read_only = args.read_only,
        transport = ?args.transport,
        "Starting ankit-mcp server"
    );

    let server = AnkiServer::new(&url, args.read_only);

    match args.transport {
        Transport::Stdio => {
            let transport = (tokio::io::stdin(), tokio::io::stdout());
            let mcp_server = server.serve(transport).await?;
            mcp_server.waiting().await?;
        }
        Transport::Http => {
            use rmcp::transport::streamable_http_server::{
                StreamableHttpServerConfig, StreamableHttpService,
                session::local::LocalSessionManager,
            };

            let bind_addr = format!("{}:{}", args.http_host, args.http_port);
            info!(bind_addr = %bind_addr, "Starting HTTP transport");

            let service: StreamableHttpService<AnkiServer, LocalSessionManager> =
                StreamableHttpService::new(
                    move || Ok(server.clone()),
                    Arc::new(LocalSessionManager::default()),
                    StreamableHttpServerConfig::default(),
                );

            let router = axum::Router::new().nest_service("/mcp", service);
            let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
            info!(bind_addr = %bind_addr, "MCP server listening on HTTP");

            axum::serve(listener, router).await?;
        }
    }

    Ok(())
}
