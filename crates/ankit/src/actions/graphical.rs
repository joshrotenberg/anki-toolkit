//! GUI-related AnkiConnect actions.
//!
//! This module provides operations for controlling Anki's graphical interface.
//!
//! # Example
//!
//! ```no_run
//! use ankit::AnkiClient;
//!
//! # async fn example() -> ankit::Result<()> {
//! let client = AnkiClient::new();
//!
//! // Open the browser with a search query
//! let card_ids = client.gui().browse("deck:Default").await?;
//!
//! // Get currently selected notes
//! let note_ids = client.gui().selected_notes().await?;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::AnkiClient;
use crate::error::Result;
use crate::types::{Ease, Note};

/// Provides access to GUI-related AnkiConnect operations.
///
/// Obtained via [`AnkiClient::gui()`].
#[derive(Debug)]
pub struct GuiActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

#[derive(Serialize)]
struct BrowseParams<'a> {
    query: &'a str,
}

#[derive(Serialize)]
struct EditNoteParams {
    note: i64,
}

#[derive(Serialize)]
struct AnswerCardParams {
    ease: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeckParams<'a> {
    name: &'a str,
}

#[derive(Serialize)]
struct ImportParams<'a> {
    path: &'a str,
}

#[derive(Serialize)]
struct SelectCardParams {
    card: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddNoteSetDataParams<'a> {
    deck: &'a str,
    model: &'a str,
    fields: HashMap<&'a str, &'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<&'a [&'a str]>,
}

/// Result of getting the current card.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentCard {
    /// The card ID.
    pub card_id: i64,
    /// The note ID.
    pub note_id: i64,
    /// The deck ID.
    pub deck_id: i64,
    /// The model ID.
    pub model_id: i64,
    /// Fields of the card.
    pub fields: serde_json::Value,
    /// Question HTML.
    pub question: String,
    /// Answer HTML.
    pub answer: String,
    /// Deck name.
    pub deck_name: String,
    /// Model name.
    pub model_name: String,
    /// Card template name.
    pub template_name: String,
    /// Available buttons (ease values).
    pub buttons: Vec<i32>,
    /// Next review intervals for each button.
    pub next_reviews: Vec<String>,
}

/// Result of a GUI import operation.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportResult {
    /// Number of notes found in the file.
    #[serde(default)]
    pub found_notes: i64,
    /// Number of notes imported.
    #[serde(default)]
    pub imported_notes: i64,
}

impl<'a> GuiActions<'a> {
    /// Open the card browser with a search query.
    ///
    /// Returns the IDs of cards matching the query.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let cards = client.gui().browse("deck:Japanese").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn browse(&self, query: &str) -> Result<Vec<i64>> {
        self.client
            .invoke("guiBrowse", BrowseParams { query })
            .await
    }

    /// Get the IDs of notes currently selected in the browser.
    pub async fn selected_notes(&self) -> Result<Vec<i64>> {
        self.client.invoke_without_params("guiSelectedNotes").await
    }

    /// Open the Add Cards dialog with a note.
    ///
    /// Returns the ID of the added note, or None if cancelled.
    pub async fn add_cards(&self, note: Note) -> Result<Option<i64>> {
        self.client.invoke("guiAddCards", note).await
    }

    /// Open the note editor for a specific note.
    pub async fn edit_note(&self, note_id: i64) -> Result<()> {
        self.client
            .invoke_void("guiEditNote", EditNoteParams { note: note_id })
            .await
    }

    /// Get information about the current card being reviewed.
    ///
    /// Returns None if not in review mode.
    pub async fn current_card(&self) -> Result<Option<CurrentCard>> {
        self.client
            .invoke_nullable_without_params("guiCurrentCard")
            .await
    }

    /// Start the card timer.
    ///
    /// This resets the timer used to track how long the user takes to answer.
    pub async fn start_timer(&self) -> Result<bool> {
        self.client.invoke_without_params("guiStartCardTimer").await
    }

    /// Show the question side of the current card.
    pub async fn show_question(&self) -> Result<bool> {
        self.client.invoke_without_params("guiShowQuestion").await
    }

    /// Show the answer side of the current card.
    pub async fn show_answer(&self) -> Result<bool> {
        self.client.invoke_without_params("guiShowAnswer").await
    }

    /// Answer the current card with the given ease.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::{AnkiClient, Ease};
    ///
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.gui().answer_card(Ease::Good).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn answer_card(&self, ease: Ease) -> Result<bool> {
        self.client
            .invoke("guiAnswerCard", AnswerCardParams { ease: ease.into() })
            .await
    }

    /// Switch to the deck overview screen for a deck.
    pub async fn deck_overview(&self, name: &str) -> Result<bool> {
        self.client
            .invoke("guiDeckOverview", DeckParams { name })
            .await
    }

    /// Switch to the deck browser screen.
    pub async fn deck_browser(&self) -> Result<bool> {
        self.client.invoke_without_params("guiDeckBrowser").await
    }

    /// Start reviewing a deck.
    pub async fn deck_review(&self, name: &str) -> Result<bool> {
        self.client
            .invoke("guiDeckReview", DeckParams { name })
            .await
    }

    /// Import a file (e.g., .apkg or .txt).
    pub async fn import_file(&self, path: &str) -> Result<ImportResult> {
        self.client
            .invoke("guiImportFile", ImportParams { path })
            .await
    }

    /// Exit Anki.
    pub async fn exit_anki(&self) -> Result<()> {
        self.client.invoke_void("guiExitAnki", ()).await
    }

    /// Check the database for errors.
    ///
    /// This is the same as Tools > Check Database in Anki.
    pub async fn check_database(&self) -> Result<bool> {
        self.client.invoke_without_params("guiCheckDatabase").await
    }

    /// Undo the last action.
    pub async fn undo(&self) -> Result<()> {
        self.client.invoke_void("guiUndo", ()).await
    }

    /// Select a specific card in the browser.
    ///
    /// Opens the browser if not already open and selects the specified card.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.gui().select_card(1234567890).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn select_card(&self, card_id: i64) -> Result<bool> {
        self.client
            .invoke("guiSelectCard", SelectCardParams { card: card_id })
            .await
    }

    /// Set data in the Add Cards dialog.
    ///
    /// This pre-fills the Add Cards dialog with the specified deck, model, fields, and tags.
    /// The dialog must be open for this to work.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::AnkiClient;
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("Front", "Question");
    /// fields.insert("Back", "Answer");
    ///
    /// client.gui().add_note_set_data("Default", "Basic", fields, Some(&["tag1", "tag2"])).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_note_set_data(
        &self,
        deck: &str,
        model: &str,
        fields: HashMap<&str, &str>,
        tags: Option<&[&str]>,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "guiAddNoteSetData",
                AddNoteSetDataParams {
                    deck,
                    model,
                    fields,
                    tags,
                },
            )
            .await
    }

    /// Play audio associated with the current card.
    ///
    /// Plays audio on either the question or answer side.
    ///
    /// # Arguments
    ///
    /// * `side` - Either "question" or "answer"
    pub async fn play_audio(&self, side: &str) -> Result<()> {
        #[derive(Serialize)]
        struct Params<'a> {
            side: &'a str,
        }
        self.client
            .invoke_void("guiPlayAudio", Params { side })
            .await
    }

    /// Get the name of the currently active profile.
    pub async fn active_profile(&self) -> Result<String> {
        self.client.invoke_without_params("getActiveProfile").await
    }
}
