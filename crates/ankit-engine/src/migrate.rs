//! Note type migration operations.
//!
//! This module provides workflows for migrating notes from one
//! note type (model) to another with field mapping.

use crate::{Error, NoteBuilder, Result};
use ankit::AnkiClient;
use std::collections::HashMap;

/// Configuration for a note type migration.
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Source model name.
    pub source_model: String,
    /// Target model name.
    pub target_model: String,
    /// Field mapping: source field -> target field.
    pub field_mapping: HashMap<String, String>,
    /// Target deck (if None, keeps original deck).
    pub target_deck: Option<String>,
    /// Whether to delete source notes after migration.
    pub delete_source: bool,
    /// Tags to add to migrated notes.
    pub add_tags: Vec<String>,
}

/// Report of a migration operation.
#[derive(Debug, Clone, Default)]
pub struct MigrationReport {
    /// Number of notes successfully migrated.
    pub migrated: usize,
    /// Number of notes that failed to migrate.
    pub failed: usize,
    /// Number of source notes deleted.
    pub deleted: usize,
    /// Errors encountered during migration.
    pub errors: Vec<MigrationError>,
}

/// Error during migration of a single note.
#[derive(Debug, Clone)]
pub struct MigrationError {
    /// The source note ID.
    pub note_id: i64,
    /// The error message.
    pub error: String,
}

/// Migration workflow engine.
#[derive(Debug)]
pub struct MigrateEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> MigrateEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Migrate notes from one model to another.
    ///
    /// # Arguments
    ///
    /// * `config` - Migration configuration
    /// * `query` - Optional query to filter notes (None = all notes of source model)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::migrate::MigrationConfig;
    /// # use std::collections::HashMap;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let mut field_mapping = HashMap::new();
    /// field_mapping.insert("Front".to_string(), "Question".to_string());
    /// field_mapping.insert("Back".to_string(), "Answer".to_string());
    ///
    /// let config = MigrationConfig {
    ///     source_model: "Basic".to_string(),
    ///     target_model: "Basic (and reversed card)".to_string(),
    ///     field_mapping,
    ///     target_deck: None,
    ///     delete_source: false,
    ///     add_tags: vec!["migrated".to_string()],
    /// };
    ///
    /// let report = engine.migrate().notes(config, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn notes(
        &self,
        config: MigrationConfig,
        query: Option<&str>,
    ) -> Result<MigrationReport> {
        // Verify models exist
        let models = self.client.models().names().await?;
        if !models.contains(&config.source_model) {
            return Err(Error::ModelNotFound(config.source_model));
        }
        if !models.contains(&config.target_model) {
            return Err(Error::ModelNotFound(config.target_model));
        }

        // Verify target model has all mapped fields
        let target_fields = self
            .client
            .models()
            .field_names(&config.target_model)
            .await?;
        for target_field in config.field_mapping.values() {
            if !target_fields.contains(target_field) {
                return Err(Error::MissingField {
                    model: config.target_model.clone(),
                    field: target_field.clone(),
                });
            }
        }

        // Find notes to migrate
        let base_query = format!("note:\"{}\"", config.source_model);
        let full_query = match query {
            Some(q) => format!("{} {}", base_query, q),
            None => base_query,
        };

        let note_ids = self.client.notes().find(&full_query).await?;
        let note_infos = self.client.notes().info(&note_ids).await?;

        let mut report = MigrationReport::default();
        let mut notes_to_delete = Vec::new();

        for info in note_infos {
            // Map fields
            let mut new_fields = HashMap::new();
            for (source_field, target_field) in &config.field_mapping {
                if let Some(field_info) = info.fields.get(source_field) {
                    new_fields.insert(target_field.clone(), field_info.value.clone());
                }
            }

            // Determine deck
            // Get deck from first card of source note
            let deck = if let Some(ref deck) = config.target_deck {
                deck.clone()
            } else {
                // Try to get the deck from the source note's cards
                if !info.cards.is_empty() {
                    let card_info = self.client.cards().info(&info.cards[..1]).await?;
                    card_info
                        .first()
                        .map(|c| c.deck_name.clone())
                        .unwrap_or_else(|| "Default".to_string())
                } else {
                    "Default".to_string()
                }
            };

            // Build new note
            let mut builder = NoteBuilder::new(&deck, &config.target_model);
            for (field, value) in &new_fields {
                builder = builder.field(field, value);
            }

            // Add original tags plus new tags
            builder = builder.tags(info.tags.iter().cloned());
            builder = builder.tags(config.add_tags.iter().cloned());

            // Allow duplicate since we're migrating
            let note = builder.allow_duplicate(true).build();

            match self.client.notes().add(note).await {
                Ok(_) => {
                    report.migrated += 1;
                    if config.delete_source {
                        notes_to_delete.push(info.note_id);
                    }
                }
                Err(e) => {
                    report.failed += 1;
                    report.errors.push(MigrationError {
                        note_id: info.note_id,
                        error: e.to_string(),
                    });
                }
            }
        }

        // Delete source notes if requested
        if !notes_to_delete.is_empty() {
            self.client.notes().delete(&notes_to_delete).await?;
            report.deleted = notes_to_delete.len();
        }

        Ok(report)
    }

    /// Preview a migration without making changes.
    ///
    /// Returns information about what would be migrated.
    pub async fn preview(
        &self,
        config: &MigrationConfig,
        query: Option<&str>,
    ) -> Result<MigrationPreview> {
        // Verify models exist
        let models = self.client.models().names().await?;
        let source_exists = models.contains(&config.source_model);
        let target_exists = models.contains(&config.target_model);

        // Get field info
        let source_fields = if source_exists {
            self.client
                .models()
                .field_names(&config.source_model)
                .await?
        } else {
            Vec::new()
        };

        let target_fields = if target_exists {
            self.client
                .models()
                .field_names(&config.target_model)
                .await?
        } else {
            Vec::new()
        };

        // Check field mapping
        let mut mapping_issues = Vec::new();
        for (source, target) in &config.field_mapping {
            if !source_fields.contains(source) {
                mapping_issues.push(format!("Source field '{}' not found", source));
            }
            if !target_fields.contains(target) {
                mapping_issues.push(format!("Target field '{}' not found", target));
            }
        }

        // Count notes
        let note_count = if source_exists {
            let base_query = format!("note:\"{}\"", config.source_model);
            let full_query = match query {
                Some(q) => format!("{} {}", base_query, q),
                None => base_query,
            };
            let notes = self.client.notes().find(&full_query).await?;
            notes.len()
        } else {
            0
        };

        Ok(MigrationPreview {
            source_model_exists: source_exists,
            target_model_exists: target_exists,
            source_fields,
            target_fields,
            notes_to_migrate: note_count,
            mapping_issues,
        })
    }
}

/// Preview of a migration operation.
#[derive(Debug, Clone)]
pub struct MigrationPreview {
    /// Whether the source model exists.
    pub source_model_exists: bool,
    /// Whether the target model exists.
    pub target_model_exists: bool,
    /// Fields in the source model.
    pub source_fields: Vec<String>,
    /// Fields in the target model.
    pub target_fields: Vec<String>,
    /// Number of notes that would be migrated.
    pub notes_to_migrate: usize,
    /// Issues with the field mapping.
    pub mapping_issues: Vec<String>,
}
