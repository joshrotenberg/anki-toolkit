//! Progress management and card state operations.
//!
//! This module provides workflows for managing card progress, including
//! resetting progress, tagging cards by performance, and bulk tag operations.

use std::collections::HashSet;

use crate::Result;
use ankit::AnkiClient;
use serde::Serialize;

/// Report from resetting deck progress.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResetReport {
    /// Number of cards reset to new state.
    pub cards_reset: usize,
    /// Deck that was reset.
    pub deck: String,
}

/// Criteria for categorizing card performance.
#[derive(Debug, Clone)]
pub struct PerformanceCriteria {
    /// Ease factor threshold for struggling (below this is struggling).
    pub struggling_ease: i64,
    /// Lapse count threshold for struggling (above this is struggling).
    pub struggling_lapses: i64,
    /// Ease factor threshold for mastered (above this is mastered).
    pub mastered_ease: i64,
    /// Minimum reps required for mastered status.
    pub mastered_min_reps: i64,
}

impl Default for PerformanceCriteria {
    fn default() -> Self {
        Self {
            struggling_ease: 2100, // Below 210%
            struggling_lapses: 3,  // More than 3 lapses
            mastered_ease: 2500,   // Above 250%
            mastered_min_reps: 5,  // At least 5 reviews
        }
    }
}

/// Report from tagging cards by performance.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TagReport {
    /// Number of notes tagged as struggling.
    pub struggling_count: usize,
    /// Number of notes tagged as mastered.
    pub mastered_count: usize,
    /// Tag used for struggling cards.
    pub struggling_tag: String,
    /// Tag used for mastered cards.
    pub mastered_tag: String,
}

/// Criteria for suspending cards.
#[derive(Debug, Clone)]
pub struct SuspendCriteria {
    /// Maximum ease factor (cards with ease below this may be suspended).
    pub max_ease: i64,
    /// Minimum lapse count (cards with lapses above this may be suspended).
    pub min_lapses: i64,
    /// Whether both conditions must be met (AND) or just one (OR).
    pub require_both: bool,
}

impl Default for SuspendCriteria {
    fn default() -> Self {
        Self {
            max_ease: 1800,     // Below 180%
            min_lapses: 5,      // More than 5 lapses
            require_both: true, // Both conditions must be met
        }
    }
}

/// Report from suspending cards by criteria.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SuspendReport {
    /// Number of cards suspended.
    pub cards_suspended: usize,
    /// Card IDs that were suspended.
    pub suspended_ids: Vec<i64>,
}

/// Comprehensive health report for a deck.
#[derive(Debug, Clone, Default, Serialize)]
pub struct HealthReport {
    /// Deck name.
    pub deck: String,
    /// Total number of cards.
    pub total_cards: usize,
    /// Number of new cards.
    pub new_cards: usize,
    /// Number of learning cards.
    pub learning_cards: usize,
    /// Number of review cards.
    pub review_cards: usize,
    /// Number of suspended cards.
    pub suspended_cards: usize,
    /// Number of buried cards.
    pub buried_cards: usize,
    /// Average ease factor (percentage * 10).
    pub avg_ease: i64,
    /// Average interval in days.
    pub avg_interval: i64,
    /// Number of leech cards (high lapses).
    pub leech_count: usize,
    /// Total lapses across all cards.
    pub total_lapses: i64,
    /// Total reviews across all cards.
    pub total_reps: i64,
}

/// Tag operation to perform.
#[derive(Debug, Clone)]
pub enum TagOperation {
    /// Add tags to notes.
    Add(String),
    /// Remove tags from notes.
    Remove(String),
    /// Replace one tag with another.
    Replace { old: String, new: String },
}

/// Report from bulk tag operation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BulkTagReport {
    /// Number of notes affected.
    pub notes_affected: usize,
    /// Operation performed.
    pub operation: String,
}

/// Criteria for smart suspension based on content similarity.
#[derive(Debug, Clone)]
pub struct SimilarityCriteria {
    /// Similarity threshold (0.0 - 1.0). Cards with similarity >= this are grouped.
    pub threshold: f64,
    /// Field to compare for similarity.
    pub field: String,
    /// Strategy for which card to keep in each similar group.
    pub keep_strategy: KeepStrategy,
    /// If true, don't actually suspend - just report what would be suspended.
    pub dry_run: bool,
}

