//! Error types for the yanki crate.

use thiserror::Error;

/// The error type for AnkiConnect operations.
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP/network error from reqwest.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// AnkiConnect returned an error message.
    #[error("AnkiConnect error: {0}")]
    AnkiConnect(String),

    /// Response was empty (no result or error).
    #[error("AnkiConnect returned empty response")]
    EmptyResponse,

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Connection refused - Anki is likely not running.
    #[error("Could not connect to Anki. Is Anki running with AnkiConnect installed?")]
    ConnectionRefused,

    /// Permission denied by AnkiConnect.
    #[error("Permission denied. Request permission first or check API key.")]
    PermissionDenied,

    /// Note validation failed.
    #[error("Note validation failed: {0}")]
    NoteValidation(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// A specialized Result type for AnkiConnect operations.
pub type Result<T> = std::result::Result<T, Error>;
