//! Type-safe query builder for Anki search syntax.
//!
//! This module provides a fluent API for constructing Anki search queries,
//! replacing error-prone string concatenation with compile-time checked methods.
//!
//! # Example
//!
//! ```
//! use ankit::QueryBuilder;
//!
//! // Build a query for due cards in a deck, excluding suspended
//! let query = QueryBuilder::new()
//!     .deck("Japanese")
//!     .is_due()
//!     .not_suspended()
//!     .build();
//!
//! assert_eq!(query, "deck:Japanese is:due -is:suspended");
//! ```
//!
//! # Complex Queries
//!
//! ```
//! use ankit::QueryBuilder;
//!
//! // Find leeches (high lapses) with low ease
//! let query = QueryBuilder::new()
//!     .deck("Vocabulary")
//!     .lapses_gte(5)
//!     .ease_lt(2.1)
//!     .not_suspended()
//!     .build();
//!
//! // Search in specific fields
//! let query = QueryBuilder::new()
//!     .field("Front", "mangiare")
//!     .build();
//!
//! // OR conditions
//! let query = QueryBuilder::new()
//!     .deck("Italian")
//!     .or(|q| q.tag("verb").tag("noun"))
//!     .build();
//! ```

/// A builder for constructing Anki search queries.
///
/// Provides a type-safe, fluent API for building queries instead of
/// manually constructing query strings.
#[derive(Debug, Clone, Default)]
#[must_use = "QueryBuilder does nothing until .build() is called"]
pub struct QueryBuilder {
    parts: Vec<String>,
}

impl QueryBuilder {
    /// Create a new empty query builder.
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    // ========================================================================
    // Location
    // ========================================================================

    /// Filter by deck name.
    ///
    /// Supports hierarchical decks with `::` separator.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().deck("Japanese").build();
    /// assert_eq!(q, "deck:Japanese");
    ///
    /// let q = QueryBuilder::new().deck("Languages::Italian").build();
    /// assert_eq!(q, "deck:Languages::Italian");
    ///
    /// // Spaces are quoted automatically
    /// let q = QueryBuilder::new().deck("My Deck").build();
    /// assert_eq!(q, "deck:\"My Deck\"");
    /// ```
    pub fn deck(mut self, name: &str) -> Self {
        self.parts.push(format!("deck:{}", quote_if_needed(name)));
        self
    }

    /// Filter by note type (model) name.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().note_type("Basic").build();
    /// assert_eq!(q, "note:Basic");
    /// ```
    pub fn note_type(mut self, model: &str) -> Self {
        self.parts.push(format!("note:{}", quote_if_needed(model)));
        self
    }

    /// Filter by card template ordinal (1-indexed).
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().card_template(2).build();
    /// assert_eq!(q, "card:2");
    /// ```
    pub fn card_template(mut self, ordinal: i32) -> Self {
        self.parts.push(format!("card:{}", ordinal));
        self
    }

    // ========================================================================
    // Card State
    // ========================================================================

    /// Filter for cards that are due for review.
    pub fn is_due(mut self) -> Self {
        self.parts.push("is:due".to_string());
        self
    }

    /// Filter for new cards (never reviewed).
    pub fn is_new(mut self) -> Self {
        self.parts.push("is:new".to_string());
        self
    }

    /// Filter for review cards.
    pub fn is_review(mut self) -> Self {
        self.parts.push("is:review".to_string());
        self
    }

    /// Filter for cards in learning phase.
    pub fn is_learn(mut self) -> Self {
        self.parts.push("is:learn".to_string());
        self
    }

    /// Filter for suspended cards.
    pub fn is_suspended(mut self) -> Self {
        self.parts.push("is:suspended".to_string());
        self
    }

    /// Filter for buried cards.
    pub fn is_buried(mut self) -> Self {
        self.parts.push("is:buried".to_string());
        self
    }

    /// Exclude suspended cards.
    pub fn not_suspended(mut self) -> Self {
        self.parts.push("-is:suspended".to_string());
        self
    }

    /// Exclude buried cards.
    pub fn not_buried(mut self) -> Self {
        self.parts.push("-is:buried".to_string());
        self
    }

    // ========================================================================
    // Tags
    // ========================================================================

    /// Filter by tag.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().tag("vocabulary").build();
    /// assert_eq!(q, "tag:vocabulary");
    /// ```
    pub fn tag(mut self, tag: &str) -> Self {
        self.parts.push(format!("tag:{}", quote_if_needed(tag)));
        self
    }

    /// Exclude notes with a specific tag.
    pub fn without_tag(mut self, tag: &str) -> Self {
        self.parts.push(format!("-tag:{}", quote_if_needed(tag)));
        self
    }

    /// Filter for notes without any tags.
    pub fn untagged(mut self) -> Self {
        self.parts.push("tag:none".to_string());
        self
    }

