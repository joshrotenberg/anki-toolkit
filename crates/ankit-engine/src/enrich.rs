//! Note enrichment operations.
//!
//! This module provides workflows for finding notes with empty fields
//! and updating them with new content.
//!
//! # Example
//!
//! ```no_run
//! use ankit_engine::Engine;
//! use ankit_engine::enrich::EnrichQuery;
//!
//! # async fn example() -> ankit_engine::Result<()> {
//! let engine = Engine::new();
//!
//! // Find notes missing the "Example" field
//! let query = EnrichQuery {
//!     search: "deck:Japanese".to_string(),
//!     empty_fields: vec!["Example".to_string()],
//! };
//!
//! let candidates = engine.enrich().find_candidates(&query).await?;
//! println!("Found {} notes needing enrichment", candidates.len());
//!
//! // Update a note with enriched content
//! use std::collections::HashMap;
//! let mut updates = HashMap::new();
//! updates.insert("Example".to_string(), "New example sentence".to_string());
//! engine.enrich().update_note(candidates[0].note_id, &updates).await?;
//! # Ok(())
//! # }
//! ```

use crate::Result;
use ankit::AnkiClient;
use serde::Serialize;
use std::collections::HashMap;

/// Query parameters for finding notes to enrich.
#[derive(Debug, Clone)]
pub struct EnrichQuery {
    /// Anki search query to filter notes.
    pub search: String,
    /// Field names that should be empty (any of these being empty qualifies the note).
    pub empty_fields: Vec<String>,
}

/// A note that is a candidate for enrichment.
#[derive(Debug, Clone, Serialize)]
pub struct EnrichCandidate {
    /// The note ID.
    pub note_id: i64,
    /// The model (note type) name.
    pub model_name: String,
    /// Current field values.
    pub fields: HashMap<String, String>,
    /// Fields that are empty and need enrichment.
    pub empty_fields: Vec<String>,
    /// Current tags on the note.
    pub tags: Vec<String>,
}

/// Report from a batch enrichment operation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EnrichReport {
    /// Number of notes successfully updated.
    pub updated: usize,
    /// Number of notes that failed to update.
    pub failed: usize,
    /// Details about failed updates.
    pub failures: Vec<EnrichFailure>,
}

/// Details about a failed enrichment.
#[derive(Debug, Clone, Serialize)]
pub struct EnrichFailure {
    /// The note ID that failed.
    pub note_id: i64,
    /// The error message.
    pub error: String,
}

/// Enrichment workflow engine.
#[derive(Debug)]
pub struct EnrichEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> EnrichEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Find notes that have empty fields matching the query criteria.
    ///
    /// Returns a list of candidates with information about which fields need enrichment.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters specifying search filter and fields to check
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::enrich::EnrichQuery;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let query = EnrichQuery {
    ///     search: "deck:\"My Deck\" note:Basic".to_string(),
    ///     empty_fields: vec!["Example".to_string(), "Pronunciation".to_string()],
    /// };
    ///
    /// let candidates = engine.enrich().find_candidates(&query).await?;
    /// for candidate in &candidates {
    ///     println!("Note {} needs: {:?}", candidate.note_id, candidate.empty_fields);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_candidates(&self, query: &EnrichQuery) -> Result<Vec<EnrichCandidate>> {
        let note_ids = self.client.notes().find(&query.search).await?;

        if note_ids.is_empty() {
            return Ok(Vec::new());
        }

        let note_infos = self.client.notes().info(&note_ids).await?;
        let mut candidates = Vec::new();

        for info in note_infos {
            // Check which specified fields are empty
            let empty: Vec<String> = query
                .empty_fields
                .iter()
                .filter(|field_name| {
                    info.fields
                        .get(*field_name)
                        .map(|f| f.value.trim().is_empty())
                        .unwrap_or(true) // Field doesn't exist = empty
                })
                .cloned()
                .collect();

            if !empty.is_empty() {
                // Convert fields to simple HashMap
                let fields: HashMap<String, String> =
                    info.fields.into_iter().map(|(k, v)| (k, v.value)).collect();

                candidates.push(EnrichCandidate {
                    note_id: info.note_id,
                    model_name: info.model_name,
                    fields,
                    empty_fields: empty,
                    tags: info.tags,
                });
            }
        }

        Ok(candidates)
    }

    /// Update a single note with new field values.
    ///
    /// # Arguments
    ///
    /// * `note_id` - The note to update
    /// * `fields` - Map of field name to new value
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use std::collections::HashMap;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("Example".to_string(), "This is an example sentence.".to_string());
    ///
    /// engine.enrich().update_note(12345, &fields).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_note(&self, note_id: i64, fields: &HashMap<String, String>) -> Result<()> {
        self.client.notes().update_fields(note_id, fields).await?;
        Ok(())
    }

    /// Update multiple notes with new field values.
    ///
    /// # Arguments
    ///
    /// * `updates` - List of (note_id, fields) pairs to update
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use std::collections::HashMap;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let updates: Vec<(i64, HashMap<String, String>)> = vec![
    ///     (12345, [("Example".to_string(), "Example 1".to_string())].into_iter().collect()),
    ///     (12346, [("Example".to_string(), "Example 2".to_string())].into_iter().collect()),
    /// ];
    ///
    /// let report = engine.enrich().update_notes(&updates).await?;
    /// println!("Updated: {}, Failed: {}", report.updated, report.failed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_notes(
        &self,
        updates: &[(i64, HashMap<String, String>)],
    ) -> Result<EnrichReport> {
        let mut report = EnrichReport::default();

        for (note_id, fields) in updates {
            match self.client.notes().update_fields(*note_id, fields).await {
                Ok(_) => report.updated += 1,
                Err(e) => {
                    report.failed += 1;
                    report.failures.push(EnrichFailure {
                        note_id: *note_id,
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(report)
    }

    /// Add a tag to notes after enrichment.
    ///
    /// Useful for marking notes as processed.
    ///
    /// # Arguments
    ///
    /// * `note_ids` - Notes to tag
    /// * `tag` - Tag to add
    pub async fn tag_enriched(&self, note_ids: &[i64], tag: &str) -> Result<()> {
        if !note_ids.is_empty() {
            self.client.notes().add_tags(note_ids, tag).await?;
        }
        Ok(())
    }
}
