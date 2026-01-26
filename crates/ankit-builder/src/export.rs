//! Export decks from Anki to TOML format.
//!
//! This module provides functionality to pull decks from Anki via AnkiConnect
//! and export them to the TOML format used by ankit-builder.
//!
//! # Example
//!
//! ```no_run
//! use ankit::AnkiClient;
//! use ankit_builder::DeckExporter;
//!
//! # async fn example() -> ankit_builder::Result<()> {
//! let client = AnkiClient::new();
//! let exporter = DeckExporter::new(&client);
//!
//! // Export a deck to DeckDefinition
//! let definition = exporter.export_deck("Japanese::Vocabulary").await?;
//!
//! // Write to TOML file
//! definition.write_toml("japanese.toml")?;
//! # Ok(())
//! # }
//! ```

use std::collections::{HashMap, HashSet};
use std::path::Path;

use ankit::AnkiClient;

use crate::error::{Error, Result};
use crate::schema::{DeckDef, DeckDefinition, ModelDef, NoteDef, PackageInfo, TemplateDef};

/// Exports decks from Anki to TOML format.
///
/// Uses AnkiConnect to fetch deck contents and convert them to a
/// [`DeckDefinition`] that can be serialized to TOML.
pub struct DeckExporter<'a> {
    client: &'a AnkiClient,
}

impl<'a> DeckExporter<'a> {
    /// Create a new exporter with the given AnkiConnect client.
    pub fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Export a deck to a [`DeckDefinition`].
    ///
    /// Fetches all notes in the specified deck, along with the models (note types)
    /// they use, and converts them to a TOML-compatible definition.
    ///
    /// # Arguments
    ///
    /// * `deck_name` - The name of the deck to export (supports hierarchical names like "Parent::Child")
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::AnkiClient;
    /// use ankit_builder::DeckExporter;
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let client = AnkiClient::new();
    /// let exporter = DeckExporter::new(&client);
    ///
    /// let definition = exporter.export_deck("My Deck").await?;
    /// println!("Exported {} notes", definition.notes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_deck(&self, deck_name: &str) -> Result<DeckDefinition> {
        // Find all notes in the deck
        let query = format!("deck:\"{}\"", deck_name);
        let note_ids = self.client.notes().find(&query).await?;

        if note_ids.is_empty() {
            // Return empty definition with just the deck
            return Ok(DeckDefinition {
                package: PackageInfo {
                    name: deck_name.to_string(),
                    version: "1.0.0".to_string(),
                    author: None,
                    description: None,
                },
                models: Vec::new(),
                decks: vec![DeckDef {
                    name: deck_name.to_string(),
                    description: None,
                    id: None,
                }],
                notes: Vec::new(),
                media: Vec::new(),
            });
        }

        // Get note info
        let note_infos = self.client.notes().info(&note_ids).await?;

        // Collect unique model names
        let model_names: HashSet<String> =
            note_infos.iter().map(|n| n.model_name.clone()).collect();

        // Fetch model details
        let mut models = Vec::new();
        for model_name in &model_names {
            let model_def = self.fetch_model(model_name).await?;
            models.push(model_def);
        }

        // Convert notes to NoteDef
        let notes: Vec<NoteDef> = note_infos
            .iter()
            .map(|note| {
                let fields: HashMap<String, String> = note
                    .fields
                    .iter()
                    .map(|(name, field)| (name.clone(), field.value.clone()))
                    .collect();

                NoteDef {
                    deck: deck_name.to_string(),
                    model: note.model_name.clone(),
                    fields,
                    tags: note.tags.clone(),
                    guid: None,
                    note_id: Some(note.note_id),
                }
            })
            .collect();

