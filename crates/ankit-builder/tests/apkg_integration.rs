//! Integration tests for .apkg generation.
//!
//! These tests build actual .apkg files and verify their contents
//! by inspecting the SQLite database and ZIP structure.

use std::collections::HashMap;
use std::io::Read;

use ankit_builder::{DeckBuilder, DeckDefinition};
use rusqlite::Connection;
use tempfile::tempdir;
use zip::ZipArchive;

/// Helper to extract and open the SQLite database from an .apkg file.
fn open_apkg_database(apkg_path: &std::path::Path) -> Connection {
    let file = std::fs::File::open(apkg_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    // Extract collection.anki2 to a temp file
    let mut db_file = archive.by_name("collection.anki2").unwrap();
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("collection.anki2");
    let mut db_bytes = Vec::new();
    db_file.read_to_end(&mut db_bytes).unwrap();
    std::fs::write(&db_path, &db_bytes).unwrap();

    // Keep temp_dir alive by leaking it (tests are short-lived anyway)
    std::mem::forget(temp_dir);

    Connection::open(&db_path).unwrap()
}

/// Helper to get the media manifest from an .apkg file.
fn get_media_manifest(apkg_path: &std::path::Path) -> HashMap<String, String> {
    let file = std::fs::File::open(apkg_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    let mut media_file = archive.by_name("media").unwrap();
    let mut content = String::new();
    media_file.read_to_string(&mut content).unwrap();

    serde_json::from_str(&content).unwrap()
}

const BASIC_TOML: &str = r#"
[package]
name = "Test Package"
version = "1.0.0"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{FrontSide}}<hr>{{Back}}"

[[decks]]
name = "Test Deck"

[[notes]]
deck = "Test Deck"
model = "Basic"
tags = ["test", "example"]

[notes.fields]
Front = "What is 2+2?"
Back = "4"

[[notes]]
deck = "Test Deck"
model = "Basic"

[notes.fields]
Front = "Capital of France?"
Back = "Paris"
"#;

#[test]
fn test_apkg_contains_expected_files() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let file = std::fs::File::open(&path).unwrap();
    let archive = ZipArchive::new(file).unwrap();

    let names: Vec<_> = archive.file_names().collect();
    assert!(
        names.contains(&"collection.anki2"),
        "Missing collection.anki2"
    );
    assert!(names.contains(&"media"), "Missing media manifest");
}

#[test]
fn test_apkg_database_schema() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    // Verify all expected tables exist
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(tables.contains(&"col".to_string()), "Missing col table");
    assert!(tables.contains(&"notes".to_string()), "Missing notes table");
    assert!(tables.contains(&"cards".to_string()), "Missing cards table");
    assert!(
        tables.contains(&"revlog".to_string()),
        "Missing revlog table"
    );
    assert!(
        tables.contains(&"graves".to_string()),
        "Missing graves table"
    );
}

#[test]
fn test_apkg_note_count() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let note_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .unwrap();

    assert_eq!(note_count, 2, "Expected 2 notes");
}

#[test]
fn test_apkg_card_count() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let card_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))
        .unwrap();

    // 2 notes * 1 template = 2 cards
    assert_eq!(card_count, 2, "Expected 2 cards");
}

#[test]
fn test_apkg_multiple_templates() {
    let toml = r#"
[package]
name = "Multi Template"

[[models]]
name = "Reversible"
fields = ["Front", "Back"]

[[models.templates]]
name = "Forward"
front = "{{Front}}"
back = "{{Back}}"

[[models.templates]]
name = "Reverse"
front = "{{Back}}"
back = "{{Front}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "Reversible"

[notes.fields]
Front = "Hello"
Back = "Hola"
"#;

    let builder = DeckBuilder::parse(toml).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let card_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))
        .unwrap();

    // 1 note * 2 templates = 2 cards
    assert_eq!(card_count, 2, "Expected 2 cards for 2 templates");
}

#[test]
fn test_apkg_note_fields() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
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

    // Fields are separated by \x1f
    assert!(fields[0].contains("What is 2+2?"));
    assert!(fields[0].contains("4"));
    assert!(fields[1].contains("Capital of France?"));
    assert!(fields[1].contains("Paris"));
}

#[test]
fn test_apkg_note_tags() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let tags: Vec<String> = conn
        .prepare("SELECT tags FROM notes ORDER BY id")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    // First note has tags, second doesn't
    assert!(tags[0].contains("test"));
    assert!(tags[0].contains("example"));
    assert!(tags[1].is_empty() || tags[1].trim().is_empty());
}

#[test]
fn test_apkg_deck_in_col() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let decks_json: String = conn
        .query_row("SELECT decks FROM col", [], |row| row.get(0))
        .unwrap();

    let decks: serde_json::Value = serde_json::from_str(&decks_json).unwrap();

    // Should have Default deck and our Test Deck
    let deck_names: Vec<&str> = decks
        .as_object()
        .unwrap()
        .values()
        .map(|d| d["name"].as_str().unwrap())
        .collect();

    assert!(deck_names.contains(&"Default"));
    assert!(deck_names.contains(&"Test Deck"));
}

#[test]
fn test_apkg_model_in_col() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let models_json: String = conn
        .query_row("SELECT models FROM col", [], |row| row.get(0))
        .unwrap();

    let models: serde_json::Value = serde_json::from_str(&models_json).unwrap();

    // Should have our Basic model
    let model = models.as_object().unwrap().values().next().unwrap();
    assert_eq!(model["name"].as_str().unwrap(), "Basic");

    // Check fields
    let fields = model["flds"].as_array().unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0]["name"].as_str().unwrap(), "Front");
    assert_eq!(fields[1]["name"].as_str().unwrap(), "Back");

    // Check templates
    let templates = model["tmpls"].as_array().unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0]["name"].as_str().unwrap(), "Card 1");
}

