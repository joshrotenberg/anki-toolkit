//! Bidirectional sync between TOML and Anki.
//!
//! This module provides functionality to synchronize a TOML deck definition
//! with Anki, supporting both push (TOML -> Anki) and pull (Anki -> TOML)
//! operations with conflict detection and resolution.
//!
//! # Example
//!
//! ```no_run
//! use ankit_builder::{DeckBuilder, SyncStrategy};
//!
//! # async fn example() -> ankit_builder::Result<()> {
//! let builder = DeckBuilder::from_file("deck.toml")?;
//!
//! // Preview what sync would do
//! let plan = builder.plan_sync().await?;
//! println!("{} to push, {} to pull, {} conflicts",
//!     plan.to_push.len(), plan.to_pull.len(), plan.conflicts.len());
//!
//! // Execute sync with default strategy
//! let result = builder.sync(SyncStrategy::default()).await?;
//! println!("Pushed: {}, Pulled: {}", result.pushed.len(), result.pulled.len());
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use ankit::AnkiClient;
use serde::Serialize;

use crate::diff::{DeckDiff, DeckDiffer, FieldChange, TagChanges};
use crate::error::Result;
use crate::schema::{DeckDefinition, NoteDef};

/// Strategy for how to handle sync operations.
#[derive(Debug, Clone)]
pub struct SyncStrategy {
    /// How to handle conflicts (note modified in both places).
    pub conflict_resolution: ConflictResolution,
    /// Pull notes that exist in Anki but not TOML.
    pub pull_new_notes: bool,
    /// Push notes that exist in TOML but not Anki.
    pub push_new_notes: bool,
    /// Sync tag changes.
    pub update_tags: bool,
}

impl Default for SyncStrategy {
    fn default() -> Self {
        Self {
            conflict_resolution: ConflictResolution::Skip,
            pull_new_notes: false,
            push_new_notes: true,
            update_tags: true,
        }
    }
}

impl SyncStrategy {
    /// Create a push-only strategy (TOML -> Anki).
    pub fn push_only() -> Self {
        Self {
            conflict_resolution: ConflictResolution::PreferToml,
            pull_new_notes: false,
            push_new_notes: true,
            update_tags: true,
        }
    }

    /// Create a pull-only strategy (Anki -> TOML).
    pub fn pull_only() -> Self {
        Self {
            conflict_resolution: ConflictResolution::PreferAnki,
            pull_new_notes: true,
            push_new_notes: false,
            update_tags: true,
        }
    }
}

/// How to resolve conflicts when a note differs between TOML and Anki.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictResolution {
    /// TOML wins - overwrite Anki with TOML values.
    PreferToml,
    /// Anki wins - update TOML with Anki values.
    PreferAnki,
    /// Fail on any conflict.
    Fail,
    /// Skip conflicting notes (don't sync them).
    #[default]
    Skip,
}

/// A plan for what sync would do, without executing it.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncPlan {
    /// Notes that would be pushed from TOML to Anki.
    pub to_push: Vec<SyncNote>,
    /// Notes that would be pulled from Anki to TOML.
    pub to_pull: Vec<SyncNote>,
    /// Notes that have conflicts (modified in both places).
    pub conflicts: Vec<SyncConflict>,
    /// Number of notes that are identical (no action needed).
    pub unchanged: usize,
}

/// A note involved in sync.
#[derive(Debug, Clone, Serialize)]
pub struct SyncNote {
    /// The Anki note ID (if known).
    pub note_id: Option<i64>,
    /// Model name.
    pub model: String,
    /// Deck name.
    pub deck: String,
    /// First field value (identifier).
    pub first_field: String,
}

/// A conflict where a note differs between TOML and Anki.
#[derive(Debug, Clone, Serialize)]
pub struct SyncConflict {
    /// The Anki note ID.
    pub note_id: i64,
    /// First field value (identifier).
    pub first_field: String,
    /// Model name.
    pub model: String,
    /// Fields that differ.
    pub field_changes: Vec<FieldChange>,
    /// Tag changes.
    pub tag_changes: TagChanges,
}

/// Result of a sync operation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncResult {
    /// Notes pushed from TOML to Anki.
    pub pushed: Vec<SyncedNote>,
    /// Notes pulled from Anki to TOML.
    pub pulled: Vec<SyncedNote>,
    /// Conflicts that were resolved.
    pub resolved_conflicts: Vec<ResolvedConflict>,
    /// Conflicts that were skipped.
    pub skipped_conflicts: Vec<SyncConflict>,
    /// Errors that occurred during sync.
    pub errors: Vec<SyncError>,
    /// Updated TOML definition (if pull_new_notes or conflicts resolved to Anki).
    pub updated_definition: Option<DeckDefinition>,
}

/// A note that was synced.
#[derive(Debug, Clone, Serialize)]
pub struct SyncedNote {
    /// The Anki note ID.
    pub note_id: i64,
    /// First field value.
    pub first_field: String,
    /// Whether this was a new note or an update.
    pub was_new: bool,
}

