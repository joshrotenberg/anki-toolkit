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

    /// Create an enrichment pipeline for batch processing.
    ///
    /// The pipeline finds candidates and provides helpers for grouping,
    /// updating, and committing changes.
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
    ///     search: "deck:Japanese".to_string(),
    ///     empty_fields: vec!["Example".to_string(), "Pronunciation".to_string()],
    /// };
    ///
    /// let mut pipeline = engine.enrich().pipeline(&query).await?;
    ///
    /// // Process by missing field for efficient batching
    /// for (field, candidates) in pipeline.by_missing_field() {
    ///     println!("Field '{}' needs {} notes enriched", field, candidates.len());
    /// }
    ///
    /// // Buffer updates - collect IDs first to avoid borrow issues
    /// let note_ids: Vec<i64> = pipeline.candidates().iter().map(|c| c.note_id).collect();
    /// for note_id in note_ids {
    ///     pipeline.update(note_id, [
    ///         ("Example".to_string(), "Generated example".to_string())
    ///     ].into_iter().collect());
    /// }
    ///
    /// // Commit all updates
    /// let report = pipeline.commit(&engine).await?;
    /// println!("Updated {} notes", report.updated);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pipeline(&self, query: &EnrichQuery) -> Result<EnrichmentPipeline> {
        let candidates = self.find_candidates(query).await?;
        Ok(EnrichmentPipeline::new(candidates))
    }
}

/// A pipeline for batch enrichment operations.
///
/// Provides helpers for grouping candidates by missing field,
/// buffering updates, and committing them in a single operation.
#[derive(Debug, Clone)]
pub struct EnrichmentPipeline {
    candidates: Vec<EnrichCandidate>,
    updates: HashMap<i64, HashMap<String, String>>,
}

impl EnrichmentPipeline {
    /// Create a new pipeline with the given candidates.
    pub fn new(candidates: Vec<EnrichCandidate>) -> Self {
        Self {
            candidates,
            updates: HashMap::new(),
        }
    }

    /// Get the candidates for enrichment.
    pub fn candidates(&self) -> &[EnrichCandidate] {
        &self.candidates
    }

    /// Get the number of candidates.
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    /// Check if there are no candidates.
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// Get candidates grouped by which field they're missing.
    ///
    /// This is useful for batch processing where you want to generate
    /// content for all notes missing a specific field at once.
    ///
    /// # Returns
    ///
    /// A map from field name to the candidates that are missing that field.
    /// A candidate may appear in multiple groups if it's missing multiple fields.
    pub fn by_missing_field(&self) -> HashMap<String, Vec<&EnrichCandidate>> {
        let mut groups: HashMap<String, Vec<&EnrichCandidate>> = HashMap::new();

        for candidate in &self.candidates {
            for field in &candidate.empty_fields {
                groups.entry(field.clone()).or_default().push(candidate);
            }
        }

        groups
    }

    /// Get candidates grouped by model name.
    ///
    /// Useful when different models need different enrichment strategies.
    pub fn by_model(&self) -> HashMap<String, Vec<&EnrichCandidate>> {
        let mut groups: HashMap<String, Vec<&EnrichCandidate>> = HashMap::new();

        for candidate in &self.candidates {
            groups
                .entry(candidate.model_name.clone())
                .or_default()
                .push(candidate);
        }

        groups
    }

    /// Buffer an update for a note.
    ///
    /// Updates are not applied until `commit()` is called.
    /// Multiple updates to the same note will be merged.
    ///
    /// # Arguments
    ///
    /// * `note_id` - The note to update
    /// * `fields` - Field values to set
    pub fn update(&mut self, note_id: i64, fields: HashMap<String, String>) {
        self.updates.entry(note_id).or_default().extend(fields);
    }

    /// Get the number of buffered updates.
    pub fn pending_updates(&self) -> usize {
        self.updates.len()
    }

    /// Get candidates that haven't been updated yet.
    pub fn pending_candidates(&self) -> Vec<&EnrichCandidate> {
        self.candidates
            .iter()
            .filter(|c| !self.updates.contains_key(&c.note_id))
            .collect()
    }

    /// Commit all buffered updates.
    ///
    /// # Arguments
    ///
    /// * `engine` - The engine to use for committing
    ///
    /// # Returns
    ///
    /// A report with counts of updated, failed, and skipped notes.
    pub async fn commit(&self, engine: &crate::Engine) -> Result<EnrichPipelineReport> {
        // Count skipped (candidates without updates)
        let skipped = self
            .candidates
            .iter()
            .filter(|c| !self.updates.contains_key(&c.note_id))
            .count();

        let mut updated = 0;
        let mut failed = Vec::new();

        // Apply updates
        for (note_id, fields) in &self.updates {
            match engine.enrich().update_note(*note_id, fields).await {
                Ok(_) => updated += 1,
                Err(e) => {
                    failed.push((*note_id, e.to_string()));
                }
            }
        }

        Ok(EnrichPipelineReport {
            updated,
            failed,
            skipped,
        })
    }