#[test]
fn test_apkg_empty_media_manifest() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let manifest = get_media_manifest(&path);
    assert!(manifest.is_empty(), "Expected empty media manifest");
}

#[test]
fn test_apkg_hierarchical_deck() {
    let toml = r#"
[package]
name = "Hierarchical"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Parent::Child::Grandchild"

[[notes]]
deck = "Parent::Child::Grandchild"
model = "Basic"

[notes.fields]
Front = "Q"
Back = "A"
"#;

    let builder = DeckBuilder::parse(toml).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let decks_json: String = conn
        .query_row("SELECT decks FROM col", [], |row| row.get(0))
        .unwrap();

    let decks: serde_json::Value = serde_json::from_str(&decks_json).unwrap();
    let deck_names: Vec<&str> = decks
        .as_object()
        .unwrap()
        .values()
        .map(|d| d["name"].as_str().unwrap())
        .collect();

    assert!(deck_names.contains(&"Parent::Child::Grandchild"));
}

#[test]
fn test_apkg_custom_css() {
    let toml = r#"
[package]
name = "Custom CSS"

[[models]]
name = "Styled"
fields = ["Front", "Back"]
css = ".card { background: blue; color: white; }"

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "Styled"

[notes.fields]
Front = "Q"
Back = "A"
"#;

    let builder = DeckBuilder::parse(toml).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let models_json: String = conn
        .query_row("SELECT models FROM col", [], |row| row.get(0))
        .unwrap();

    let models: serde_json::Value = serde_json::from_str(&models_json).unwrap();
    let model = models.as_object().unwrap().values().next().unwrap();

    assert_eq!(
        model["css"].as_str().unwrap(),
        ".card { background: blue; color: white; }"
    );
}

#[test]
fn test_apkg_sort_field() {
    let toml = r#"
[package]
name = "Sort Field"

[[models]]
name = "Custom Sort"
fields = ["A", "B", "C"]
sort_field = "B"

[[models.templates]]
name = "Card 1"
front = "{{A}}"
back = "{{B}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "Custom Sort"

[notes.fields]
A = "First"
B = "Second"
C = "Third"
"#;

    let builder = DeckBuilder::parse(toml).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    let models_json: String = conn
        .query_row("SELECT models FROM col", [], |row| row.get(0))
        .unwrap();

    let models: serde_json::Value = serde_json::from_str(&models_json).unwrap();
    let model = models.as_object().unwrap().values().next().unwrap();

    // sortf should be 1 (index of "B")
    assert_eq!(model["sortf"].as_i64().unwrap(), 1);

    // sfld in notes should be "Second"
    let sfld: String = conn
        .query_row("SELECT sfld FROM notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(sfld, "Second");
}

#[test]
fn test_apkg_deterministic_ids() {
    // Building the same deck twice should produce the same model/deck IDs
    let builder1 = DeckBuilder::parse(BASIC_TOML).unwrap();
    let builder2 = DeckBuilder::parse(BASIC_TOML).unwrap();

    let dir = tempdir().unwrap();
    let path1 = dir.path().join("test1.apkg");
    let path2 = dir.path().join("test2.apkg");

    builder1.write_apkg(&path1).unwrap();
    builder2.write_apkg(&path2).unwrap();

    let conn1 = open_apkg_database(&path1);
    let conn2 = open_apkg_database(&path2);

    // Get model IDs
    let models1: String = conn1
        .query_row("SELECT models FROM col", [], |row| row.get(0))
        .unwrap();
    let models2: String = conn2
        .query_row("SELECT models FROM col", [], |row| row.get(0))
        .unwrap();

    let m1: serde_json::Value = serde_json::from_str(&models1).unwrap();
    let m2: serde_json::Value = serde_json::from_str(&models2).unwrap();

    let id1: i64 = m1
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let id2: i64 = m2
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .parse()
        .unwrap();

    assert_eq!(id1, id2, "Model IDs should be deterministic");
}

#[test]
fn test_apkg_cards_reference_correct_notes() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    // Every card should reference a valid note
    let orphan_cards: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cards WHERE nid NOT IN (SELECT id FROM notes)",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(orphan_cards, 0, "All cards should reference valid notes");
}

#[test]
fn test_apkg_cards_new_state() {
    let builder = DeckBuilder::parse(BASIC_TOML).unwrap();
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.apkg");

    builder.write_apkg(&path).unwrap();

    let conn = open_apkg_database(&path);

    // All cards should be in new state (type=0, queue=0)
    let non_new: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cards WHERE type != 0 OR queue != 0",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(non_new, 0, "All cards should be in new state");
}

#[test]
fn test_schema_validation_missing_model() {
    let toml = r#"
[package]
name = "Invalid"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "NonExistent"

[notes.fields]
Front = "Q"
"#;

    let result = DeckDefinition::parse(toml);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("model not found"));
}

#[test]
fn test_schema_validation_missing_deck() {
    let toml = r#"
[package]
name = "Invalid"

[[models]]
name = "Basic"
fields = ["Front"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Front}}"

[[notes]]
deck = "NonExistent"
model = "Basic"

[notes.fields]
Front = "Q"
"#;

    let result = DeckDefinition::parse(toml);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("deck not found"));
}

#[test]
fn test_schema_validation_invalid_field() {
    let toml = r#"
[package]
name = "Invalid"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test"

[[notes]]
deck = "Test"
model = "Basic"

[notes.fields]
Front = "Q"
InvalidField = "X"
"#;

    let result = DeckDefinition::parse(toml);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("field") && err.contains("not found"));
}
