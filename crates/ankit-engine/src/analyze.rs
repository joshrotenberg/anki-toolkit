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

    /// Generate a comprehensive study report.
    ///
    /// Combines multiple statistics into a single overview including activity summary,
    /// performance metrics, problem cards, and upcoming workload.
    ///
    /// # Arguments
    ///
    /// * `deck` - Deck to analyze (use "*" for all decks)
    /// * `days` - Number of days to include in the report
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let report = engine.analyze().study_report("Japanese", 7).await?;
    ///
    /// println!("Study Report for {}", report.deck);
    /// println!("Reviews: {} ({:.1}/day)", report.total_reviews, report.average_reviews_per_day);
    /// println!("Retention: {:.1}%", report.retention_rate * 100.0);
    /// println!("Study streak: {} days", report.study_streak);
    /// println!("Leeches: {}", report.leeches.len());
    /// println!("Due tomorrow: {}", report.due_tomorrow);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn study_report(&self, deck: &str, days: u32) -> Result<StudyReport> {
        let mut report = StudyReport {
            deck: deck.to_string(),
            period_days: days,
            ..Default::default()
        };

        // Get daily review counts
        let daily_reviews = self.client.statistics().cards_reviewed_by_day().await?;
        let take_days = days as usize;
        let recent: Vec<_> = daily_reviews.into_iter().take(take_days).collect();

        // Calculate activity metrics
        for (date, count) in &recent {
            report.total_reviews += *count as usize;
            report.daily_stats.push(ReportDailyStats {
                date: date.clone(),
                reviews: *count as usize,
            });
        }

        if !recent.is_empty() {
            report.average_reviews_per_day = report.total_reviews as f64 / recent.len() as f64;
        }

        // Calculate study streak (consecutive days with reviews from most recent)
        report.study_streak = recent.iter().take_while(|(_, count)| *count > 0).count() as u32;

        // Build query for deck-specific stats
        let review_query = if deck == "*" {
            "is:review".to_string()
        } else {
            format!("deck:\"{}\" is:review", deck)
        };

        let review_card_ids = self.client.cards().find(&review_query).await?;

        if !review_card_ids.is_empty() {
            let cards = self.client.cards().info(&review_card_ids).await?;

            // Calculate retention and ease
            let total_lapses: i64 = cards.iter().map(|c| c.lapses).sum();
            let total_reps: i64 = cards.iter().map(|c| c.reps).sum();

            if total_reps > 0 {
                report.retention_rate = 1.0 - (total_lapses as f64 / total_reps as f64);
            }

            let ease_values: Vec<i64> = cards
                .iter()
                .filter(|c| c.ease_factor > 0)
                .map(|c| c.ease_factor)
                .collect();

            if !ease_values.is_empty() {
                report.average_ease =
                    ease_values.iter().sum::<i64>() as f64 / ease_values.len() as f64;
            }

            // Find problem cards
            for card in &cards {
                // Leeches: 8+ lapses (Anki default)
                if card.lapses >= 8 {
                    report.leeches.push(card.card_id);
                }
                // Low ease: below 200% (2000)
                if card.ease_factor > 0 && card.ease_factor < 2000 {
                    report.low_ease_cards.push(card.card_id);
                }
            }

            // Count relearning cards
            report.relearning_cards = cards.iter().filter(|c| c.card_type == 3).count();
        }

        // Get cards studied in period (rated:N query)
        if deck != "*" {
            let rated_query = format!("deck:\"{}\" rated:{}", deck, days);
            let rated_cards = self.client.cards().find(&rated_query).await?;

            if !rated_cards.is_empty() {
                let card_infos = self.client.cards().info(&rated_cards).await?;

                // Count by type
                for card in &card_infos {
                    match card.card_type {
                        0 => report.new_cards_studied += 1,
                        2 => report.review_cards_studied += 1,
                        _ => {}
                    }
                }
            }
        }

        // Get upcoming workload
        let due_tomorrow_query = if deck == "*" {
            "prop:due=1".to_string()
        } else {
            format!("deck:\"{}\" prop:due=1", deck)
        };
        let due_tomorrow_cards = self.client.cards().find(&due_tomorrow_query).await?;
        report.due_tomorrow = due_tomorrow_cards.len();

        let due_week_query = if deck == "*" {
            "prop:due<=7".to_string()
        } else {
            format!("deck:\"{}\" prop:due<=7", deck)
        };
        let due_week_cards = self.client.cards().find(&due_week_query).await?;
        report.due_this_week = due_week_cards.len();

        Ok(report)
    }

    /// Compare two decks for overlap and differences.
    ///
    /// Analyzes notes in both decks based on a key field, identifying:
    /// - Notes unique to each deck
    /// - Exact matches (identical key field values)
    /// - Similar notes (fuzzy matching above threshold)
    ///
    /// # Arguments
    ///
    /// * `deck_a` - Name of the first deck
    /// * `deck_b` - Name of the second deck
    /// * `options` - Comparison options (key field and similarity threshold)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # use ankit_engine::analyze::CompareOptions;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let comparison = engine.analyze()
    ///     .compare_decks("Japanese::Core", "Japanese::Extra", CompareOptions {
    ///         key_field: "Front".to_string(),
    ///         similarity_threshold: 0.85,
    ///     })
    ///     .await?;
    ///
    /// println!("Only in Core: {}", comparison.only_in_a.len());
    /// println!("Only in Extra: {}", comparison.only_in_b.len());
    /// println!("Exact matches: {}", comparison.exact_matches.len());
    /// println!("Similar: {}", comparison.similar.len());
    ///
    /// for pair in &comparison.similar {
    ///     println!("  {:.0}% similar: '{}' vs '{}'",
    ///         pair.similarity * 100.0,
    ///         pair.note_a.key_value,
    ///         pair.note_b.key_value);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn compare_decks(
        &self,
        deck_a: &str,
        deck_b: &str,
        options: CompareOptions,
    ) -> Result<DeckComparison> {
        let mut comparison = DeckComparison {
            deck_a: deck_a.to_string(),
            deck_b: deck_b.to_string(),
            key_field: options.key_field.clone(),
            similarity_threshold: options.similarity_threshold,
            ..Default::default()
        };

        // Get notes from both decks
        let query_a = format!("deck:\"{}\"", deck_a);
        let query_b = format!("deck:\"{}\"", deck_b);

        let note_ids_a = self.client.notes().find(&query_a).await?;
        let note_ids_b = self.client.notes().find(&query_b).await?;

        if note_ids_a.is_empty() && note_ids_b.is_empty() {
            return Ok(comparison);
        }

        // Get note info
        let notes_a = if note_ids_a.is_empty() {
            Vec::new()
        } else {
            self.client.notes().info(&note_ids_a).await?
        };

        let notes_b = if note_ids_b.is_empty() {
            Vec::new()
        } else {
            self.client.notes().info(&note_ids_b).await?
        };

        // Extract key field values
        let extract_key = |note: &ankit::NoteInfo| -> Option<(i64, String, Vec<String>)> {
            note.fields
                .get(&options.key_field)
                .map(|f| (note.note_id, f.value.trim().to_string(), note.tags.clone()))
        };

        let keys_a: Vec<_> = notes_a.iter().filter_map(extract_key).collect();
        let keys_b: Vec<_> = notes_b.iter().filter_map(extract_key).collect();

        // Build lookup map for deck B (for exact matching from A)
        let map_b: HashMap<String, (i64, Vec<String>)> = keys_b
            .iter()
            .map(|(id, key, tags)| (key.to_lowercase(), (*id, tags.clone())))
            .collect();

        // Track which notes have been matched
        let mut matched_in_a: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut matched_in_b: std::collections::HashSet<i64> = std::collections::HashSet::new();

        // Find exact matches
        for (note_id_a, key_a, tags_a) in &keys_a {
            let key_lower = key_a.to_lowercase();
            if let Some((note_id_b, tags_b)) = map_b.get(&key_lower) {
                matched_in_a.insert(*note_id_a);
                matched_in_b.insert(*note_id_b);

                comparison.exact_matches.push((
                    ComparisonNote {
                        note_id: *note_id_a,
                        key_value: key_a.clone(),
                        tags: tags_a.clone(),
                    },
                    ComparisonNote {
                        note_id: *note_id_b,
                        key_value: key_a.clone(), // Same value
                        tags: tags_b.clone(),
                    },
                ));
            }
        }

        // Find similar matches (only for unmatched notes)
        if options.similarity_threshold < 1.0 {
            for (note_id_a, key_a, tags_a) in &keys_a {
                if matched_in_a.contains(note_id_a) {
                    continue;
                }

                for (note_id_b, key_b, tags_b) in &keys_b {
                    if matched_in_b.contains(note_id_b) {
                        continue;
                    }

                    let similarity = string_similarity(key_a, key_b);
                    if similarity >= options.similarity_threshold {
                        matched_in_a.insert(*note_id_a);
                        matched_in_b.insert(*note_id_b);

                        comparison.similar.push(SimilarPair {
                            note_a: ComparisonNote {
                                note_id: *note_id_a,
                                key_value: key_a.clone(),
                                tags: tags_a.clone(),
                            },
                            note_b: ComparisonNote {
                                note_id: *note_id_b,
                                key_value: key_b.clone(),
                                tags: tags_b.clone(),
                            },
                            similarity,
                        });

                        break; // Move to next note in A
                    }
                }
            }
        }

        // Collect unmatched notes
        for (note_id_a, key_a, tags_a) in &keys_a {
            if !matched_in_a.contains(note_id_a) {
                comparison.only_in_a.push(ComparisonNote {
                    note_id: *note_id_a,
                    key_value: key_a.clone(),
                    tags: tags_a.clone(),
                });
            }
        }

        for (note_id_b, key_b, tags_b) in &keys_b {
            if !matched_in_b.contains(note_id_b) {
                comparison.only_in_b.push(ComparisonNote {
                    note_id: *note_id_b,
                    key_value: key_b.clone(),
                    tags: tags_b.clone(),
                });
            }
        }

        Ok(comparison)
    }
}

