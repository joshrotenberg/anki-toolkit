//! Media management tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CleanupMediaParams {
    /// If true, only report what would be deleted
    pub dry_run: bool,
}

/// Audit media files to find orphaned files and missing references.
pub fn audit_media(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("audit_media")
        .description("Audit media files to find orphaned files and missing references.")
        .read_only()
        .handler_with_state_no_params(state, |state: Arc<AnkiState>| async move {
            debug!("Auditing media");

            let audit = state
                .engine
                .media()
                .audit()
                .await
                .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

            Ok(CallToolResult::text(
                serde_json::to_string_pretty(&audit).unwrap(),
            ))
        })
        .expect("valid tool")
}

/// Clean up orphaned media files. Set dry_run=true to preview without deleting.
pub fn cleanup_media(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("cleanup_media")
        .description("Clean up orphaned media files. Set dry_run=true to preview without deleting.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: CleanupMediaParams| async move {
                if !params.dry_run {
                    state.check_write("cleanup_media")?;
                }
                debug!(dry_run = params.dry_run, "Cleaning up media");

                let report = state
                    .engine
                    .media()
                    .cleanup_orphaned(params.dry_run)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                let action = if params.dry_run {
                    "Would delete"
                } else {
                    info!(count = report.files_deleted, "Media files deleted");
                    "Deleted"
                };

                Ok(CallToolResult::text(format!(
                    "{} {} files",
                    action, report.files_deleted
                )))
            },
        )
        .build()
        .expect("valid tool")
}