/// A conflict that was resolved.
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedConflict {
    /// The Anki note ID.
    pub note_id: i64,
    /// First field value.
    pub first_field: String,
    /// How it was resolved.
    pub resolution: String,
}

/// An error that occurred during sync.
#[derive(Debug, Clone, Serialize)]
pub struct SyncError {
    /// Description of what failed.
    pub description: String,
    /// The first field of the note (if applicable).
    pub first_field: Option<String>,
    /// The error message.
    pub error: String,
}

/// Synchronizes a TOML definition with Anki.
pub struct DeckSyncer<'a> {
    client: &'a AnkiClient,
    definition: DeckDefinition,
}

impl<'a> DeckSyncer<'a> {
    /// Create a new syncer.
    pub fn new(client: &'a AnkiClient, definition: DeckDefinition) -> Self {
        Self { client, definition }
    }

    /// Plan what sync would do without executing it.
    pub async fn plan(&self) -> Result<SyncPlan> {
        let differ = DeckDiffer::new(self.client, &self.definition);
        let diff = differ.diff().await?;

        Ok(self.diff_to_plan(diff))
    }

    /// Convert a diff to a sync plan.
    fn diff_to_plan(&self, diff: DeckDiff) -> SyncPlan {
        let mut plan = SyncPlan {
            unchanged: diff.unchanged,
            ..Default::default()
        };

        // Notes only in TOML -> to_push
        for note in diff.toml_only {
            plan.to_push.push(SyncNote {
                note_id: None,
                model: note.model,
                deck: note.deck,
                first_field: note.first_field,
            });
        }

        // Notes only in Anki -> to_pull
        for note in diff.anki_only {
            plan.to_pull.push(SyncNote {
                note_id: note.note_id,
                model: note.model,
                deck: note.deck,
                first_field: note.first_field,
            });
        }

        // Modified notes -> conflicts
        for modified in diff.modified {
            plan.conflicts.push(SyncConflict {
                note_id: modified.note_id,
                first_field: modified.first_field,
                model: modified.model,
                field_changes: modified.field_changes,
                tag_changes: modified.tag_changes,
            });
        }

        plan
    }

