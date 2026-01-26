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
    fn test_normalize_key_nested_html() {
        assert_eq!(normalize_key("<div><p>Nested</p></div>"), "nested");
        assert_eq!(normalize_key("<a href='url'>Link</a>"), "link");
    }

    #[test]
    fn test_normalize_key_empty() {
        assert_eq!(normalize_key(""), "");
        assert_eq!(normalize_key("   "), "");
        assert_eq!(normalize_key("<>"), "");
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<b>Bold</b> text"), "Bold text");
        assert_eq!(strip_html("No tags"), "No tags");
    }

    #[test]
    fn test_strip_html_unclosed_tags() {
        assert_eq!(strip_html("<p>Unclosed"), "Unclosed");
        assert_eq!(strip_html("Text<br>More"), "TextMore");
    }

    #[test]
    fn test_strip_html_attributes() {
        assert_eq!(strip_html("<a href=\"url\">Link</a>"), "Link");
        assert_eq!(
            strip_html("<div class=\"foo\" id=\"bar\">Content</div>"),
            "Content"
        );
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

    #[test]
    fn test_tag_changes_both() {
        let both = TagChanges {
            added: vec!["new".to_string()],
            removed: vec!["old".to_string()],
        };
        assert!(!both.is_empty());
    }

    #[test]
    fn test_deck_diff_default() {
        let diff = DeckDiff::default();
        assert!(diff.toml_only.is_empty());
        assert!(diff.anki_only.is_empty());
        assert!(diff.modified.is_empty());
        assert_eq!(diff.unchanged, 0);
    }

    #[test]
    fn test_note_diff_construction() {
        let note = NoteDiff {
            note_id: Some(12345),
            model: "Basic".to_string(),
            deck: "Test".to_string(),
            first_field: "Question".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        assert_eq!(note.note_id, Some(12345));
        assert_eq!(note.model, "Basic");
        assert_eq!(note.deck, "Test");
        assert_eq!(note.first_field, "Question");
        assert_eq!(note.tags.len(), 2);
    }

    #[test]
    fn test_field_change_construction() {
        let change = FieldChange {
            field: "Back".to_string(),
            toml_value: "TOML answer".to_string(),
            anki_value: "Anki answer".to_string(),
        };

        assert_eq!(change.field, "Back");
        assert_eq!(change.toml_value, "TOML answer");
        assert_eq!(change.anki_value, "Anki answer");
    }

    #[test]
    fn test_modified_note_construction() {
        let modified = ModifiedNote {
            note_id: 67890,
            first_field: "Word".to_string(),
            model: "Vocabulary".to_string(),
            field_changes: vec![FieldChange {
                field: "Definition".to_string(),
                toml_value: "new def".to_string(),
                anki_value: "old def".to_string(),
            }],
            tag_changes: TagChanges {
                added: vec!["updated".to_string()],
                removed: vec![],
            },
        };

        assert_eq!(modified.note_id, 67890);
        assert_eq!(modified.first_field, "Word");
        assert_eq!(modified.model, "Vocabulary");
        assert_eq!(modified.field_changes.len(), 1);
        assert!(!modified.tag_changes.is_empty());
    }

    #[test]
    fn test_note_key_equality() {
        let key1 = NoteKey {
            deck: "Test".to_string(),
            model: "Basic".to_string(),
            first_field: "hello".to_string(),
        };

        let key2 = NoteKey {
            deck: "Test".to_string(),
            model: "Basic".to_string(),
            first_field: "hello".to_string(),
        };

        let key3 = NoteKey {
            deck: "Test".to_string(),
            model: "Basic".to_string(),
            first_field: "world".to_string(),
        };

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_note_key_hash() {
        use std::collections::HashSet;

        let key1 = NoteKey {
            deck: "Test".to_string(),
            model: "Basic".to_string(),
            first_field: "hello".to_string(),
        };

        let key2 = NoteKey {
            deck: "Test".to_string(),
            model: "Basic".to_string(),
            first_field: "hello".to_string(),
        };

        let mut set = HashSet::new();
        set.insert(key1);
        assert!(set.contains(&key2));
    }
}
