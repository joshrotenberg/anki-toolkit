//! Note-related AnkiConnect actions.
//!
//! This module provides operations for managing Anki notes, including
//! adding, finding, updating, and deleting notes.
//!
//! # Example
//!
//! ```no_run
//! use ankit::{AnkiClient, NoteBuilder};
//!
//! # async fn example() -> ankit::Result<()> {
//! let client = AnkiClient::new();
//!
//! // Add a note using the builder
//! let note = NoteBuilder::new("Default", "Basic")
//!     .field("Front", "Hello")
//!     .field("Back", "World")
//!     .tag("test")
//!     .build();
//!
//! let note_id = client.notes().add(note).await?;
//! println!("Created note: {}", note_id);
//!
//! // Find notes
//! let note_ids = client.notes().find("deck:Default").await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use serde::Serialize;

use crate::client::AnkiClient;
use crate::error::Result;
use crate::types::{CanAddResult, Note, NoteInfo, NoteModTime};

/// Provides access to note-related AnkiConnect operations.
///
/// Obtained via [`AnkiClient::notes()`].
#[derive(Debug)]
pub struct NoteActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

// Parameter structs for actions
#[derive(Serialize)]
struct AddNoteParams {
    note: Note,
}

#[derive(Serialize)]
struct FindNotesParams<'a> {
    query: &'a str,
}

#[derive(Serialize)]
struct NotesInfoParams<'a> {
    notes: &'a [i64],
}

#[derive(Serialize)]
struct UpdateNoteFieldsParams<'a> {
    note: UpdateNoteFieldsInner<'a>,
}

#[derive(Serialize)]
struct UpdateNoteFieldsInner<'a> {
    id: i64,
    fields: &'a HashMap<String, String>,
}

#[derive(Serialize)]
struct DeleteNotesParams<'a> {
    notes: &'a [i64],
}

#[derive(Serialize)]
struct AddNotesParams<'a> {
    notes: &'a [Note],
}

#[derive(Serialize)]
struct CanAddNotesParams<'a> {
    notes: &'a [Note],
}

#[derive(Serialize)]
struct TagsParams<'a> {
    notes: &'a [i64],
    tags: &'a str,
}

#[derive(Serialize)]
struct ReplaceTagsParams<'a> {
    notes: &'a [i64],
    tag_to_replace: &'a str,
    replace_with_tag: &'a str,
}

#[derive(Serialize)]
struct ReplaceTagsAllParams<'a> {
    tag_to_replace: &'a str,
    replace_with_tag: &'a str,
}

#[derive(Serialize)]
struct UpdateNoteParams<'a> {
    note: UpdateNoteInner<'a>,
}

#[derive(Serialize)]
struct UpdateNoteInner<'a> {
    id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<&'a HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<&'a [String]>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateNoteModelParams<'a> {
    note: i64,
    model_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_map: Option<&'a HashMap<String, String>>,
}

#[derive(Serialize)]
struct UpdateNoteTagsParams<'a> {
    note: i64,
    tags: &'a [String],
}

