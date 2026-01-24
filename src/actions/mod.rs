//! Action modules for AnkiConnect operations.
//!
//! Each module provides a set of related operations grouped by domain.

mod cards;
mod decks;
mod graphical;
mod media;
mod miscellaneous;
mod models;
mod notes;
mod statistics;

pub use cards::CardActions;
pub use decks::DeckActions;
pub use graphical::{CurrentCard, GuiActions, ImportResult};
pub use media::MediaActions;
pub use miscellaneous::{ApiReflectResult, MiscActions, MultiAction, PermissionResult};
pub use models::ModelActions;
pub use notes::NoteActions;
pub use statistics::{ReviewEntry, StatisticsActions};
