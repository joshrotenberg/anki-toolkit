//! Study statistics and problem card detection.
//!
//! This module provides analytics workflows for understanding study
//! patterns and identifying cards that need attention.

use crate::Result;
use ankit::AnkiClient;
use serde::Serialize;

/// Summary of study activity.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StudySummary {
    /// Total number of reviews in the period.
    pub total_reviews: usize,
    /// Number of unique cards reviewed.
    pub unique_cards: usize,
    /// Total time spent studying in seconds.
    pub total_time_seconds: u64,
    /// Average reviews per day.
    pub avg_reviews_per_day: f64,
    /// Daily breakdown.
    pub daily: Vec<DailyStats>,
}

/// Study statistics for a single day.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DailyStats {
    /// Date in YYYY-MM-DD format.
    pub date: String,
    /// Number of reviews.
    pub reviews: usize,
    /// Time spent in seconds.
    pub time_seconds: u64,
}

/// A card identified as problematic.
#[derive(Debug, Clone, Serialize)]
pub struct ProblemCard {
    /// The card ID.
    pub card_id: i64,
    /// The note ID.
    pub note_id: i64,
    /// Number of lapses (times forgotten).
    pub lapses: i64,
    /// Total number of reviews.
    pub reps: i64,
    /// Current ease factor (percentage * 10).
    pub ease: i64,
    /// Current interval in days.
    pub interval: i64,
    /// The deck name.
    pub deck_name: String,
    /// Front field content (first field).
    pub front: String,
    /// Reason this card was flagged.
    pub reason: ProblemReason,
}

/// Reason a card was flagged as problematic.
#[derive(Debug, Clone, Serialize)]
pub enum ProblemReason {
    /// Card has been forgotten many times.
    HighLapseCount(i64),
    /// Card has very low ease factor.
    LowEase(i64),
    /// Card has been reviewed many times but still has short interval.
    PoorRetention { reps: i64, interval: i64 },
}

/// Criteria for finding problem cards.
#[derive(Debug, Clone)]
pub struct ProblemCriteria {
    /// Minimum lapse count to flag.
    pub min_lapses: i64,
    /// Maximum ease factor to flag (e.g., 2000 = 200%).
    pub max_ease: i64,
    /// Minimum reps with max interval for poor retention.
    pub min_reps_for_retention: i64,
    /// Maximum interval with high reps for poor retention.
    pub max_interval_for_retention: i64,
}

impl Default for ProblemCriteria {
    fn default() -> Self {
        Self {
            min_lapses: 5,
            max_ease: 2000, // 200%
            min_reps_for_retention: 10,
            max_interval_for_retention: 7,
        }
    }
}