impl Default for SimilarityCriteria {
    fn default() -> Self {
        Self {
            threshold: 0.85,
            field: "Front".to_string(),
            keep_strategy: KeepStrategy::MostMature,
            dry_run: false,
        }
    }
}

/// Strategy for which card to keep when suspending similar cards.
#[derive(Debug, Clone, Copy, Default)]
pub enum KeepStrategy {
    /// Keep the card with the highest interval (most mature).
    #[default]
    MostMature,
    /// Keep the card with the lowest interval (least mature).
    LeastMature,
    /// Keep the card with the highest ease factor.
    HighestEase,
    /// Keep the card with the most reviews.
    MostReviewed,
}

/// A group of similar cards.
#[derive(Debug, Clone, Serialize)]
pub struct SimilarGroup {
    /// Card ID of the card to keep.
    pub keep: i64,
    /// Card IDs of cards to suspend.
    pub suspend: Vec<i64>,
    /// The field value these cards share (from the kept card).
    pub field_value: String,
    /// Similarity score within the group (minimum pairwise).
    pub min_similarity: f64,
}

/// Report from smart suspension.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SmartSuspendReport {
    /// Number of cards analyzed.
    pub cards_analyzed: usize,
    /// Number of similar groups found.
    pub groups_found: usize,
    /// Number of cards suspended.
    pub cards_suspended: usize,
    /// Number of cards kept (one per group).
    pub cards_kept: usize,
    /// Details of each similar group.
    pub groups: Vec<SimilarGroup>,
    /// Whether this was a dry run.
    pub dry_run: bool,
}