/// Calculate string similarity using normalized Levenshtein distance.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
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

    // Use two rows instead of full matrix for memory efficiency
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

            curr[j] = (prev[j] + 1) // deletion
                .min(curr[j - 1] + 1) // insertion
                .min(prev[j - 1] + cost); // substitution
        }

        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Comprehensive study report combining multiple statistics.
///
/// Provides a complete overview of study activity, performance, problem areas,
/// and upcoming workload for a deck over a specified time period.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StudyReport {
    /// The deck name (or "*" for all decks).
    pub deck: String,
    /// Number of days covered by this report.
    pub period_days: u32,

    // Activity summary
    /// Total number of reviews in the period.
    pub total_reviews: usize,
    /// Total time spent studying in minutes.
    pub total_time_minutes: u64,
    /// Average reviews per day.
    pub average_reviews_per_day: f64,
    /// Consecutive days with at least one review.
    pub study_streak: u32,

    // Performance metrics
    /// Estimated retention rate (0.0 - 1.0).
    pub retention_rate: f64,
    /// Average ease factor (percentage * 10, e.g., 2500 = 250%).
    pub average_ease: f64,

    // Cards reviewed breakdown
    /// Number of new cards studied in the period.
    pub new_cards_studied: usize,
    /// Number of review cards studied in the period.
    pub review_cards_studied: usize,
    /// Number of cards in relearning state.
    pub relearning_cards: usize,

    // Problem areas (card IDs)
    /// Card IDs flagged as leeches (high lapses).
    pub leeches: Vec<i64>,
    /// Card IDs with low ease factor (below 200%).
    pub low_ease_cards: Vec<i64>,

    // Upcoming workload
    /// Number of cards due tomorrow.
    pub due_tomorrow: usize,
    /// Number of cards due within the next 7 days.
    pub due_this_week: usize,

    // Daily breakdown
    /// Statistics for each day in the period.
    pub daily_stats: Vec<ReportDailyStats>,
}

