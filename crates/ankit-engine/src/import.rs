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

    /// Smart add a single note with duplicate checking and tag suggestions.
    ///
    /// Combines validation, duplicate detection, and tag suggestions into
    /// a single atomic operation.
    ///
    /// # Arguments
    ///
    /// * `note` - Note to add
    /// * `options` - Options controlling duplicate handling and tag suggestions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::{Engine, NoteBuilder};
    /// # use ankit_engine::import::SmartAddOptions;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let note = NoteBuilder::new("Japanese", "Basic")
    ///     .field("Front", "hello")
    ///     .field("Back", "world")
    ///     .build();
    ///
    /// let result = engine.import().smart_add(&note, SmartAddOptions::default()).await?;
    ///
    /// match result.status {
    ///     ankit_engine::import::SmartAddStatus::Added => {
    ///         println!("Added note: {:?}", result.note_id);
    ///     }
    ///     ankit_engine::import::SmartAddStatus::RejectedDuplicate { existing_id } => {
    ///         println!("Duplicate of note {}", existing_id);
    ///     }
    ///     _ => {}
    /// }
    ///
    /// if !result.suggested_tags.is_empty() {
    ///     println!("Suggested tags: {:?}", result.suggested_tags);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn smart_add(&self, note: &Note, options: SmartAddOptions) -> Result<SmartAddResult> {
        let mut result = SmartAddResult {
            note_id: None,
            status: SmartAddStatus::Added,
            suggested_tags: Vec::new(),
            similar_notes: Vec::new(),
        };

        // Check for empty fields if requested
        if options.check_empty_fields {
            let empty_fields: Vec<String> = note
                .fields
                .iter()
                .filter(|(_, v)| v.trim().is_empty())
                .map(|(k, _)| k.clone())
                .collect();

            if !empty_fields.is_empty() {
                result.status = SmartAddStatus::RejectedEmptyFields {
                    fields: empty_fields,
                };
                return Ok(result);
            }
        }

        // Validate the note
        let validation = self.validate(std::slice::from_ref(note)).await?;
        if let Some(v) = validation.first() {
            if !v.valid {
                result.status = SmartAddStatus::RejectedInvalid {
                    errors: v.errors.clone(),
                };
                return Ok(result);
            }
        }

        // Check for duplicates using the model's first field
        if options.check_duplicates {
            // Get the model's field names to find the canonical "first" field
            let model_fields = self.client.models().field_names(&note.model_name).await?;
            let first_field_name = model_fields.first().cloned();

            if let Some(field_name) = first_field_name {
                if let Some(field_value) = note.fields.get(&field_name) {
                    if !field_value.trim().is_empty() {
                        // Search for notes with same first field value in the same deck
                        let query = format!(
                            "deck:\"{}\" \"{}:{}\"",
                            note.deck_name,
                            field_name,
                            field_value.replace('\"', "\\\"")
                        );

                        let existing = self.client.notes().find(&query).await?;

                        if !existing.is_empty() {
                            result.similar_notes = existing.clone();

                            // Collect tags from similar notes for suggestions
                            if options.suggest_tags {
                                let note_infos = self.client.notes().info(&existing).await?;
                                let mut tag_counts: std::collections::HashMap<String, usize> =
                                    std::collections::HashMap::new();

                                for info in &note_infos {
                                    for tag in &info.tags {
                                        *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                                    }
                                }

                                // Sort by frequency and take top suggestions
                                let mut tags: Vec<_> = tag_counts.into_iter().collect();
                                tags.sort_by(|a, b| b.1.cmp(&a.1));
                                result.suggested_tags = tags
                                    .into_iter()
                                    .take(5)
                                    .map(|(tag, _)| tag)
                                    .filter(|t| !note.tags.contains(t))
                                    .collect();
                            }

                            if options.reject_on_duplicate {
                                result.status = SmartAddStatus::RejectedDuplicate {
                                    existing_id: existing[0],
                                };
                                return Ok(result);
                            }
                        }
                    }
                }
            }
        }

        // Suggest tags from similar content even if no exact duplicates
        if options.suggest_tags && result.suggested_tags.is_empty() {
            // Search for notes in the same deck with the same model
            let query = format!("deck:\"{}\" note:\"{}\"", note.deck_name, note.model_name);
            let similar = self.client.notes().find(&query).await?;

            if !similar.is_empty() {
                // Sample up to 50 notes for tag suggestions
                let sample: Vec<_> = similar.into_iter().take(50).collect();
                let note_infos = self.client.notes().info(&sample).await?;

                let mut tag_counts: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();

                for info in &note_infos {
                    for tag in &info.tags {
                        *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                    }
                }

                // Sort by frequency and take top suggestions
                let mut tags: Vec<_> = tag_counts.into_iter().collect();
                tags.sort_by(|a, b| b.1.cmp(&a.1));
                result.suggested_tags = tags
                    .into_iter()
                    .take(5)
                    .map(|(tag, _)| tag)
                    .filter(|t| !note.tags.contains(t))
                    .collect();
            }
        }

        // Add the note
        let mut note_to_add = note.clone();

        // If we found duplicates but aren't rejecting, allow the duplicate
        if !result.similar_notes.is_empty() && !options.reject_on_duplicate {
            let options = note_to_add.options.get_or_insert_with(Default::default);
            options.allow_duplicate = Some(true);
        }

        match self.client.notes().add(note_to_add).await {
            Ok(note_id) => {
                result.note_id = Some(note_id);
                if !result.similar_notes.is_empty() {
                    result.status = SmartAddStatus::AddedWithWarning {
                        warning: format!(
                            "Potential duplicate of {} existing note(s)",
                            result.similar_notes.len()
                        ),
                    };
                } else {
                    result.status = SmartAddStatus::Added;
                }
            }
            Err(e) => {
                result.status = SmartAddStatus::RejectedInvalid {
                    errors: vec![e.to_string()],
                };
            }
        }

        Ok(result)
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