/// Progress management workflow engine.
#[derive(Debug)]
pub struct ProgressEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> ProgressEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Reset all cards in a deck to new state.
    ///
    /// This clears all learning progress for the deck.
    ///
    /// # Arguments
    ///
    /// * `deck` - Name of the deck to reset
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.progress().reset_deck("Test Deck").await?;
    /// println!("Reset {} cards", report.cards_reset);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset_deck(&self, deck: &str) -> Result<ResetReport> {
        let query = format!("deck:\"{}\"", deck);
        let card_ids = self.client.cards().find(&query).await?;

        if !card_ids.is_empty() {
            self.client.cards().forget(&card_ids).await?;
        }

        Ok(ResetReport {
            cards_reset: card_ids.len(),
            deck: deck.to_string(),
        })
    }

    /// Tag cards based on their performance.
    ///
    /// Cards are categorized as "struggling" or "mastered" based on
    /// ease factor, lapse count, and review count.
    ///
    /// # Arguments
    ///
    /// * `query` - Anki search query to filter cards
    /// * `criteria` - Criteria for categorization
    /// * `struggling_tag` - Tag to apply to struggling cards
    /// * `mastered_tag` - Tag to apply to mastered cards
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::progress::PerformanceCriteria;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.progress()
    ///     .tag_by_performance(
    ///         "deck:Japanese",
    ///         PerformanceCriteria::default(),
    ///         "struggling",
    ///         "mastered"
    ///     )
    ///     .await?;
    /// println!("{} struggling, {} mastered", report.struggling_count, report.mastered_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn tag_by_performance(
        &self,
        query: &str,
        criteria: PerformanceCriteria,
        struggling_tag: &str,
        mastered_tag: &str,
    ) -> Result<TagReport> {
        let card_ids = self.client.cards().find(query).await?;

        if card_ids.is_empty() {
            return Ok(TagReport {
                struggling_tag: struggling_tag.to_string(),
                mastered_tag: mastered_tag.to_string(),
                ..Default::default()
            });
        }

        let cards = self.client.cards().info(&card_ids).await?;

        let mut struggling_notes = HashSet::new();
        let mut mastered_notes = HashSet::new();

        for card in cards {
            let is_struggling = card.ease_factor > 0
                && (card.ease_factor < criteria.struggling_ease
                    || card.lapses > criteria.struggling_lapses);

            let is_mastered = card.ease_factor >= criteria.mastered_ease
                && card.reps >= criteria.mastered_min_reps;

            if is_struggling {
                struggling_notes.insert(card.note_id);
            } else if is_mastered {
                mastered_notes.insert(card.note_id);
            }
        }

        // Apply tags
        let struggling_ids: Vec<_> = struggling_notes.into_iter().collect();
        let mastered_ids: Vec<_> = mastered_notes.into_iter().collect();

        if !struggling_ids.is_empty() {
            self.client
                .notes()
                .add_tags(&struggling_ids, struggling_tag)
                .await?;
        }

        if !mastered_ids.is_empty() {
            self.client
                .notes()
                .add_tags(&mastered_ids, mastered_tag)
                .await?;
        }

        Ok(TagReport {
            struggling_count: struggling_ids.len(),
            mastered_count: mastered_ids.len(),
            struggling_tag: struggling_tag.to_string(),
            mastered_tag: mastered_tag.to_string(),
        })
    }

    /// Suspend cards matching performance criteria.
    ///
    /// # Arguments
    ///
    /// * `query` - Anki search query to filter cards
    /// * `criteria` - Criteria for suspension
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::progress::SuspendCriteria;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.progress()
    ///     .suspend_by_criteria("deck:Japanese", SuspendCriteria::default())
    ///     .await?;
    /// println!("Suspended {} cards", report.cards_suspended);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn suspend_by_criteria(
        &self,
        query: &str,
        criteria: SuspendCriteria,
    ) -> Result<SuspendReport> {
        let card_ids = self.client.cards().find(query).await?;

        if card_ids.is_empty() {
            return Ok(SuspendReport::default());
        }

        let cards = self.client.cards().info(&card_ids).await?;

        let mut to_suspend = Vec::new();

        for card in cards {
            // Skip already suspended cards
            if card.queue == -1 {
                continue;
            }

            let low_ease = card.ease_factor > 0 && card.ease_factor < criteria.max_ease;
            let high_lapses = card.lapses >= criteria.min_lapses;

            let should_suspend = if criteria.require_both {
                low_ease && high_lapses
            } else {
                low_ease || high_lapses
            };

            if should_suspend {
                to_suspend.push(card.card_id);
            }
        }

        if !to_suspend.is_empty() {
            self.client.cards().suspend(&to_suspend).await?;
        }

        Ok(SuspendReport {
            cards_suspended: to_suspend.len(),
            suspended_ids: to_suspend,
        })
    }

    /// Get comprehensive health report for a deck.
    ///
    /// # Arguments
    ///
    /// * `deck` - Deck name to analyze
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.progress().deck_health("Japanese").await?;
    /// println!("Total: {}, Suspended: {}, Leeches: {}",
    ///     report.total_cards, report.suspended_cards, report.leech_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deck_health(&self, deck: &str) -> Result<HealthReport> {
        let query = format!("deck:\"{}\"", deck);
        let card_ids = self.client.cards().find(&query).await?;

        if card_ids.is_empty() {
            return Ok(HealthReport {
                deck: deck.to_string(),
                ..Default::default()
            });
        }

        let cards = self.client.cards().info(&card_ids).await?;

        let mut report = HealthReport {
            deck: deck.to_string(),
            total_cards: cards.len(),
            ..Default::default()
        };

        let mut total_ease: i64 = 0;
        let mut ease_count: usize = 0;
        let mut total_interval: i64 = 0;
        let mut interval_count: usize = 0;

        for card in &cards {
            // Card type: 0=new, 1=learning, 2=review, 3=relearning
            // Queue: -1=suspended, -2=sibling buried, -3=manually buried, 0=new, 1=learning, 2=review
            match card.queue {
                -1 => report.suspended_cards += 1,
                -2 | -3 => report.buried_cards += 1,
                0 => report.new_cards += 1,
                1 | 3 => report.learning_cards += 1,
                2 => report.review_cards += 1,
                _ => {}
            }

            if card.ease_factor > 0 {
                total_ease += card.ease_factor;
                ease_count += 1;
            }

            if card.interval > 0 {
                total_interval += card.interval;
                interval_count += 1;
            }

            report.total_lapses += card.lapses;
            report.total_reps += card.reps;

            // Leech threshold: 8+ lapses (Anki's default)
            if card.lapses >= 8 {
                report.leech_count += 1;
            }
        }

        if ease_count > 0 {
            report.avg_ease = total_ease / ease_count as i64;
        }

        if interval_count > 0 {
            report.avg_interval = total_interval / interval_count as i64;
        }

        Ok(report)
    }

    /// Perform bulk tag operation on notes matching a query.
    ///
    /// # Arguments
    ///
    /// * `query` - Anki search query to filter notes
    /// * `operation` - Tag operation to perform
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::progress::TagOperation;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// // Add tags
    /// let report = engine.progress()
    ///     .bulk_tag("deck:Japanese", TagOperation::Add("needs-review".to_string()))
    ///     .await?;
    ///
    /// // Remove tags
    /// let report = engine.progress()
    ///     .bulk_tag("deck:Japanese", TagOperation::Remove("old-tag".to_string()))
    ///     .await?;
    ///
    /// // Replace tags
    /// let report = engine.progress()
    ///     .bulk_tag("deck:Japanese", TagOperation::Replace {
    ///         old: "v1".to_string(),
    ///         new: "v2".to_string(),
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bulk_tag(&self, query: &str, operation: TagOperation) -> Result<BulkTagReport> {
        let note_ids = self.client.notes().find(query).await?;

        if note_ids.is_empty() {
            return Ok(BulkTagReport {
                operation: format!("{:?}", operation),
                ..Default::default()
            });
        }

        let op_description = match &operation {
            TagOperation::Add(tags) => {
                self.client.notes().add_tags(&note_ids, tags).await?;
                format!("Added '{}'", tags)
            }
            TagOperation::Remove(tags) => {
                self.client.notes().remove_tags(&note_ids, tags).await?;
                format!("Removed '{}'", tags)
            }
            TagOperation::Replace { old, new } => {
                // Replace on specific notes
                self.client
                    .notes()
                    .replace_tags(&note_ids, old, new)
                    .await?;
                format!("Replaced '{}' with '{}'", old, new)
            }
        };

        Ok(BulkTagReport {
            notes_affected: note_ids.len(),
            operation: op_description,
        })
    }

    /// Suspend similar cards to reduce interference during learning.
    ///
    /// This workflow analyzes cards for content similarity and suspends
    /// all but one card from each group of similar cards.
    ///
    /// # Arguments
    ///
    /// * `query` - Anki search query to filter cards
    /// * `criteria` - Similarity criteria and options
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::progress::{SimilarityCriteria, KeepStrategy};
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// // First do a dry run to see what would be suspended
    /// let report = engine.progress()
    ///     .smart_suspend("deck:Japanese", SimilarityCriteria {
    ///         threshold: 0.85,
    ///         field: "Front".to_string(),
    ///         keep_strategy: KeepStrategy::MostMature,
    ///         dry_run: true,
    ///     })
    ///     .await?;
    ///
    /// println!("Would suspend {} cards in {} groups",
    ///     report.cards_suspended, report.groups_found);
    ///
    /// // Then actually suspend
    /// let report = engine.progress()
    ///     .smart_suspend("deck:Japanese", SimilarityCriteria {
    ///         dry_run: false,
    ///         ..SimilarityCriteria::default()
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn smart_suspend(
        &self,
        query: &str,
        criteria: SimilarityCriteria,
    ) -> Result<SmartSuspendReport> {
        let card_ids = self.client.cards().find(query).await?;

        if card_ids.is_empty() {
            return Ok(SmartSuspendReport {
                dry_run: criteria.dry_run,
                ..Default::default()
            });
        }

        let cards = self.client.cards().info(&card_ids).await?;

        // Get note info for field values
        let note_ids: Vec<i64> = cards.iter().map(|c| c.note_id).collect();
        let unique_note_ids: Vec<i64> = note_ids
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let notes = self.client.notes().info(&unique_note_ids).await?;

        // Build card -> field value mapping
        let note_fields: std::collections::HashMap<i64, String> = notes
            .into_iter()
            .filter_map(|n| {
                n.fields
                    .get(&criteria.field)
                    .map(|f| (n.note_id, f.value.trim().to_string()))
            })
            .collect();

        // Build list of (card_id, note_id, field_value, interval, ease, reps) for comparison
        let mut card_data: Vec<(i64, i64, String, i64, i64, i64)> = Vec::new();
        for card in &cards {
            // Skip already suspended cards
            if card.queue == -1 {
                continue;
            }

            if let Some(field_value) = note_fields.get(&card.note_id) {
                if !field_value.is_empty() {
                    card_data.push((
                        card.card_id,
                        card.note_id,
                        field_value.clone(),
                        card.interval,
                        card.ease_factor,
                        card.reps,
                    ));
                }
            }
        }

        if card_data.len() < 2 {
            return Ok(SmartSuspendReport {
                cards_analyzed: card_data.len(),
                dry_run: criteria.dry_run,
                ..Default::default()
            });
        }

        // Find similar groups using union-find
        let n = card_data.len();
        let mut parent: Vec<usize> = (0..n).collect();

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], i: usize, j: usize) {
            let pi = find(parent, i);
            let pj = find(parent, j);
            if pi != pj {
                parent[pi] = pj;
            }
        }

        // Compare all pairs and union similar cards
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = string_similarity(&card_data[i].2, &card_data[j].2);
                if sim >= criteria.threshold {
                    union(&mut parent, i, j);
                }
            }
        }

        // Group cards by their root
        let mut groups_map: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            groups_map.entry(root).or_default().push(i);
        }

        // Process groups with more than one card
        let mut report = SmartSuspendReport {
            cards_analyzed: card_data.len(),
            dry_run: criteria.dry_run,
            ..Default::default()
        };

        let mut to_suspend: Vec<i64> = Vec::new();

        for indices in groups_map.values() {
            if indices.len() < 2 {
                continue;
            }

            // Select which card to keep based on strategy
            let keep_idx = match criteria.keep_strategy {
                KeepStrategy::MostMature => {
                    *indices.iter().max_by_key(|&&i| card_data[i].3).unwrap()
                }
                KeepStrategy::LeastMature => {
                    *indices.iter().min_by_key(|&&i| card_data[i].3).unwrap()
                }
                KeepStrategy::HighestEase => {
                    *indices.iter().max_by_key(|&&i| card_data[i].4).unwrap()
                }
                KeepStrategy::MostReviewed => {
                    *indices.iter().max_by_key(|&&i| card_data[i].5).unwrap()
                }
            };

            let suspend_ids: Vec<i64> = indices
                .iter()
                .filter(|&&i| i != keep_idx)
                .map(|&i| card_data[i].0)
                .collect();

            // Calculate minimum similarity within group
            let mut min_sim = 1.0f64;
            for &i in indices {
                for &j in indices {
                    if i < j {
                        let sim = string_similarity(&card_data[i].2, &card_data[j].2);
                        min_sim = min_sim.min(sim);
                    }
                }
            }

            to_suspend.extend(&suspend_ids);

            report.groups.push(SimilarGroup {
                keep: card_data[keep_idx].0,
                suspend: suspend_ids,
                field_value: card_data[keep_idx].2.clone(),
                min_similarity: min_sim,
            });
        }

        report.groups_found = report.groups.len();
        report.cards_suspended = to_suspend.len();
        report.cards_kept = report.groups_found;

        // Actually suspend if not a dry run
        if !criteria.dry_run && !to_suspend.is_empty() {
            self.client.cards().suspend(&to_suspend).await?;
        }

        Ok(report)
    }
}

/// Calculate string similarity using normalized Levenshtein distance.
fn string_similarity(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 1.0;
    }

    if a_lower.is_empty() || b_lower.is_empty() {
        return 0.0;
    }

    let distance = levenshtein_distance(&a_lower, &b_lower);
    let max_len = a_lower.chars().count().max(b_lower.chars().count());

    1.0 - (distance as f64 / max_len as f64)
}

/// Calculate the Levenshtein distance between two strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;

        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };

            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }

        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}
