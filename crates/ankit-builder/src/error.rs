//! Error types for ankit-builder.

use thiserror::Error;

/// Result type for ankit-builder operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during deck building.
#[derive(Debug, Error)]
pub enum Error {
    /// TOML parsing error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Model not found in deck definition.
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// Deck not found in deck definition.
    #[error("deck not found: {0}")]
    DeckNotFound(String),

    /// Field not found in model.
    #[error("field '{field}' not found in model '{model}'")]
    FieldNotFound {
        /// Model name.
        model: String,
        /// Field name.
        field: String,
    },

    /// Missing required field.
    #[error("missing required field '{field}' in model '{model}'")]
    MissingRequiredField {
        /// Model name.
        model: String,
        /// Field name.
        field: String,
    },

    /// Media file not found.
    #[error("media file not found: {0}")]
    MediaNotFound(String),

    /// Invalid deck definition.
    #[error("invalid deck definition: {0}")]
    InvalidDefinition(String),

    /// SQLite error (apkg feature).
    #[cfg(feature = "apkg")]
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// ZIP error (apkg feature).
    #[cfg(feature = "apkg")]
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// AnkiConnect error (connect feature).
    #[cfg(feature = "connect")]
    #[error("AnkiConnect error: {0}")]
    AnkiConnect(#[from] ankit::Error),
}
