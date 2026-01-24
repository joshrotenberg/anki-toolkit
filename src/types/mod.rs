//! Domain types for AnkiConnect.
//!
//! This module contains the data structures used to represent Anki entities
//! like decks, notes, cards, and models.

mod card;
mod deck;
mod media;
mod model;
mod note;

pub use card::{CardAnswer, CardInfo, CardModTime, Ease};
pub use deck::{DeckConfig, DeckStats, LapseConfig, NewCardConfig, ReviewConfig};
pub use media::{MediaData, StoreMediaParams};
pub use model::{
    CardTemplate, CreateModelParams, FieldFont, FieldsOnTemplates, FindReplaceParams, ModelField,
    ModelInfo, ModelStyling,
};
pub use note::{
    CanAddResult, DuplicateScope, DuplicateScopeOptions, MediaAttachment, Note, NoteBuilder,
    NoteField, NoteInfo, NoteModTime, NoteOptions,
};
