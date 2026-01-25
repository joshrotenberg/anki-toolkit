//! TOML schema types for deck definitions.
//!
//! # Example TOML
//!
//! ```toml
//! [package]
//! name = "Spanish Vocabulary"
//! version = "1.0.0"
//! author = "Your Name"
//!
//! [[models]]
//! name = "Basic Spanish"
//! fields = ["Spanish", "English", "Example"]
//! # Optional: specify which field to sort by (default: first field)
//! sort_field = "Spanish"
//!
//! [[models.templates]]
//! name = "Spanish -> English"
//! front = "{{Spanish}}"
//! back = "{{FrontSide}}<hr>{{English}}<br><i>{{Example}}</i>"
//!
//! [[models.templates]]
//! name = "English -> Spanish"
//! front = "{{English}}"
//! back = "{{FrontSide}}<hr>{{Spanish}}"
//!
//! [[decks]]
//! name = "Spanish::Vocabulary"
//! description = "Core Spanish vocabulary"
//!
//! [[notes]]
//! deck = "Spanish::Vocabulary"
//! model = "Basic Spanish"
//! tags = ["chapter1", "nouns"]
//!
//! [notes.fields]
//! Spanish = "el gato"
//! English = "the cat"
//! Example = "El gato es negro."
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};

/// Root structure for a deck definition file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckDefinition {
    /// Package metadata.
    pub package: PackageInfo,

    /// Model (note type) definitions.
    #[serde(default)]
    pub models: Vec<ModelDef>,

    /// Deck definitions.
    #[serde(default)]
    pub decks: Vec<DeckDef>,

    /// Note definitions.
    #[serde(default)]
    pub notes: Vec<NoteDef>,

    /// Media file definitions.
    #[serde(default)]
    pub media: Vec<MediaDef>,
}

impl DeckDefinition {
    /// Load a deck definition from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse a deck definition from a TOML string.
    pub fn parse(content: &str) -> Result<Self> {
        let def: DeckDefinition = toml::from_str(content)?;
        def.validate()?;
        Ok(def)
    }

    /// Validate the deck definition for consistency.
    pub fn validate(&self) -> Result<()> {
        // Check that all notes reference valid models
        let model_names: std::collections::HashSet<_> =
            self.models.iter().map(|m| m.name.as_str()).collect();

        for note in &self.notes {
            if !model_names.contains(note.model.as_str()) {
                return Err(Error::ModelNotFound(note.model.clone()));
            }

            // Check that note fields match model fields
            let model = self.models.iter().find(|m| m.name == note.model).unwrap();
            for field_name in note.fields.keys() {
                if !model.fields.contains(field_name) {
                    return Err(Error::FieldNotFound {
                        model: note.model.clone(),
                        field: field_name.clone(),
                    });
                }
            }
        }

        // Check that all notes reference valid decks
        let deck_names: std::collections::HashSet<_> =
            self.decks.iter().map(|d| d.name.as_str()).collect();

        for note in &self.notes {
            if !deck_names.contains(note.deck.as_str()) {
                return Err(Error::DeckNotFound(note.deck.clone()));
            }
        }

        Ok(())
    }

    /// Get a model by name.
    pub fn get_model(&self, name: &str) -> Option<&ModelDef> {
        self.models.iter().find(|m| m.name == name)
    }

    /// Get a deck by name.
    pub fn get_deck(&self, name: &str) -> Option<&DeckDef> {
        self.decks.iter().find(|d| d.name == name)
    }

    /// Get notes for a specific deck.
    pub fn notes_for_deck(&self, deck_name: &str) -> impl Iterator<Item = &NoteDef> {
        self.notes.iter().filter(move |n| n.deck == deck_name)
    }
}

/// Package metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name (used as default deck name if no decks defined).
    pub name: String,

    /// Package version.
    #[serde(default = "default_version")]
    pub version: String,

    /// Package author.
    #[serde(default)]
    pub author: Option<String>,

    /// Package description.
    #[serde(default)]
    pub description: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Model (note type) definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDef {
    /// Model name (must be unique).
    pub name: String,

    /// Field names in order.
    pub fields: Vec<String>,

    /// Card templates.
    pub templates: Vec<TemplateDef>,

    /// CSS styling for cards.
    #[serde(default)]
    pub css: Option<String>,

    /// Which field to sort by (default: first field).
    #[serde(default)]
    pub sort_field: Option<String>,

    /// Model ID (auto-generated if not specified).
    #[serde(default)]
    pub id: Option<i64>,
}

