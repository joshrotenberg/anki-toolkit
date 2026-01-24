//! Note-related types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A new note to be added to Anki.
///
/// Use [`NoteBuilder`] for a more ergonomic way to construct notes.
///
/// # Field Values
///
/// Field values are HTML. If you need literal `<` or `>`, use `&lt;` and `&gt;`.
/// Field names are case-sensitive and must match the model's field names exactly.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    /// The deck to add the note to.
    pub deck_name: String,
    /// The note type (model) name.
    pub model_name: String,
    /// Field values, keyed by field name.
    pub fields: HashMap<String, String>,
    /// Tags for the note.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Audio attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Vec<MediaAttachment>>,
    /// Video attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<Vec<MediaAttachment>>,
    /// Picture attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<Vec<MediaAttachment>>,
    /// Options for duplicate handling, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<NoteOptions>,
}

/// A media attachment for a note (audio, video, or picture).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAttachment {
    /// URL to download the media from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Base64-encoded media data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Local file path to read media from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Filename to save the media as.
    pub filename: String,
    /// Fields to insert the media reference into.
    pub fields: Vec<String>,
    /// Optional hash to skip download if file already exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_hash: Option<String>,
}

/// Options for adding notes.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteOptions {
    /// Allow duplicate notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_duplicate: Option<bool>,
    /// Scope for duplicate checking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_scope: Option<DuplicateScope>,
    /// Additional options for duplicate scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_scope_options: Option<DuplicateScopeOptions>,
}

/// Scope for duplicate note checking.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateScope {
    /// Check for duplicates within the target deck only.
    Deck,
    /// Check for duplicates across the entire collection.
    DeckRoot,
}

/// Additional options for duplicate scope checking.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateScopeOptions {
    /// Deck name to check for duplicates in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deck_name: Option<String>,
    /// Check child decks as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_children: Option<bool>,
    /// Check all note types, not just the specified one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_all_models: Option<bool>,
}

/// Information about an existing note.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInfo {
    /// The note ID.
    pub note_id: i64,
    /// The note type (model) name.
    pub model_name: String,
    /// Tags on the note.
    pub tags: Vec<String>,
    /// Field values and metadata.
    pub fields: HashMap<String, NoteField>,
    /// Card IDs generated from this note.
    #[serde(default)]
    pub cards: Vec<i64>,
}

/// A field value with metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct NoteField {
    /// The field value (HTML).
    pub value: String,
    /// The field's position in the note type.
    pub order: i32,
}

/// Result of checking if a note can be added.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanAddResult {
    /// Whether the note can be added.
    pub can_add: bool,
    /// Error message if the note cannot be added.
    #[serde(default)]
    pub error: Option<String>,
}

/// Modification time information for a note.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteModTime {
    /// The note ID.
    pub note_id: i64,
    /// Modification timestamp (seconds since epoch).
    #[serde(rename = "mod")]
    pub mod_time: i64,
}

/// Builder for creating notes with a fluent API.
///
/// # Example
///
/// ```
/// use yanki::NoteBuilder;
///
/// let note = NoteBuilder::new("My Deck", "Basic")
///     .field("Front", "What is the capital of France?")
///     .field("Back", "Paris")
///     .tag("geography")
///     .tag("europe")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct NoteBuilder {
    deck_name: String,
    model_name: String,
    fields: HashMap<String, String>,
    tags: Vec<String>,
    audio: Option<Vec<MediaAttachment>>,
    video: Option<Vec<MediaAttachment>>,
    picture: Option<Vec<MediaAttachment>>,
    options: Option<NoteOptions>,
}

impl NoteBuilder {
    /// Create a new note builder.
    ///
    /// # Arguments
    ///
    /// * `deck` - The deck name to add the note to
    /// * `model` - The note type (model) name
    pub fn new(deck: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            deck_name: deck.into(),
            model_name: model.into(),
            ..Default::default()
        }
    }

    /// Set a field value.
    ///
    /// Field names are case-sensitive and must match the model exactly.
    /// Values are HTML - use `&lt;` for literal `<`.
    pub fn field(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(name.into(), value.into());
        self
    }

    /// Add a tag to the note.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the note.
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    /// Add an audio attachment.
    pub fn audio(mut self, attachment: MediaAttachment) -> Self {
        self.audio.get_or_insert_with(Vec::new).push(attachment);
        self
    }

    /// Add a video attachment.
    pub fn video(mut self, attachment: MediaAttachment) -> Self {
        self.video.get_or_insert_with(Vec::new).push(attachment);
        self
    }

    /// Add a picture attachment.
    pub fn picture(mut self, attachment: MediaAttachment) -> Self {
        self.picture.get_or_insert_with(Vec::new).push(attachment);
        self
    }

    /// Allow duplicate notes.
    pub fn allow_duplicate(mut self, allow: bool) -> Self {
        self.options
            .get_or_insert_with(NoteOptions::default)
            .allow_duplicate = Some(allow);
        self
    }

    /// Set the duplicate checking scope.
    pub fn duplicate_scope(mut self, scope: DuplicateScope) -> Self {
        self.options
            .get_or_insert_with(NoteOptions::default)
            .duplicate_scope = Some(scope);
        self
    }

    /// Set the deck to check for duplicates in.
    pub fn duplicate_scope_deck(mut self, deck: impl Into<String>) -> Self {
        let options = self.options.get_or_insert_with(NoteOptions::default);
        options
            .duplicate_scope_options
            .get_or_insert_with(DuplicateScopeOptions::default)
            .deck_name = Some(deck.into());
        self
    }

    /// Build the note.
    pub fn build(self) -> Note {
        Note {
            deck_name: self.deck_name,
            model_name: self.model_name,
            fields: self.fields,
            tags: self.tags,
            audio: self.audio,
            video: self.video,
            picture: self.picture,
            options: self.options,
        }
    }
}
