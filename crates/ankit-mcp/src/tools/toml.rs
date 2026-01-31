//! TOML deck definition tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Error, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportDeckTomlParams {
    /// Deck name to export
    pub deck: String,
    /// Optional path to write TOML file directly (if omitted, returns content)
    #[serde(default)]
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiffDeckTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    pub toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    pub toml_path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlanSyncTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    pub toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    pub toml_path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncDeckTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    pub toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    pub toml_path: Option<String>,
    /// Sync strategy: "push_only", "pull_only", or "bidirectional"
    #[serde(default = "default_sync_strategy")]
    pub strategy: String,
    /// Conflict resolution: "prefer_toml", "prefer_anki", "fail", or "skip"
    #[serde(default = "default_conflict_resolution")]
    pub conflict_resolution: String,
}

fn default_sync_strategy() -> String {
    "push_only".to_string()
}

fn default_conflict_resolution() -> String {
    "skip".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportDeckTomlParams {
    /// TOML definition content (mutually exclusive with toml_path)
    #[serde(default)]
    pub toml_content: Option<String>,
    /// Path to TOML file (mutually exclusive with toml_content)
    #[serde(default)]
    pub toml_path: Option<String>,
}

/// Helper to resolve TOML content from either inline content or file path.
fn resolve_toml_content(
    toml_content: Option<String>,
    toml_path: Option<String>,
) -> Result<String, Error> {
    match (toml_content, toml_path) {
        (Some(content), None) => Ok(content),
        (None, Some(path)) => std::fs::read_to_string(&path)
            .map_err(|e| Error::tool(format!("Failed to read TOML file '{}': {}", path, e))),
        (Some(_), Some(_)) => Err(Error::tool(
            "Provide either toml_content or toml_path, not both",
        )),
        (None, None) => Err(Error::tool("Must provide either toml_content or toml_path")),
    }
}

/// Export a deck from Anki to TOML format.
pub fn export_deck_toml(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("export_deck_toml")
        .description("Export a deck from Anki to TOML format. Returns the TOML content, or writes to output_path if provided.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ExportDeckTomlParams| async move {
                debug!(deck = %params.deck, output_path = ?params.output_path, "Exporting deck to TOML");

                let builder =
                    ankit_builder::DeckBuilder::from_anki(state.engine.client(), &params.deck)
                        .await
                        .map_err(|e| Error::tool(e.to_string()))?;

                let toml = builder
                    .definition()
                    .to_toml()
                    .map_err(|e| Error::tool(e.to_string()))?;

                // Write to file if output_path provided, otherwise return content
                if let Some(path) = params.output_path {
                    std::fs::write(&path, &toml)
                        .map_err(|e| Error::tool(format!("Failed to write to '{}': {}", path, e)))?;
                    let note_count = builder.definition().notes.len();
                    info!(deck = %params.deck, path = %path, notes = note_count, "Deck exported to file");
                    Ok(CallToolResult::text(format!(
                        "Exported {} notes to '{}'",
                        note_count, path
                    )))
                } else {
                    info!(deck = %params.deck, "Deck exported to TOML");
                    Ok(CallToolResult::text(toml))
                }
            },
        )
        .build()
        .expect("valid tool")
}