    /// Execute sync with the given strategy.
    pub async fn sync(mut self, strategy: SyncStrategy) -> Result<SyncResult> {
        let differ = DeckDiffer::new(self.client, &self.definition);
        let diff = differ.diff().await?;

        let mut result = SyncResult::default();
        let mut definition_modified = false;

        // Handle notes only in TOML (push to Anki)
        if strategy.push_new_notes {
            for note_diff in &diff.toml_only {
                match self.push_new_note(note_diff).await {
                    Ok(note_id) => {
                        result.pushed.push(SyncedNote {
                            note_id,
                            first_field: note_diff.first_field.clone(),
                            was_new: true,
                        });
                    }
                    Err(e) => {
                        result.errors.push(SyncError {
                            description: "Failed to push new note".to_string(),
                            first_field: Some(note_diff.first_field.clone()),
                            error: e.to_string(),
                        });
                    }
                }
            }
        }

        // Handle notes only in Anki (pull to TOML)
        if strategy.pull_new_notes {
            for note_diff in &diff.anki_only {
                if let Some(note_id) = note_diff.note_id {
                    match self.pull_note(note_id, &note_diff.deck).await {
                        Ok(note_def) => {
                            self.definition.notes.push(note_def);
                            definition_modified = true;
                            result.pulled.push(SyncedNote {
                                note_id,
                                first_field: note_diff.first_field.clone(),
                                was_new: true,
                            });
                        }
                        Err(e) => {
                            result.errors.push(SyncError {
                                description: "Failed to pull note".to_string(),
                                first_field: Some(note_diff.first_field.clone()),
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Handle conflicts (modified in both)
        for modified in diff.modified {
            let conflict = SyncConflict {
                note_id: modified.note_id,
                first_field: modified.first_field.clone(),
                model: modified.model.clone(),
                field_changes: modified.field_changes.clone(),
                tag_changes: modified.tag_changes.clone(),
            };

            match strategy.conflict_resolution {
                ConflictResolution::Fail => {
                    return Err(crate::error::Error::SyncConflict(format!(
                        "Conflict on note '{}' - use a different conflict resolution strategy",
                        modified.first_field
                    )));
                }
                ConflictResolution::Skip => {
                    result.skipped_conflicts.push(conflict);
                }
                ConflictResolution::PreferToml => {
                    // Push TOML values to Anki
                    match self
                        .push_updates(modified.note_id, &modified.field_changes, &strategy)
                        .await
                    {
                        Ok(_) => {
                            result.resolved_conflicts.push(ResolvedConflict {
                                note_id: modified.note_id,
                                first_field: modified.first_field,
                                resolution: "Pushed TOML values to Anki".to_string(),
                            });
                        }
                        Err(e) => {
                            result.errors.push(SyncError {
                                description: "Failed to push conflict resolution".to_string(),
                                first_field: Some(conflict.first_field.clone()),
                                error: e.to_string(),
                            });
                            result.skipped_conflicts.push(conflict);
                        }
                    }
                }
                ConflictResolution::PreferAnki => {
                    // Update TOML with Anki values
                    self.update_definition_from_anki(
                        &modified.first_field,
                        &modified.model,
                        &modified.field_changes,
                        &modified.tag_changes,
                    );
                    definition_modified = true;
                    result.resolved_conflicts.push(ResolvedConflict {
                        note_id: modified.note_id,
                        first_field: modified.first_field,
                        resolution: "Updated TOML with Anki values".to_string(),
                    });
                }
            }
        }

        if definition_modified {
            result.updated_definition = Some(self.definition);
        }

        Ok(result)
    }

    /// Push a new note from TOML to Anki.
    async fn push_new_note(&self, note_diff: &crate::diff::NoteDiff) -> Result<i64> {
        // Find the note in our definition
        let note_def = self
            .definition
            .notes
            .iter()
            .find(|n| {
                let model = self.definition.get_model(&n.model);
                if let Some(model) = model {
                    if let Some(first_field_name) = model.fields.first() {
                        let first_field =
                            n.fields.get(first_field_name).cloned().unwrap_or_default();
                        return first_field == note_diff.first_field && n.model == note_diff.model;
                    }
                }
                false
            })
            .ok_or_else(|| {
                crate::error::Error::InvalidDefinition(format!(
                    "Note '{}' not found in definition",
                    note_diff.first_field
                ))
            })?;

        // Create the note in Anki
        let note =
            ankit::NoteBuilder::new(&note_def.deck, &note_def.model).tags(note_def.tags.clone());

        let mut note = note;
        for (field, value) in &note_def.fields {
            note = note.field(field, value);
        }

        let note_id = self.client.notes().add(note.build()).await?;
        Ok(note_id)
    }

    /// Pull a note from Anki to TOML definition.
    async fn pull_note(&self, note_id: i64, deck: &str) -> Result<NoteDef> {
        let note_infos = self.client.notes().info(&[note_id]).await?;
        let note_info = note_infos.into_iter().next().ok_or_else(|| {
            crate::error::Error::InvalidDefinition(format!("Note {} not found in Anki", note_id))
        })?;

        let fields: HashMap<String, String> = note_info
            .fields
            .into_iter()
            .map(|(name, field)| (name, field.value))
            .collect();

        Ok(NoteDef {
            deck: deck.to_string(),
            model: note_info.model_name,
            fields,
            tags: note_info.tags,
            guid: None,
        })
    }

    /// Push field updates to Anki for conflict resolution.
    async fn push_updates(
        &self,
        note_id: i64,
        field_changes: &[FieldChange],
        strategy: &SyncStrategy,
    ) -> Result<()> {
        // Build the fields to update (use TOML values)
        let fields: HashMap<String, String> = field_changes
            .iter()
            .map(|fc| (fc.field.clone(), fc.toml_value.clone()))
            .collect();

        if !fields.is_empty() {
            self.client.notes().update_fields(note_id, &fields).await?;
        }

        // Update tags if enabled
        if strategy.update_tags {
            // We'd need the full tag lists to do this properly
            // For now, skip tag sync on conflicts
        }

        Ok(())
    }

    /// Update the TOML definition with Anki values for a note.
    fn update_definition_from_anki(
        &mut self,
        first_field: &str,
        model_name: &str,
        field_changes: &[FieldChange],
        tag_changes: &TagChanges,
    ) {
        // Find the note in our definition
        for note in &mut self.definition.notes {
            let model = self.definition.models.iter().find(|m| m.name == note.model);
            if let Some(model) = model {
                if let Some(first_field_name) = model.fields.first() {
                    let note_first_field = note
                        .fields
                        .get(first_field_name)
                        .cloned()
                        .unwrap_or_default();
                    if note_first_field == first_field && note.model == model_name {
                        // Update fields with Anki values
                        for fc in field_changes {
                            note.fields.insert(fc.field.clone(), fc.anki_value.clone());
                        }

                        // Update tags
                        for tag in &tag_changes.removed {
                            // Tags in Anki but not TOML - add to TOML
                            if !note.tags.contains(tag) {
                                note.tags.push(tag.clone());
                            }
                        }
                        for tag in &tag_changes.added {
                            // Tags in TOML but not Anki - remove from TOML
                            note.tags.retain(|t| t != tag);
                        }

                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_strategy_default() {
        let strategy = SyncStrategy::default();
        assert_eq!(strategy.conflict_resolution, ConflictResolution::Skip);
        assert!(!strategy.pull_new_notes);
        assert!(strategy.push_new_notes);
        assert!(strategy.update_tags);
    }

    #[test]
    fn test_sync_strategy_push_only() {
        let strategy = SyncStrategy::push_only();
        assert_eq!(strategy.conflict_resolution, ConflictResolution::PreferToml);
        assert!(!strategy.pull_new_notes);
        assert!(strategy.push_new_notes);
    }

    #[test]
    fn test_sync_strategy_pull_only() {
        let strategy = SyncStrategy::pull_only();
        assert_eq!(strategy.conflict_resolution, ConflictResolution::PreferAnki);
        assert!(strategy.pull_new_notes);
        assert!(!strategy.push_new_notes);
    }
}
