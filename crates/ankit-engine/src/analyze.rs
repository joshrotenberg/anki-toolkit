//! Study statistics and problem card detection.
//!
//! This module provides analytics workflows for understanding study
//! patterns and identifying cards that need attention.

use std::collections::HashMap;

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

    /// Perform a comprehensive audit of a deck.
    ///
    /// Returns detailed information about deck contents including card counts,
    /// tag distribution, empty fields, duplicates, and scheduling state.
    ///
    /// # Arguments
    ///
    /// * `deck` - Deck name to audit
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let audit = engine.analyze().deck_audit("Japanese").await?;
    ///
    /// println!("Deck: {}", audit.deck);
    /// println!("Total cards: {}", audit.total_cards);
    /// println!("Total notes: {}", audit.total_notes);
    /// println!("Leeches: {}", audit.leech_count);
    /// println!("Suspended: {}", audit.suspended_count);
    /// println!("New: {}, Learning: {}, Review: {}",
    ///     audit.new_cards, audit.learning_cards, audit.review_cards);
    ///
    /// for (model, count) in &audit.cards_by_model {
    ///     println!("  {}: {} cards", model, count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deck_audit(&self, deck: &str) -> Result<DeckAudit> {
        let mut audit = DeckAudit {
            deck: deck.to_string(),
            ..Default::default()
        };

        let query = format!("deck:\"{}\"", deck);

        // Get all cards in deck
        let card_ids = self.client.cards().find(&query).await?;
        audit.total_cards = card_ids.len();

        if card_ids.is_empty() {
            return Ok(audit);
        }

        // Get card info for scheduling and model analysis
        let cards = self.client.cards().info(&card_ids).await?;

        // Count by model and scheduling state
        let mut ease_sum: i64 = 0;
        let mut ease_count: usize = 0;

        for card in &cards {
            // Count by model
            *audit
                .cards_by_model
                .entry(card.model_name.clone())
                .or_insert(0) += 1;

            // Count by scheduling state (card_type: 0=new, 1=learning, 2=review, 3=relearning)
            match card.card_type {
                0 => audit.new_cards += 1,
                1 | 3 => audit.learning_cards += 1,
                2 => audit.review_cards += 1,
                _ => {}
            }

            // Check suspended (queue == -1)
            if card.queue == -1 {
                audit.suspended_count += 1;
            }

            // Check leech (high lapses, default threshold 8)
            if card.lapses >= 8 {
                audit.leech_count += 1;
            }

            // Accumulate ease for average
            if card.ease_factor > 0 {
                ease_sum += card.ease_factor;
                ease_count += 1;
            }
        }

        // Calculate average ease
        if ease_count > 0 {
            audit.average_ease = ease_sum as f64 / ease_count as f64;
        }

        // Get all notes in deck
        let note_ids = self.client.notes().find(&query).await?;
        audit.total_notes = note_ids.len();

        if !note_ids.is_empty() {
            let notes = self.client.notes().info(&note_ids).await?;

            // Tag distribution and untagged count
            for note in &notes {
                if note.tags.is_empty() {
                    audit.untagged_notes += 1;
                } else {
                    for tag in &note.tags {
                        *audit.tag_distribution.entry(tag.clone()).or_insert(0) += 1;
                    }
                }
            }

            // Empty field analysis - collect all field names and check which are empty
            let mut field_names: HashMap<String, bool> = HashMap::new();
            for note in &notes {
                for (field_name, field_value) in &note.fields {
                    field_names.insert(field_name.clone(), true);
                    if field_value.value.trim().is_empty() {
                        *audit
                            .empty_field_counts
                            .entry(field_name.clone())
                            .or_insert(0) += 1;
                    }
                }
            }

            // Duplicate detection - use first field as key
            let mut seen_values: HashMap<String, usize> = HashMap::new();
            for note in &notes {
                // Get the first field value (sorted by order)
                if let Some(first_field) = note
                    .fields
                    .values()
                    .min_by_key(|f| f.order)
                    .map(|f| f.value.trim().to_lowercase())
                {
                    if !first_field.is_empty() {
                        *seen_values.entry(first_field).or_insert(0) += 1;
                    }
                }
            }

            // Count duplicates (values that appear more than once)
            audit.duplicate_count = seen_values.values().filter(|&&count| count > 1).count();
        }

        Ok(audit)
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

/// Comprehensive audit of a deck's contents and health.
///
/// Combines multiple analyses into a single report including card counts,
/// tag distribution, empty fields, duplicates, and scheduling state.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DeckAudit {
    /// The deck name.
    pub deck: String,
    /// Total number of cards.
    pub total_cards: usize,
    /// Total number of notes.
    pub total_notes: usize,

    // Card counts by model
    /// Number of cards per note type (model).
    pub cards_by_model: HashMap<String, usize>,

    // Tag coverage
    /// Number of notes per tag.
    pub tag_distribution: HashMap<String, usize>,
    /// Number of notes without any tags.
    pub untagged_notes: usize,

    // Field analysis
    /// Number of notes with each field empty (field name -> count).
    pub empty_field_counts: HashMap<String, usize>,

    // Duplicates
    /// Number of potential duplicate notes detected.
    pub duplicate_count: usize,

    // Problem cards
    /// Number of leech cards (high lapses).
    pub leech_count: usize,
    /// Number of suspended cards.
    pub suspended_count: usize,

    // Scheduling summary
    /// Number of new cards (never reviewed).
    pub new_cards: usize,
    /// Number of cards in learning phase.
    pub learning_cards: usize,
    /// Number of review cards.
    pub review_cards: usize,
    /// Average ease factor (percentage * 10, e.g., 2500 = 250%).
    pub average_ease: f64,
}