    // ========================================================================
    // Properties
    // ========================================================================

    /// Filter for cards with interval greater than N days.
    pub fn interval_gt(mut self, days: i32) -> Self {
        self.parts.push(format!("prop:ivl>{}", days));
        self
    }

    /// Filter for cards with interval less than N days.
    pub fn interval_lt(mut self, days: i32) -> Self {
        self.parts.push(format!("prop:ivl<{}", days));
        self
    }

    /// Filter for cards with interval equal to N days.
    pub fn interval_eq(mut self, days: i32) -> Self {
        self.parts.push(format!("prop:ivl={}", days));
        self
    }

    /// Filter for cards with ease factor greater than value.
    ///
    /// Ease is expressed as a decimal (e.g., 2.5 = 250%).
    pub fn ease_gt(mut self, ease: f32) -> Self {
        self.parts.push(format!("prop:ease>{:.2}", ease));
        self
    }

    /// Filter for cards with ease factor less than value.
    ///
    /// Ease is expressed as a decimal (e.g., 2.5 = 250%).
    pub fn ease_lt(mut self, ease: f32) -> Self {
        self.parts.push(format!("prop:ease<{:.2}", ease));
        self
    }

    /// Filter for cards with lapses greater than or equal to N.
    ///
    /// Useful for finding leeches.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// // Find potential leeches
    /// let q = QueryBuilder::new().lapses_gte(8).build();
    /// assert_eq!(q, "prop:lapses>=8");
    /// ```
    pub fn lapses_gte(mut self, n: i32) -> Self {
        self.parts.push(format!("prop:lapses>={}", n));
        self
    }

    /// Filter for cards with exactly N lapses.
    pub fn lapses_eq(mut self, n: i32) -> Self {
        self.parts.push(format!("prop:lapses={}", n));
        self
    }

    /// Filter for cards with reps greater than or equal to N.
    pub fn reps_gte(mut self, n: i32) -> Self {
        self.parts.push(format!("prop:reps>={}", n));
        self
    }

    /// Filter for cards due within N days.
    ///
    /// Use 0 for due today, negative for overdue.
    pub fn due_in_days(mut self, days: i32) -> Self {
        self.parts.push(format!("prop:due={}", days));
        self
    }

    /// Filter for cards due before N days from now.
    pub fn due_before_days(mut self, days: i32) -> Self {
        self.parts.push(format!("prop:due<{}", days));
        self
    }

    // ========================================================================
    // Time-based
    // ========================================================================

    /// Filter for cards added within the last N days.
    pub fn added_within_days(mut self, days: i32) -> Self {
        self.parts.push(format!("added:{}", days));
        self
    }

    /// Filter for cards reviewed/rated within the last N days.
    pub fn rated_within_days(mut self, days: i32) -> Self {
        self.parts.push(format!("rated:{}", days));
        self
    }

    /// Filter for cards edited within the last N days.
    pub fn edited_within_days(mut self, days: i32) -> Self {
        self.parts.push(format!("edited:{}", days));
        self
    }

    /// Filter for cards first reviewed within the last N days.
    pub fn introduced_within_days(mut self, days: i32) -> Self {
        self.parts.push(format!("introduced:{}", days));
        self
    }

    // ========================================================================
    // Content Search
    // ========================================================================

    /// Search for text in any field.
    ///
    /// The text is matched as an exact phrase.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().contains("to eat").build();
    /// assert_eq!(q, "\"to eat\"");
    /// ```
    pub fn contains(mut self, text: &str) -> Self {
        self.parts.push(format!("\"{}\"", escape_quotes(text)));
        self
    }

    /// Search for a word in any field.
    ///
    /// Unlike `contains`, this matches word boundaries.
    pub fn word(mut self, word: &str) -> Self {
        self.parts.push(quote_if_needed(word));
        self
    }

    /// Search for text in a specific field.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().field("Front", "hello").build();
    /// assert_eq!(q, "Front:hello");
    ///
    /// let q = QueryBuilder::new().field("Front", "hello world").build();
    /// assert_eq!(q, "Front:\"hello world\"");
    /// ```
    pub fn field(mut self, field_name: &str, text: &str) -> Self {
        self.parts
            .push(format!("{}:{}", field_name, quote_if_needed(text)));
        self
    }

    /// Search with regex in a specific field.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().field_regex("Front", r"^to\s+").build();
    /// assert_eq!(q, r"Front:re:^to\s+");
    /// ```
    pub fn field_regex(mut self, field_name: &str, pattern: &str) -> Self {
        self.parts.push(format!("{}:re:{}", field_name, pattern));
        self
    }

