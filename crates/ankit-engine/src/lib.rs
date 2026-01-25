//! High-level workflow operations for Anki via AnkiConnect.
//!
//! This crate provides ergonomic, high-level operations built on top of the
//! [`ankit`] client library. While `ankit` provides 1:1 API bindings, `ankit-engine`
//! combines multiple API calls into cohesive workflows.
//!
//! # Quick Start
//!
//! ```no_run
//! use ankit_engine::Engine;
//!
//! # async fn example() -> ankit_engine::Result<()> {
//! let engine = Engine::new();
//!
//! // High-level workflows
//! let stats = engine.analyze().study_summary("Japanese", 30).await?;
//! println!("Cards reviewed: {}", stats.total_reviews);
//!
//! // Direct client access when needed
//! let version = engine.client().misc().version().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Feature Flags
//!
//! All workflow modules are enabled by default. Disable with:
//!
//! ```toml
//! [dependencies]
//! ankit-engine = { version = "0.1", default-features = false, features = ["analyze"] }
//! ```
//!
//! Available features:
//! - `import` - Bulk import with duplicate handling
//! - `export` - Deck and review history export
//! - `organize` - Deck cloning, merging, reorganization
//! - `analyze` - Study statistics and problem card detection
//! - `migrate` - Note type migration with field mapping
//! - `media` - Media audit and cleanup
//! - `progress` - Card state management and performance tagging
//! - `enrich` - Find and update notes with empty fields
//! - `deduplicate` - Duplicate detection and removal

mod error;

#[cfg(feature = "analyze")]
pub mod analyze;

#[cfg(feature = "export")]
pub mod export;

#[cfg(feature = "import")]
pub mod import;

#[cfg(feature = "media")]
pub mod media;

#[cfg(feature = "migrate")]
pub mod migrate;

#[cfg(feature = "organize")]
pub mod organize;

#[cfg(feature = "progress")]
pub mod progress;

#[cfg(feature = "enrich")]
pub mod enrich;

#[cfg(feature = "deduplicate")]
pub mod deduplicate;

pub use error::{Error, Result};

// Re-export ankit types for convenience
pub use ankit::{
    AnkiClient, CanAddResult, CardAnswer, CardInfo, CardModTime, CardTemplate, ClientBuilder,
    CreateModelParams, DeckConfig, DeckStats, DuplicateScope, Ease, FieldFont, FindReplaceParams,
    LapseConfig, MediaAttachment, ModelField, ModelStyling, NewCardConfig, Note, NoteBuilder,
    NoteField, NoteInfo, NoteModTime, NoteOptions, ReviewConfig, StoreMediaParams,
};

#[cfg(feature = "analyze")]
use analyze::AnalyzeEngine;

#[cfg(feature = "export")]
use export::ExportEngine;

#[cfg(feature = "import")]
use import::ImportEngine;

#[cfg(feature = "media")]
use media::MediaEngine;

#[cfg(feature = "migrate")]
use migrate::MigrateEngine;

#[cfg(feature = "organize")]
use organize::OrganizeEngine;

#[cfg(feature = "progress")]
use progress::ProgressEngine;

#[cfg(feature = "enrich")]
use enrich::EnrichEngine;

#[cfg(feature = "deduplicate")]
use deduplicate::DeduplicateEngine;

/// High-level workflow engine for Anki operations.
///
/// The engine wraps an [`AnkiClient`] and provides access to workflow modules
/// that combine multiple API calls into cohesive operations.
///
/// # Example
///
/// ```no_run
/// use ankit_engine::Engine;
///
/// # async fn example() -> ankit_engine::Result<()> {
/// // Create with default client settings
/// let engine = Engine::new();
///
/// // Or with a custom client
/// let client = ankit_engine::AnkiClient::builder()
///     .url("http://localhost:8765")
///     .build();
/// let engine = Engine::from_client(client);
///
/// // Access workflow modules
/// let stats = engine.analyze().study_summary("Default", 7).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Engine {
    client: AnkiClient,
}

impl Engine {
    /// Create a new engine with default client settings.
    ///
    /// Connects to AnkiConnect at `http://127.0.0.1:8765`.
    pub fn new() -> Self {
        Self {
            client: AnkiClient::new(),
        }
    }

    /// Create an engine from an existing client.
    pub fn from_client(client: AnkiClient) -> Self {
        Self { client }
    }

    /// Get a reference to the underlying client.
    ///
    /// Use this for direct API access when workflows don't cover your use case.
    pub fn client(&self) -> &AnkiClient {
        &self.client
    }

    /// Access import workflows.
    ///
    /// Provides bulk import with duplicate detection and conflict resolution.
    #[cfg(feature = "import")]
    pub fn import(&self) -> ImportEngine<'_> {
        ImportEngine::new(&self.client)
    }

    /// Access export workflows.
    ///
    /// Provides deck export and review history extraction.
    #[cfg(feature = "export")]
    pub fn export(&self) -> ExportEngine<'_> {
        ExportEngine::new(&self.client)
    }

    /// Access organization workflows.
    ///
    /// Provides deck cloning, merging, and tag-based reorganization.
    #[cfg(feature = "organize")]
    pub fn organize(&self) -> OrganizeEngine<'_> {
        OrganizeEngine::new(&self.client)
    }

    /// Access analysis workflows.
    ///
    /// Provides study statistics and problem card (leech) detection.
    #[cfg(feature = "analyze")]
    pub fn analyze(&self) -> AnalyzeEngine<'_> {
        AnalyzeEngine::new(&self.client)
    }

    /// Access migration workflows.
    ///
    /// Provides note type migration with field mapping.
    #[cfg(feature = "migrate")]
    pub fn migrate(&self) -> MigrateEngine<'_> {
        MigrateEngine::new(&self.client)
    }

    /// Access media workflows.
    ///
    /// Provides media audit and cleanup operations.
    #[cfg(feature = "media")]
    pub fn media(&self) -> MediaEngine<'_> {
        MediaEngine::new(&self.client)
    }

    /// Access progress management workflows.
    ///
    /// Provides card state management, performance tagging, and bulk operations.
    #[cfg(feature = "progress")]
    pub fn progress(&self) -> ProgressEngine<'_> {
        ProgressEngine::new(&self.client)
    }

    /// Access enrichment workflows.
    ///
    /// Provides tools for finding notes with empty fields and updating them.
    #[cfg(feature = "enrich")]
    pub fn enrich(&self) -> EnrichEngine<'_> {
        EnrichEngine::new(&self.client)
    }

    /// Access deduplication workflows.
    ///
    /// Provides duplicate detection and removal based on key fields.
    #[cfg(feature = "deduplicate")]
    pub fn deduplicate(&self) -> DeduplicateEngine<'_> {
        DeduplicateEngine::new(&self.client)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