impl ModelDef {
    /// Get the sort field index.
    pub fn sort_field_index(&self) -> usize {
        if let Some(ref name) = self.sort_field {
            self.fields.iter().position(|f| f == name).unwrap_or(0)
        } else {
            0
        }
    }
}

/// Card template definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDef {
    /// Template name.
    pub name: String,

    /// Front template (question side).
    pub front: String,

    /// Back template (answer side).
    pub back: String,
}

/// Deck definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckDef {
    /// Deck name (use :: for hierarchy, e.g., "Parent::Child").
    pub name: String,

    /// Deck description.
    #[serde(default)]
    pub description: Option<String>,

    /// Deck ID (auto-generated if not specified).
    #[serde(default)]
    pub id: Option<i64>,
}

/// Note definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDef {
    /// Deck name to add note to.
    pub deck: String,

    /// Model name for this note.
    pub model: String,

    /// Field values.
    pub fields: HashMap<String, String>,

    /// Tags for this note.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Custom GUID (auto-generated if not specified).
    #[serde(default)]
    pub guid: Option<String>,
}

impl NoteDef {
    /// Get field values in model field order.
    pub fn fields_ordered(&self, model: &ModelDef) -> Vec<String> {
        model
            .fields
            .iter()
            .map(|f| self.fields.get(f).cloned().unwrap_or_default())
            .collect()
    }

    /// Get tags as a space-separated string with surrounding spaces.
    pub fn tags_string(&self) -> String {
        if self.tags.is_empty() {
            String::new()
        } else {
            format!(" {} ", self.tags.join(" "))
        }
    }
}

/// Media file definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDef {
    /// Filename as referenced in note fields (e.g., "audio.mp3").
    pub name: String,

    /// Path to the source file.
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_definition() {
        let toml = r#"
[package]
name = "Test Deck"
version = "1.0.0"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{FrontSide}}<hr>{{Back}}"

[[decks]]
name = "Test Deck"

[[notes]]
deck = "Test Deck"
model = "Basic"
tags = ["test"]

[notes.fields]
Front = "Question"
Back = "Answer"
"#;

        let def = DeckDefinition::parse(toml).unwrap();
        assert_eq!(def.package.name, "Test Deck");
        assert_eq!(def.models.len(), 1);
        assert_eq!(def.models[0].fields, vec!["Front", "Back"]);
        assert_eq!(def.decks.len(), 1);
        assert_eq!(def.notes.len(), 1);
        assert_eq!(def.notes[0].fields.get("Front").unwrap(), "Question");
    }

    #[test]
    fn test_invalid_model_reference() {
        let toml = r#"
[package]
name = "Test"

[[decks]]
name = "Test Deck"

[[notes]]
deck = "Test Deck"
model = "NonExistent"

[notes.fields]
Front = "Q"
"#;

        let result = DeckDefinition::parse(toml);
        assert!(matches!(result, Err(Error::ModelNotFound(_))));
    }

    #[test]
    fn test_invalid_deck_reference() {
        let toml = r#"
[package]
name = "Test"

[[models]]
name = "Basic"
fields = ["Front"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Front}}"

[[notes]]
deck = "NonExistent"
model = "Basic"

[notes.fields]
Front = "Q"
"#;

        let result = DeckDefinition::parse(toml);
        assert!(matches!(result, Err(Error::DeckNotFound(_))));
    }

    #[test]
    fn test_invalid_field_reference() {
        let toml = r#"
[package]
name = "Test"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test Deck"

[[notes]]
deck = "Test Deck"
model = "Basic"

[notes.fields]
Front = "Q"
InvalidField = "X"
"#;

        let result = DeckDefinition::parse(toml);
        assert!(matches!(result, Err(Error::FieldNotFound { .. })));
    }

    #[test]
    fn test_fields_ordered() {
        let model = ModelDef {
            name: "Test".to_string(),
            fields: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            templates: vec![],
            css: None,
            sort_field: None,
            id: None,
        };

        let mut fields = HashMap::new();
        fields.insert("C".to_string(), "third".to_string());
        fields.insert("A".to_string(), "first".to_string());
        // B is missing, should be empty

        let note = NoteDef {
            deck: "Test".to_string(),
            model: "Test".to_string(),
            fields,
            tags: vec![],
            guid: None,
        };

        let ordered = note.fields_ordered(&model);
        assert_eq!(ordered, vec!["first", "", "third"]);
    }
}
