//! Cloze deletion helpers for creating flashcards with fill-in-the-blank sections.
//!
//! Anki's cloze deletion feature lets you create cards where portions of text are
//! hidden, prompting you to recall the missing information.
//!
//! # Cloze Syntax
//!
//! Cloze deletions use the format `{{c1::text}}` where:
//! - `c1` is the cloze number (c1, c2, c3, etc.)
//! - `text` is the content that will be hidden
//!
//! Optionally, you can add a hint: `{{c1::text::hint}}`
//!
//! # Example
//!
//! ```
//! use ankit_builder::cloze::{cloze, cloze_hint, ClozeBuilder};
//!
//! // Simple cloze
//! let text = cloze(1, "Paris");
//! assert_eq!(text, "{{c1::Paris}}");
//!
//! // With hint
//! let text = cloze_hint(1, "Paris", "capital city");
//! assert_eq!(text, "{{c1::Paris::capital city}}");
//!
//! // Build a full sentence
//! let sentence = format!(
//!     "The capital of France is {}.",
//!     cloze(1, "Paris")
//! );
//! assert_eq!(sentence, "The capital of France is {{c1::Paris}}.");
//!
//! // Using the builder for multiple cloze deletions
//! let mut builder = ClozeBuilder::new();
//! let c1 = builder.add("Paris");
//! let c2 = builder.add("France");
//! let text = format!("{} is the capital of {}.", c1, c2);
//! assert_eq!(text, "{{c1::Paris}} is the capital of {{c2::France}}.");
//! ```

/// Create a cloze deletion with the given number.
///
/// # Arguments
///
/// * `number` - The cloze number (1, 2, 3, etc.)
/// * `text` - The text to be hidden
///
/// # Example
///
/// ```
/// use ankit_builder::cloze::cloze;
///
/// let text = cloze(1, "mitochondria");
/// assert_eq!(text, "{{c1::mitochondria}}");
///
/// // Multiple clozes in one note
/// let sentence = format!(
///     "The {} is the powerhouse of the {}.",
///     cloze(1, "mitochondria"),
///     cloze(2, "cell")
/// );
/// ```
pub fn cloze(number: u32, text: &str) -> String {
    format!("{{{{c{}::{}}}}}", number, text)
}

/// Create a cloze deletion with a hint.
///
/// The hint is shown when the card is displayed to help recall the answer.
///
/// # Arguments
///
/// * `number` - The cloze number (1, 2, 3, etc.)
/// * `text` - The text to be hidden
/// * `hint` - A hint to display
///
/// # Example
///
/// ```
/// use ankit_builder::cloze::cloze_hint;
///
/// let text = cloze_hint(1, "1789", "year");
/// assert_eq!(text, "{{c1::1789::year}}");
/// ```
pub fn cloze_hint(number: u32, text: &str, hint: &str) -> String {
    format!("{{{{c{}::{}::{}}}}}", number, text, hint)
}

/// Builder for creating multiple cloze deletions with auto-incrementing numbers.
///
/// # Example
///
/// ```
/// use ankit_builder::cloze::ClozeBuilder;
///
/// let mut builder = ClozeBuilder::new();
/// let c1 = builder.add("hydrogen");
/// let c2 = builder.add("helium");
/// let c3 = builder.add_with_hint("lithium", "Li");
///
/// let text = format!("The first three elements are {}, {}, and {}.", c1, c2, c3);
/// assert_eq!(
///     text,
///     "The first three elements are {{c1::hydrogen}}, {{c2::helium}}, and {{c3::lithium::Li}}."
/// );
/// ```
#[derive(Debug, Default)]
pub struct ClozeBuilder {
    counter: u32,
}

impl ClozeBuilder {
    /// Create a new cloze builder starting at c1.
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Add a cloze deletion and return the formatted string.
    ///
    /// Automatically increments the cloze number.
    pub fn add(&mut self, text: &str) -> String {
        self.counter += 1;
        cloze(self.counter, text)
    }

    /// Add a cloze deletion with a hint and return the formatted string.
    ///
    /// Automatically increments the cloze number.
    pub fn add_with_hint(&mut self, text: &str, hint: &str) -> String {
        self.counter += 1;
        cloze_hint(self.counter, text, hint)
    }

    /// Get the current cloze number (the last number used).
    pub fn current(&self) -> u32 {
        self.counter
    }

    /// Reset the counter to start from c1 again.
    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloze() {
        assert_eq!(cloze(1, "test"), "{{c1::test}}");
        assert_eq!(cloze(2, "another"), "{{c2::another}}");
        assert_eq!(cloze(10, "big number"), "{{c10::big number}}");
    }

    #[test]
    fn test_cloze_hint() {
        assert_eq!(cloze_hint(1, "Paris", "city"), "{{c1::Paris::city}}");
        assert_eq!(cloze_hint(3, "H2O", "formula"), "{{c3::H2O::formula}}");
    }

    #[test]
    fn test_cloze_builder() {
        let mut builder = ClozeBuilder::new();

        assert_eq!(builder.current(), 0);

        let c1 = builder.add("first");
        assert_eq!(c1, "{{c1::first}}");
        assert_eq!(builder.current(), 1);

        let c2 = builder.add("second");
        assert_eq!(c2, "{{c2::second}}");
        assert_eq!(builder.current(), 2);

        let c3 = builder.add_with_hint("third", "hint");
        assert_eq!(c3, "{{c3::third::hint}}");
        assert_eq!(builder.current(), 3);

        builder.reset();
        assert_eq!(builder.current(), 0);

        let c1_again = builder.add("new first");
        assert_eq!(c1_again, "{{c1::new first}}");
    }

    #[test]
    fn test_cloze_in_sentence() {
        let sentence = format!(
            "The {} is the capital of {}.",
            cloze(1, "Paris"),
            cloze(2, "France")
        );
        assert_eq!(
            sentence,
            "The {{c1::Paris}} is the capital of {{c2::France}}."
        );
    }

    #[test]
    fn test_cloze_builder_sentence() {
        let mut builder = ClozeBuilder::new();
        let sentence = format!(
            "{} discovered {} in {}.",
            builder.add("Marie Curie"),
            builder.add("radium"),
            builder.add_with_hint("1898", "year")
        );
        assert_eq!(
            sentence,
            "{{c1::Marie Curie}} discovered {{c2::radium}} in {{c3::1898::year}}."
        );
    }
}
