//! Deck-related types.

use serde::{Deserialize, Serialize};

/// Statistics for a deck.
///
/// Note: The deck ID is provided as the key in the HashMap returned by
/// [`DeckActions::stats()`](crate::actions::DeckActions::stats), not as a field here.
#[derive(Debug, Clone, Deserialize)]
pub struct DeckStats {
    /// The deck name.
    pub name: String,
    /// Number of new cards.
    #[serde(default, alias = "newCount", alias = "new_count")]
    pub new_count: i64,
    /// Number of cards in learning.
    #[serde(default, alias = "learnCount", alias = "learn_count")]
    pub learn_count: i64,
    /// Number of cards due for review.
    #[serde(default, alias = "reviewCount", alias = "review_count")]
    pub review_count: i64,
    /// Total number of cards in the deck.
    #[serde(default, alias = "totalInDeck", alias = "total_in_deck")]
    pub total_in_deck: i64,
}

/// Configuration for a deck.
///
/// This represents the study options for a deck, including settings for
/// new cards, reviews, and lapses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckConfig {
    /// The config ID.
    pub id: i64,
    /// The config name.
    pub name: String,
    /// Maximum reviews per day.
    #[serde(default)]
    pub max_taken: i64,
    /// Whether to replay question audio when showing answer.
    #[serde(default)]
    pub replayq: bool,
    /// Whether this is the autoplay setting.
    #[serde(default)]
    pub autoplay: bool,
    /// Timer setting.
    #[serde(default)]
    pub timer: i64,
    /// New card settings.
    pub new: NewCardConfig,
    /// Review settings.
    pub rev: ReviewConfig,
    /// Lapse settings.
    pub lapse: LapseConfig,
}

/// Configuration for new cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCardConfig {
    /// Learning steps in minutes.
    #[serde(default)]
    pub delays: Vec<f64>,
    /// Order of new cards (0 = random, 1 = due).
    #[serde(default)]
    pub order: i64,
    /// Initial ease factor (as integer, e.g., 2500 = 250%).
    #[serde(default)]
    pub initial_factor: i64,
    /// Whether to separate new cards by day.
    #[serde(default)]
    pub separate: bool,
    /// Graduating interval in days.
    #[serde(default)]
    pub ints: Vec<i64>,
    /// Maximum new cards per day.
    #[serde(default)]
    pub per_day: i64,
}

/// Configuration for reviews.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewConfig {
    /// Maximum reviews per day.
    #[serde(default)]
    pub per_day: i64,
    /// Easy bonus multiplier.
    #[serde(default)]
    pub ease4: f64,
    /// Interval modifier.
    #[serde(default)]
    pub fuzz: f64,
    /// Minimum interval.
    #[serde(default)]
    pub min_space: i64,
    /// Maximum interval in days.
    #[serde(default)]
    pub max_ivl: i64,
    /// Whether to bury related reviews.
    #[serde(default)]
    pub bury: bool,
    /// Hard interval multiplier.
    #[serde(default)]
    pub hard_factor: f64,
}

/// Configuration for lapses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LapseConfig {
    /// Relearning steps in minutes.
    #[serde(default)]
    pub delays: Vec<f64>,
    /// Leech threshold.
    #[serde(default)]
    pub leech_fails: i64,
    /// Leech action (0 = suspend, 1 = tag only).
    #[serde(default)]
    pub leech_action: i64,
    /// Minimum interval after lapse.
    #[serde(default)]
    pub min_int: i64,
    /// New interval multiplier after lapse.
    #[serde(default)]
    pub mult: f64,
}