        Ok(DeckDefinition {
            package: PackageInfo {
                name: deck_name.to_string(),
                version: "1.0.0".to_string(),
                author: None,
                description: None,
            },
            models,
            decks: vec![DeckDef {
                name: deck_name.to_string(),
                description: None,
                id: None,
            }],
            notes,
            media: Vec::new(),
        })
    }

    /// Export multiple decks to a single [`DeckDefinition`].
    ///
    /// Useful for exporting a parent deck and its children, or a set of related decks.
    ///
    /// # Arguments
    ///
    /// * `deck_names` - The names of the decks to export
    /// * `package_name` - The name for the package (used in `[package]` section)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::AnkiClient;
    /// use ankit_builder::DeckExporter;
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let client = AnkiClient::new();
    /// let exporter = DeckExporter::new(&client);
    ///
    /// let definition = exporter
    ///     .export_decks(&["Japanese::N5", "Japanese::N4"], "Japanese JLPT")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_decks(
        &self,
        deck_names: &[&str],
        package_name: &str,
    ) -> Result<DeckDefinition> {
        let mut all_notes = Vec::new();
        let mut model_names: HashSet<String> = HashSet::new();
        let mut decks = Vec::new();

        for deck_name in deck_names {
            // Find all notes in this deck
            let query = format!("deck:\"{}\"", deck_name);
            let note_ids = self.client.notes().find(&query).await?;

            // Add deck to list
            decks.push(DeckDef {
                name: deck_name.to_string(),
                description: None,
                id: None,
            });

            if note_ids.is_empty() {
                continue;
            }

            // Get note info
            let note_infos = self.client.notes().info(&note_ids).await?;

            // Collect model names
            for note in &note_infos {
                model_names.insert(note.model_name.clone());
            }

            // Convert notes
            for note in note_infos {
                let fields: HashMap<String, String> = note
                    .fields
                    .iter()
                    .map(|(name, field)| (name.clone(), field.value.clone()))
                    .collect();

                all_notes.push(NoteDef {
                    deck: deck_name.to_string(),
                    model: note.model_name.clone(),
                    fields,
                    tags: note.tags,
                    guid: None,
                    note_id: Some(note.note_id),
                });
            }
        }

        // Fetch model details
        let mut models = Vec::new();
        for model_name in &model_names {
            let model_def = self.fetch_model(model_name).await?;
            models.push(model_def);
        }

        Ok(DeckDefinition {
            package: PackageInfo {
                name: package_name.to_string(),
                version: "1.0.0".to_string(),
                author: None,
                description: None,
            },
            models,
            decks,
            notes: all_notes,
            media: Vec::new(),
        })
    }

    /// Fetch model definition from Anki.
    async fn fetch_model(&self, model_name: &str) -> Result<ModelDef> {
        // Get field names
        let fields = self.client.models().field_names(model_name).await?;

        // Get templates
        let templates_map = self.client.models().templates(model_name).await?;
        let templates: Vec<TemplateDef> = templates_map
            .into_iter()
            .map(|(name, template)| TemplateDef {
                name,
                front: template.front,
                back: template.back,
            })
            .collect();

        // Get CSS styling
        let styling = self.client.models().styling(model_name).await?;
        let css = if styling.css.is_empty() {
            None
        } else {
            Some(styling.css)
        };

        Ok(ModelDef {
            name: model_name.to_string(),
            fields,
            templates,
            css,
            sort_field: None,
            id: None,
            markdown_fields: vec![],
        })
    }
}

impl DeckDefinition {
    /// Write the deck definition to a TOML file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckDefinition;
    ///
    /// # fn example() -> ankit_builder::Result<()> {
    /// let definition = DeckDefinition::from_file("input.toml")?;
    /// // ... modify definition ...
    /// definition.write_toml("output.toml")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_toml(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = self.to_toml()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Serialize the deck definition to a TOML string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckDefinition;
    ///
    /// # fn example() -> ankit_builder::Result<()> {
    /// let definition = DeckDefinition::from_file("input.toml")?;
    /// let toml_string = definition.to_toml()?;
    /// println!("{}", toml_string);
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::TomlSerialize(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_toml_roundtrip() {
        let toml_input = r#"
[package]
name = "Test Deck"
version = "1.0.0"

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
tags = ["test"]

[notes.fields]
Front = "Question"
Back = "Answer"
"#;

        let definition = DeckDefinition::parse(toml_input).unwrap();
        let toml_output = definition.to_toml().unwrap();

        // Parse the output and verify it matches
        let definition2 = DeckDefinition::parse(&toml_output).unwrap();
        assert_eq!(definition.package.name, definition2.package.name);
        assert_eq!(definition.models.len(), definition2.models.len());
        assert_eq!(definition.notes.len(), definition2.notes.len());
    }

    #[test]
    fn test_note_id_serialization() {
        let toml_input = r#"
[package]
name = "Test"
version = "1.0.0"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "Basic"
note_id = 1234567890

[notes.fields]
Front = "Q"
Back = "A"
"#;

        // Parse TOML with note_id
        let definition = DeckDefinition::parse(toml_input).unwrap();
        assert_eq!(definition.notes[0].note_id, Some(1234567890));

        // Serialize and verify note_id is preserved
        let toml_output = definition.to_toml().unwrap();
        assert!(
            toml_output.contains("note_id = 1234567890"),
            "note_id should be in output: {}",
            toml_output
        );

        // Roundtrip
        let definition2 = DeckDefinition::parse(&toml_output).unwrap();
        assert_eq!(definition2.notes[0].note_id, Some(1234567890));
    }

    #[test]
    fn test_note_id_omitted_when_none() {
        let toml_input = r#"
[package]
name = "Test"
version = "1.0.0"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "Basic"

[notes.fields]
Front = "Q"
Back = "A"
"#;

        let definition = DeckDefinition::parse(toml_input).unwrap();
        assert_eq!(definition.notes[0].note_id, None);

        // Serialize and verify note_id is NOT in output
        let toml_output = definition.to_toml().unwrap();
        assert!(
            !toml_output.contains("note_id"),
            "note_id should be omitted when None: {}",
            toml_output
        );
    }
}
