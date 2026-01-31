//! Enrichment tools.

use std::collections::HashMap;
use std::sync::Arc;

use ankit_engine::enrich::EnrichQuery;
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindEnrichCandidatesParams {
    /// Anki search query to filter notes
    pub query: String,
    /// Field names to check for empty values
    pub empty_fields: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnrichNoteParams {
    /// Note ID to update
    pub note_id: i64,
    /// Field values to set (field_name -> value)
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnrichNotesParams {
    /// Updates to apply: list of (note_id, fields) pairs
    pub updates: Vec<EnrichNoteUpdate>,
    /// Optional tag to add to enriched notes
    #[serde(default)]
    pub tag_enriched: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnrichNoteUpdate {
    /// Note ID to update
    pub note_id: i64,
    /// Field values to set
    pub fields: HashMap<String, String>,
}

/// Find notes with empty fields that need enrichment.
pub fn find_enrich_candidates(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("find_enrich_candidates")
        .description("Find notes with empty fields that need enrichment. Returns candidates with their current field values and which fields are empty.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: FindEnrichCandidatesParams| async move {
                debug!(query = %params.query, empty_fields = ?params.empty_fields, "Finding enrich candidates");

                let query = EnrichQuery {
                    search: params.query,
                    empty_fields: params.empty_fields,
                };

                let candidates = state
                    .engine
                    .enrich()
                    .find_candidates(&query)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                debug!(count = candidates.len(), "Found enrich candidates");
                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&candidates).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Update a single note with new field values for enrichment.
pub fn enrich_note(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("enrich_note")
        .description("Update a single note with new field values for enrichment.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: EnrichNoteParams| async move {
                state.check_write("enrich_note")?;
                debug!(note_id = params.note_id, "Enriching note");

                state
                    .engine
                    .enrich()
                    .update_note(params.note_id, &params.fields)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(note_id = params.note_id, "Note enriched");
                Ok(CallToolResult::text(format!(
                    "Enriched note {}",
                    params.note_id
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Update multiple notes with enriched content.
pub fn enrich_notes(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("enrich_notes")
        .description(
            "Update multiple notes with enriched content. Optionally tag them as enriched.",
        )
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: EnrichNotesParams| async move {
                state.check_write("enrich_notes")?;
                debug!(count = params.updates.len(), "Enriching notes");

                let updates: Vec<(i64, HashMap<String, String>)> = params
                    .updates
                    .into_iter()
                    .map(|u| (u.note_id, u.fields))
                    .collect();

                let report = state
                    .engine
                    .enrich()
                    .update_notes(&updates)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                // Tag enriched notes if requested
                if let Some(tag) = params.tag_enriched {
                    let note_ids: Vec<i64> = updates.iter().map(|(id, _)| *id).collect();
                    state
                        .engine
                        .enrich()
                        .tag_enriched(&note_ids, &tag)
                        .await
                        .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;
                }

                info!(
                    updated = report.updated,
                    failed = report.failed,
                    "Notes enriched"
                );
                Ok(CallToolResult::text(format!(
                    "Enriched {} notes ({} failed)",
                    report.updated, report.failed
                )))
            },
        )
        .build()
        .expect("valid tool")
}
