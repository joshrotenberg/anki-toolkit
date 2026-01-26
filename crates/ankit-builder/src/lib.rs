//! TOML-based Anki deck builder with .apkg generation and AnkiConnect import.
//!
//! This crate provides a unified way to define Anki decks using TOML and then either:
//! - Generate .apkg files for direct import into Anki
//! - Import directly into a running Anki instance via AnkiConnect
//!
//! # Features
//!
//! - `apkg` (default): Enable .apkg file generation
//! - `connect` (default): Enable AnkiConnect import
//!
//! # Example TOML Format
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
//!
//! [[models.templates]]
//! name = "Spanish -> English"
//! front = "{{Spanish}}"
//! back = "{{FrontSide}}<hr>{{English}}"
//!
//! [[decks]]
//! name = "Spanish::Vocabulary"
//! description = "Core Spanish vocabulary"
//!
//! [[notes]]
//! deck = "Spanish::Vocabulary"
//! model = "Basic Spanish"
//! tags = ["chapter1"]
//!
//! [notes.fields]
//! Spanish = "el gato"
//! English = "the cat"
//! Example = "El gato es negro."
//! ```
//!
//! # Usage
//!
//! ## Generate .apkg file
//!
//! ```no_run
//! use ankit_builder::{DeckDefinition, ApkgBuilder};
//!
//! let definition = DeckDefinition::from_file("my_deck.toml").unwrap();
//! let builder = ApkgBuilder::new(definition);
//! builder.write_to_file("my_deck.apkg").unwrap();
//! ```
//!
//! ## Import via AnkiConnect
//!
//! ```no_run
//! use ankit_builder::{DeckDefinition, ConnectImporter};
//!
//! # async fn example() -> ankit_builder::Result<()> {
//! let definition = DeckDefinition::from_file("my_deck.toml")?;
//! let importer = ConnectImporter::new(definition);
//! let result = importer.import().await?;
//! println!("Created {} notes", result.notes_created);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod schema;

#[cfg(feature = "apkg")]
mod sql;

#[cfg(feature = "apkg")]
mod apkg;

#[cfg(feature = "connect")]
mod connect;

#[cfg(feature = "connect")]
mod diff;

#[cfg(feature = "connect")]
mod export;

pub use error::{Error, Result};
pub use schema::{DeckDef, DeckDefinition, MediaDef, ModelDef, NoteDef, PackageInfo, TemplateDef};

#[cfg(feature = "apkg")]
pub use apkg::ApkgBuilder;

#[cfg(feature = "connect")]
pub use connect::{ConnectImporter, ImportResult};

#[cfg(feature = "connect")]
pub use diff::{DeckDiff, FieldChange, ModifiedNote, NoteDiff, TagChanges};

#[cfg(feature = "connect")]
pub use export::DeckExporter;

/// Unified builder that can output to either .apkg or AnkiConnect.
///
/// `DeckBuilder` provides a high-level interface for working with deck definitions,
/// supporting both offline `.apkg` file generation and live import via AnkiConnect.
///
/// # Features
///
/// - `apkg` (default): Enable `.apkg` file generation via [`write_apkg()`](Self::write_apkg)
/// - `connect` (default): Enable AnkiConnect import via [`import_connect()`](Self::import_connect)
///
/// # Example
///
/// ```no_run
/// use ankit_builder::DeckBuilder;
///
/// # fn main() -> ankit_builder::Result<()> {
/// // Load from TOML file
/// let builder = DeckBuilder::from_file("vocabulary.toml")?;
///
/// // Generate .apkg file
/// builder.write_apkg("vocabulary.apkg")?;
/// # Ok(())
/// # }
/// ```
///
/// # Async Example (AnkiConnect)
///
/// ```no_run
/// use ankit_builder::DeckBuilder;
///
/// # async fn example() -> ankit_builder::Result<()> {
/// let builder = DeckBuilder::from_file("vocabulary.toml")?;
/// let result = builder.import_connect().await?;
/// println!("Created {} notes", result.notes_created);
/// # Ok(())
/// # }
/// ```
pub struct DeckBuilder {
    definition: DeckDefinition,
    #[cfg(feature = "apkg")]
    media_base_path: Option<std::path::PathBuf>,
}

