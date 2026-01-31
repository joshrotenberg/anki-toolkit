//! Import tools.

use std::collections::HashMap;
use std::sync::Arc;

use ankit_engine::{NoteBuilder, import::OnDuplicate};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportNote {
    /// Deck name
    pub deck: String,
    /// Model (note type) name
    pub model: String,
    /// Field values
    pub fields: HashMap<String, String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportNotesParams {
    /// Notes to import
    pub notes: Vec<ImportNote>,
    /// How to handle duplicates: "skip", "update", or "allow"
    #[serde(default = "default_on_duplicate")]
    pub on_duplicate: String,
}

fn default_on_duplicate() -> String {
    "skip".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateNotesParams {
    /// Notes to validate
    pub notes: Vec<ImportNote>,
}

/// Import multiple notes with duplicate handling.
pub fn import_notes(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("import_notes")
        .description("Import multiple notes with duplicate handling. on_duplicate can be 'skip', 'update', or 'allow'.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ImportNotesParams| async move {
                state.check_write("import_notes")?;
                debug!(
                    count = params.notes.len(),
                    on_duplicate = %params.on_duplicate,
                    "Importing notes"
                );

                let on_duplicate = match params.on_duplicate.as_str() {
                    "update" => OnDuplicate::Update,
                    "allow" => OnDuplicate::Allow,
                    _ => OnDuplicate::Skip,
                };

                let notes: Vec<_> = params
                    .notes
                    .iter()
                    .map(|n| {
                        let mut builder = NoteBuilder::new(&n.deck, &n.model);
                        for (field, value) in &n.fields {
                            builder = builder.field(field, value);
                        }
                        builder.tags(n.tags.clone()).build()
                    })
                    .collect();

                let report = state
                    .engine
                    .import()
                    .notes(&notes, on_duplicate)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    added = report.added,
                    skipped = report.skipped,
                    updated = report.updated,
                    failed = report.failed,
                    "Import completed"
                );
                Ok(CallToolResult::text(format!(
                    "Import complete: {} added, {} skipped, {} updated, {} failed",
                    report.added, report.skipped, report.updated, report.failed
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Validate notes before importing. Checks if decks and models exist.
pub fn validate_notes(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("validate_notes")
        .description("Validate notes before importing. Checks if decks and models exist.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ValidateNotesParams| async move {
                debug!(count = params.notes.len(), "Validating notes");

                let notes: Vec<_> = params
                    .notes
                    .iter()
                    .map(|n| {
                        let mut builder = NoteBuilder::new(&n.deck, &n.model);
                        for (field, value) in &n.fields {
                            builder = builder.field(field, value);
                        }
                        builder.tags(n.tags.clone()).build()
                    })
                    .collect();

                let results = state
                    .engine
                    .import()
                    .validate(&notes)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                let valid_count = results.iter().filter(|r| r.valid).count();
                let invalid: Vec<_> = results
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| !r.valid)
                    .map(|(i, r)| format!("Note {}: {}", i, r.errors.join(", ")))
                    .collect();

                let message = if invalid.is_empty() {
                    format!("All {} notes are valid", valid_count)
                } else {
                    format!(
                        "{} valid, {} invalid:\n{}",
                        valid_count,
                        invalid.len(),
                        invalid.join("\n")
                    )
                };

                Ok(CallToolResult::text(message))
            },
        )
        .build()
        .expect("valid tool")
}
