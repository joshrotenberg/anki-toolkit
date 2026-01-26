//! Markdown to HTML conversion for note fields.
//!
//! This module provides bidirectional conversion between Markdown and HTML
//! for note fields marked with `markdown_fields` in the model definition.
//!
//! # Example
//!
//! ```
//! use ankit_builder::markdown::{markdown_to_html, html_to_markdown};
//!
//! let md = "**bold** and *italic*";
//! let html = markdown_to_html(md);
//! assert!(html.contains("<strong>bold</strong>"));
//!
//! let back = html_to_markdown(&html);
//! assert!(back.contains("**bold**"));
//! ```

use pulldown_cmark::{Options, Parser, html};

/// Convert Markdown to HTML.
///
/// Supports common Markdown features:
/// - **bold** and *italic*
/// - Lists (ordered and unordered)
/// - Links and images
/// - Code blocks and inline code
/// - Blockquotes
/// - Line breaks
pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Anki prefers <br> for line breaks, clean up trailing newlines
    html_output = html_output.trim().to_string();

    // Convert <p> blocks to simpler format for Anki
    // Single paragraphs don't need <p> tags
    if html_output.starts_with("<p>") && html_output.ends_with("</p>") {
        let inner = &html_output[3..html_output.len() - 4];
        if !inner.contains("<p>") {
            return inner.to_string();
        }
    }

    // Replace paragraph breaks with <br><br> for Anki compatibility
    html_output = html_output.replace("</p>\n<p>", "<br><br>");
    html_output = html_output.replace("<p>", "");
    html_output = html_output.replace("</p>", "");

    html_output
}

/// Convert HTML to Markdown.
///
/// Best-effort conversion that handles common HTML elements:
/// - `<b>`, `<strong>` -> `**bold**`
/// - `<i>`, `<em>` -> `*italic*`
/// - `<ul>`, `<ol>`, `<li>` -> lists
/// - `<a>` -> `[text](url)`
/// - `<br>` -> newlines
///
/// Note: Some HTML styling may be lost in conversion (e.g., colors, fonts).
pub fn html_to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

/// Check if a string appears to be HTML (contains HTML tags).
pub fn is_html(s: &str) -> bool {
    s.contains('<') && s.contains('>')
}

/// Check if a string appears to be Markdown (contains markdown syntax).
pub fn is_markdown(s: &str) -> bool {
    // Check for common markdown patterns
    s.contains("**")
        || s.contains("__")
        || (s.contains('*') && !s.contains('<'))
        || (s.contains('_') && !s.contains('<'))
        || s.contains("```")
        || s.contains("- ")
        || s.contains("1. ")
        || s.starts_with('#')
        || s.contains("\n# ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_bold() {
        let md = "**bold text**";
        let html = markdown_to_html(md);
        assert!(html.contains("<strong>bold text</strong>"));
    }

    #[test]
    fn test_markdown_to_html_italic() {
        let md = "*italic text*";
        let html = markdown_to_html(md);
        assert!(html.contains("<em>italic text</em>"));
    }

    #[test]
    fn test_markdown_to_html_list() {
        let md = "- item 1\n- item 2";
        let html = markdown_to_html(md);
        assert!(html.contains("<li>"));
        assert!(html.contains("item 1"));
        assert!(html.contains("item 2"));
    }

    #[test]
    fn test_markdown_to_html_combined() {
        let md = "**bold** and *italic*";
        let html = markdown_to_html(md);
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_html_to_markdown_bold() {
        let html = "<strong>bold</strong>";
        let md = html_to_markdown(html);
        assert!(md.contains("**bold**"));
    }

    #[test]
    fn test_html_to_markdown_italic() {
        let html = "<em>italic</em>";
        let md = html_to_markdown(html);
        assert!(md.contains("*italic*") || md.contains("_italic_"));
    }

    #[test]
    fn test_html_to_markdown_list() {
        let html = "<ul><li>item 1</li><li>item 2</li></ul>";
        let md = html_to_markdown(html);
        assert!(md.contains("item 1"));
        assert!(md.contains("item 2"));
    }

    #[test]
    fn test_roundtrip_simple() {
        let original = "**bold** and *italic*";
        let html = markdown_to_html(original);
        let back = html_to_markdown(&html);
        // Roundtrip should preserve meaning, though format may differ slightly
        assert!(back.contains("bold"));
        assert!(back.contains("italic"));
    }

    #[test]
    fn test_is_html() {
        assert!(is_html("<b>bold</b>"));
        assert!(is_html("<p>paragraph</p>"));
        assert!(!is_html("plain text"));
        assert!(!is_html("**markdown**"));
    }

    #[test]
    fn test_is_markdown() {
        assert!(is_markdown("**bold**"));
        assert!(is_markdown("*italic*"));
        assert!(is_markdown("- list item"));
        assert!(!is_markdown("plain text"));
    }
}
