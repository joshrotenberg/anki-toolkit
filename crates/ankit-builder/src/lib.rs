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

pub use error::{Error, Result};
pub use schema::{DeckDef, DeckDefinition, MediaDef, ModelDef, NoteDef, PackageInfo, TemplateDef};

#[cfg(feature = "apkg")]
pub use apkg::ApkgBuilder;

#[cfg(feature = "connect")]
pub use connect::{ConnectImporter, ImportResult};

/// Unified builder that can output to either .apkg or AnkiConnect.
pub struct DeckBuilder {
    definition: DeckDefinition,
    #[cfg(feature = "apkg")]
    media_base_path: Option<std::path::PathBuf>,
}

impl DeckBuilder {
    /// Create a new builder from a deck definition.
    pub fn new(definition: DeckDefinition) -> Self {
        Self {
            definition,
            #[cfg(feature = "apkg")]
            media_base_path: None,
        }
    }

    /// Load a deck definition from a TOML file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let definition = DeckDefinition::from_file(path)?;
        Ok(Self::new(definition))
    }

    /// Load a deck definition from a TOML string.
    pub fn parse(content: &str) -> Result<Self> {
        let definition = DeckDefinition::parse(content)?;
        Ok(Self::new(definition))
    }

    /// Set the base path for resolving media file paths.
    #[cfg(feature = "apkg")]
    pub fn media_base_path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.media_base_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Get the underlying deck definition.
    pub fn definition(&self) -> &DeckDefinition {
        &self.definition
    }

    /// Write to an .apkg file.
    #[cfg(feature = "apkg")]
    pub fn write_apkg(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let mut builder = ApkgBuilder::new(self.definition.clone());
        if let Some(ref media_path) = self.media_base_path {
            builder = builder.media_base_path(media_path);
        }
        builder.write_to_file(path)
    }

    /// Import via AnkiConnect.
    #[cfg(feature = "connect")]
    pub async fn import_connect(&self) -> Result<ImportResult> {
        let importer = ConnectImporter::new(self.definition.clone());
        importer.import().await
    }

    /// Import via AnkiConnect in batch mode.
    #[cfg(feature = "connect")]
    pub async fn import_connect_batch(&self) -> Result<ImportResult> {
        let importer = ConnectImporter::new(self.definition.clone());
        importer.import_batch().await
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
