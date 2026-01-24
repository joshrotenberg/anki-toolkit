//! Statistics-related AnkiConnect actions.
//!
//! This module provides operations for retrieving study statistics and review data.
//!
//! # Example
//!
//! ```no_run
//! use yanki::AnkiClient;
//!
//! # async fn example() -> yanki::Result<()> {
//! let client = AnkiClient::new();
//!
//! // Get number of cards reviewed today
//! let count = client.statistics().cards_reviewed_today().await?;
//! println!("Cards reviewed today: {}", count);
//!
//! // Get review counts by day
//! let by_day = client.statistics().cards_reviewed_by_day().await?;
//! for (date, count) in by_day {
//!     println!("{}: {} reviews", date, count);
//! }
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::AnkiClient;
use crate::error::Result;

/// Provides access to statistics-related AnkiConnect operations.
///
/// Obtained via [`AnkiClient::statistics()`].
#[derive(Debug)]
pub struct StatisticsActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

#[derive(Serialize)]
struct CollectionStatsParams {
    #[serde(rename = "wholeCollection")]
    whole_collection: bool,
}

#[derive(Serialize)]
struct CardReviewsParams<'a> {
    deck: &'a str,
    #[serde(rename = "startID")]
    start_id: i64,
}

#[derive(Serialize)]
struct ReviewsOfCardsParams<'a> {
    cards: &'a [i64],
}

#[derive(Serialize)]
struct LatestReviewIdParams<'a> {
    deck: &'a str,
}

#[derive(Serialize)]
struct InsertReviewsParams<'a> {
    reviews: &'a [ReviewEntry],
}

/// A single review entry for insertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewEntry {
    /// The card ID.
    pub card_id: i64,
    /// Review timestamp (milliseconds since epoch).
    #[serde(rename = "id")]
    pub review_id: i64,
    /// Ease factor used.
    pub ease: i32,
    /// Interval before review (negative = seconds, positive = days).
    #[serde(rename = "ivl")]
    pub interval: i64,
    /// Interval after review (negative = seconds, positive = days).
    #[serde(rename = "lastIvl")]
    pub last_interval: i64,
    /// New ease factor after review.
    pub factor: i64,
    /// Time spent answering (milliseconds).
    pub time: i64,
    /// Review type (0 = learning, 1 = review, 2 = relearn, 3 = cram).
    #[serde(rename = "type")]
    pub review_type: i32,
}

impl ReviewEntry {
    /// Create a new review entry.
    pub fn new(card_id: i64, review_id: i64) -> Self {
        Self {
            card_id,
            review_id,
            ease: 3,
            interval: 1,
            last_interval: -60,
            factor: 2500,
            time: 10000,
            review_type: 1,
        }
    }

    /// Set the ease rating.
    pub fn ease(mut self, ease: i32) -> Self {
        self.ease = ease;
        self
    }

    /// Set the interval (positive = days, negative = seconds).
    pub fn interval(mut self, interval: i64) -> Self {
        self.interval = interval;
        self
    }

    /// Set the previous interval.
    pub fn last_interval(mut self, interval: i64) -> Self {
        self.last_interval = interval;
        self
    }

    /// Set the ease factor.
    pub fn factor(mut self, factor: i64) -> Self {
        self.factor = factor;
        self
    }

    /// Set the time spent in milliseconds.
    pub fn time(mut self, time: i64) -> Self {
        self.time = time;
        self
    }

    /// Set the review type.
    pub fn review_type(mut self, review_type: i32) -> Self {
        self.review_type = review_type;
        self
    }
}

impl<'a> StatisticsActions<'a> {
    /// Get the number of cards reviewed today.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let count = client.statistics().cards_reviewed_today().await?;
    /// println!("Reviewed {} cards today", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cards_reviewed_today(&self) -> Result<i64> {
        self.client
            .invoke_without_params("getNumCardsReviewedToday")
            .await
    }

    /// Get card review counts by day.
    ///
    /// Returns a list of (date, count) pairs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let by_day = client.statistics().cards_reviewed_by_day().await?;
    /// for (date, count) in by_day {
    ///     println!("{}: {} reviews", date, count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cards_reviewed_by_day(&self) -> Result<Vec<(String, i64)>> {
        self.client
            .invoke_without_params("getNumCardsReviewedByDay")
            .await
    }

    /// Get collection statistics as HTML.
    ///
    /// If `whole_collection` is true, returns stats for all decks.
    /// Otherwise, returns stats for the current deck only.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let html = client.statistics().collection_html(true).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn collection_html(&self, whole_collection: bool) -> Result<String> {
        self.client
            .invoke(
                "getCollectionStatsHTML",
                CollectionStatsParams { whole_collection },
            )
            .await
    }

    /// Get reviews for a deck since a given review ID.
    ///
    /// Returns a map of card ID to list of review timestamps.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let reviews = client.statistics().reviews_since("Default", 0).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reviews_since(
        &self,
        deck: &str,
        start_id: i64,
    ) -> Result<HashMap<String, Vec<Vec<i64>>>> {
        self.client
            .invoke("cardReviews", CardReviewsParams { deck, start_id })
            .await
    }

    /// Get reviews for specific cards.
    ///
    /// Returns a map of card ID to list of review entries.
    pub async fn reviews_for_cards(
        &self,
        card_ids: &[i64],
    ) -> Result<HashMap<String, Vec<ReviewEntry>>> {
        self.client
            .invoke(
                "getReviewsOfCards",
                ReviewsOfCardsParams { cards: card_ids },
            )
            .await
    }

    /// Get the latest review ID for a deck.
    ///
    /// Useful for incremental syncing of review data.
    pub async fn latest_review_id(&self, deck: &str) -> Result<i64> {
        self.client
            .invoke("getLatestReviewID", LatestReviewIdParams { deck })
            .await
    }

    /// Insert review entries into the database.
    ///
    /// This can be used to restore review history from a backup.
    pub async fn insert(&self, reviews: &[ReviewEntry]) -> Result<()> {
        self.client
            .invoke_void("insertReviews", InsertReviewsParams { reviews })
            .await
    }
}
