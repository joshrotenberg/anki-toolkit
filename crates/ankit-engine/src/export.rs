//! Deck and review history export operations.
//!
//! This module provides high-level export workflows for extracting
//! deck contents and review history.

use crate::Result;
use ankit::AnkiClient;
use serde::Serialize;

/// Exported note with all fields and metadata.
#[derive(Debug, Clone, Serialize)]
pub struct ExportedNote {
    /// The note ID.
    pub note_id: i64,
    /// The model (note type) name.
    pub model_name: String,
    /// The deck name.
    pub deck_name: String,
    /// Field values keyed by field name.
    pub fields: std::collections::HashMap<String, String>,
    /// Tags on the note.
    pub tags: Vec<String>,
}

/// Exported card with scheduling information.
#[derive(Debug, Clone, Serialize)]
pub struct ExportedCard {
    /// The card ID.
    pub card_id: i64,
    /// The note ID this card belongs to.
    pub note_id: i64,
    /// The deck name.
    pub deck_name: String,
    /// Number of reviews.
    pub reps: i64,
    /// Number of lapses.
    pub lapses: i64,
    /// Current interval in days.
    pub interval: i64,
    /// Due date (days since collection creation, or negative for learning).
    pub due: i64,
    /// Ease factor (as integer, e.g., 2500 = 250%).
    pub ease_factor: i64,
    /// Card type (0 = new, 1 = learning, 2 = review, 3 = relearning).
    pub card_type: i32,
    /// Queue (-1 = suspended, -2 = sibling buried, -3 = manually buried,
    /// 0 = new, 1 = learning, 2 = review, 3 = day learn, 4 = preview).
    pub queue: i32,
    /// Last modification timestamp (seconds since epoch).
    pub mod_time: i64,
}

/// Export of deck contents.
#[derive(Debug, Clone, Serialize)]
pub struct DeckExport {
    /// Deck name.
    pub deck_name: String,
    /// All notes in the deck.
    pub notes: Vec<ExportedNote>,
    /// All cards in the deck.
    pub cards: Vec<ExportedCard>,
}

/// Export workflow engine.
#[derive(Debug)]
pub struct ExportEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> ExportEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Export all notes and cards from a deck.
    ///
    /// # Arguments
    ///
    /// * `deck_name` - Name of the deck to export
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let export = engine.export().deck("Japanese").await?;
    /// println!("Exported {} notes", export.notes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deck(&self, deck_name: &str) -> Result<DeckExport> {
        // Find all notes in deck
        let query = format!("deck:\"{}\"", deck_name);
        let note_ids = self.client.notes().find(&query).await?;
        let note_infos = self.client.notes().info(&note_ids).await?;

        // Find all cards in deck
        let card_ids = self.client.cards().find(&query).await?;
        let card_infos = self.client.cards().info(&card_ids).await?;

        // Convert to export format
        let notes = note_infos
            .into_iter()
            .map(|info| ExportedNote {
                note_id: info.note_id,
                model_name: info.model_name,
                deck_name: deck_name.to_string(),
                fields: info.fields.into_iter().map(|(k, v)| (k, v.value)).collect(),
                tags: info.tags,
            })
            .collect();

        let cards = card_infos
            .into_iter()
            .map(|info| ExportedCard {
                card_id: info.card_id,
                note_id: info.note_id,
                deck_name: info.deck_name,
                reps: info.reps,
                lapses: info.lapses,
                interval: info.interval,
                due: info.due,
                ease_factor: info.ease_factor,
                card_type: info.card_type,
                queue: info.queue,
                mod_time: info.mod_time,
            })
            .collect();

        Ok(DeckExport {
            deck_name: deck_name.to_string(),
            notes,
            cards,
        })
    }

    /// Export review history for cards.
    ///
    /// # Arguments
    ///
    /// * `query` - Anki search query to select cards
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let reviews = engine.export().reviews("deck:Japanese").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reviews(&self, query: &str) -> Result<Vec<CardReviewHistory>> {
        let card_ids = self.client.cards().find(query).await?;

        if card_ids.is_empty() {
            return Ok(Vec::new());
        }

        let reviews = self
            .client
            .statistics()
            .reviews_for_cards(&card_ids)
            .await?;

        // Convert HashMap<String, Vec<ReviewEntry>> to Vec<CardReviewHistory>
        let mut result = Vec::new();

        for (card_id_str, card_reviews) in reviews {
            let card_id: i64 = card_id_str.parse().unwrap_or(0);
            let entries: Vec<ExportedReviewEntry> = card_reviews
                .iter()
                .map(|r| ExportedReviewEntry {
                    timestamp: r.review_id,
                    ease: r.ease,
                    interval: r.interval,
                    last_interval: r.last_interval,
                    time_ms: r.time,
                })
                .collect();
            result.push(CardReviewHistory {
                card_id,
                reviews: entries,
            });
        }

        Ok(result)
    }
}

/// Review history for a single card.
#[derive(Debug, Clone, Serialize)]
pub struct CardReviewHistory {
    /// The card ID.
    pub card_id: i64,
    /// Review entries in chronological order.
    pub reviews: Vec<ExportedReviewEntry>,
}

/// A single review entry.
#[derive(Debug, Clone, Serialize)]
pub struct ExportedReviewEntry {
    /// Review timestamp (milliseconds since epoch).
    pub timestamp: i64,
    /// Ease button pressed (1-4).
    pub ease: i32,
    /// Resulting interval.
    pub interval: i64,
    /// Previous interval.
    pub last_interval: i64,
    /// Time spent on review in milliseconds.
    pub time_ms: i64,
}