impl DeckBuilder {
    /// Create a new builder from a deck definition.
    ///
    /// Use this when you have a programmatically constructed [`DeckDefinition`].
    /// For loading from TOML, prefer [`from_file()`](Self::from_file) or
    /// [`parse()`](Self::parse).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// // Typically you would load from TOML:
    /// let builder = DeckBuilder::from_file("deck.toml").unwrap();
    ///
    /// // Or parse from a string:
    /// let toml = r#"
    /// [package]
    /// name = "My Deck"
    /// version = "1.0.0"
    /// "#;
    /// let builder = DeckBuilder::parse(toml).unwrap();
    /// ```
    pub fn new(definition: DeckDefinition) -> Self {
        Self {
            definition,
            #[cfg(feature = "apkg")]
            media_base_path: None,
        }
    }

    /// Load a deck definition from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains invalid TOML.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # fn main() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("my_deck.toml")?;
    /// println!("Loaded deck: {}", builder.definition().package.name);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let definition = DeckDefinition::from_file(path)?;
        Ok(Self::new(definition))
    }

    /// Load a deck definition from a TOML string.
    ///
    /// Useful for testing or when the TOML content is embedded or dynamically generated.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains invalid TOML.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit_builder::DeckBuilder;
    ///
    /// let toml = r#"
    /// [package]
    /// name = "Test Deck"
    /// version = "1.0.0"
    ///
    /// [[models]]
    /// name = "Basic"
    /// fields = ["Front", "Back"]
    ///
    /// [[models.templates]]
    /// name = "Card 1"
    /// front = "{{Front}}"
    /// back = "{{Back}}"
    ///
    /// [[decks]]
    /// name = "Test Deck"
    /// "#;
    ///
    /// let builder = DeckBuilder::parse(toml).unwrap();
    /// assert_eq!(builder.definition().package.name, "Test Deck");
    /// ```
    pub fn parse(content: &str) -> Result<Self> {
        let definition = DeckDefinition::parse(content)?;
        Ok(Self::new(definition))
    }

    /// Set the base path for resolving media file paths.
    ///
    /// When your TOML definition references media files with relative paths,
    /// this sets the directory from which those paths are resolved.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # fn main() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("decks/vocabulary.toml")?
    ///     .media_base_path("decks/media");
    ///
    /// // Media files like "audio.mp3" will be loaded from "decks/media/audio.mp3"
    /// builder.write_apkg("vocabulary.apkg")?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "apkg")]
    pub fn media_base_path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.media_base_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Get the underlying deck definition.
    ///
    /// Use this to inspect the parsed TOML structure, including package metadata,
    /// models, decks, and notes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # fn main() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("my_deck.toml")?;
    /// let def = builder.definition();
    ///
    /// println!("Package: {} v{}", def.package.name, def.package.version);
    /// println!("Models: {}", def.models.len());
    /// println!("Decks: {}", def.decks.len());
    /// println!("Notes: {}", def.notes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn definition(&self) -> &DeckDefinition {
        &self.definition
    }

    /// Write the deck definition to an `.apkg` file.
    ///
    /// Generates a complete Anki package file that can be imported directly
    /// into Anki via File > Import. The generated file includes all notes,
    /// cards, and referenced media files.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The output path cannot be written to
    /// - Media files referenced in the definition cannot be read
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # fn main() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("vocabulary.toml")?;
    /// builder.write_apkg("vocabulary.apkg")?;
    /// println!("Created vocabulary.apkg");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "apkg")]
    pub fn write_apkg(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let mut builder = ApkgBuilder::new(self.definition.clone());
        if let Some(ref media_path) = self.media_base_path {
            builder = builder.media_base_path(media_path);
        }
        builder.write_to_file(path)
    }

    /// Import the deck definition via AnkiConnect.
    ///
    /// Imports notes one at a time into a running Anki instance. Creates
    /// any missing decks automatically.
    ///
    /// # Requirements
    ///
    /// - Anki must be running with the AnkiConnect add-on installed
    /// - Note types (models) must already exist in Anki
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot connect to Anki (not running or AnkiConnect not installed)
    /// - A referenced model does not exist in Anki
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("vocabulary.toml")?;
    /// let result = builder.import_connect().await?;
    ///
    /// println!("Decks created: {}", result.decks_created);
    /// println!("Notes created: {}", result.notes_created);
    /// println!("Notes skipped: {}", result.notes_skipped);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "connect")]
    pub async fn import_connect(&self) -> Result<ImportResult> {
        let importer = ConnectImporter::new(self.definition.clone());
        importer.import().await
    }

    /// Import the deck definition via AnkiConnect in batch mode.
    ///
    /// More efficient than [`import_connect()`](Self::import_connect) for large
    /// decks as it uses a single API call to add all notes.
    ///
    /// # Requirements
    ///
    /// - Anki must be running with the AnkiConnect add-on installed
    /// - Note types (models) must already exist in Anki
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("large_deck.toml")?;
    /// let result = builder.import_connect_batch().await?;
    /// println!("Imported {} notes", result.notes_created);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "connect")]
    pub async fn import_connect_batch(&self) -> Result<ImportResult> {
        let importer = ConnectImporter::new(self.definition.clone());
        importer.import_batch().await
    }

    /// Compare the TOML definition against the live state in Anki.
    ///
    /// Shows what's different between the TOML definition and Anki:
    /// - Notes only in TOML (would be added on import)
    /// - Notes only in Anki (exist in Anki but not in TOML)
    /// - Modified notes (exist in both but have differences)
    ///
    /// Uses the first field value (normalized) as the key for matching notes.
    ///
    /// # Requirements
    ///
    /// - Anki must be running with the AnkiConnect add-on installed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("deck.toml")?;
    /// let diff = builder.diff_connect().await?;
    ///
    /// println!("Notes only in TOML: {}", diff.toml_only.len());
    /// println!("Notes only in Anki: {}", diff.anki_only.len());
    /// println!("Modified notes: {}", diff.modified.len());
    /// println!("Unchanged notes: {}", diff.unchanged);
    ///
    /// for note in &diff.modified {
    ///     println!("Modified: {} ({} field changes)",
    ///         note.first_field, note.field_changes.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "connect")]
    pub async fn diff_connect(&self) -> Result<DeckDiff> {
        let client = ankit::AnkiClient::new();
        self.diff_connect_with_client(&client).await
    }

    /// Compare the TOML definition against Anki using a custom client.
    ///
    /// Like [`diff_connect()`](Self::diff_connect) but allows using a custom
    /// [`AnkiClient`](ankit::AnkiClient) with non-default settings.
    #[cfg(feature = "connect")]
    pub async fn diff_connect_with_client(&self, client: &ankit::AnkiClient) -> Result<DeckDiff> {
        let differ = diff::DeckDiffer::new(client, &self.definition);
        differ.diff().await
    }

    /// Export a deck from Anki to a [`DeckBuilder`].
    ///
    /// Fetches all notes in the specified deck from a running Anki instance
    /// via AnkiConnect and creates a `DeckBuilder` that can be used to write
    /// to TOML or .apkg files.
    ///
    /// # Requirements
    ///
    /// - Anki must be running with the AnkiConnect add-on installed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::AnkiClient;
    /// use ankit_builder::DeckBuilder;
    ///
    /// # async fn example() -> ankit_builder::Result<()> {
    /// let client = AnkiClient::new();
    /// let builder = DeckBuilder::from_anki(&client, "Japanese::Vocabulary").await?;
    ///
    /// // Write to TOML
    /// builder.definition().write_toml("japanese.toml")?;
    ///
    /// // Or write to .apkg
    /// builder.write_apkg("japanese.apkg")?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "connect")]
    pub async fn from_anki(client: &ankit::AnkiClient, deck_name: &str) -> Result<Self> {
        let exporter = DeckExporter::new(client);
        let definition = exporter.export_deck(deck_name).await?;
        Ok(Self::new(definition))
    }

    /// Write the deck definition to a TOML file.
    ///
    /// Convenience method that calls [`DeckDefinition::write_toml()`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_builder::DeckBuilder;
    ///
    /// # fn main() -> ankit_builder::Result<()> {
    /// let builder = DeckBuilder::from_file("deck.toml")?;
    /// builder.write_toml("deck_copy.toml")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_toml(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        self.definition.write_toml(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_TOML: &str = r#"
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
description = "A test deck"

[[notes]]
deck = "Test Deck"
model = "Basic"
tags = ["test"]

[notes.fields]
Front = "What is 2+2?"
Back = "4"

[[notes]]
deck = "Test Deck"
model = "Basic"

[notes.fields]
Front = "Capital of France?"
Back = "Paris"
"#;

    #[test]
    fn test_deck_builder_parse() {
        let builder = DeckBuilder::parse(EXAMPLE_TOML).unwrap();
        assert_eq!(builder.definition().package.name, "Test Deck");
        assert_eq!(builder.definition().notes.len(), 2);
    }

    #[test]
    #[cfg(feature = "apkg")]
    fn test_write_apkg() {
        let builder = DeckBuilder::parse(EXAMPLE_TOML).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.apkg");
        builder.write_apkg(&path).unwrap();
        assert!(path.exists());
    }
}
