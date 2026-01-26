//! Diff TOML definition vs Anki state.
//!
//! This module provides functionality to compare a TOML deck definition
//! against the live state in Anki, showing what's different.
//!
//! # Example
//!
//! ```no_run
//! use ankit_builder::DeckBuilder;
//!
//! # async fn example() -> ankit_builder::Result<()> {
//! let builder = DeckBuilder::from_file("deck.toml")?;
//! let diff = builder.diff_connect().await?;
//!
//! println!("Notes only in TOML: {}", diff.toml_only.len());
//! println!("Notes only in Anki: {}", diff.anki_only.len());
//! println!("Modified notes: {}", diff.modified.len());
//! println!("Unchanged notes: {}", diff.unchanged);
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use ankit::AnkiClient;
use serde::Serialize;

use crate::error::Result;
use crate::schema::{DeckDefinition, NoteDef};

/// Result of comparing a TOML definition against Anki state.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DeckDiff {
    /// Notes in TOML but not in Anki.
    pub toml_only: Vec<NoteDiff>,
    /// Notes in Anki but not in TOML.
    pub anki_only: Vec<NoteDiff>,
    /// Notes that exist in both but have differences.
    pub modified: Vec<ModifiedNote>,
    /// Number of notes that are identical.
    pub unchanged: usize,
}

/// A note that exists in only one place.
#[derive(Debug, Clone, Serialize)]
pub struct NoteDiff {
    /// The note ID (only set for Anki notes).
    pub note_id: Option<i64>,
    /// Model (note type) name.
    pub model: String,
    /// Deck name.
    pub deck: String,
    /// Value of the first field (used as identifier).
    pub first_field: String,
    /// Tags on the note.
    pub tags: Vec<String>,
}

/// A note that exists in both TOML and Anki but has differences.
#[derive(Debug, Clone, Serialize)]
pub struct ModifiedNote {
    /// The Anki note ID.
    pub note_id: i64,
    /// Value of the first field (used as identifier).
    pub first_field: String,
    /// Model name.
    pub model: String,
    /// Fields that differ between TOML and Anki.
    pub field_changes: Vec<FieldChange>,
    /// Tag changes between TOML and Anki.
    pub tag_changes: TagChanges,
}

/// A field that differs between TOML and Anki.
#[derive(Debug, Clone, Serialize)]
pub struct FieldChange {
    /// Field name.
    pub field: String,
    /// Value in TOML definition.
    pub toml_value: String,
    /// Value in Anki.
    pub anki_value: String,
}

/// Tag differences between TOML and Anki.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TagChanges {
    /// Tags in TOML but not in Anki.
    pub added: Vec<String>,
    /// Tags in Anki but not in TOML.
    pub removed: Vec<String>,
}

impl TagChanges {
    /// Check if there are any tag changes.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

/// Compares a TOML deck definition against Anki state.
pub struct DeckDiffer<'a> {
    client: &'a AnkiClient,
    definition: &'a DeckDefinition,
}

impl<'a> DeckDiffer<'a> {
    /// Create a new differ with the given client and definition.
    pub fn new(client: &'a AnkiClient, definition: &'a DeckDefinition) -> Self {
        Self { client, definition }
    }

