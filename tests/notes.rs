//! Tests for note AnkiConnect actions.

mod common;

use common::{mock_action, mock_anki_error, mock_anki_response, setup_mock_server};
use yanki::{AnkiClient, NoteBuilder};

#[tokio::test]
async fn test_add_note() {
    let server = setup_mock_server().await;
    mock_action(&server, "addNote", mock_anki_response(1234567890_i64)).await;

    let client = AnkiClient::builder().url(server.uri()).build();

    let note = NoteBuilder::new("Default", "Basic")
        .field("Front", "Hello")
        .field("Back", "World")
        .tag("test")
        .build();

    let note_id = client.notes().add(note).await.unwrap();
    assert_eq!(note_id, 1234567890);
}

#[tokio::test]
async fn test_find_notes() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "findNotes",
        mock_anki_response(vec![1_i64, 2, 3, 4, 5]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let notes = client.notes().find("deck:Default").await.unwrap();

    assert_eq!(notes, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_notes_info() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![serde_json::json!({
            "noteId": 1234567890_i64,
            "modelName": "Basic",
            "tags": ["test", "vocabulary"],
            "fields": {
                "Front": {"value": "Hello", "order": 0},
                "Back": {"value": "World", "order": 1}
            },
            "cards": [9876543210_i64]
        })]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let notes = client.notes().info(&[1234567890]).await.unwrap();

    assert_eq!(notes.len(), 1);
    let note = &notes[0];
    assert_eq!(note.note_id, 1234567890);
    assert_eq!(note.model_name, "Basic");
    assert_eq!(note.tags, vec!["test", "vocabulary"]);
    assert_eq!(note.fields.get("Front").unwrap().value, "Hello");
}

#[tokio::test]
async fn test_delete_notes() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "deleteNotes",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.notes().delete(&[1234567890]).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_duplicate_note_error() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "addNote",
        mock_anki_error("cannot create note because it is a duplicate"),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();

    let note = NoteBuilder::new("Default", "Basic")
        .field("Front", "Hello")
        .field("Back", "World")
        .build();

    let result = client.notes().add(note).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("duplicate"));
}

#[test]
fn test_note_builder() {
    let note = NoteBuilder::new("My Deck", "Basic")
        .field("Front", "Question")
        .field("Back", "Answer")
        .tag("tag1")
        .tags(["tag2", "tag3"])
        .allow_duplicate(true)
        .build();

    assert_eq!(note.deck_name, "My Deck");
    assert_eq!(note.model_name, "Basic");
    assert_eq!(note.fields.get("Front"), Some(&"Question".to_string()));
    assert_eq!(note.fields.get("Back"), Some(&"Answer".to_string()));
    assert_eq!(note.tags, vec!["tag1", "tag2", "tag3"]);
    assert!(note.options.is_some());
    assert_eq!(note.options.unwrap().allow_duplicate, Some(true));
}
