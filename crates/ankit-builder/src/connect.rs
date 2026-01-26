//! AnkiConnect import functionality.
//!
//! This module provides [`ConnectImporter`] for importing deck definitions directly
//! into a running Anki instance via AnkiConnect.
//!
//! # Requirements
//!
//! - Anki must be running
//! - The [AnkiConnect](https://foosoft.net/projects/anki-connect/) add-on must be installed
//! - Note types (models) referenced in the definition must already exist in Anki
//!
//! # Example
//!
//! ```no_run
//! use ankit_builder::{ConnectImporter, DeckDefinition};
//!
//! # async fn example() -> ankit_builder::Result<()> {
//! let definition = DeckDefinition::from_file("vocabulary.toml")?;
//! let importer = ConnectImporter::new(definition);
//!
//! // Optionally validate before importing
//! let missing_models = importer.validate_models().await?;
//! if !missing_models.is_empty() {
//!     eprintln!("Missing models: {:?}", missing_models);
//!     return Ok(());
//! }
//!
//! let result = importer.import().await?;
//! println!("Created {} notes", result.notes_created);
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use ankit::{AnkiClient, NoteBuilder};

use crate::error::{Error, Result};
use crate::schema::DeckDefinition;

/// Imports deck definitions into Anki via AnkiConnect.
///
/// `ConnectImporter` handles the live import of notes into a running Anki
/// instance. It automatically creates missing decks but requires that all
/// referenced note types (models) already exist.
///
/// # Import Methods
///
/// - [`import()`](Self::import): Adds notes one at a time (safer, better error tracking)
/// - [`import_batch()`](Self::import_batch): Adds all notes in a single call (faster)
///
/// # Validation
///
/// Use [`validate_models()`](Self::validate_models) and [`validate_decks()`](Self::validate_decks)
/// to check prerequisites before importing.
pub struct ConnectImporter {
    definition: DeckDefinition,
    client: AnkiClient,
}

/// Result of an import operation.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Number of decks created.
    pub decks_created: usize,
    /// Number of notes created.
    pub notes_created: usize,
    /// Number of notes skipped (duplicates or errors).
    pub notes_skipped: usize,
    /// Errors encountered (note index -> error message).
    pub errors: HashMap<usize, String>,
}

impl ConnectImporter {
    /// Create a new importer from a deck definition.
    pub fn new(definition: DeckDefinition) -> Self {
        Self {
            definition,
            client: AnkiClient::new(),
        }
    }

    /// Create a new importer with a custom AnkiConnect client.
    pub fn with_client(definition: DeckDefinition, client: AnkiClient) -> Self {
        Self { definition, client }
    }

