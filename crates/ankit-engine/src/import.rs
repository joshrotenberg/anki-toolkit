//! Bulk import operations with duplicate handling.
//!
//! This module provides high-level import workflows that combine validation,
//! duplicate detection, and batch operations.
//!
//! # Example
//!
//! ```no_run
//! use ankit_engine::{Engine, NoteBuilder};
//! use ankit_engine::import::OnDuplicate;
//!
//! # async fn example() -> ankit_engine::Result<()> {
//! let engine = Engine::new();
//!
//! let notes = vec![
//!     NoteBuilder::new("Japanese", "Basic")
//!         .field("Front", "hello")
//!         .field("Back", "world")
//!         .build(),
//! ];
//!
//! let report = engine.import().notes(&notes, OnDuplicate::Skip).await?;
//! println!("Added: {}, Skipped: {}", report.added, report.skipped);
//! # Ok(())
//! # }
//! ```

use crate::{Note, Result};
use ankit::AnkiClient;

/// Strategy for handling duplicate notes during import.
#[derive(Debug, Clone, Copy, Default)]
pub enum OnDuplicate {
    /// Skip duplicate notes (default).
    #[default]
    Skip,
    /// Update existing notes with new field values.
    Update,
    /// Allow duplicates to be created.
    Allow,
}

/// Report of an import operation.
#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    /// Number of notes successfully added.
    pub added: usize,
    /// Number of notes skipped (duplicates).
    pub skipped: usize,
    /// Number of notes updated (when using OnDuplicate::Update).
    pub updated: usize,
    /// Number of notes that failed to import.
    pub failed: usize,
    /// Details about failed imports.
    pub failures: Vec<ImportFailure>,
}

/// Details about a failed import.
#[derive(Debug, Clone)]
pub struct ImportFailure {
    /// Index of the note in the input list.
    pub index: usize,
    /// Error message.
    pub error: String,
}

/// Import workflow engine.
#[derive(Debug)]
pub struct ImportEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> ImportEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Import notes with duplicate handling.
    ///
    /// Validates notes, checks for duplicates, and imports in batches.
    ///
    /// # Arguments
    ///
    /// * `notes` - Notes to import
    /// * `on_duplicate` - Strategy for handling duplicates
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::{Engine, NoteBuilder};
    /// # use ankit_engine::import::OnDuplicate;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let notes = vec![
    ///     NoteBuilder::new("Default", "Basic")
    ///         .field("Front", "Q1")
    ///         .field("Back", "A1")
    ///         .build(),
    /// ];
    ///
    /// let report = engine.import().notes(&notes, OnDuplicate::Skip).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn notes(&self, notes: &[Note], on_duplicate: OnDuplicate) -> Result<ImportReport> {
        if notes.is_empty() {
            return Ok(ImportReport::default());
        }

        let mut report = ImportReport::default();

        // Check which notes can be added
        let can_add = self.client.notes().can_add_detailed(notes).await?;

        match on_duplicate {
            OnDuplicate::Skip => {
                // Filter to only notes that can be added
                let addable: Vec<_> = notes
                    .iter()
                    .zip(can_add.iter())
                    .filter(|(_, result)| result.can_add)
                    .map(|(note, _)| note.clone())
                    .collect();

                report.skipped = notes.len() - addable.len();

                if !addable.is_empty() {
                    let results = self.client.notes().add_many(&addable).await?;
                    for (i, result) in results.iter().enumerate() {
                        if result.is_some() {
                            report.added += 1;
                        } else {
                            report.failed += 1;
                            report.failures.push(ImportFailure {
                                index: i,
                                error: "Failed to add note".to_string(),
                            });
                        }
                    }
                }
            }
            OnDuplicate::Allow => {
                // Add all notes, allowing duplicates
                let notes_with_allow: Vec<_> = notes
                    .iter()
                    .map(|n| {
                        let mut note = n.clone();
                        let options = note.options.get_or_insert_with(Default::default);
                        options.allow_duplicate = Some(true);
                        note
                    })
                    .collect();

                let results = self.client.notes().add_many(&notes_with_allow).await?;
                for (i, result) in results.iter().enumerate() {
                    if result.is_some() {
                        report.added += 1;
                    } else {
                        report.failed += 1;
                        report.failures.push(ImportFailure {
                            index: i,
                            error: "Failed to add note".to_string(),
                        });
                    }
                }
            }
            OnDuplicate::Update => {
                // For duplicates, find and update existing notes
                for (i, (note, result)) in notes.iter().zip(can_add.iter()).enumerate() {
                    if result.can_add {
                        // Not a duplicate, add it
                        match self.client.notes().add(note.clone()).await {
                            Ok(_) => report.added += 1,
                            Err(e) => {
                                report.failed += 1;
                                report.failures.push(ImportFailure {
                                    index: i,
                                    error: e.to_string(),
                                });
                            }
                        }
                    } else {
                        // Duplicate - find and update
                        // Use the first field value to search for duplicates
                        if let Some((field_name, field_value)) = note.fields.iter().next() {
                            let query =
                                format!("\"{}:{}\"", field_name, field_value.replace('\"', "\\\""));
                            match self.client.notes().find(&query).await {
                                Ok(existing) if !existing.is_empty() => {
                                    // Update the first match
                                    match self
                                        .client
                                        .notes()
                                        .update_fields(existing[0], &note.fields)
                                        .await
                                    {
                                        Ok(_) => report.updated += 1,
                                        Err(e) => {
                                            report.failed += 1;
                                            report.failures.push(ImportFailure {
                                                index: i,
                                                error: e.to_string(),
                                            });
                                        }
                                    }
                                }
                                _ => {
                                    report.skipped += 1;
                                }
                            }
                        } else {
                            report.skipped += 1;
                        }
                    }
                }
            }
        }

        Ok(report)
    }

    /// Validate notes before import without actually importing.
    ///
    /// Returns detailed validation results for each note.
    pub async fn validate(&self, notes: &[Note]) -> Result<Vec<ValidationResult>> {
        // Check model and deck existence
        let models = self.client.models().names().await?;
        let decks = self.client.decks().names().await?;

        let mut results = Vec::with_capacity(notes.len());

        for note in notes {
            let mut errors = Vec::new();

            // Check model exists
            if !models.contains(&note.model_name) {
                errors.push(format!("Model '{}' not found", note.model_name));
            } else {
                // Check fields match model
                let model_fields = self.client.models().field_names(&note.model_name).await?;
                for field_name in note.fields.keys() {
                    if !model_fields.contains(field_name) {
                        errors.push(format!("Unknown field '{}'", field_name));
                    }
                }
            }

            // Check deck exists
            if !decks.contains(&note.deck_name) {
                errors.push(format!("Deck '{}' not found", note.deck_name));
            }

            results.push(ValidationResult {
                valid: errors.is_empty(),
                errors,
            });
        }

        Ok(results)
    }
}

/// Result of validating a single note.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the note is valid.
    pub valid: bool,
    /// Validation errors, if any.
    pub errors: Vec<String>,
}
