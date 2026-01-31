//! Tag management tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddTagsParams {
    /// Note IDs to add tags to
    pub note_ids: Vec<i64>,
    /// Tags to add (space-separated string, e.g., "tag1 tag2")
    pub tags: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveTagsParams {
    /// Note IDs to remove tags from
    pub note_ids: Vec<i64>,
    /// Tags to remove (space-separated string, e.g., "tag1 tag2")
    pub tags: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReplaceTagsAllParams {
    /// Tag to replace
    pub old_tag: String,
    /// Replacement tag
    pub new_tag: String,
}

/// Add tags to notes. Tags are space-separated.
pub fn add_tags(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("add_tags")
        .description("Add tags to notes. Tags are space-separated (e.g., 'tag1 tag2').")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: AddTagsParams| async move {
                state.check_write("add_tags")?;
                debug!(count = params.note_ids.len(), tags = %params.tags, "Adding tags");

                state
                    .engine
                    .client()
                    .notes()
                    .add_tags(&params.note_ids, &params.tags)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = params.note_ids.len(), tags = %params.tags, "Tags added");
                Ok(CallToolResult::text(format!(
                    "Added tags '{}' to {} notes",
                    params.tags,
                    params.note_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Remove tags from notes. Tags are space-separated.
pub fn remove_tags(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("remove_tags")
        .description("Remove tags from notes. Tags are space-separated (e.g., 'tag1 tag2').")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: RemoveTagsParams| async move {
                state.check_write("remove_tags")?;
                debug!(count = params.note_ids.len(), tags = %params.tags, "Removing tags");

                state
                    .engine
                    .client()
                    .notes()
                    .remove_tags(&params.note_ids, &params.tags)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = params.note_ids.len(), tags = %params.tags, "Tags removed");
                Ok(CallToolResult::text(format!(
                    "Removed tags '{}' from {} notes",
                    params.tags,
                    params.note_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Replace a tag with another across all notes in the collection.
pub fn replace_tags_all(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("replace_tags_all")
        .description("Replace a tag with another across all notes in the collection.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ReplaceTagsAllParams| async move {
                state.check_write("replace_tags_all")?;
                debug!(old = %params.old_tag, new = %params.new_tag, "Replacing tag globally");

                state
                    .engine
                    .client()
                    .notes()
                    .replace_tags_all(&params.old_tag, &params.new_tag)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(old = %params.old_tag, new = %params.new_tag, "Tag replaced globally");
                Ok(CallToolResult::text(format!(
                    "Replaced tag '{}' with '{}' across all notes",
                    params.old_tag, params.new_tag
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Remove all tags that are not used by any notes.
pub fn clear_unused_tags(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("clear_unused_tags")
        .description("Remove all tags that are not used by any notes.")
        .handler_no_params_with_state(state, |state: Arc<AnkiState>| async move {
            state.check_write("clear_unused_tags")?;
            debug!("Clearing unused tags");

            state
                .engine
                .client()
                .notes()
                .clear_unused_tags()
                .await
                .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

            info!("Unused tags cleared");
            Ok(CallToolResult::text("Cleared all unused tags"))
        })
        .expect("valid tool")
}