    /// Search with wildcard in a specific field.
    ///
    /// Use `*` for any sequence, `_` for single character.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().field_wildcard("Front", "*tion").build();
    /// assert_eq!(q, "Front:*tion");
    /// ```
    pub fn field_wildcard(mut self, field_name: &str, pattern: &str) -> Self {
        self.parts.push(format!("{}:{}", field_name, pattern));
        self
    }

    /// Filter for notes where a field is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new().field_empty("Example").build();
    /// assert_eq!(q, "Example:");
    /// ```
    pub fn field_empty(mut self, field_name: &str) -> Self {
        self.parts.push(format!("{}:", field_name));
        self
    }

    // ========================================================================
    // Flags
    // ========================================================================

    /// Filter by flag color.
    ///
    /// Flag values: 0 (no flag), 1 (red), 2 (orange), 3 (green), 4 (blue), 5 (pink), 6 (turquoise), 7 (purple).
    pub fn flag(mut self, flag: i32) -> Self {
        self.parts.push(format!("flag:{}", flag));
        self
    }

    /// Filter for cards with any flag.
    pub fn has_flag(mut self) -> Self {
        self.parts.push("-flag:0".to_string());
        self
    }

    /// Filter for cards without any flag.
    pub fn no_flag(mut self) -> Self {
        self.parts.push("flag:0".to_string());
        self
    }

    // ========================================================================
    // Combinators
    // ========================================================================

    /// Combine conditions with OR.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new()
    ///     .deck("Italian")
    ///     .or(|q| q.tag("verb").tag("noun"))
    ///     .build();
    ///
    /// assert_eq!(q, "deck:Italian (tag:verb OR tag:noun)");
    /// ```
    pub fn or<F>(mut self, f: F) -> Self
    where
        F: FnOnce(OrBuilder) -> OrBuilder,
    {
        let or_builder = f(OrBuilder::new());
        let or_query = or_builder.build();
        if !or_query.is_empty() {
            self.parts.push(format!("({})", or_query));
        }
        self
    }

    /// Negate a condition.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new()
    ///     .deck("Test")
    ///     .not(|q| q.tag("exclude"))
    ///     .build();
    ///
    /// assert_eq!(q, "deck:Test -tag:exclude");
    /// ```
    pub fn not<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder) -> QueryBuilder,
    {
        let inner = f(QueryBuilder::new());
        for part in inner.parts {
            if let Some(stripped) = part.strip_prefix('-') {
                // Double negative - remove the dash
                self.parts.push(stripped.to_string());
            } else {
                self.parts.push(format!("-{}", part));
            }
        }
        self
    }

    /// Add a raw query string.
    ///
    /// Use this as an escape hatch for query syntax not covered by the builder.
    ///
    /// # Example
    ///
    /// ```
    /// use ankit::QueryBuilder;
    ///
    /// let q = QueryBuilder::new()
    ///     .deck("Test")
    ///     .raw("prop:pos>5")
    ///     .build();
    ///
    /// assert_eq!(q, "deck:Test prop:pos>5");
    /// ```
    pub fn raw(mut self, query: &str) -> Self {
        self.parts.push(query.to_string());
        self
    }

    // ========================================================================
    // Terminal
    // ========================================================================

    /// Build the final query string.
    pub fn build(self) -> String {
        self.parts.join(" ")
    }
}

/// Builder for OR conditions.
#[derive(Debug, Clone, Default)]
pub struct OrBuilder {
    parts: Vec<String>,
}

impl OrBuilder {
    fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Add a tag to the OR group.
    pub fn tag(mut self, tag: &str) -> Self {
        self.parts.push(format!("tag:{}", quote_if_needed(tag)));
        self
    }

    /// Add a deck to the OR group.
    pub fn deck(mut self, name: &str) -> Self {
        self.parts.push(format!("deck:{}", quote_if_needed(name)));
        self
    }

    /// Add a note type to the OR group.
    pub fn note_type(mut self, model: &str) -> Self {
        self.parts.push(format!("note:{}", quote_if_needed(model)));
        self
    }

    /// Add a field search to the OR group.
    pub fn field(mut self, field_name: &str, text: &str) -> Self {
        self.parts
            .push(format!("{}:{}", field_name, quote_if_needed(text)));
        self
    }

    /// Add a raw query to the OR group.
    pub fn raw(mut self, query: &str) -> Self {
        self.parts.push(query.to_string());
        self
    }

    /// Add is:new to the OR group.
    pub fn is_new(mut self) -> Self {
        self.parts.push("is:new".to_string());
        self
    }

    /// Add is:due to the OR group.
    pub fn is_due(mut self) -> Self {
        self.parts.push("is:due".to_string());
        self
    }

    /// Add is:review to the OR group.
    pub fn is_review(mut self) -> Self {
        self.parts.push("is:review".to_string());
        self
    }