/// Analysis workflow engine.
#[derive(Debug)]
pub struct AnalyzeEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> AnalyzeEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Get a summary of study activity.
    ///
    /// # Arguments
    ///
    /// * `deck` - Deck to analyze (use "*" for all decks)
    /// * `days` - Number of days to include
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let stats = engine.analyze().study_summary("Japanese", 30).await?;
    /// println!("Reviewed {} cards", stats.total_reviews);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn study_summary(&self, deck: &str, days: u32) -> Result<StudySummary> {
        let daily_reviews = self.client.statistics().cards_reviewed_by_day().await?;

        let mut summary = StudySummary::default();
        let take_days = days as usize;

        // Take last N days
        let recent: Vec<_> = daily_reviews.into_iter().take(take_days).collect();

        for (date, count) in &recent {
            summary.total_reviews += *count as usize;
            summary.daily.push(DailyStats {
                date: date.clone(),
                reviews: *count as usize,
                time_seconds: 0, // Would need review data for this
            });
        }

        if !recent.is_empty() {
            summary.avg_reviews_per_day = summary.total_reviews as f64 / recent.len() as f64;
        }

        // Get unique cards reviewed
        if deck != "*" {
            let query = format!("deck:\"{}\" rated:{}", deck, days);
            let cards = self.client.cards().find(&query).await?;
            summary.unique_cards = cards.len();
        }

        Ok(summary)
    }

    /// Find problem cards (leeches).
    ///
    /// # Arguments
    ///
    /// * `query` - Anki search query to filter cards
    /// * `criteria` - Criteria for identifying problems
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::analyze::ProblemCriteria;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let problems = engine.analyze()
    ///     .find_problems("deck:Japanese", ProblemCriteria::default())
    ///     .await?;
    /// for card in problems {
    ///     println!("Problem card: {} - {:?}", card.front, card.reason);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_problems(
        &self,
        query: &str,
        criteria: ProblemCriteria,
    ) -> Result<Vec<ProblemCard>> {
        let card_ids = self.client.cards().find(query).await?;

        if card_ids.is_empty() {
            return Ok(Vec::new());
        }

        let cards = self.client.cards().info(&card_ids).await?;
        let mut problems = Vec::new();

        for card in cards {
            let reason = if card.lapses >= criteria.min_lapses {
                Some(ProblemReason::HighLapseCount(card.lapses))
            } else if card.ease_factor > 0 && card.ease_factor <= criteria.max_ease {
                Some(ProblemReason::LowEase(card.ease_factor))
            } else if card.reps >= criteria.min_reps_for_retention
                && card.interval <= criteria.max_interval_for_retention
            {
                Some(ProblemReason::PoorRetention {
                    reps: card.reps,
                    interval: card.interval,
                })
            } else {
                None
            };

            if let Some(reason) = reason {
                // Get the note to get the front field
                let note_info = self.client.notes().info(&[card.note_id]).await?;
                let front = note_info
                    .first()
                    .and_then(|n| n.fields.values().next())
                    .map(|f| f.value.clone())
                    .unwrap_or_default();

                problems.push(ProblemCard {
                    card_id: card.card_id,
                    note_id: card.note_id,
                    lapses: card.lapses,
                    reps: card.reps,
                    ease: card.ease_factor,
                    interval: card.interval,
                    deck_name: card.deck_name.clone(),
                    front,
                    reason,
                });
            }
        }

        Ok(problems)
    }

    /// Get retention statistics for a deck.
    ///
    /// # Arguments
    ///
    /// * `deck` - Deck to analyze
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let retention = engine.analyze().retention_stats("Japanese").await?;
    /// println!("Average ease: {}%", retention.avg_ease / 10);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retention_stats(&self, deck: &str) -> Result<RetentionStats> {
        let query = format!("deck:\"{}\" is:review", deck);
        let card_ids = self.client.cards().find(&query).await?;

        if card_ids.is_empty() {
            return Ok(RetentionStats::default());
        }

        let cards = self.client.cards().info(&card_ids).await?;
        let ease_factors = self.client.cards().get_ease(&card_ids).await?;

        let total_lapses: i64 = cards.iter().map(|c| c.lapses).sum();
        let total_reps: i64 = cards.iter().map(|c| c.reps).sum();
        let avg_ease: i64 = if !ease_factors.is_empty() {
            ease_factors.iter().sum::<i64>() / ease_factors.len() as i64
        } else {
            0
        };
        let avg_interval: i64 = if !cards.is_empty() {
            cards.iter().map(|c| c.interval).sum::<i64>() / cards.len() as i64
        } else {
            0
        };

        Ok(RetentionStats {
            total_cards: cards.len(),
            total_reviews: total_reps as usize,
            total_lapses: total_lapses as usize,
            avg_ease,
            avg_interval,
            retention_rate: if total_reps > 0 {
                1.0 - (total_lapses as f64 / total_reps as f64)
            } else {
                0.0
            },
        })
    }
}

/// Retention statistics for a deck.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RetentionStats {
    /// Total number of review cards.
    pub total_cards: usize,
    /// Total number of reviews.
    pub total_reviews: usize,
    /// Total number of lapses.
    pub total_lapses: usize,
    /// Average ease factor (percentage * 10).
    pub avg_ease: i64,
    /// Average interval in days.
    pub avg_interval: i64,
    /// Estimated retention rate (0.0 - 1.0).
    pub retention_rate: f64,
}
