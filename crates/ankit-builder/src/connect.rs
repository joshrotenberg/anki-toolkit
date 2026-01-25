//! AnkiConnect import functionality.
//!
//! Imports deck definitions directly into a running Anki instance via AnkiConnect.

use std::collections::HashMap;

use ankit::{AnkiClient, NoteBuilder};

use crate::error::{Error, Result};
use crate::schema::DeckDefinition;

/// Builder for importing decks via AnkiConnect.
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

            for (field, value) in &note_def.fields {
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
                for (field, value) in &note_def.fields {
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
