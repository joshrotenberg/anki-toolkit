//! Tests for import workflow operations.

mod common;

use ankit_engine::NoteBuilder;
use ankit_engine::import::{SmartAddOptions, SmartAddStatus};
use common::{
    engine_for_mock, mock_action, mock_action_times, mock_anki_response, setup_mock_server,
};

#[tokio::test]
async fn test_smart_add_success() {
    let server = setup_mock_server().await;

    // Mock modelNames for validation
    mock_action(&server, "modelNames", mock_anki_response(vec!["Basic"])).await;

    // Mock deckNames for validation
    mock_action(&server, "deckNames", mock_anki_response(vec!["Japanese"])).await;

    // Mock modelFieldNames - called twice (validation + duplicate check)
    mock_action_times(
        &server,
        "modelFieldNames",
        mock_anki_response(vec!["Front", "Back"]),
        2,
    )
    .await;

    // Mock findNotes - called twice (duplicate check + tag suggestions)
    // Both return empty for simplicity (no duplicates, no similar notes)
    mock_action_times(
        &server,
        "findNotes",
        mock_anki_response(Vec::<i64>::new()),
        2,
    )
    .await;

    // Mock addNote - success
    mock_action(&server, "addNote", mock_anki_response(12345_i64)).await;

    let engine = engine_for_mock(&server);
    let note = NoteBuilder::new("Japanese", "Basic")
        .field("Front", "hello")
        .field("Back", "world")
        .build();

    let result = engine
        .import()
        .smart_add(&note, SmartAddOptions::default())
        .await
        .unwrap();

    assert!(matches!(result.status, SmartAddStatus::Added));
    assert_eq!(result.note_id, Some(12345));
    assert!(result.similar_notes.is_empty());
    // No similar notes found, so no tag suggestions
    assert!(result.suggested_tags.is_empty());
}

#[tokio::test]
async fn test_smart_add_rejected_duplicate() {
    let server = setup_mock_server().await;

    // Mock modelNames for validation
    mock_action(&server, "modelNames", mock_anki_response(vec!["Basic"])).await;

    // Mock deckNames for validation
    mock_action(&server, "deckNames", mock_anki_response(vec!["Japanese"])).await;

    // Mock modelFieldNames - called twice (validation + duplicate check)
    mock_action_times(
        &server,
        "modelFieldNames",
        mock_anki_response(vec!["Front", "Back"]),
        2,
    )
    .await;

    // Mock findNotes for duplicate check - found duplicate!
    mock_action(&server, "findNotes", mock_anki_response(vec![999_i64])).await;

    // Mock notesInfo for getting tags from duplicate
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![serde_json::json!({
            "noteId": 999_i64,
            "modelName": "Basic",
            "tags": ["existing-tag"],
            "fields": {
                "Front": {"value": "hello", "order": 0},
                "Back": {"value": "existing back", "order": 1}
            }
        })]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let note = NoteBuilder::new("Japanese", "Basic")
        .field("Front", "hello")
        .field("Back", "world")
        .build();

    let result = engine
        .import()
        .smart_add(&note, SmartAddOptions::default())
        .await
        .unwrap();

    assert!(matches!(
        result.status,
        SmartAddStatus::RejectedDuplicate { existing_id: 999 }
    ));
    assert_eq!(result.note_id, None);
    assert_eq!(result.similar_notes, vec![999]);
    assert!(result.suggested_tags.contains(&"existing-tag".to_string()));
}

