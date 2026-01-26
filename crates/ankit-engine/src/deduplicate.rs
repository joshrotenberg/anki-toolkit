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

    #[test]
    fn test_normalize_key_empty() {
        assert_eq!(normalize_key(""), "");
        assert_eq!(normalize_key("   "), "");
        assert_eq!(normalize_key("<>"), "");
    }

    #[test]
    fn test_normalize_key_html_attributes() {
        assert_eq!(normalize_key("<a href=\"url\">Link</a>"), "link");
        assert_eq!(
            normalize_key("<div class=\"foo\" id=\"bar\">Content</div>"),
            "content"
        );
    }

    #[test]
    fn test_normalize_key_unclosed_tags() {
        assert_eq!(normalize_key("<p>Unclosed"), "unclosed");
        assert_eq!(normalize_key("Text<br>More"), "textmore");
    }

    #[test]
    fn test_normalize_key_newlines() {
        assert_eq!(normalize_key("hello\nworld"), "hello world");
        assert_eq!(normalize_key("hello\r\nworld"), "hello world");
        assert_eq!(normalize_key("hello\tworld"), "hello world");
    }

    #[test]
    fn test_keep_strategy_default() {
        let strategy = KeepStrategy::default();
        assert!(matches!(strategy, KeepStrategy::First));
    }

    #[test]
    fn test_dedupe_query_construction() {
        let query = DedupeQuery {
            search: "deck:Test".to_string(),
            key_field: "Front".to_string(),
            keep: KeepStrategy::MostContent,
        };

        assert_eq!(query.search, "deck:Test");
        assert_eq!(query.key_field, "Front");
        assert!(matches!(query.keep, KeepStrategy::MostContent));
    }

    #[test]
    fn test_duplicate_group_construction() {
        let group = DuplicateGroup {
            key_value: "hello".to_string(),
            keep_note_id: 1000,
            duplicate_note_ids: vec![1001, 1002, 1003],
        };

        assert_eq!(group.key_value, "hello");
        assert_eq!(group.keep_note_id, 1000);
        assert_eq!(group.duplicate_note_ids.len(), 3);
        assert!(group.duplicate_note_ids.contains(&1001));
    }

    #[test]
    fn test_duplicate_group_serialization() {
        let group = DuplicateGroup {
            key_value: "test".to_string(),
            keep_note_id: 123,
            duplicate_note_ids: vec![456, 789],
        };

        let json = serde_json::to_string(&group).unwrap();
        assert!(json.contains("\"key_value\":\"test\""));
        assert!(json.contains("\"keep_note_id\":123"));
        assert!(json.contains("\"duplicate_note_ids\":[456,789]"));
    }

    #[test]
    fn test_dedupe_report_default() {
        let report = DedupeReport::default();
        assert_eq!(report.groups_found, 0);
        assert_eq!(report.deleted, 0);
        assert_eq!(report.kept, 0);
        assert!(report.details.is_empty());
    }

    #[test]
    fn test_dedupe_report_construction() {
        let group = DuplicateGroup {
            key_value: "word".to_string(),
            keep_note_id: 100,
            duplicate_note_ids: vec![101, 102],
        };

        let report = DedupeReport {
            groups_found: 1,
            deleted: 2,
            kept: 1,
            details: vec![group],
        };

        assert_eq!(report.groups_found, 1);
        assert_eq!(report.deleted, 2);
        assert_eq!(report.kept, 1);
        assert_eq!(report.details.len(), 1);
    }

    #[test]
    fn test_dedupe_report_serialization() {
        let report = DedupeReport {
            groups_found: 2,
            deleted: 5,
            kept: 2,
            details: vec![],
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"groups_found\":2"));
        assert!(json.contains("\"deleted\":5"));
        assert!(json.contains("\"kept\":2"));
    }

    #[test]
    fn test_note_for_dedupe_construction() {
        let note = NoteForDedupe {
            note_id: 12345,
            non_empty_count: 3,
            tag_count: 2,
        };

        assert_eq!(note.note_id, 12345);
        assert_eq!(note.non_empty_count, 3);
        assert_eq!(note.tag_count, 2);
    }

    #[test]
    fn test_keep_strategy_variants() {
        // Verify all variants can be constructed
        let _first = KeepStrategy::First;
        let _last = KeepStrategy::Last;
        let _most_content = KeepStrategy::MostContent;
        let _most_tags = KeepStrategy::MostTags;
    }
}
