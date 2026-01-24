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

#[tokio::test]
async fn test_add_many_notes() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "addNotes",
        mock_anki_response(vec![Some(1000_i64), Some(1001), Option::<i64>::None]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();

    let notes = vec![
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Q1")
            .field("Back", "A1")
            .build(),
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Q2")
            .field("Back", "A2")
            .build(),
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Duplicate")
            .field("Back", "Duplicate")
            .build(),
    ];

    let ids = client.notes().add_many(&notes).await.unwrap();

    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0], Some(1000));
    assert_eq!(ids[1], Some(1001));
    assert_eq!(ids[2], None); // Failed (e.g., duplicate)
}

#[tokio::test]
async fn test_can_add_notes() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "canAddNotes",
        mock_anki_response(vec![true, true, false]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();

    let notes = vec![
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Q1")
            .field("Back", "A1")
            .build(),
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Q2")
            .field("Back", "A2")
            .build(),
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Duplicate")
            .field("Back", "Duplicate")
            .build(),
    ];

    let can_add = client.notes().can_add(&notes).await.unwrap();

    assert_eq!(can_add, vec![true, true, false]);
}

#[tokio::test]
async fn test_can_add_detailed() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "canAddNotesWithErrorDetail",
        mock_anki_response(vec![
            serde_json::json!({"canAdd": true, "error": null}),
            serde_json::json!({"canAdd": false, "error": "duplicate note"}),
        ]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();

    let notes = vec![
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Q1")
            .field("Back", "A1")
            .build(),
        NoteBuilder::new("Default", "Basic")
            .field("Front", "Duplicate")
            .field("Back", "Duplicate")
            .build(),
    ];

    let results = client.notes().can_add_detailed(&notes).await.unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].can_add);
    assert!(results[0].error.is_none());
    assert!(!results[1].can_add);
    assert!(results[1].error.as_ref().unwrap().contains("duplicate"));
}

#[tokio::test]
async fn test_update_fields() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "updateNoteFields",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();

    let mut fields = std::collections::HashMap::new();
    fields.insert("Front".to_string(), "Updated front".to_string());
    fields.insert("Back".to_string(), "Updated back".to_string());

    let result = client.notes().update_fields(1234567890, &fields).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_tags() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "getNoteTags",
        mock_anki_response(vec!["tag1", "tag2", "vocabulary"]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let tags = client.notes().get_tags(1234567890).await.unwrap();

    assert_eq!(tags, vec!["tag1", "tag2", "vocabulary"]);
}

#[tokio::test]
async fn test_add_tags() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "addTags",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.notes().add_tags(&[1234567890], "new-tag").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_tags() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "removeTags",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.notes().remove_tags(&[1234567890], "old-tag").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_unused_tags() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "clearUnusedTags",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.notes().clear_unused_tags().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_replace_tags() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "replaceTags",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client
        .notes()
        .replace_tags(&[1234567890], "old-tag", "new-tag")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_replace_tags_all() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "replaceTagsInAllNotes",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.notes().replace_tags_all("old-tag", "new-tag").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_notes_mod_time() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "notesModTime",
        mock_anki_response(vec![
            serde_json::json!({"noteId": 123, "mod": 1705330000}),
            serde_json::json!({"noteId": 456, "mod": 1705330100}),
        ]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let times = client.notes().mod_time(&[123, 456]).await.unwrap();

    assert_eq!(times.len(), 2);
    assert_eq!(times[0].note_id, 123);
    assert_eq!(times[0].mod_time, 1705330000);
}

#[tokio::test]
async fn test_remove_empty_notes() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "removeEmptyNotes",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.notes().remove_empty().await;

    assert!(result.is_ok());
}