/// Compare a TOML deck definition against the current state in Anki.
pub fn diff_deck_toml(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("diff_deck_toml")
        .description("Compare a TOML deck definition against the current state in Anki. Shows notes only in TOML, only in Anki, and modified notes.")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: DiffDeckTomlParams| async move {
                debug!("Diffing TOML against Anki");

                let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
                let builder = ankit_builder::DeckBuilder::parse(&toml_content)
                    .map_err(|e| Error::tool(e.to_string()))?;

                let diff = builder
                    .diff_connect_with_client(state.engine.client())
                    .await
                    .map_err(|e| Error::tool(e.to_string()))?;

                debug!(
                    toml_only = diff.toml_only.len(),
                    anki_only = diff.anki_only.len(),
                    modified = diff.modified.len(),
                    unchanged = diff.unchanged,
                    "Diff completed"
                );

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&diff).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Preview what sync would do between a TOML definition and Anki.
pub fn plan_sync_toml(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("plan_sync_toml")
        .description(
            "Preview what sync would do between a TOML definition and Anki without making changes.",
        )
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: PlanSyncTomlParams| async move {
                debug!("Planning TOML sync");

                let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
                let builder = ankit_builder::DeckBuilder::parse(&toml_content)
                    .map_err(|e| Error::tool(e.to_string()))?;

                let plan = builder
                    .plan_sync_with_client(state.engine.client())
                    .await
                    .map_err(|e| Error::tool(e.to_string()))?;

                debug!(
                    to_push = plan.to_push.len(),
                    to_pull = plan.to_pull.len(),
                    conflicts = plan.conflicts.len(),
                    "Sync plan generated"
                );

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&plan).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Sync a TOML deck definition with Anki.
pub fn sync_deck_toml(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("sync_deck_toml")
        .description("Sync a TOML deck definition with Anki. Strategy can be 'push_only' (TOML -> Anki), 'pull_only' (Anki -> TOML), or 'bidirectional'. Returns sync results and optionally updated TOML.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: SyncDeckTomlParams| async move {
                state.check_write("sync_deck_toml")?;
                debug!(strategy = %params.strategy, "Syncing TOML with Anki");

                let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
                let builder = ankit_builder::DeckBuilder::parse(&toml_content)
                    .map_err(|e| Error::tool(e.to_string()))?;

                let conflict_resolution = match params.conflict_resolution.as_str() {
                    "prefer_toml" => ankit_builder::ConflictResolution::PreferToml,
                    "prefer_anki" => ankit_builder::ConflictResolution::PreferAnki,
                    "fail" => ankit_builder::ConflictResolution::Fail,
                    _ => ankit_builder::ConflictResolution::Skip,
                };

                let strategy = match params.strategy.as_str() {
                    "pull_only" => ankit_builder::SyncStrategy::pull_only(),
                    "bidirectional" => ankit_builder::SyncStrategy {
                        conflict_resolution,
                        pull_new_notes: true,
                        push_new_notes: true,
                        update_tags: true,
                    },
                    _ => ankit_builder::SyncStrategy::push_only(),
                };

                let result = builder
                    .sync_with_client(state.engine.client(), strategy)
                    .await
                    .map_err(|e| Error::tool(e.to_string()))?;

                info!(
                    pushed = result.pushed.len(),
                    pulled = result.pulled.len(),
                    resolved = result.resolved_conflicts.len(),
                    skipped = result.skipped_conflicts.len(),
                    errors = result.errors.len(),
                    "Sync completed"
                );

                // Build response with results and optionally updated TOML
                let mut response = serde_json::json!({
                    "pushed": result.pushed.len(),
                    "pulled": result.pulled.len(),
                    "resolved_conflicts": result.resolved_conflicts.len(),
                    "skipped_conflicts": result.skipped_conflicts.len(),
                    "errors": result.errors,
                });

                if let Some(updated_def) = result.updated_definition {
                    if let Ok(updated_toml) = updated_def.to_toml() {
                        response["updated_toml"] = serde_json::Value::String(updated_toml);
                    }
                }

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&response).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Import a TOML deck definition into Anki.
pub fn import_deck_toml(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("import_deck_toml")
        .description("Import a TOML deck definition into Anki. Creates decks and adds notes.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ImportDeckTomlParams| async move {
                state.check_write("import_deck_toml")?;
                debug!("Importing TOML to Anki");

                let toml_content = resolve_toml_content(params.toml_content, params.toml_path)?;
                let builder = ankit_builder::DeckBuilder::parse(&toml_content)
                    .map_err(|e| Error::tool(e.to_string()))?;

                let result = builder
                    .import_connect_batch()
                    .await
                    .map_err(|e| Error::tool(e.to_string()))?;

                info!(
                    decks_created = result.decks_created,
                    notes_created = result.notes_created,
                    notes_skipped = result.notes_skipped,
                    "TOML imported"
                );

                Ok(CallToolResult::text(format!(
                    "Imported: {} decks created, {} notes created, {} notes skipped",
                    result.decks_created, result.notes_created, result.notes_skipped
                )))
            },
        )
        .build()
        .expect("valid tool")
}
