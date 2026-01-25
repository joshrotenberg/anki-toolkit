//! Error types for ankit-engine.

use std::fmt;

/// Result type for ankit-engine operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during engine operations.
#[derive(Debug)]
pub enum Error {
    /// An error from the underlying ankit client.
    Client(ankit::Error),

    /// A deck was not found.
    DeckNotFound(String),

    /// A model (note type) was not found.
    ModelNotFound(String),

    /// A required field is missing from a note.
    MissingField {
        /// The model name.
        model: String,
        /// The missing field name.
        field: String,
    },

    /// No notes matched the query.
    NoNotesFound(String),

    /// An operation was cancelled.
    Cancelled,

    /// A validation error occurred.
    Validation(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Client(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Client(e) => write!(f, "{}", e),
            Error::DeckNotFound(name) => write!(f, "deck not found: {}", name),
            Error::ModelNotFound(name) => write!(f, "model not found: {}", name),
            Error::MissingField { model, field } => {
                write!(f, "missing field '{}' for model '{}'", field, model)
            }
            Error::NoNotesFound(query) => write!(f, "no notes found for query: {}", query),
            Error::Cancelled => write!(f, "operation cancelled"),
            Error::Validation(msg) => write!(f, "validation error: {}", msg),
        }
    }
}

impl From<ankit::Error> for Error {
    fn from(err: ankit::Error) -> Self {
        Error::Client(err)
    }
}
