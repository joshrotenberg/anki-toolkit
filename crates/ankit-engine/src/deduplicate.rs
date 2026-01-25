//! Duplicate note detection and removal.
//!
//! This module provides workflows for finding and removing duplicate notes
//! based on a key field.
//!
//! # Example
//!
//! ```no_run
//! use ankit_engine::Engine;
//! use ankit_engine::deduplicate::{DedupeQuery, KeepStrategy};
//!
//! # async fn example() -> ankit_engine::Result<()> {
//! let engine = Engine::new();
//!
//! // Find duplicates based on the "Front" field
//! let query = DedupeQuery {
//!     search: "deck:Japanese".to_string(),
//!     key_field: "Front".to_string(),
//!     keep: KeepStrategy::First,
//! };
//!
//! let groups = engine.deduplicate().find_duplicates(&query).await?;
//! println!("Found {} duplicate groups", groups.len());
//!
//! // Remove duplicates (keeps the first, deletes the rest)
//! let report = engine.deduplicate().remove_duplicates(&query).await?;
//! println!("Deleted {} duplicate notes", report.deleted);
//! # Ok(())
//! # }
//! ```

use crate::Result;
use ankit::AnkiClient;
use serde::Serialize;
use std::collections::HashMap;

/// Strategy for which duplicate to keep.
#[derive(Debug, Clone, Copy, Default)]
pub enum KeepStrategy {
    /// Keep the first note found (by note ID, oldest).
    #[default]
    First,
    /// Keep the last note found (by note ID, newest).
    Last,
    /// Keep the note with the most non-empty fields.
    MostContent,
    /// Keep the note with the most tags.
    MostTags,
}

/// Query parameters for finding duplicates.
#[derive(Debug, Clone)]
pub struct DedupeQuery {
    /// Anki search query to filter notes.
    pub search: String,
    /// Field name to use as the duplicate key.
    pub key_field: String,
    /// Strategy for which duplicate to keep.
    pub keep: KeepStrategy,
}

/// A group of duplicate notes.
#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    /// The key value that these notes share.
    pub key_value: String,
    /// The note ID that will be kept.
    pub keep_note_id: i64,
    /// The note IDs that are duplicates (to be deleted).
    pub duplicate_note_ids: Vec<i64>,
}

/// Information about a note for duplicate comparison.
#[derive(Debug, Clone)]
struct NoteForDedupe {
    note_id: i64,
    non_empty_count: usize,
    tag_count: usize,
}

/// Report from a deduplication operation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DedupeReport {
    /// Number of duplicate groups found.
    pub groups_found: usize,
    /// Number of notes deleted.
    pub deleted: usize,
    /// Number of notes kept (one per group).
    pub kept: usize,
    /// Details about deleted notes per key.
    pub details: Vec<DuplicateGroup>,
}

