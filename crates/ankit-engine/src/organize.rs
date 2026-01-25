//! Deck organization operations.
//!
//! This module provides high-level workflows for deck cloning,
//! merging, and tag-based reorganization.

use crate::{Error, NoteBuilder, Result};
use ankit::AnkiClient;

/// Report of a deck clone operation.
#[derive(Debug, Clone, Default)]
pub struct CloneReport {
    /// Number of notes cloned.
    pub notes_cloned: usize,
    /// Number of notes that failed to clone.
    pub notes_failed: usize,
    /// Name of the destination deck.
    pub destination: String,
}

/// Report of a deck merge operation.
#[derive(Debug, Clone, Default)]
pub struct MergeReport {
    /// Number of cards moved.
    pub cards_moved: usize,
    /// Source decks that were merged.
    pub sources: Vec<String>,
    /// Destination deck.
    pub destination: String,
}

/// Organization workflow engine.
#[derive(Debug)]
pub struct OrganizeEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> OrganizeEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Clone a deck with all its notes.
    ///
    /// Creates a new deck with copies of all notes from the source deck.
    /// Scheduling information is not preserved (cards start as new).
    ///
    /// # Arguments
    ///
    /// * `source` - Name of the deck to clone
    /// * `destination` - Name for the new deck
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.organize().clone_deck("Japanese", "Japanese Copy").await?;
    /// println!("Cloned {} notes", report.notes_cloned);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clone_deck(&self, source: &str, destination: &str) -> Result<CloneReport> {
        // Verify source exists
        let decks = self.client.decks().names().await?;
        if !decks.contains(&source.to_string()) {
            return Err(Error::DeckNotFound(source.to_string()));
        }

        // Create destination deck
        self.client.decks().create(destination).await?;

        // Get all notes from source
        let query = format!("deck:\"{}\"", source);
        let note_ids = self.client.notes().find(&query).await?;
        let note_infos = self.client.notes().info(&note_ids).await?;

        let mut report = CloneReport {
            destination: destination.to_string(),
            ..Default::default()
        };

        // Clone each note
        for info in note_infos {
            let mut builder = NoteBuilder::new(destination, &info.model_name);

            for (field_name, field_info) in info.fields {
                builder = builder.field(field_name, field_info.value);
            }

            builder = builder.tags(info.tags);

            // Allow duplicates in the new deck
            let note = builder.allow_duplicate(true).build();

            match self.client.notes().add(note).await {
                Ok(_) => report.notes_cloned += 1,
                Err(_) => report.notes_failed += 1,
            }
        }

        Ok(report)
    }

    /// Merge multiple decks into one.
    ///
    /// Moves all cards from source decks into the destination deck.
    /// Does not delete the source decks.
    ///
    /// # Arguments
    ///
    /// * `sources` - Names of decks to merge
    /// * `destination` - Name of the destination deck
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.organize()
    ///     .merge_decks(&["Deck A", "Deck B"], "Combined")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn merge_decks(&self, sources: &[&str], destination: &str) -> Result<MergeReport> {
        // Create destination if it doesn't exist
        self.client.decks().create(destination).await?;

        let mut report = MergeReport {
            destination: destination.to_string(),
            sources: sources.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };

        // Move cards from each source
        for source in sources {
            let query = format!("deck:\"{}\"", source);
            let card_ids = self.client.cards().find(&query).await?;

            if !card_ids.is_empty() {
                self.client
                    .decks()
                    .move_cards(&card_ids, destination)
                    .await?;
                report.cards_moved += card_ids.len();
            }
        }

        Ok(report)
    }

    /// Move notes matching a tag to a different deck.
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag to search for
    /// * `destination` - Deck to move matching notes to
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let moved = engine.organize()
    ///     .move_by_tag("verb", "Japanese::Grammar::Verbs")
    ///     .await?;
    /// println!("Moved {} cards", moved);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn move_by_tag(&self, tag: &str, destination: &str) -> Result<usize> {
        // Create destination if needed
        self.client.decks().create(destination).await?;

        // Find cards with tag
        let query = format!("tag:{}", tag);
        let card_ids = self.client.cards().find(&query).await?;

        if !card_ids.is_empty() {
            self.client
                .decks()
                .move_cards(&card_ids, destination)
                .await?;
        }

        Ok(card_ids.len())
    }

    /// Reorganize cards by tag into subdecks.
    ///
    /// For each unique tag, creates a subdeck under the parent deck
    /// and moves matching cards there.
    ///
    /// # Arguments
    ///
    /// * `source_deck` - Deck to reorganize
    /// * `parent_deck` - Parent deck for new subdecks
    /// * `tags` - Tags to use for organization
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.organize()
    ///     .reorganize_by_tags("Japanese", "Japanese", &["verb", "noun", "adjective"])
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reorganize_by_tags(
        &self,
        source_deck: &str,
        parent_deck: &str,
        tags: &[&str],
    ) -> Result<ReorganizeReport> {
        let mut report = ReorganizeReport::default();

        for tag in tags {
            let subdeck = format!("{}::{}", parent_deck, tag);

            // Find cards in source deck with this tag
            let query = format!("deck:\"{}\" tag:{}", source_deck, tag);
            let card_ids = self.client.cards().find(&query).await?;

            if !card_ids.is_empty() {
                self.client.decks().create(&subdeck).await?;
                self.client.decks().move_cards(&card_ids, &subdeck).await?;
                report
                    .moved
                    .push((tag.to_string(), subdeck, card_ids.len()));
            }
        }

        Ok(report)
    }
}

/// Report of a reorganization operation.
#[derive(Debug, Clone, Default)]
pub struct ReorganizeReport {
    /// List of (tag, destination deck, card count) for each reorganization.
    pub moved: Vec<(String, String, usize)>,
}
