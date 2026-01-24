//! Card-related types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::NoteField;

/// Information about a card.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardInfo {
    /// The card ID.
    pub card_id: i64,
    /// The note ID this card was generated from.
    #[serde(default, alias = "nid")]
    pub note_id: i64,
    /// The deck ID.
    #[serde(default)]
    pub deck_id: i64,
    /// The deck this card belongs to.
    #[serde(default)]
    pub deck_name: String,
    /// The note type (model) name.
    #[serde(default)]
    pub model_name: String,
    /// The card's question side (HTML).
    #[serde(default)]
    pub question: String,
    /// The card's answer side (HTML).
    #[serde(default)]
    pub answer: String,
    /// Field values from the note.
    #[serde(default)]
    pub fields: HashMap<String, NoteField>,
    /// The card type (0 = new, 1 = learning, 2 = review, 3 = relearning).
    #[serde(default, rename = "type")]
    pub card_type: i32,
    /// The queue the card is in (-1 = suspended, -2 = sibling buried, -3 = manually buried,
    /// 0 = new, 1 = learning, 2 = review, 3 = day learn, 4 = preview).
    #[serde(default)]
    pub queue: i32,
    /// Due position/date (meaning depends on card type).
    #[serde(default)]
    pub due: i64,
    /// Current interval in days.
    #[serde(default)]
    pub interval: i64,
    /// Ease factor (as integer, e.g., 2500 = 250%).
    #[serde(default, alias = "factor")]
    pub ease_factor: i64,
    /// Number of reviews.
    #[serde(default)]
    pub reps: i64,
    /// Number of lapses.
    #[serde(default)]
    pub lapses: i64,
    /// Number of reviews left today.
    #[serde(default)]
    pub left: i64,
    /// Last modification timestamp.
    #[serde(default, alias = "mod")]
    pub mod_time: i64,
}

/// Modification time information for a card.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardModTime {
    /// The card ID.
    pub card_id: i64,
    /// Modification timestamp (seconds since epoch).
    #[serde(rename = "mod")]
    pub mod_time: i64,
}

/// Answer ease for reviewing cards.
///
/// The meaning of each ease depends on the card state:
/// - For new/learning cards: Again, Hard, Good, Easy
/// - For review cards: Again (lapse), Hard, Good, Easy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum Ease {
    /// Mark the card as failed (Again).
    Again = 1,
    /// Mark the card as hard.
    Hard = 2,
    /// Mark the card as good.
    Good = 3,
    /// Mark the card as easy.
    Easy = 4,
}

/// Answer for a card review.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardAnswer {
    /// The card ID to answer.
    pub card_id: i64,
    /// The ease rating.
    pub ease: Ease,
}

impl CardAnswer {
    /// Create a new card answer.
    pub fn new(card_id: i64, ease: Ease) -> Self {
        Self { card_id, ease }
    }
}

impl From<Ease> for i32 {
    fn from(ease: Ease) -> i32 {
        ease as i32
    }
}
