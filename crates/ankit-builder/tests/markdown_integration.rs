//! Integration tests for Markdown field support.
//!
//! These tests verify that markdown fields are correctly converted
//! to HTML when building .apkg files and that the conversions work
//! correctly in both directions.

use std::io::Read;

use ankit_builder::{DeckBuilder, DeckDefinition};
use rusqlite::Connection;
use tempfile::tempdir;
use zip::ZipArchive;

/// Helper to extract and open the SQLite database from an .apkg file.
fn open_apkg_database(apkg_path: &std::path::Path) -> Connection {
    let file = std::fs::File::open(apkg_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    let mut db_file = archive.by_name("collection.anki2").unwrap();
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("collection.anki2");
    let mut db_bytes = Vec::new();
    db_file.read_to_end(&mut db_bytes).unwrap();
    std::fs::write(&db_path, &db_bytes).unwrap();

    std::mem::forget(temp_dir);

    Connection::open(&db_path).unwrap()
}

const MARKDOWN_TOML: &str = r#"
[package]
name = "Markdown Test"
version = "1.0.0"

[[models]]
name = "MarkdownModel"
fields = ["Question", "Answer", "Notes"]
markdown_fields = ["Answer", "Notes"]

[[models.templates]]
name = "Card 1"
front = "{{Question}}"
back = "{{FrontSide}}<hr>{{Answer}}<br>{{Notes}}"

[[decks]]
name = "Markdown Deck"

[[notes]]
deck = "Markdown Deck"
model = "MarkdownModel"
tags = ["test"]

[notes.fields]
Question = "What is **bold** text?"
Answer = "Text wrapped in **double asterisks** or __double underscores__"
Notes = "- Item 1\n- Item 2\n- Item 3"

[[notes]]
deck = "Markdown Deck"
model = "MarkdownModel"

[notes.fields]
Question = "How do you make a link?"
Answer = "Use [text](url) syntax"
Notes = "*Italic* and **bold** can be combined"
"#;

#[test]
fn test_markdown_fields_converted_to_html_in_apkg() {
    let builder = DeckBuilder::parse(MARKDOWN_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let fields: Vec<String> = conn
        .prepare("SELECT flds FROM notes ORDER BY id")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    // First note - Answer and Notes should be HTML, Question should remain as-is
    // Fields are separated by \x1f
    let note1_fields: Vec<&str> = fields[0].split('\x1f').collect();

    // Question field is NOT in markdown_fields, so markdown syntax stays as-is
    assert!(
        note1_fields[0].contains("**bold**"),
        "Question should keep markdown syntax: {}",
        note1_fields[0]
    );

    // Answer field IS in markdown_fields, so should be converted to HTML
    assert!(
        note1_fields[1].contains("<strong>"),
        "Answer should have HTML strong tags: {}",
        note1_fields[1]
    );

    // Notes field IS in markdown_fields, list should be HTML
    assert!(
        note1_fields[2].contains("<li>"),
        "Notes should have HTML list tags: {}",
        note1_fields[2]
    );
}

#[test]
fn test_markdown_to_html_on_definition() {
    let mut definition = DeckDefinition::parse(MARKDOWN_TOML).unwrap();

    // Before conversion, fields have markdown
    let answer = definition.notes[0].fields.get("Answer").unwrap();
    assert!(answer.contains("**double asterisks**"));

    // Convert markdown to HTML
    definition.markdown_to_html();

    // After conversion, markdown fields have HTML
    let answer = definition.notes[0].fields.get("Answer").unwrap();
    assert!(
        answer.contains("<strong>"),
        "Answer should have HTML after conversion: {}",
        answer
    );

    // Non-markdown field should be unchanged
    let question = definition.notes[0].fields.get("Question").unwrap();
    assert!(
        question.contains("**bold**"),
        "Question should keep markdown: {}",
        question
    );
}

#[test]
fn test_html_to_markdown_on_definition() {
    // Start with HTML content
    let toml_with_html = r#"
[package]
name = "HTML Test"
version = "1.0.0"

[[models]]
name = "HtmlModel"
fields = ["Front", "Back"]
markdown_fields = ["Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "HtmlModel"

[notes.fields]
Front = "Question"
Back = "<strong>bold</strong> and <em>italic</em>"
"#;

    let mut definition = DeckDefinition::parse(toml_with_html).unwrap();

    // Convert HTML to markdown
    definition.html_to_markdown();

    let back = definition.notes[0].fields.get("Back").unwrap();
    assert!(
        back.contains("**bold**") || back.contains("__bold__"),
        "Back should have markdown bold: {}",
        back
    );
    assert!(
        back.contains("*italic*") || back.contains("_italic_"),
        "Back should have markdown italic: {}",
        back
    );
}

#[test]
fn test_fields_as_html_helper() {
    let definition = DeckDefinition::parse(MARKDOWN_TOML).unwrap();
    let note = &definition.notes[0];
    let model = definition.get_model(&note.model).unwrap();

    let html_fields = note.fields_as_html(&model.markdown_fields);

    // Question is not in markdown_fields - stays as markdown
    assert!(
        html_fields.get("Question").unwrap().contains("**bold**"),
        "Question should remain markdown"
    );

    // Answer is in markdown_fields - converted to HTML
    assert!(
        html_fields.get("Answer").unwrap().contains("<strong>"),
        "Answer should be HTML"
    );

    // Notes is in markdown_fields - list converted to HTML
    assert!(
        html_fields.get("Notes").unwrap().contains("<li>"),
        "Notes should have HTML list"
    );
}

#[test]
fn test_set_markdown_fields() {
    let mut definition = DeckDefinition::parse(MARKDOWN_TOML).unwrap();

    // Initially has Answer and Notes as markdown fields
    let model = definition.get_model("MarkdownModel").unwrap();
    assert_eq!(model.markdown_fields.len(), 2);

    // Change to only Question
    definition.set_markdown_fields("MarkdownModel", &["Question"]);

    let model = definition.get_model("MarkdownModel").unwrap();
    assert_eq!(model.markdown_fields, vec!["Question"]);
}

#[test]
fn test_markdown_roundtrip() {
    let original_md = "**bold** and *italic* with a [link](https://example.com)";

    let html = ankit_builder::markdown::markdown_to_html(original_md);
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("href=\"https://example.com\""));

    let back_to_md = ankit_builder::markdown::html_to_markdown(&html);
    // Roundtrip may not be exact but should preserve meaning
    assert!(
        back_to_md.contains("bold"),
        "Should preserve bold text: {}",
        back_to_md
    );
    assert!(
        back_to_md.contains("italic"),
        "Should preserve italic text: {}",
        back_to_md
    );
    assert!(
        back_to_md.contains("example.com"),
        "Should preserve link: {}",
        back_to_md
    );
}

#[test]
fn test_markdown_list_conversion() {
    let md = "- First item\n- Second item\n- Third item";
    let html = ankit_builder::markdown::markdown_to_html(md);

    assert!(html.contains("<ul>"), "Should have ul tag: {}", html);
    assert!(html.contains("<li>"), "Should have li tags: {}", html);
    assert!(html.contains("First item"), "Should have content: {}", html);
}

#[test]
fn test_markdown_code_block() {
    let md = "```rust\nfn main() {}\n```";
    let html = ankit_builder::markdown::markdown_to_html(md);

    assert!(
        html.contains("<code>") || html.contains("<pre>"),
        "Should have code formatting: {}",
        html
    );
    assert!(html.contains("fn main()"), "Should preserve code: {}", html);
}

#[test]
fn test_empty_markdown_fields_no_conversion() {
    let toml_no_md = r#"
[package]
name = "No Markdown"
version = "1.0.0"

[[models]]
name = "PlainModel"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "PlainModel"

[notes.fields]
Front = "**stays as markdown**"
Back = "*also stays*"
"#;

    let builder = DeckBuilder::parse(toml_no_md).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let fields: String = conn
        .query_row("SELECT flds FROM notes", [], |row| row.get(0))
        .unwrap();

    // No markdown_fields defined, so content should be unchanged
    assert!(
        fields.contains("**stays as markdown**"),
        "Front should keep markdown: {}",
        fields
    );
    assert!(
        fields.contains("*also stays*"),
        "Back should keep markdown: {}",
        fields
    );
}

#[test]
fn test_toml_serialization_preserves_markdown_fields() {
    let definition = DeckDefinition::parse(MARKDOWN_TOML).unwrap();
    let toml_output = definition.to_toml().unwrap();

    // Re-parse and verify markdown_fields survived
    let reparsed = DeckDefinition::parse(&toml_output).unwrap();
    let model = reparsed.get_model("MarkdownModel").unwrap();

    assert!(
        model.markdown_fields.contains(&"Answer".to_string()),
        "Should preserve Answer in markdown_fields"
    );
    assert!(
        model.markdown_fields.contains(&"Notes".to_string()),
        "Should preserve Notes in markdown_fields"
    );
}