impl<'a> NoteActions<'a> {
    /// Add a new note.
    ///
    /// Returns the ID of the created note.
    ///
    /// # Note on Duplicates
    ///
    /// By default, AnkiConnect will reject duplicate notes. Use
    /// [`NoteBuilder::allow_duplicate()`](crate::NoteBuilder::allow_duplicate)
    /// to override this behavior.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::{AnkiClient, NoteBuilder};
    ///
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let note = NoteBuilder::new("Japanese", "Basic")
    ///     .field("Front", "hello")
    ///     .field("Back", "world")
    ///     .tags(["vocabulary", "common"])
    ///     .build();
    ///
    /// let note_id = client.notes().add(note).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add(&self, note: Note) -> Result<i64> {
        self.client.invoke("addNote", AddNoteParams { note }).await
    }

    /// Find notes matching a query.
    ///
    /// Returns a list of note IDs. Use [`info()`](Self::info) to get full note details.
    ///
    /// # Query Syntax
    ///
    /// Uses Anki's search syntax:
    /// - `deck:DeckName` - notes in a specific deck
    /// - `tag:TagName` - notes with a specific tag
    /// - `"exact phrase"` - exact phrase match
    /// - `field:value` - search in a specific field
    /// - `-tag:excluded` - exclude notes with a tag
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// // Find all notes in the Japanese deck with the "verb" tag
    /// let notes = client.notes().find("deck:Japanese tag:verb").await?;
    /// println!("Found {} notes", notes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find(&self, query: &str) -> Result<Vec<i64>> {
        self.client
            .invoke("findNotes", FindNotesParams { query })
            .await
    }

    /// Get detailed information about notes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let note_ids = client.notes().find("deck:Default").await?;
    /// let notes = client.notes().info(&note_ids).await?;
    ///
    /// for note in notes {
    ///     println!("Note {}: {:?}", note.note_id, note.tags);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn info(&self, note_ids: &[i64]) -> Result<Vec<NoteInfo>> {
        self.client
            .invoke("notesInfo", NotesInfoParams { notes: note_ids })
            .await
    }

    /// Update a note's field values.
    ///
    /// # Warning
    ///
    /// If the note is currently displayed in Anki's browser, changes may not
    /// persist due to a known AnkiConnect limitation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # use std::collections::HashMap;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("Front".to_string(), "Updated front".to_string());
    /// fields.insert("Back".to_string(), "Updated back".to_string());
    ///
    /// client.notes().update_fields(1234567890, &fields).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_fields(
        &self,
        note_id: i64,
        fields: &HashMap<String, String>,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "updateNoteFields",
                UpdateNoteFieldsParams {
                    note: UpdateNoteFieldsInner {
                        id: note_id,
                        fields,
                    },
                },
            )
            .await
    }

    /// Delete notes.
    ///
    /// This also deletes all cards generated from the notes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let note_ids = vec![1234567890, 1234567891];
    /// client.notes().delete(&note_ids).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, note_ids: &[i64]) -> Result<()> {
        self.client
            .invoke_void("deleteNotes", DeleteNotesParams { notes: note_ids })
            .await
    }

    /// Add multiple notes at once.
    ///
    /// Returns a list of note IDs. If a note could not be created (e.g., duplicate),
    /// the corresponding entry will be `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::{AnkiClient, NoteBuilder};
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let notes = vec![
    ///     NoteBuilder::new("Default", "Basic")
    ///         .field("Front", "Q1").field("Back", "A1").build(),
    ///     NoteBuilder::new("Default", "Basic")
    ///         .field("Front", "Q2").field("Back", "A2").build(),
    /// ];
    ///
    /// let ids = client.notes().add_many(&notes).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_many(&self, notes: &[Note]) -> Result<Vec<Option<i64>>> {
        self.client
            .invoke("addNotes", AddNotesParams { notes })
            .await
    }

    /// Check if notes can be added without actually adding them.
    ///
    /// Returns a boolean for each note indicating whether it can be added.
    pub async fn can_add(&self, notes: &[Note]) -> Result<Vec<bool>> {
        self.client
            .invoke("canAddNotes", CanAddNotesParams { notes })
            .await
    }

    /// Check if notes can be added, with detailed error information.
    ///
    /// Returns detailed results for each note including error messages.
    pub async fn can_add_detailed(&self, notes: &[Note]) -> Result<Vec<CanAddResult>> {
        self.client
            .invoke("canAddNotesWithErrorDetail", CanAddNotesParams { notes })
            .await
    }

    /// Get the tags for a specific note.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let tags = client.notes().get_tags(1234567890).await?;
    /// println!("Tags: {:?}", tags);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tags(&self, note_id: i64) -> Result<Vec<String>> {
        #[derive(Serialize)]
        struct Params {
            note: i64,
        }
        self.client
            .invoke("getNoteTags", Params { note: note_id })
            .await
    }

    /// Add tags to notes.
    ///
    /// Tags are provided as a space-separated string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.notes().add_tags(&[1234567890], "tag1 tag2").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_tags(&self, note_ids: &[i64], tags: &str) -> Result<()> {
        self.client
            .invoke_void(
                "addTags",
                TagsParams {
                    notes: note_ids,
                    tags,
                },
            )
            .await
    }

    /// Remove tags from notes.
    ///
    /// Tags are provided as a space-separated string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.notes().remove_tags(&[1234567890], "tag1 tag2").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_tags(&self, note_ids: &[i64], tags: &str) -> Result<()> {
        self.client
            .invoke_void(
                "removeTags",
                TagsParams {
                    notes: note_ids,
                    tags,
                },
            )
            .await
    }

    /// Remove all tags that are not used by any notes.
    pub async fn clear_unused_tags(&self) -> Result<()> {
        self.client
            .invoke_void("clearUnusedTags", serde_json::json!({}))
            .await
    }

    /// Replace a tag on specific notes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.notes().replace_tags(&[1234567890], "old-tag", "new-tag").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn replace_tags(&self, note_ids: &[i64], old_tag: &str, new_tag: &str) -> Result<()> {
        self.client
            .invoke_void(
                "replaceTags",
                ReplaceTagsParams {
                    notes: note_ids,
                    tag_to_replace: old_tag,
                    replace_with_tag: new_tag,
                },
            )
            .await
    }

    /// Replace a tag on all notes in the collection.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.notes().replace_tags_all("old-tag", "new-tag").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn replace_tags_all(&self, old_tag: &str, new_tag: &str) -> Result<()> {
        self.client
            .invoke_void(
                "replaceTagsInAllNotes",
                ReplaceTagsAllParams {
                    tag_to_replace: old_tag,
                    replace_with_tag: new_tag,
                },
            )
            .await
    }

    /// Get modification times for notes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let mod_times = client.notes().mod_time(&[1234567890]).await?;
    /// for mt in mod_times {
    ///     println!("Note {} modified at {}", mt.note_id, mt.mod_time);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mod_time(&self, note_ids: &[i64]) -> Result<Vec<NoteModTime>> {
        self.client
            .invoke("notesModTime", NotesInfoParams { notes: note_ids })
            .await
    }

    /// Remove notes that have no cards.
    ///
    /// This can happen if all card templates were deleted from a note type.
    pub async fn remove_empty(&self) -> Result<()> {
        self.client
            .invoke_void("removeEmptyNotes", serde_json::json!({}))
            .await
    }

    /// Update a note's fields and/or tags in a single operation.
    ///
    /// More efficient than calling `update_fields` and tag operations separately.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # use std::collections::HashMap;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("Front".to_string(), "Updated question".to_string());
    ///
    /// let tags = vec!["updated".to_string(), "reviewed".to_string()];
    ///
    /// client.notes().update(1234567890, Some(&fields), Some(&tags)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(
        &self,
        note_id: i64,
        fields: Option<&HashMap<String, String>>,
        tags: Option<&[String]>,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "updateNote",
                UpdateNoteParams {
                    note: UpdateNoteInner {
                        id: note_id,
                        fields,
                        tags,
                    },
                },
            )
            .await
    }

    /// Change a note's model (note type) with optional field mapping.
    ///
    /// If `field_map` is provided, it maps old field names to new field names.
    /// Fields not in the map will be discarded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # use std::collections::HashMap;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// // Map old fields to new fields
    /// let mut field_map = HashMap::new();
    /// field_map.insert("Front".to_string(), "Question".to_string());
    /// field_map.insert("Back".to_string(), "Answer".to_string());
    ///
    /// client.notes().update_model(1234567890, "New Model", Some(&field_map)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_model(
        &self,
        note_id: i64,
        model_name: &str,
        field_map: Option<&HashMap<String, String>>,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "updateNoteModel",
                UpdateNoteModelParams {
                    note: note_id,
                    model_name,
                    field_map,
                },
            )
            .await
    }

    /// Set all tags for a note, replacing any existing tags.
    ///
    /// Unlike `add_tags` and `remove_tags`, this atomically replaces all tags.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let new_tags = vec!["vocabulary".to_string(), "chapter1".to_string()];
    /// client.notes().set_tags(1234567890, &new_tags).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_tags(&self, note_id: i64, tags: &[String]) -> Result<()> {
        self.client
            .invoke_void(
                "updateNoteTags",
                UpdateNoteTagsParams {
                    note: note_id,
                    tags,
                },
            )
            .await
    }

    /// Get all tags in the collection.
    ///
    /// Returns all tags that exist in any note.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let all_tags = client.notes().all_tags().await?;
    /// println!("Collection has {} tags", all_tags.len());
    /// for tag in all_tags {
    ///     println!("  {}", tag);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn all_tags(&self) -> Result<Vec<String>> {
        self.client.invoke("getTags", serde_json::json!({})).await
    }
}
