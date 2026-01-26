//! Error types for ankit-engine.
//!
//! Errors from engine workflows fall into two categories:
//!
//! 1. **Client errors**: Wrapped from the underlying [`ankit::Error`] type
//! 2. **Workflow errors**: Specific to engine operations (e.g., deck not found)
//!
//! # Example
//!
//! ```no_run
//! use ankit_engine::{Engine, Error};
//!
//! # async fn example() {
//! let engine = Engine::new();
//!
//! match engine.analyze().study_summary("NonexistentDeck", 7).await {
//!     Ok(stats) => println!("Reviews: {}", stats.total_reviews),
//!     Err(Error::DeckNotFound(name)) => {
//!         eprintln!("Deck '{}' not found", name);
//!     }
//!     Err(Error::Client(ankit::Error::ConnectionRefused)) => {
//!         eprintln!("Is Anki running?");
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! # }
//! ```

use std::fmt;

/// Result type for ankit-engine operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during engine operations.
///
/// Engine errors wrap lower-level client errors and add workflow-specific
/// error variants for common failure cases.
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

    /// An I/O error occurred.
    Io(std::io::Error),

    /// A backup operation failed.
    Backup(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Client(e) => Some(e),
            Error::Io(e) => Some(e),
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
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Backup(msg) => write!(f, "backup error: {}", msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<ankit::Error> for Error {
    fn from(err: ankit::Error) -> Self {
        Error::Client(err)
    }
}
