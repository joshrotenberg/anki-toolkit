//! A complete, async-first Rust client for the AnkiConnect API.
//!
//! This crate provides type-safe access to all AnkiConnect actions, allowing you
//! to programmatically interact with Anki from Rust applications.
//!
//! # Quick Start
//!
//! ```no_run
//! use yanki::AnkiClient;
//!
//! # async fn example() -> yanki::Result<()> {
//! // Create a client with default settings (localhost:8765)
//! let client = AnkiClient::new();
//!
//! // Check that AnkiConnect is running
//! let version = client.misc().version().await?;
//! println!("AnkiConnect version: {}", version);
//! # Ok(())
//! # }
//! ```
//!
//! # Client Configuration
//!
//! Use the builder pattern for custom configuration:
//!
//! ```no_run
//! use std::time::Duration;
//! use yanki::AnkiClient;
//!
//! let client = AnkiClient::builder()
//!     .url("http://localhost:8765")
//!     .api_key("your-api-key")
//!     .timeout(Duration::from_secs(60))
//!     .build();
//! ```
//!
//! # Action Groups
//!
//! Operations are organized into groups accessible from the client:
//!
//! - [`AnkiClient::cards()`] - Find, inspect, suspend, and answer cards
//! - [`AnkiClient::decks()`] - Create, delete, and configure decks
//! - [`AnkiClient::gui()`] - Control Anki's graphical interface
//! - [`AnkiClient::media()`] - Store, retrieve, and manage media files
//! - [`AnkiClient::models()`] - Manage note types, fields, and templates
//! - [`AnkiClient::notes()`] - Add, find, update, and delete notes
//! - [`AnkiClient::statistics()`] - Review history and collection statistics
//! - [`AnkiClient::misc()`] - Version, sync, profiles, and other miscellaneous operations
//!
//! # Requirements
//!
//! - Anki must be running with the [AnkiConnect](https://ankiweb.net/shared/info/2055492159) add-on installed
//! - By default, the client connects to `http://127.0.0.1:8765`

pub mod actions;
pub mod client;
pub mod error;
mod request;
pub mod types;

pub use client::{AnkiClient, ClientBuilder};
pub use error::{Error, Result};
pub use types::{
    CanAddResult, CardAnswer, CardInfo, CardModTime, CardTemplate, CreateModelParams, DeckConfig,
    DeckStats, DuplicateScope, Ease, FieldFont, FindReplaceParams, MediaAttachment, ModelField,
    ModelStyling, Note, NoteBuilder, NoteField, NoteInfo, NoteModTime, NoteOptions,
    StoreMediaParams,
};