#[tokio::test]
async fn test_smart_add_duplicate_allowed() {
    let server = setup_mock_server().await;

    // Mock modelNames for validation
    mock_action(&server, "modelNames", mock_anki_response(vec!["Basic"])).await;

    // Mock deckNames for validation
    mock_action(&server, "deckNames", mock_anki_response(vec!["Japanese"])).await;

    // Mock modelFieldNames - called twice (validation + duplicate check)
    mock_action_times(
        &server,
        "modelFieldNames",
        mock_anki_response(vec!["Front", "Back"]),
        2,
    )
    .await;

    // Mock findNotes - only called once (duplicate check finds match, tags populated from that)
    mock_action(&server, "findNotes", mock_anki_response(vec![999_i64])).await;

    // Mock notesInfo - only called once (for duplicate's tags)
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![serde_json::json!({
            "noteId": 999_i64,
            "modelName": "Basic",
            "tags": ["vocab"],
            "fields": {
                "Front": {"value": "hello", "order": 0},
                "Back": {"value": "existing", "order": 1}
            }
        })]),
    )
    .await;

    // Mock addNote - success (with allow_duplicate)
    mock_action(&server, "addNote", mock_anki_response(12346_i64)).await;

    let engine = engine_for_mock(&server);
    let note = NoteBuilder::new("Japanese", "Basic")
        .field("Front", "hello")
        .field("Back", "world")
        .build();

    let options = SmartAddOptions {
        reject_on_duplicate: false, // Allow duplicates with warning
        ..Default::default()
    };

    let result = engine.import().smart_add(&note, options).await.unwrap();

    assert!(matches!(
        result.status,
        SmartAddStatus::AddedWithWarning { .. }
    ));
    assert_eq!(result.note_id, Some(12346));
    assert_eq!(result.similar_notes, vec![999]);
    // Tags came from the duplicate note
    assert!(result.suggested_tags.contains(&"vocab".to_string()));
}

#[tokio::test]
async fn test_smart_add_rejected_empty_fields() {
    let server = setup_mock_server().await;

    let engine = engine_for_mock(&server);
    let note = NoteBuilder::new("Japanese", "Basic")
        .field("Front", "hello")
        .field("Back", "   ") // Empty/whitespace
        .build();

    let result = engine
        .import()
        .smart_add(&note, SmartAddOptions::default())
        .await
        .unwrap();

    assert!(matches!(
        result.status,
        SmartAddStatus::RejectedEmptyFields { ref fields } if fields.contains(&"Back".to_string())
    ));
    assert_eq!(result.note_id, None);
}

#[tokio::test]
async fn test_smart_add_rejected_invalid_model() {
    let server = setup_mock_server().await;

    // Mock modelNames - model doesn't exist
    mock_action(
        &server,
        "modelNames",
        mock_anki_response(vec!["Basic", "Cloze"]),
    )
    .await;

    // Mock deckNames
    mock_action(&server, "deckNames", mock_anki_response(vec!["Japanese"])).await;

    let engine = engine_for_mock(&server);
    let note = NoteBuilder::new("Japanese", "NonExistentModel")
        .field("Front", "hello")
        .field("Back", "world")
        .build();

    let result = engine
        .import()
        .smart_add(&note, SmartAddOptions::default())
        .await
        .unwrap();

    assert!(matches!(
        result.status,
        SmartAddStatus::RejectedInvalid { .. }
    ));
    assert_eq!(result.note_id, None);
}

#[tokio::test]
async fn test_smart_add_no_checks() {
    let server = setup_mock_server().await;

    // Mock modelNames for validation
    mock_action(&server, "modelNames", mock_anki_response(vec!["Basic"])).await;

    // Mock deckNames for validation
    mock_action(&server, "deckNames", mock_anki_response(vec!["Japanese"])).await;

    // Mock modelFieldNames for validation only (no duplicate check)
    mock_action(
        &server,
        "modelFieldNames",
        mock_anki_response(vec!["Front", "Back"]),
    )
    .await;

    // Mock addNote - success
    mock_action(&server, "addNote", mock_anki_response(12347_i64)).await;

    let engine = engine_for_mock(&server);
    let note = NoteBuilder::new("Japanese", "Basic")
        .field("Front", "hello")
        .field("Back", "world")
        .build();

    let options = SmartAddOptions {
        check_duplicates: false,
        suggest_tags: false,
        check_empty_fields: false,
        ..Default::default()
    };

    let result = engine.import().smart_add(&note, options).await.unwrap();

    assert!(matches!(result.status, SmartAddStatus::Added));
    assert_eq!(result.note_id, Some(12347));
    assert!(result.suggested_tags.is_empty());
}
