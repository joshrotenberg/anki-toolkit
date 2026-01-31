//! Miscellaneous tools (sync, version).

use std::sync::Arc;

use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

/// Get the AnkiConnect version. Useful for checking if Anki is running.
pub fn version(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("version")
        .description("Get the AnkiConnect version. Useful for checking if Anki is running.")
        .read_only()
        .handler_with_state_no_params(state, |state: Arc<AnkiState>| async move {
            debug!("Getting AnkiConnect version");

            let version = state
                .engine
                .client()
                .misc()
                .version()
                .await
                .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

            Ok(CallToolResult::text(format!(
                "AnkiConnect version: {}",
                version
            )))
        })
        .expect("valid tool")
}

/// Sync the Anki collection with AnkiWeb.
pub fn sync(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("sync")
        .description("Sync the Anki collection with AnkiWeb.")
        .handler_with_state_no_params(state, |state: Arc<AnkiState>| async move {
            state.check_write("sync")?;
            debug!("Syncing with AnkiWeb");

            state
                .engine
                .client()
                .misc()
                .sync()
                .await
                .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

            info!("Sync completed");
            Ok(CallToolResult::text("Sync completed successfully"))
        })
        .expect("valid tool")
}
