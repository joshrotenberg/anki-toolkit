//! Analysis tools.

use std::sync::Arc;

use ankit_engine::analyze::ProblemCriteria;
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::debug;

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StudySummaryParams {
    /// Deck name (use "*" for all decks)
    pub deck: String,
    /// Number of days to include
    pub days: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindProblemsParams {
    /// Anki search query
    pub query: String,
    /// Minimum lapse count to flag (default: 5)
    #[serde(default = "default_min_lapses")]
    pub min_lapses: i64,
}

fn default_min_lapses() -> i64 {
    5
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetentionStatsParams {
    /// Deck name
    pub deck: String,
}

/// Get study summary statistics for a deck over a number of days.
pub fn study_summary(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("study_summary")
        .description("Get study summary statistics for a deck over a number of days.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: StudySummaryParams| async move {
                debug!(deck = %params.deck, days = params.days, "Getting study summary");

                let stats = state
                    .engine
                    .analyze()
                    .study_summary(&params.deck, params.days)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&stats).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Find problem cards (leeches) that may need attention.
pub fn find_problems(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("find_problems")
        .description("Find problem cards (leeches) that may need attention.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: FindProblemsParams| async move {
                debug!(
                    query = %params.query,
                    min_lapses = params.min_lapses,
                    "Finding problems"
                );

                let criteria = ProblemCriteria {
                    min_lapses: params.min_lapses,
                    ..Default::default()
                };

                let problems = state
                    .engine
                    .analyze()
                    .find_problems(&params.query, criteria)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                debug!(count = problems.len(), "Found problem cards");
                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&problems).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Get retention statistics for a deck including average ease and retention rate.
pub fn retention_stats(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("retention_stats")
        .description(
            "Get retention statistics for a deck including average ease and retention rate.",
        )
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: RetentionStatsParams| async move {
                debug!(deck = %params.deck, "Getting retention stats");

                let stats = state
                    .engine
                    .analyze()
                    .retention_stats(&params.deck)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&stats).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}