    /// Commit all buffered updates and tag the updated notes.
    ///
    /// # Arguments
    ///
    /// * `engine` - The engine to use for committing
    /// * `tag` - Tag to add to successfully updated notes
    pub async fn commit_and_tag(
        &self,
        engine: &crate::Engine,
        tag: &str,
    ) -> Result<EnrichPipelineReport> {
        let report = self.commit(engine).await?;

        // Tag successfully updated notes
        if report.updated > 0 {
            let updated_ids: Vec<i64> = self
                .updates
                .keys()
                .filter(|id| !report.failed.iter().any(|(fid, _)| fid == *id))
                .copied()
                .collect();

            if !updated_ids.is_empty() {
                engine.enrich().tag_enriched(&updated_ids, tag).await?;
            }
        }

        Ok(report)
    }
}

/// Report from an enrichment pipeline commit.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EnrichPipelineReport {
    /// Number of notes successfully updated.
    pub updated: usize,
    /// Notes that failed to update (note_id, error message).
    pub failed: Vec<(i64, String)>,
    /// Number of candidates that were not updated (no update buffered).
    pub skipped: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enrich_query_construction() {
        let query = EnrichQuery {
            search: "deck:Test".to_string(),
            empty_fields: vec!["Example".to_string(), "Audio".to_string()],
        };

        assert_eq!(query.search, "deck:Test");
        assert_eq!(query.empty_fields.len(), 2);
        assert!(query.empty_fields.contains(&"Example".to_string()));
    }

    #[test]
    fn test_enrich_candidate_construction() {
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "Hello".to_string());
        fields.insert("Back".to_string(), "World".to_string());

        let candidate = EnrichCandidate {
            note_id: 12345,
            model_name: "Basic".to_string(),
            fields,
            empty_fields: vec!["Example".to_string()],
            tags: vec!["tag1".to_string()],
        };

        assert_eq!(candidate.note_id, 12345);
        assert_eq!(candidate.model_name, "Basic");
        assert_eq!(candidate.fields.len(), 2);
        assert_eq!(candidate.empty_fields.len(), 1);
        assert_eq!(candidate.tags.len(), 1);
    }

    #[test]
    fn test_enrich_candidate_serialization() {
        let candidate = EnrichCandidate {
            note_id: 100,
            model_name: "Vocab".to_string(),
            fields: HashMap::new(),
            empty_fields: vec!["Definition".to_string()],
            tags: vec![],
        };

        let json = serde_json::to_string(&candidate).unwrap();
        assert!(json.contains("\"note_id\":100"));
        assert!(json.contains("\"model_name\":\"Vocab\""));
    }

    #[test]
    fn test_enrich_report_default() {
        let report = EnrichReport::default();
        assert_eq!(report.updated, 0);
        assert_eq!(report.failed, 0);
        assert!(report.failures.is_empty());
    }

    #[test]
    fn test_enrich_report_construction() {
        let failure = EnrichFailure {
            note_id: 999,
            error: "Not found".to_string(),
        };

        let report = EnrichReport {
            updated: 5,
            failed: 1,
            failures: vec![failure],
        };

        assert_eq!(report.updated, 5);
        assert_eq!(report.failed, 1);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].note_id, 999);
    }

    #[test]
    fn test_enrich_failure_construction() {
        let failure = EnrichFailure {
            note_id: 12345,
            error: "Field not found".to_string(),
        };

        assert_eq!(failure.note_id, 12345);
        assert_eq!(failure.error, "Field not found");
    }

    #[test]
    fn test_enrich_failure_serialization() {
        let failure = EnrichFailure {
            note_id: 456,
            error: "Connection error".to_string(),
        };

        let json = serde_json::to_string(&failure).unwrap();
        assert!(json.contains("\"note_id\":456"));
        assert!(json.contains("\"error\":\"Connection error\""));
    }

    #[test]
    fn test_enrichment_pipeline_new_empty() {
        let pipeline = EnrichmentPipeline::new(vec![]);
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.len(), 0);
        assert_eq!(pipeline.pending_updates(), 0);
    }

    #[test]
    fn test_enrichment_pipeline_with_candidates() {
        let candidate = EnrichCandidate {
            note_id: 1,
            model_name: "Basic".to_string(),
            fields: HashMap::new(),
            empty_fields: vec!["Back".to_string()],
            tags: vec![],
        };

        let pipeline = EnrichmentPipeline::new(vec![candidate]);
        assert!(!pipeline.is_empty());
        assert_eq!(pipeline.len(), 1);
        assert_eq!(pipeline.candidates().len(), 1);
    }

    #[test]
    fn test_enrichment_pipeline_update() {
        let candidate = EnrichCandidate {
            note_id: 100,
            model_name: "Basic".to_string(),
            fields: HashMap::new(),
            empty_fields: vec!["Back".to_string()],
            tags: vec![],
        };

        let mut pipeline = EnrichmentPipeline::new(vec![candidate]);
        assert_eq!(pipeline.pending_updates(), 0);

        let mut fields = HashMap::new();
        fields.insert("Back".to_string(), "Answer".to_string());
        pipeline.update(100, fields);

        assert_eq!(pipeline.pending_updates(), 1);
    }

    #[test]
    fn test_enrichment_pipeline_update_merge() {
        let mut pipeline = EnrichmentPipeline::new(vec![]);

        let mut fields1 = HashMap::new();
        fields1.insert("Field1".to_string(), "Value1".to_string());
        pipeline.update(100, fields1);

        let mut fields2 = HashMap::new();
        fields2.insert("Field2".to_string(), "Value2".to_string());
        pipeline.update(100, fields2);

        // Should still be 1 update (merged)
        assert_eq!(pipeline.pending_updates(), 1);
    }

    #[test]
    fn test_enrichment_pipeline_pending_candidates() {
        let candidates = vec![
            EnrichCandidate {
                note_id: 1,
                model_name: "Basic".to_string(),
                fields: HashMap::new(),
                empty_fields: vec!["Back".to_string()],
                tags: vec![],
            },
            EnrichCandidate {
                note_id: 2,
                model_name: "Basic".to_string(),
                fields: HashMap::new(),
                empty_fields: vec!["Back".to_string()],
                tags: vec![],
            },
        ];

        let mut pipeline = EnrichmentPipeline::new(candidates);
        assert_eq!(pipeline.pending_candidates().len(), 2);

        let mut fields = HashMap::new();
        fields.insert("Back".to_string(), "Answer".to_string());
        pipeline.update(1, fields);

        assert_eq!(pipeline.pending_candidates().len(), 1);
        assert_eq!(pipeline.pending_candidates()[0].note_id, 2);
    }

    #[test]
    fn test_enrichment_pipeline_by_missing_field() {
        let candidates = vec![
            EnrichCandidate {
                note_id: 1,
                model_name: "Basic".to_string(),
                fields: HashMap::new(),
                empty_fields: vec!["Field1".to_string()],
                tags: vec![],
            },
            EnrichCandidate {
                note_id: 2,
                model_name: "Basic".to_string(),
                fields: HashMap::new(),
                empty_fields: vec!["Field1".to_string(), "Field2".to_string()],
                tags: vec![],
            },
        ];

        let pipeline = EnrichmentPipeline::new(candidates);
        let by_field = pipeline.by_missing_field();

        assert_eq!(by_field.get("Field1").unwrap().len(), 2);
        assert_eq!(by_field.get("Field2").unwrap().len(), 1);
    }

    #[test]
    fn test_enrichment_pipeline_by_model() {
        let candidates = vec![
            EnrichCandidate {
                note_id: 1,
                model_name: "Basic".to_string(),
                fields: HashMap::new(),
                empty_fields: vec![],
                tags: vec![],
            },
            EnrichCandidate {
                note_id: 2,
                model_name: "Cloze".to_string(),
                fields: HashMap::new(),
                empty_fields: vec![],
                tags: vec![],
            },
            EnrichCandidate {
                note_id: 3,
                model_name: "Basic".to_string(),
                fields: HashMap::new(),
                empty_fields: vec![],
                tags: vec![],
            },
        ];

        let pipeline = EnrichmentPipeline::new(candidates);
        let by_model = pipeline.by_model();

        assert_eq!(by_model.get("Basic").unwrap().len(), 2);
        assert_eq!(by_model.get("Cloze").unwrap().len(), 1);
    }

    #[test]
    fn test_enrich_pipeline_report_default() {
        let report = EnrichPipelineReport::default();
        assert_eq!(report.updated, 0);
        assert!(report.failed.is_empty());
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn test_enrich_pipeline_report_construction() {
        let report = EnrichPipelineReport {
            updated: 10,
            failed: vec![(100, "Error".to_string())],
            skipped: 5,
        };

        assert_eq!(report.updated, 10);
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.skipped, 5);
    }

    #[test]
    fn test_enrich_pipeline_report_serialization() {
        let report = EnrichPipelineReport {
            updated: 3,
            failed: vec![],
            skipped: 2,
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"updated\":3"));
        assert!(json.contains("\"skipped\":2"));
    }
}
