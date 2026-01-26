//! Error types for the ankit crate.
//!
//! This module provides error handling for AnkiConnect operations.
//!
//! # Error Handling
//!
//! The most common errors you'll encounter are:
//!
//! - [`Error::ConnectionRefused`]: Anki is not running or AnkiConnect is not installed
//! - [`Error::AnkiConnect`]: The operation failed (e.g., deck not found, invalid query)
//! - [`Error::PermissionDenied`]: API key required or request needs approval
//!
//! # Example
//!
//! ```no_run
//! use ankit::{AnkiClient, Error};
//!
//! # async fn example() {
//! let client = AnkiClient::new();
//!
//! match client.decks().names().await {
//!     Ok(decks) => println!("Found {} decks", decks.len()),
//!     Err(Error::ConnectionRefused) => {
//!         eprintln!("Please start Anki with AnkiConnect installed");
//!     }
//!     Err(Error::PermissionDenied) => {
//!         eprintln!("Please configure your API key or approve the request in Anki");
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! # }
//! ```

use thiserror::Error;

/// The error type for AnkiConnect operations.
///
/// # Common Patterns
///
/// ## Checking if Anki is Running
///
/// ```no_run
/// use ankit::{AnkiClient, Error};
///
/// # async fn example() -> ankit::Result<()> {
/// let client = AnkiClient::new();
///
/// // Try a simple operation to check connectivity
/// match client.misc().version().await {
///     Ok(version) => println!("Connected to AnkiConnect v{}", version),
///     Err(Error::ConnectionRefused) => {
///         return Err(Error::ConnectionRefused);
///     }
///     Err(e) => return Err(e),
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Handling Duplicate Notes
///
/// ```no_run
/// use ankit::{AnkiClient, NoteBuilder, Error};
///
/// # async fn example() -> ankit::Result<()> {
/// let client = AnkiClient::new();
/// let note = NoteBuilder::new("Default", "Basic")
///     .field("Front", "Hello")
///     .field("Back", "World")
///     .build();
///
/// match client.notes().add(note).await {
///     Ok(id) => println!("Created note {}", id),
///     Err(Error::AnkiConnect(msg)) if msg.contains("duplicate") => {
///         println!("Note already exists");
///     }
///     Err(e) => return Err(e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP/network error from reqwest.
    ///
    /// Typically indicates network issues unrelated to Anki.
    /// For connection issues, see [`Error::ConnectionRefused`].
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// AnkiConnect returned an error message.
    ///
    /// The message string contains details about what went wrong.
    /// Common messages include:
    /// - "cannot create note because it is a duplicate"
    /// - "deck was not found"
    /// - "model was not found"
    #[error("AnkiConnect error: {0}")]
    AnkiConnect(String),

    /// Response was empty (no result or error).
    ///
    /// This is unexpected and may indicate an AnkiConnect bug.
    #[error("AnkiConnect returned empty response")]
    EmptyResponse,

    /// JSON serialization/deserialization error.
    ///
    /// May occur if AnkiConnect returns unexpected data formats.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Connection refused - Anki is likely not running.
    ///
    /// This error occurs when:
    /// - Anki is not running
    /// - The AnkiConnect add-on is not installed
    /// - AnkiConnect is configured on a different port
    #[error("Could not connect to Anki. Is Anki running with AnkiConnect installed?")]
    ConnectionRefused,

    /// Permission denied by AnkiConnect.
    ///
    /// This occurs when:
    /// - An API key is required but not provided
    /// - The provided API key is incorrect
    /// - A request requires user approval in the Anki UI
    #[error("Permission denied. Request permission first or check API key.")]
    PermissionDenied,

    /// Note validation failed.
    ///
    /// The note could not be added due to validation issues
    /// (e.g., missing required fields).
    #[error("Note validation failed: {0}")]
    NoteValidation(String),

    /// Invalid configuration.
    ///
    /// A configuration value was invalid or inconsistent.
    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// A specialized Result type for AnkiConnect operations.
pub type Result<T> = std::result::Result<T, Error>;