    /// Add is:learn to the OR group.
    pub fn is_learn(mut self) -> Self {
        self.parts.push("is:learn".to_string());
        self
    }

    fn build(self) -> String {
        self.parts.join(" OR ")
    }
}

/// Quote a value if it contains spaces or special characters.
fn quote_if_needed(s: &str) -> String {
    if s.contains(' ') || s.contains('"') || s.contains('(') || s.contains(')') {
        format!("\"{}\"", escape_quotes(s))
    } else {
        s.to_string()
    }
}

/// Escape double quotes in a string.
fn escape_quotes(s: &str) -> String {
    s.replace('"', "\\\"")
}

impl std::fmt::Display for QueryBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let q = QueryBuilder::new().build();
        assert_eq!(q, "");
    }

    #[test]
    fn test_deck() {
        let q = QueryBuilder::new().deck("Japanese").build();
        assert_eq!(q, "deck:Japanese");
    }

    #[test]
    fn test_deck_with_spaces() {
        let q = QueryBuilder::new().deck("My Deck").build();
        assert_eq!(q, "deck:\"My Deck\"");
    }

    #[test]
    fn test_hierarchical_deck() {
        let q = QueryBuilder::new()
            .deck("Languages::Italian::Verbs")
            .build();
        assert_eq!(q, "deck:Languages::Italian::Verbs");
    }

    #[test]
    fn test_card_states() {
        let q = QueryBuilder::new().is_due().is_new().build();
        assert_eq!(q, "is:due is:new");

        let q = QueryBuilder::new().not_suspended().not_buried().build();
        assert_eq!(q, "-is:suspended -is:buried");
    }

    #[test]
    fn test_tags() {
        let q = QueryBuilder::new().tag("vocab").without_tag("hard").build();
        assert_eq!(q, "tag:vocab -tag:hard");
    }

    #[test]
    fn test_properties() {
        let q = QueryBuilder::new()
            .lapses_gte(5)
            .ease_lt(2.1)
            .interval_gt(30)
            .build();
        assert_eq!(q, "prop:lapses>=5 prop:ease<2.10 prop:ivl>30");
    }

    #[test]
    fn test_time_filters() {
        let q = QueryBuilder::new()
            .added_within_days(7)
            .rated_within_days(1)
            .build();
        assert_eq!(q, "added:7 rated:1");
    }

    #[test]
    fn test_content_search() {
        let q = QueryBuilder::new().contains("to eat").build();
        assert_eq!(q, "\"to eat\"");

        let q = QueryBuilder::new().field("Front", "hello").build();
        assert_eq!(q, "Front:hello");

        let q = QueryBuilder::new().field("Front", "hello world").build();
        assert_eq!(q, "Front:\"hello world\"");
    }

    #[test]
    fn test_field_regex() {
        let q = QueryBuilder::new().field_regex("Front", r"^to\s+").build();
        assert_eq!(q, r"Front:re:^to\s+");
    }

    #[test]
    fn test_field_empty() {
        let q = QueryBuilder::new().field_empty("Example").build();
        assert_eq!(q, "Example:");
    }

    #[test]
    fn test_or_combinator() {
        let q = QueryBuilder::new()
            .deck("Italian")
            .or(|q| q.tag("verb").tag("noun"))
            .build();
        assert_eq!(q, "deck:Italian (tag:verb OR tag:noun)");
    }

    #[test]
    fn test_not_combinator() {
        let q = QueryBuilder::new()
            .deck("Test")
            .not(|q| q.tag("exclude"))
            .build();
        assert_eq!(q, "deck:Test -tag:exclude");
    }

    #[test]
    fn test_complex_query() {
        let q = QueryBuilder::new()
            .deck("Japanese")
            .is_due()
            .not_suspended()
            .lapses_gte(3)
            .or(|q| q.tag("verb").tag("noun").tag("adjective"))
            .build();
        assert_eq!(
            q,
            "deck:Japanese is:due -is:suspended prop:lapses>=3 (tag:verb OR tag:noun OR tag:adjective)"
        );
    }

    #[test]
    fn test_raw_escape_hatch() {
        let q = QueryBuilder::new().deck("Test").raw("prop:pos>5").build();
        assert_eq!(q, "deck:Test prop:pos>5");
    }

    #[test]
    fn test_display() {
        let q = QueryBuilder::new().deck("Test").is_due();
        assert_eq!(format!("{}", q), "deck:Test is:due");
    }

    #[test]
    fn test_flags() {
        let q = QueryBuilder::new().flag(1).build();
        assert_eq!(q, "flag:1");

        let q = QueryBuilder::new().has_flag().build();
        assert_eq!(q, "-flag:0");

        let q = QueryBuilder::new().no_flag().build();
        assert_eq!(q, "flag:0");
    }
}
