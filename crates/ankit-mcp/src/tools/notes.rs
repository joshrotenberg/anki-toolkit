//! Note management tools.

use std::collections::HashMap;
use std::sync::Arc;

use ankit_engine::NoteBuilder;
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddNoteParams {
    /// Deck name to add the note to
    pub deck: String,
    /// Note type (model) name
    pub model: String,
    /// Field values (field_name -> value)
    pub fields: HashMap<String, String>,
    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindNotesParams {
    /// Anki search query (e.g., "deck:Japanese tag:verb")
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNotesInfoParams {
    /// Note IDs to get info for
    pub note_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateNoteParams {
    /// Note ID to update
    pub note_id: i64,
    /// Field values to update (field_name -> value)
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteNotesParams {
    /// Note IDs to delete
    pub note_ids: Vec<i64>,
}

/// Add a single flashcard note to Anki. Returns the new note ID.
pub fn add_note(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("add_note")
        .description("Add a single flashcard note to Anki. Returns the new note ID.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: AddNoteParams| async move {
                state.check_write("add_note")?;
                debug!(deck = %params.deck, model = %params.model, "Adding note");

                let mut builder = NoteBuilder::new(&params.deck, &params.model);
                for (field, value) in &params.fields {
                    builder = builder.field(field, value);
                }
                builder = builder.tags(params.tags);

                let note_id = state
                    .engine
                    .client()
                    .notes()
                    .add(builder.build())
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(note_id, "Note created");
                Ok(CallToolResult::text(format!(
                    "Created note with ID: {}",
                    note_id
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Search for notes using Anki query syntax. Returns note IDs.
pub fn find_notes(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("find_notes")
        .description(
            "Search for notes using Anki query syntax (e.g., 'deck:Japanese tag:verb'). Returns note IDs.",
        )
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: FindNotesParams| async move {
                debug!(query = %params.query, "Finding notes");

                let note_ids = state
                    .engine
                    .client()
                    .notes()
                    .find(&params.query)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                debug!(count = note_ids.len(), "Found notes");
                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&note_ids).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Get detailed information about notes by their IDs.
pub fn get_notes_info(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("get_notes_info")
        .description("Get detailed information about notes by their IDs.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: GetNotesInfoParams| async move {
                debug!(count = params.note_ids.len(), "Getting notes info");

                let notes = state
                    .engine
                    .client()
                    .notes()
                    .info(&params.note_ids)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&notes).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Update a note's field values.
pub fn update_note(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("update_note")
        .description("Update a note's field values.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: UpdateNoteParams| async move {
                state.check_write("update_note")?;
                debug!(note_id = params.note_id, "Updating note");

                state
                    .engine
                    .client()
                    .notes()
                    .update_fields(params.note_id, &params.fields)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(note_id = params.note_id, "Note updated");
                Ok(CallToolResult::text(format!(
                    "Updated note {}",
                    params.note_id
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Delete notes by their IDs. This also deletes all cards generated from the notes.
pub fn delete_notes(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("delete_notes")
        .description(
            "Delete notes by their IDs. This also deletes all cards generated from the notes.",
        )
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: DeleteNotesParams| async move {
                state.check_write("delete_notes")?;
                debug!(count = params.note_ids.len(), "Deleting notes");

                state
                    .engine
                    .client()
                    .notes()
                    .delete(&params.note_ids)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = params.note_ids.len(), "Notes deleted");
                Ok(CallToolResult::text(format!(
                    "Deleted {} notes",
                    params.note_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}