    /// Compute the diff between TOML and Anki.
    ///
    /// Uses the first field value (normalized) as the key for matching
    /// notes between TOML and Anki.
    pub async fn diff(&self) -> Result<DeckDiff> {
        let mut result = DeckDiff::default();

        // Get all decks in the definition
        let deck_names: Vec<&str> = self
            .definition
            .decks
            .iter()
            .map(|d| d.name.as_str())
            .collect();

        // Build a map of TOML notes by (deck, model, first_field_normalized)
        let mut toml_notes: HashMap<NoteKey, &NoteDef> = HashMap::new();
        for note in &self.definition.notes {
            let model = self.definition.get_model(&note.model);
            if let Some(model) = model {
                if let Some(first_field_name) = model.fields.first() {
                    let first_field_value = note
                        .fields
                        .get(first_field_name)
                        .cloned()
                        .unwrap_or_default();
                    let key = NoteKey {
                        deck: note.deck.clone(),
                        model: note.model.clone(),
                        first_field: normalize_key(&first_field_value),
                    };
                    toml_notes.insert(key, note);
                }
            }
        }

        // Fetch notes from Anki for each deck
        let mut anki_notes: HashMap<NoteKey, AnkiNote> = HashMap::new();

        for deck_name in &deck_names {
            let query = format!("deck:\"{}\"", deck_name);
            let note_ids = self.client.notes().find(&query).await?;

            if note_ids.is_empty() {
                continue;
            }

            let note_infos = self.client.notes().info(&note_ids).await?;

            for note in note_infos {
                // Get first field value
                let first_field_value = get_first_field_value(&note.fields);
                let key = NoteKey {
                    deck: deck_name.to_string(),
                    model: note.model_name.clone(),
                    first_field: normalize_key(&first_field_value),
                };

                // Convert fields to simple HashMap
                let fields: HashMap<String, String> = note
                    .fields
                    .iter()
                    .map(|(name, field)| (name.clone(), field.value.clone()))
                    .collect();

                anki_notes.insert(
                    key,
                    AnkiNote {
                        note_id: note.note_id,
                        model_name: note.model_name,
                        fields,
                        tags: note.tags,
                        first_field_value,
                    },
                );
            }
        }

        // Compare TOML notes against Anki
        for (key, toml_note) in &toml_notes {
            if let Some(anki_note) = anki_notes.get(key) {
                // Note exists in both - check for modifications
                let (field_changes, tag_changes) = self.compare_note(toml_note, anki_note);

                if field_changes.is_empty() && tag_changes.is_empty() {
                    result.unchanged += 1;
                } else {
                    result.modified.push(ModifiedNote {
                        note_id: anki_note.note_id,
                        first_field: anki_note.first_field_value.clone(),
                        model: anki_note.model_name.clone(),
                        field_changes,
                        tag_changes,
                    });
                }
            } else {
                // Note only in TOML
                let model = self.definition.get_model(&toml_note.model);
                let first_field = if let Some(model) = model {
                    model
                        .fields
                        .first()
                        .and_then(|f| toml_note.fields.get(f))
                        .cloned()
                        .unwrap_or_default()
                } else {
                    String::new()
                };

                result.toml_only.push(NoteDiff {
                    note_id: None,
                    model: toml_note.model.clone(),
                    deck: toml_note.deck.clone(),
                    first_field,
                    tags: toml_note.tags.clone(),
                });
            }
        }

        // Find notes only in Anki
        for (key, anki_note) in &anki_notes {
            if !toml_notes.contains_key(key) {
                result.anki_only.push(NoteDiff {
                    note_id: Some(anki_note.note_id),
                    model: anki_note.model_name.clone(),
                    deck: key.deck.clone(),
                    first_field: anki_note.first_field_value.clone(),
                    tags: anki_note.tags.clone(),
                });
            }
        }

        Ok(result)
    }

    /// Compare a TOML note against an Anki note.
    fn compare_note(
        &self,
        toml_note: &NoteDef,
        anki_note: &AnkiNote,
    ) -> (Vec<FieldChange>, TagChanges) {
        let mut field_changes = Vec::new();

        // Compare fields
        let model = self.definition.get_model(&toml_note.model);
        if let Some(model) = model {
            for field_name in &model.fields {
                let toml_value = toml_note
                    .fields
                    .get(field_name)
                    .cloned()
                    .unwrap_or_default();
                let anki_value = anki_note
                    .fields
                    .get(field_name)
                    .cloned()
                    .unwrap_or_default();

                if toml_value != anki_value {
                    field_changes.push(FieldChange {
                        field: field_name.clone(),
                        toml_value,
                        anki_value,
                    });
                }
            }
        }

        // Compare tags
        let toml_tags: std::collections::HashSet<_> = toml_note.tags.iter().collect();
        let anki_tags: std::collections::HashSet<_> = anki_note.tags.iter().collect();

        let added: Vec<String> = toml_tags
            .difference(&anki_tags)
            .map(|s| (*s).clone())
            .collect();
        let removed: Vec<String> = anki_tags
            .difference(&toml_tags)
            .map(|s| (*s).clone())
            .collect();

        let tag_changes = TagChanges { added, removed };

        (field_changes, tag_changes)
    }
}

/// Key for identifying a note (deck + model + first field).
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct NoteKey {
    deck: String,
    model: String,
    first_field: String,
}

/// Temporary struct for Anki note data.
struct AnkiNote {
    note_id: i64,
    model_name: String,
    fields: HashMap<String, String>,
    tags: Vec<String>,
    first_field_value: String,
}

/// Normalize a key value for comparison.
///
/// - Trims whitespace
/// - Converts to lowercase
/// - Strips HTML tags
fn normalize_key(value: &str) -> String {
    let trimmed = value.trim();
    let lower = trimmed.to_lowercase();
    strip_html(&lower)
}

/// Strip HTML tags from a string.
fn strip_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;

    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
}

/// Get the first field value from a note's fields map.
fn get_first_field_value(fields: &HashMap<String, ankit::NoteField>) -> String {
    // Find the field with order 0
    fields
        .values()
        .find(|f| f.order == 0)
        .map(|f| f.value.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_key() {
        assert_eq!(normalize_key("  Hello World  "), "hello world");
        assert_eq!(normalize_key("<b>Bold</b>"), "bold");
        assert_eq!(normalize_key("  <i>Italic</i>  "), "italic");
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<b>Bold</b> text"), "Bold text");
        assert_eq!(strip_html("No tags"), "No tags");
    }

    #[test]
    fn test_tag_changes_is_empty() {
        let empty = TagChanges::default();
        assert!(empty.is_empty());

        let with_added = TagChanges {
            added: vec!["new".to_string()],
            removed: vec![],
        };
        assert!(!with_added.is_empty());

        let with_removed = TagChanges {
            added: vec![],
            removed: vec!["old".to_string()],
        };
        assert!(!with_removed.is_empty());
    }
}
