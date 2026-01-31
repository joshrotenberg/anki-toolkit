//! Export tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::debug;

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportDeckParams {
    /// Deck name to export
    pub deck: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportReviewsParams {
    /// Anki search query to select cards
    pub query: String,
}

/// Export all notes and cards from a deck as JSON.
pub fn export_deck(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("export_deck")
        .description("Export all notes and cards from a deck as JSON.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ExportDeckParams| async move {
                debug!(deck = %params.deck, "Exporting deck");

                let export = state
                    .engine
                    .export()
                    .deck(&params.deck)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&export).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Export review history for cards matching an Anki query.
pub fn export_reviews(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("export_reviews")
        .description("Export review history for cards matching an Anki query.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ExportReviewsParams| async move {
                debug!(query = %params.query, "Exporting reviews");

                let reviews = state
                    .engine
                    .export()
                    .reviews(&params.query)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&reviews).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}
