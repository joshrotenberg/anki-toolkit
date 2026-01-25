//! Card-related AnkiConnect actions.
//!
//! This module provides operations for finding and inspecting cards.
//! Note that cards are generated from notes - one note can produce multiple cards.
//!
//! # Example
//!
//! ```no_run
//! use ankit::AnkiClient;
//!
//! # async fn example() -> ankit::Result<()> {
//! let client = AnkiClient::new();
//!
//! // Find cards due today
//! let due_cards = client.cards().find("is:due").await?;
//! println!("Cards due: {}", due_cards.len());
//!
//! // Get card details
//! if !due_cards.is_empty() {
//!     let info = client.cards().info(&due_cards[..5.min(due_cards.len())]).await?;
//!     for card in info {
//!         println!("Card {} in deck {}", card.card_id, card.deck_name);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use serde::Serialize;

use crate::client::AnkiClient;
use crate::error::Result;
use crate::types::{CardAnswer, CardInfo, CardModTime};

/// Provides access to card-related AnkiConnect operations.
///
/// Obtained via [`AnkiClient::cards()`].
#[derive(Debug)]
pub struct CardActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

// Parameter structs
#[derive(Serialize)]
struct FindCardsParams<'a> {
    query: &'a str,
}

#[derive(Serialize)]
struct CardsInfoParams<'a> {
    cards: &'a [i64],
}

#[derive(Serialize)]
struct SuspendParams<'a> {
    cards: &'a [i64],
}

#[derive(Serialize)]
struct SuspendedParams {
    card: i64,
}

#[derive(Serialize)]
struct GetIntervalsParams<'a> {
    cards: &'a [i64],
    complete: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetEaseFactorsParams<'a> {
    cards: &'a [i64],
    ease_factors: &'a [i64],
}

#[derive(Serialize)]
struct AnswerCardsParams<'a> {
    answers: &'a [CardAnswer],
}

impl<'a> CardActions<'a> {
    /// Find cards matching a query.
    ///
    /// Returns a list of card IDs. Use [`info()`](Self::info) to get full card details.
    ///
    /// # Query Syntax
    ///
    /// Uses Anki's search syntax:
    /// - `deck:DeckName` - cards in a specific deck
    /// - `is:due` - cards that are due
    /// - `is:new` - new cards
    /// - `is:review` - review cards
    /// - `is:suspended` - suspended cards
    /// - `rated:1` - cards rated today
    /// - `-is:suspended` - exclude suspended cards
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// // Find all due cards in the Japanese deck
    /// let cards = client.cards().find("deck:Japanese is:due").await?;
    /// println!("Found {} due cards", cards.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find(&self, query: &str) -> Result<Vec<i64>> {
        self.client
            .invoke("findCards", FindCardsParams { query })
            .await
    }

    /// Get detailed information about cards.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let card_ids = client.cards().find("is:due").await?;
    /// let cards = client.cards().info(&card_ids).await?;
    ///
    /// for card in cards {
    ///     println!("Card {} (note {}): {} reps, {} lapses",
    ///         card.card_id, card.note_id, card.reps, card.lapses);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn info(&self, card_ids: &[i64]) -> Result<Vec<CardInfo>> {
        self.client
            .invoke("cardsInfo", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Convert card IDs to their corresponding note IDs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let card_ids = client.cards().find("is:due").await?;
    /// let note_ids = client.cards().to_notes(&card_ids).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn to_notes(&self, card_ids: &[i64]) -> Result<Vec<i64>> {
        self.client
            .invoke("cardsToNotes", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Get modification times for cards.
    pub async fn mod_time(&self, card_ids: &[i64]) -> Result<Vec<CardModTime>> {
        self.client
            .invoke("cardsModTime", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Suspend cards.
    ///
    /// Suspended cards will not appear in reviews.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.cards().suspend(&[1234567890]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn suspend(&self, card_ids: &[i64]) -> Result<bool> {
        self.client
            .invoke("suspend", SuspendParams { cards: card_ids })
            .await
    }

    /// Unsuspend cards.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.cards().unsuspend(&[1234567890]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsuspend(&self, card_ids: &[i64]) -> Result<bool> {
        self.client
            .invoke("unsuspend", SuspendParams { cards: card_ids })
            .await
    }

    /// Check if a single card is suspended.
    ///
    /// Returns `true` if the card is suspended.
    pub async fn is_suspended(&self, card_id: i64) -> Result<bool> {
        self.client
            .invoke("suspended", SuspendedParams { card: card_id })
            .await
    }

    /// Check if multiple cards are suspended.
    ///
    /// Returns `Some(true)` if suspended, `Some(false)` if not, `None` if card doesn't exist.
    pub async fn are_suspended(&self, card_ids: &[i64]) -> Result<Vec<Option<bool>>> {
        self.client
            .invoke("areSuspended", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Check if cards are due for review.
    pub async fn are_due(&self, card_ids: &[i64]) -> Result<Vec<bool>> {
        self.client
            .invoke("areDue", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Get intervals for cards.
    ///
    /// If `complete` is false, returns only the current interval.
    /// If `complete` is true, returns the full interval history.
    pub async fn intervals(
        &self,
        card_ids: &[i64],
        complete: bool,
    ) -> Result<Vec<serde_json::Value>> {
        self.client
            .invoke(
                "getIntervals",
                GetIntervalsParams {
                    cards: card_ids,
                    complete,
                },
            )
            .await
    }

    /// Get ease factors for cards.
    ///
    /// Ease factors are returned as integers (e.g., 2500 = 250%).
    pub async fn get_ease(&self, card_ids: &[i64]) -> Result<Vec<i64>> {
        self.client
            .invoke("getEaseFactors", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Set ease factors for cards.
    ///
    /// Ease factors should be integers (e.g., 2500 = 250%).
    /// Returns success status for each card.
    pub async fn set_ease(&self, card_ids: &[i64], ease_factors: &[i64]) -> Result<Vec<bool>> {
        self.client
            .invoke(
                "setEaseFactors",
                SetEaseFactorsParams {
                    cards: card_ids,
                    ease_factors,
                },
            )
            .await
    }

    /// Forget cards, making them new again.
    ///
    /// This resets the card's learning progress.
    pub async fn forget(&self, card_ids: &[i64]) -> Result<()> {
        self.client
            .invoke_void("forgetCards", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Put cards back into the learning queue.
    pub async fn relearn(&self, card_ids: &[i64]) -> Result<()> {
        self.client
            .invoke_void("relearnCards", CardsInfoParams { cards: card_ids })
            .await
    }

    /// Answer cards programmatically.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::{AnkiClient, CardAnswer, Ease};
    ///
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let answers = vec![
    ///     CardAnswer::new(1234567890, Ease::Good),
    ///     CardAnswer::new(1234567891, Ease::Easy),
    /// ];
    ///
    /// let results = client.cards().answer(&answers).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn answer(&self, answers: &[CardAnswer]) -> Result<Vec<bool>> {
        self.client
            .invoke("answerCards", AnswerCardsParams { answers })
            .await
    }
}
