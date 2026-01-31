//! Progress management tools.

use std::sync::Arc;

use ankit_engine::progress::{PerformanceCriteria, SuspendCriteria, TagOperation};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResetDeckProgressParams {
    /// Deck name to reset
    pub deck: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TagByPerformanceParams {
    /// Anki search query to filter cards
    pub query: String,
    /// Tag for struggling cards
    #[serde(default = "default_struggling_tag")]
    pub struggling_tag: String,
    /// Tag for mastered cards
    #[serde(default = "default_mastered_tag")]
    pub mastered_tag: String,
    /// Ease threshold for struggling (cards below this are struggling)
    #[serde(default = "default_struggling_ease")]
    pub struggling_ease: i64,
    /// Lapse threshold for struggling (cards above this are struggling)
    #[serde(default = "default_struggling_lapses")]
    pub struggling_lapses: i64,
    /// Ease threshold for mastered (cards above this are mastered)
    #[serde(default = "default_mastered_ease")]
    pub mastered_ease: i64,
    /// Minimum reps for mastered status
    #[serde(default = "default_mastered_min_reps")]
    pub mastered_min_reps: i64,
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuspendByCriteriaParams {
    /// Anki search query to filter cards
    pub query: String,
    /// Maximum ease factor (cards below this may be suspended)
    #[serde(default = "default_suspend_max_ease")]
    pub max_ease: i64,
    /// Minimum lapse count (cards above this may be suspended)
    #[serde(default = "default_suspend_min_lapses")]
    pub min_lapses: i64,
    /// Whether both conditions must be met (default: true)
    #[serde(default = "default_require_both")]
    pub require_both: bool,
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeckHealthReportParams {
    /// Deck name to analyze
    pub deck: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BulkTagOperationParams {
    /// Anki search query to filter notes
    pub query: String,
    /// Operation type: "add", "remove", or "replace"
    pub operation: String,
    /// Tags for add/remove, or old_tag for replace
    pub tags: String,
    /// New tag (only for replace operation)
    #[serde(default)]
    pub new_tag: Option<String>,
}

/// Reset all cards in a deck to new state, clearing learning progress.
pub fn reset_deck_progress(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("reset_deck_progress")
        .description("Reset all cards in a deck to new state, clearing learning progress.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ResetDeckProgressParams| async move {
                state.check_write("reset_deck_progress")?;
                debug!(deck = %params.deck, "Resetting deck progress");

                let report = state
                    .engine
                    .progress()
                    .reset_deck(&params.deck)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(cards_reset = report.cards_reset, deck = %report.deck, "Deck progress reset");
                Ok(CallToolResult::text(format!(
                    "Reset {} cards in deck '{}'",
                    report.cards_reset, report.deck
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Tag cards based on performance.
pub fn tag_by_performance(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("tag_by_performance")
        .description("Tag cards based on performance. Adds 'struggling' tag to cards with low ease or high lapses, 'mastered' tag to cards with high ease and many reviews.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: TagByPerformanceParams| async move {
                state.check_write("tag_by_performance")?;
                debug!(query = %params.query, "Tagging by performance");

                let criteria = PerformanceCriteria {
                    struggling_ease: params.struggling_ease,
                    struggling_lapses: params.struggling_lapses,
                    mastered_ease: params.mastered_ease,
                    mastered_min_reps: params.mastered_min_reps,
                };

                let report = state
                    .engine
                    .progress()
                    .tag_by_performance(
                        &params.query,
                        criteria,
                        &params.struggling_tag,
                        &params.mastered_tag,
                    )
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    struggling = report.struggling_count,
                    mastered = report.mastered_count,
                    "Cards tagged by performance"
                );
                Ok(CallToolResult::text(format!(
                    "Tagged {} as '{}', {} as '{}'",
                    report.struggling_count,
                    report.struggling_tag,
                    report.mastered_count,
                    report.mastered_tag
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Suspend cards matching criteria (low ease and/or high lapses).
pub fn suspend_by_criteria(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("suspend_by_criteria")
        .description("Suspend cards matching criteria (low ease and/or high lapses). By default requires both conditions.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: SuspendByCriteriaParams| async move {
                state.check_write("suspend_by_criteria")?;
                debug!(query = %params.query, "Suspending by criteria");

                let criteria = SuspendCriteria {
                    max_ease: params.max_ease,
                    min_lapses: params.min_lapses,
                    require_both: params.require_both,
                };

                let report = state
                    .engine
                    .progress()
                    .suspend_by_criteria(&params.query, criteria)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(cards_suspended = report.cards_suspended, "Cards suspended");
                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&report).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Get comprehensive health report for a deck.
pub fn deck_health_report(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("deck_health_report")
        .description("Get comprehensive health report for a deck including card counts by state, average ease, leeches, and more.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: DeckHealthReportParams| async move {
                debug!(deck = %params.deck, "Getting deck health report");

                let report = state
                    .engine
                    .progress()
                    .deck_health(&params.deck)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&report).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Perform bulk tag operation on notes.
pub fn bulk_tag_operation(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("bulk_tag_operation")
        .description(
            "Perform bulk tag operation on notes. Operation can be 'add', 'remove', or 'replace'.",
        )
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: BulkTagOperationParams| async move {
                state.check_write("bulk_tag_operation")?;
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
                        return Err(tower_mcp::Error::tool(format!(
                            "Invalid operation '{}'. Use 'add', 'remove', or 'replace'",
                            params.operation
                        )));
                    }
                };

                let report = state
                    .engine
                    .progress()
                    .bulk_tag(&params.query, operation)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    notes_affected = report.notes_affected,
                    "Bulk tag operation complete"
                );
                Ok(CallToolResult::text(format!(
                    "{} on {} notes",
                    report.operation, report.notes_affected
                )))
            },
        )
        .build()
        .expect("valid tool")
}
