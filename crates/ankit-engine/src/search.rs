//! Content search helpers for finding notes.
//!
//! This module provides a simplified API for searching notes by content,
//! abstracting away Anki's query syntax.
//!
//! # Example
//!
//! ```no_run
//! # use ankit_engine::Engine;
//! # async fn example() -> ankit_engine::Result<()> {
//! let engine = Engine::new();
//!
//! // Search for text in any field
//! let notes = engine.search().text("conjugation", Some("Japanese")).await?;
//!
//! // Search in a specific field
//! let notes = engine.search().field("Front", "mangiare", None).await?;
//!
//! // Regex search
//! let notes = engine.search().regex("Back", r"^to\s+", None).await?;
//! # Ok(())
//! # }
//! ```

use ankit::{AnkiClient, NoteInfo, QueryBuilder};

use crate::Result;

/// Content search engine for finding notes.
#[derive(Debug)]
pub struct SearchEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> SearchEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Search for text in any field.
    ///
    /// Returns notes containing the exact phrase in any field.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to search for (exact phrase match)
    /// * `deck` - Optional deck to limit search to
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// // Search all decks
    /// let notes = engine.search().text("example sentence", None).await?;
    ///
    /// // Search specific deck
    /// let notes = engine.search().text("verb", Some("Italian")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn text(&self, text: &str, deck: Option<&str>) -> Result<Vec<NoteInfo>> {
        let mut qb = QueryBuilder::new().contains(text);
        if let Some(d) = deck {
            qb = qb.deck(d);
        }
        self.execute_query(&qb.build()).await
    }

    /// Search for text in a specific field.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the field to search in
    /// * `text` - Text to search for
    /// * `deck` - Optional deck to limit search to
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let notes = engine.search().field("Front", "mangiare", Some("Italian")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn field(
        &self,
        field_name: &str,
        text: &str,
        deck: Option<&str>,
    ) -> Result<Vec<NoteInfo>> {
        let mut qb = QueryBuilder::new().field(field_name, text);
        if let Some(d) = deck {
            qb = qb.deck(d);
        }
        self.execute_query(&qb.build()).await
    }

    /// Search with regex in a specific field.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the field to search in
    /// * `pattern` - Regex pattern
    /// * `deck` - Optional deck to limit search to
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// // Find notes where Back field starts with "to "
    /// let notes = engine.search().regex("Back", r"^to\s+", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn regex(
        &self,
        field_name: &str,
        pattern: &str,
        deck: Option<&str>,
    ) -> Result<Vec<NoteInfo>> {
        let mut qb = QueryBuilder::new().field_regex(field_name, pattern);
        if let Some(d) = deck {
            qb = qb.deck(d);
        }
        self.execute_query(&qb.build()).await
    }

    /// Search with wildcards in a specific field.
    ///
    /// Use `*` for any sequence of characters, `_` for single character.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the field to search in
    /// * `pattern` - Wildcard pattern (e.g., `*tion`, `h_llo`)
    /// * `deck` - Optional deck to limit search to
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// // Find notes ending with "tion"
    /// let notes = engine.search().wildcard("Front", "*tion", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wildcard(
        &self,
        field_name: &str,
        pattern: &str,
        deck: Option<&str>,
    ) -> Result<Vec<NoteInfo>> {
        let mut qb = QueryBuilder::new().field_wildcard(field_name, pattern);
        if let Some(d) = deck {
            qb = qb.deck(d);
        }
        self.execute_query(&qb.build()).await
    }

    /// Find notes where a field is empty.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the field to check
    /// * `deck` - Optional deck to limit search to
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// // Find notes missing examples
    /// let notes = engine.search().empty_field("Example", Some("Vocabulary")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn empty_field(&self, field_name: &str, deck: Option<&str>) -> Result<Vec<NoteInfo>> {
        let mut qb = QueryBuilder::new().field_empty(field_name);
        if let Some(d) = deck {
            qb = qb.deck(d);
        }
        self.execute_query(&qb.build()).await
    }

    /// Search with a custom query built using QueryBuilder.
    ///
    /// This allows combining the content search with full QueryBuilder capabilities.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::QueryBuilder;
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// let query = QueryBuilder::new()
    ///     .deck("Japanese")
    ///     .is_due()
    ///     .not_suspended()
    ///     .lapses_gte(3)
    ///     .build();
    ///
    /// let notes = engine.search().query(&query).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query(&self, query: &str) -> Result<Vec<NoteInfo>> {
        self.execute_query(query).await
    }

    /// Execute a query and return full note info.
    async fn execute_query(&self, query: &str) -> Result<Vec<NoteInfo>> {
        let note_ids = self.client.notes().find(query).await?;
        if note_ids.is_empty() {
            return Ok(Vec::new());
        }
        let notes = self.client.notes().info(&note_ids).await?;
        Ok(notes)
    }
}
