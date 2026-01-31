//! Organization tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveByTagParams {
    /// Tag to search for
    pub tag: String,
    /// Destination deck name
    pub destination: String,
}

/// Move all notes with a specific tag to a destination deck.
pub fn move_by_tag(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("move_by_tag")
        .description("Move all notes with a specific tag to a destination deck.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: MoveByTagParams| async move {
                state.check_write("move_by_tag")?;
                debug!(tag = %params.tag, destination = %params.destination, "Moving by tag");

                let count = state
                    .engine
                    .organize()
                    .move_by_tag(&params.tag, &params.destination)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    count,
                    tag = %params.tag,
                    destination = %params.destination,
                    "Cards moved"
                );
                Ok(CallToolResult::text(format!(
                    "Moved {} cards with tag '{}' to '{}'",
                    count, params.tag, params.destination
                )))
            },
        )
        .build()
        .expect("valid tool")
}
