//! Deduplication tools.

use std::sync::Arc;

use ankit_engine::deduplicate::{DedupeQuery, KeepStrategy};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindDuplicatesParams {
    /// Anki search query to filter notes
    pub query: String,
    /// Field name to use as the duplicate key
    pub key_field: String,
    /// Strategy for which duplicate to keep: "first", "last", "most_content", or "most_tags"
    #[serde(default = "default_keep_strategy")]
    pub keep: String,
}

fn default_keep_strategy() -> String {
    "first".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveDuplicatesParams {
    /// Anki search query to filter notes
    pub query: String,
    /// Field name to use as the duplicate key
    pub key_field: String,
    /// Strategy for which duplicate to keep: "first", "last", "most_content", or "most_tags"
    #[serde(default = "default_keep_strategy")]
    pub keep: String,
}

fn parse_keep_strategy(s: &str) -> KeepStrategy {
    match s {
        "last" => KeepStrategy::Last,
        "most_content" => KeepStrategy::MostContent,
        "most_tags" => KeepStrategy::MostTags,
        _ => KeepStrategy::First,
    }
}

/// Find duplicate notes based on a key field.
pub fn find_duplicates(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("find_duplicates")
        .description("Find duplicate notes based on a key field. Returns groups of duplicates with which note would be kept.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: FindDuplicatesParams| async move {
                debug!(
                    query = %params.query,
                    key_field = %params.key_field,
                    keep = %params.keep,
                    "Finding duplicates"
                );

                let keep = parse_keep_strategy(&params.keep);

                let query = DedupeQuery {
                    search: params.query,
                    key_field: params.key_field,
                    keep,
                };

                let groups = state
                    .engine
                    .deduplicate()
                    .find_duplicates(&query)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                let total_dups: usize = groups.iter().map(|g| g.duplicate_note_ids.len()).sum();
                debug!(
                    groups = groups.len(),
                    total_duplicates = total_dups,
                    "Found duplicates"
                );

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&groups).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Preview deduplication without making changes.
pub fn preview_deduplicate(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("preview_deduplicate")
        .description("Preview deduplication without making changes. Shows what would be deleted.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: FindDuplicatesParams| async move {
                debug!(
                    query = %params.query,
                    key_field = %params.key_field,
                    "Previewing deduplication"
                );

                let keep = parse_keep_strategy(&params.keep);

                let query = DedupeQuery {
                    search: params.query,
                    key_field: params.key_field,
                    keep,
                };

                let report = state
                    .engine
                    .deduplicate()
                    .preview(&query)
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

/// Remove duplicate notes.
pub fn remove_duplicates(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("remove_duplicates")
        .description("Remove duplicate notes. Keeps one note per duplicate group based on the keep strategy and deletes the rest.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: RemoveDuplicatesParams| async move {
                state.check_write("remove_duplicates")?;
                debug!(
                    query = %params.query,
                    key_field = %params.key_field,
                    keep = %params.keep,
                    "Removing duplicates"
                );

                let keep = parse_keep_strategy(&params.keep);

                let query = DedupeQuery {
                    search: params.query,
                    key_field: params.key_field,
                    keep,
                };

                let report = state
                    .engine
                    .deduplicate()
                    .remove_duplicates(&query)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    deleted = report.deleted,
                    kept = report.kept,
                    "Duplicates removed"
                );
                Ok(CallToolResult::text(format!(
                    "Removed {} duplicate notes (kept {} unique)",
                    report.deleted, report.kept
                )))
            },
        )
        .build()
        .expect("valid tool")
}