    /// Import the deck definition into Anki.
    ///
    /// This will:
    /// 1. Create any missing decks
    /// 2. Add all notes (using existing models)
    ///
    /// Note: Models must already exist in Anki. This method does not create models.
    pub async fn import(&self) -> Result<ImportResult> {
        let mut result = ImportResult {
            decks_created: 0,
            notes_created: 0,
            notes_skipped: 0,
            errors: HashMap::new(),
        };

        // Create decks if they don't exist
        let existing_decks = self.client.decks().names().await?;
        for deck in &self.definition.decks {
            if !existing_decks.contains(&deck.name) {
                self.client.decks().create(&deck.name).await?;
                result.decks_created += 1;
            }
        }

        // Verify models exist
        let existing_models = self.client.models().names().await?;
        for model in &self.definition.models {
            if !existing_models.contains(&model.name) {
                return Err(Error::ModelNotFound(format!(
                    "Model '{}' does not exist in Anki. Create it manually first.",
                    model.name
                )));
            }
        }

        // Add notes
        for (i, note_def) in self.definition.notes.iter().enumerate() {
            let mut builder = NoteBuilder::new(&note_def.deck, &note_def.model);

            // Get markdown fields for this model
            let markdown_fields = self
                .definition
                .get_model(&note_def.model)
                .map(|m| m.markdown_fields.clone())
                .unwrap_or_default();

            // Convert markdown to HTML for markdown fields
            let fields = note_def.fields_as_html(&markdown_fields);

            for (field, value) in &fields {
                builder = builder.field(field, value);
            }

            for tag in &note_def.tags {
                builder = builder.tag(tag);
            }

            let note = builder.build();

            match self.client.notes().add(note).await {
                Ok(_) => {
                    result.notes_created += 1;
                }
                Err(e) => {
                    result.notes_skipped += 1;
                    result.errors.insert(i, e.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Import notes in batches for better performance.
    ///
    /// Note: Models must already exist in Anki. This method does not create models.
    pub async fn import_batch(&self) -> Result<ImportResult> {
        let mut result = ImportResult {
            decks_created: 0,
            notes_created: 0,
            notes_skipped: 0,
            errors: HashMap::new(),
        };

        // Create decks if they don't exist
        let existing_decks = self.client.decks().names().await?;
        for deck in &self.definition.decks {
            if !existing_decks.contains(&deck.name) {
                self.client.decks().create(&deck.name).await?;
                result.decks_created += 1;
            }
        }

        // Verify models exist
        let existing_models = self.client.models().names().await?;
        for model in &self.definition.models {
            if !existing_models.contains(&model.name) {
                return Err(Error::ModelNotFound(format!(
                    "Model '{}' does not exist in Anki. Create it manually first.",
                    model.name
                )));
            }
        }

        // Build notes for batch add
        let notes: Vec<_> = self
            .definition
            .notes
            .iter()
            .map(|note_def| {
                let mut builder = NoteBuilder::new(&note_def.deck, &note_def.model);

                // Get markdown fields for this model
                let markdown_fields = self
                    .definition
                    .get_model(&note_def.model)
                    .map(|m| m.markdown_fields.clone())
                    .unwrap_or_default();

                // Convert markdown to HTML for markdown fields
                let fields = note_def.fields_as_html(&markdown_fields);

                for (field, value) in &fields {
                    builder = builder.field(field, value);
                }
                for tag in &note_def.tags {
                    builder = builder.tag(tag);
                }
                builder.build()
            })
            .collect();

        // Add notes in batch
        let results = self.client.notes().add_many(&notes).await?;

        for (i, note_result) in results.iter().enumerate() {
            match note_result {
                Some(_) => result.notes_created += 1,
                None => {
                    result.notes_skipped += 1;
                    result.errors.insert(i, "Failed to add note".to_string());
                }
            }
        }

        Ok(result)
    }

    /// Check if all required models exist in Anki.
    ///
    /// Returns a list of model names that are defined in the TOML but do not
    /// exist in Anki. An empty list means all models are available.
    ///
    /// Unlike decks, models cannot be created automatically and must exist
    /// before import. Use this to warn users or fail early.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::{ConnectImporter, DeckDefinition};
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let definition = DeckDefinition::from_file("deck.toml")?;
    /// let importer = ConnectImporter::new(definition);
    ///
    /// let missing = importer.validate_models().await?;
    /// if !missing.is_empty() {
    ///     eprintln!("Please create these note types in Anki first:");
    ///     for model in &missing {
    ///         eprintln!("  - {}", model);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_models(&self) -> Result<Vec<String>> {
        let existing_models = self.client.models().names().await?;
        let missing: Vec<String> = self
            .definition
            .models
            .iter()
            .filter(|m| !existing_models.contains(&m.name))
            .map(|m| m.name.clone())
            .collect();
        Ok(missing)
    }

    /// Check if all required decks exist in Anki.
    ///
    /// Returns a list of deck names that are defined in the TOML but do not
    /// exist in Anki. An empty list means all decks are available.
    ///
    /// Note that [`import()`](Self::import) automatically creates missing decks,
    /// so this is mainly useful for informational purposes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::{ConnectImporter, DeckDefinition};
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let definition = DeckDefinition::from_file("deck.toml")?;
    /// let importer = ConnectImporter::new(definition);
    ///
    /// let missing = importer.validate_decks().await?;
    /// if !missing.is_empty() {
    ///     println!("Will create {} new decks", missing.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_decks(&self) -> Result<Vec<String>> {
        let existing_decks = self.client.decks().names().await?;
        let missing: Vec<String> = self
            .definition
            .decks
            .iter()
            .filter(|d| !existing_decks.contains(&d.name))
            .map(|d| d.name.clone())
            .collect();
        Ok(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_result_default() {
        let result = ImportResult {
            decks_created: 0,
            notes_created: 0,
            notes_skipped: 0,
            errors: HashMap::new(),
        };
        assert_eq!(result.decks_created, 0);
    }
}