/// Deduplication workflow engine.
#[derive(Debug)]
pub struct DeduplicateEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> DeduplicateEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Find groups of duplicate notes.
    ///
    /// Notes are considered duplicates if they have the same value in the key field.
    /// Returns groups where each group has the note to keep and notes to delete.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters specifying search filter, key field, and keep strategy
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::deduplicate::{DedupeQuery, KeepStrategy};
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let query = DedupeQuery {
    ///     search: "deck:Vocabulary".to_string(),
    ///     key_field: "Word".to_string(),
    ///     keep: KeepStrategy::MostContent,
    /// };
    ///
    /// let groups = engine.deduplicate().find_duplicates(&query).await?;
    /// for group in &groups {
    ///     println!("'{}': keep {}, delete {:?}",
    ///         group.key_value, group.keep_note_id, group.duplicate_note_ids);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_duplicates(&self, query: &DedupeQuery) -> Result<Vec<DuplicateGroup>> {
        let note_ids = self.client.notes().find(&query.search).await?;

        if note_ids.is_empty() {
            return Ok(Vec::new());
        }

        let note_infos = self.client.notes().info(&note_ids).await?;

        // Group notes by key field value
        let mut groups: HashMap<String, Vec<NoteForDedupe>> = HashMap::new();

        for info in note_infos {
            // Get the key field value
            let key_value = info
                .fields
                .get(&query.key_field)
                .map(|f| normalize_key(&f.value))
                .unwrap_or_default();

            // Skip notes with empty key
            if key_value.is_empty() {
                continue;
            }

            // Count non-empty fields
            let non_empty_count = info
                .fields
                .values()
                .filter(|f| !f.value.trim().is_empty())
                .count();

            groups.entry(key_value).or_default().push(NoteForDedupe {
                note_id: info.note_id,
                non_empty_count,
                tag_count: info.tags.len(),
            });
        }

        // Convert to DuplicateGroups (only groups with more than one note)
        let mut result = Vec::new();

        for (key, mut notes) in groups {
            if notes.len() <= 1 {
                continue;
            }

            // Sort notes based on keep strategy
            match query.keep {
                KeepStrategy::First => {
                    notes.sort_by_key(|n| n.note_id);
                }
                KeepStrategy::Last => {
                    notes.sort_by_key(|n| std::cmp::Reverse(n.note_id));
                }
                KeepStrategy::MostContent => {
                    // Sort by non-empty count descending, then by note_id ascending for ties
                    notes.sort_by(|a, b| {
                        b.non_empty_count
                            .cmp(&a.non_empty_count)
                            .then_with(|| a.note_id.cmp(&b.note_id))
                    });
                }
                KeepStrategy::MostTags => {
                    // Sort by tag count descending, then by note_id ascending for ties
                    notes.sort_by(|a, b| {
                        b.tag_count
                            .cmp(&a.tag_count)
                            .then_with(|| a.note_id.cmp(&b.note_id))
                    });
                }
            }

            let keep_note_id = notes[0].note_id;
            let duplicate_note_ids: Vec<i64> = notes[1..].iter().map(|n| n.note_id).collect();

            result.push(DuplicateGroup {
                key_value: key,
                keep_note_id,
                duplicate_note_ids,
            });
        }

        // Sort by key for consistent output
        result.sort_by(|a, b| a.key_value.cmp(&b.key_value));

        Ok(result)
    }

    /// Preview deduplication without making changes.
    ///
    /// Returns the same information as `find_duplicates` but formatted as a report.
    pub async fn preview(&self, query: &DedupeQuery) -> Result<DedupeReport> {
        let groups = self.find_duplicates(query).await?;

        let deleted: usize = groups.iter().map(|g| g.duplicate_note_ids.len()).sum();

        Ok(DedupeReport {
            groups_found: groups.len(),
            deleted,
            kept: groups.len(),
            details: groups,
        })
    }

    /// Remove duplicate notes.
    ///
    /// Keeps one note per duplicate group (based on keep strategy) and deletes the rest.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters specifying search filter, key field, and keep strategy
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::deduplicate::{DedupeQuery, KeepStrategy};
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let query = DedupeQuery {
    ///     search: "deck:Vocabulary tag:imported".to_string(),
    ///     key_field: "Word".to_string(),
    ///     keep: KeepStrategy::MostContent,
    /// };
    ///
    /// let report = engine.deduplicate().remove_duplicates(&query).await?;
    /// println!("Deleted {} duplicates, kept {}", report.deleted, report.kept);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_duplicates(&self, query: &DedupeQuery) -> Result<DedupeReport> {
        let groups = self.find_duplicates(query).await?;

        if groups.is_empty() {
            return Ok(DedupeReport::default());
        }

        // Collect all note IDs to delete
        let to_delete: Vec<i64> = groups
            .iter()
            .flat_map(|g| g.duplicate_note_ids.iter().copied())
            .collect();

        let deleted_count = to_delete.len();
        let kept_count = groups.len();

        // Delete the duplicates
        if !to_delete.is_empty() {
            self.client.notes().delete(&to_delete).await?;
        }

        Ok(DedupeReport {
            groups_found: groups.len(),
            deleted: deleted_count,
            kept: kept_count,
            details: groups,
        })
    }

    /// Delete specific duplicate notes.
    ///
    /// Use this after reviewing the results from `find_duplicates` to selectively
    /// delete duplicates.
    ///
    /// # Arguments
    ///
    /// * `note_ids` - Note IDs to delete
    pub async fn delete_notes(&self, note_ids: &[i64]) -> Result<usize> {
        if note_ids.is_empty() {
            return Ok(0);
        }

        self.client.notes().delete(note_ids).await?;
        Ok(note_ids.len())
    }
}

/// Normalize a key value for comparison.
///
/// Strips HTML, collapses whitespace, and converts to lowercase.
fn normalize_key(value: &str) -> String {
    // Simple HTML stripping (remove tags)
    let mut result = String::with_capacity(value.len());
    let mut in_tag = false;

    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Collapse whitespace and trim
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_key() {
        assert_eq!(normalize_key("hello"), "hello");
        assert_eq!(normalize_key("Hello World"), "hello world");
        assert_eq!(normalize_key("  hello   world  "), "hello world");
        assert_eq!(normalize_key("<b>hello</b>"), "hello");
        assert_eq!(
            normalize_key("<div>Hello <span>World</span></div>"),
            "hello world"
        );
    }
}