/// Daily statistics for a study report.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReportDailyStats {
    /// Date in YYYY-MM-DD format.
    pub date: String,
    /// Number of reviews on this day.
    pub reviews: usize,
}

/// Options for comparing two decks.
#[derive(Debug, Clone)]
pub struct CompareOptions {
    /// Field name to use as the comparison key (e.g., "Front").
    pub key_field: String,
    /// Similarity threshold for fuzzy matching (0.0 - 1.0).
    /// Cards with similarity >= this value are considered similar.
    /// Set to 1.0 for exact matches only.
    pub similarity_threshold: f64,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            key_field: "Front".to_string(),
            similarity_threshold: 0.9,
        }
    }
}

/// Result of comparing two decks.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DeckComparison {
    /// Name of the first deck.
    pub deck_a: String,
    /// Name of the second deck.
    pub deck_b: String,
    /// Field used for comparison.
    pub key_field: String,
    /// Similarity threshold used.
    pub similarity_threshold: f64,

    /// Notes only in deck A (not in B).
    pub only_in_a: Vec<ComparisonNote>,
    /// Notes only in deck B (not in A).
    pub only_in_b: Vec<ComparisonNote>,
    /// Notes with exact matching key field values.
    pub exact_matches: Vec<(ComparisonNote, ComparisonNote)>,
    /// Notes with similar (but not exact) key field values.
    pub similar: Vec<SimilarPair>,
}

/// A note in a comparison result.
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonNote {
    /// The note ID.
    pub note_id: i64,
    /// The value of the key field.
    pub key_value: String,
    /// The note's tags.
    pub tags: Vec<String>,
}

/// A pair of similar notes from two decks.
#[derive(Debug, Clone, Serialize)]
pub struct SimilarPair {
    /// Note from deck A.
    pub note_a: ComparisonNote,
    /// Note from deck B.
    pub note_b: ComparisonNote,
    /// Similarity score (0.0 - 1.0).
    pub similarity: f64,
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