/// Options for smart add operation.
#[derive(Debug, Clone)]
pub struct SmartAddOptions {
    /// Check for duplicate notes before adding.
    pub check_duplicates: bool,
    /// Suggest tags based on similar notes.
    pub suggest_tags: bool,
    /// Reject the note if a duplicate is found (otherwise add with warning).
    pub reject_on_duplicate: bool,
    /// Check for empty required fields.
    pub check_empty_fields: bool,
}

impl Default for SmartAddOptions {
    fn default() -> Self {
        Self {
            check_duplicates: true,
            suggest_tags: true,
            reject_on_duplicate: true,
            check_empty_fields: true,
        }
    }
}

/// Status of a smart add operation.
#[derive(Debug, Clone)]
pub enum SmartAddStatus {
    /// Note was successfully added.
    Added,
    /// Note was added despite being a potential duplicate.
    AddedWithWarning {
        /// Reason for the warning.
        warning: String,
    },
    /// Note was rejected as a duplicate.
    RejectedDuplicate {
        /// ID of the existing duplicate note.
        existing_id: i64,
    },
    /// Note was rejected due to empty required fields.
    RejectedEmptyFields {
        /// Names of empty fields.
        fields: Vec<String>,
    },
    /// Note was rejected due to validation errors.
    RejectedInvalid {
        /// Validation error messages.
        errors: Vec<String>,
    },
}

/// Result of a smart add operation.
#[derive(Debug, Clone)]
pub struct SmartAddResult {
    /// The note ID if successfully added, None if rejected.
    pub note_id: Option<i64>,
    /// Status of the operation.
    pub status: SmartAddStatus,
    /// Suggested tags based on similar notes.
    pub suggested_tags: Vec<String>,
    /// IDs of similar notes found (potential duplicates).
    pub similar_notes: Vec<i64>,
}
