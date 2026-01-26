//! Tests for enrich workflow operations.

mod common;

use ankit_engine::enrich::EnrichQuery;
use common::{
    engine_for_mock, mock_action, mock_action_times, mock_anki_response, setup_mock_server,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_pipeline_find_candidates() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action(&server, "findNotes", mock_anki_response(vec![1_i64, 2, 3])).await;

    // Mock notesInfo
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 1_i64,
                "modelName": "Basic",
                "tags": ["vocab"],
                "fields": {
                    "Front": {"value": "hello", "order": 0},
                    "Back": {"value": "world", "order": 1},
                    "Example": {"value": "", "order": 2}  // empty
                }
            }),
            serde_json::json!({
                "noteId": 2_i64,
                "modelName": "Basic",
                "tags": ["vocab"],
                "fields": {
                    "Front": {"value": "goodbye", "order": 0},
                    "Back": {"value": "", "order": 1},  // empty
                    "Example": {"value": "", "order": 2}  // empty
                }
            }),
            serde_json::json!({
                "noteId": 3_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "test", "order": 0},
                    "Back": {"value": "has value", "order": 1},  // not empty
                    "Example": {"value": "also has value", "order": 2}  // not empty
                }
            }),
        ]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let query = EnrichQuery {
        search: "deck:Test".to_string(),
        empty_fields: vec!["Example".to_string(), "Back".to_string()],
    };

    let pipeline = engine.enrich().pipeline(&query).await.unwrap();

    // Should have 2 candidates (notes 1 and 2 have empty fields)
    assert_eq!(pipeline.len(), 2);
    assert!(!pipeline.is_empty());

    // Check by_missing_field grouping
    let by_field = pipeline.by_missing_field();
    assert_eq!(by_field.get("Example").map(|v| v.len()), Some(2)); // notes 1 and 2
    assert_eq!(by_field.get("Back").map(|v| v.len()), Some(1)); // only note 2

    // Check by_model grouping
    let by_model = pipeline.by_model();
    assert_eq!(by_model.get("Basic").map(|v| v.len()), Some(2)); // only notes 1 and 2 (note 3 has no empty fields)
}

#[tokio::test]
async fn test_pipeline_update_and_commit() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action(&server, "findNotes", mock_anki_response(vec![1_i64, 2])).await;

    // Mock notesInfo
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 1_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "hello", "order": 0},
                    "Example": {"value": "", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 2_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "world", "order": 0},
                    "Example": {"value": "", "order": 1}
                }
            }),
        ]),
    )
    .await;

    // Mock updateNoteFields - called twice
    mock_action_times(
        &server,
        "updateNoteFields",
        mock_anki_response(serde_json::Value::Null),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let query = EnrichQuery {
        search: "deck:Test".to_string(),
        empty_fields: vec!["Example".to_string()],
    };

    let mut pipeline = engine.enrich().pipeline(&query).await.unwrap();

    // Initially no pending updates
    assert_eq!(pipeline.pending_updates(), 0);
    assert_eq!(pipeline.pending_candidates().len(), 2);

    // Add updates
    let mut fields1 = HashMap::new();
    fields1.insert("Example".to_string(), "Example for hello".to_string());
    pipeline.update(1, fields1);

    let mut fields2 = HashMap::new();
    fields2.insert("Example".to_string(), "Example for world".to_string());
    pipeline.update(2, fields2);

    // Now have pending updates
    assert_eq!(pipeline.pending_updates(), 2);
    assert_eq!(pipeline.pending_candidates().len(), 0);

    // Commit
    let report = pipeline.commit(&engine).await.unwrap();
    assert_eq!(report.updated, 2);
    assert_eq!(report.skipped, 0);
    assert!(report.failed.is_empty());
}

#[tokio::test]
async fn test_pipeline_partial_update() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action(&server, "findNotes", mock_anki_response(vec![1_i64, 2, 3])).await;

    // Mock notesInfo
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 1_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "a", "order": 0},
                    "Example": {"value": "", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 2_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "b", "order": 0},
                    "Example": {"value": "", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 3_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "c", "order": 0},
                    "Example": {"value": "", "order": 1}
                }
            }),
        ]),
    )
    .await;

    // Mock updateNoteFields - only called once (only note 1 updated)
    mock_action(
        &server,
        "updateNoteFields",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let engine = engine_for_mock(&server);
    let query = EnrichQuery {
        search: "deck:Test".to_string(),
        empty_fields: vec!["Example".to_string()],
    };

    let mut pipeline = engine.enrich().pipeline(&query).await.unwrap();

    // Only update note 1, leave 2 and 3
    let mut fields = HashMap::new();
    fields.insert("Example".to_string(), "Only this one".to_string());
    pipeline.update(1, fields);

    assert_eq!(pipeline.pending_updates(), 1);
    assert_eq!(pipeline.pending_candidates().len(), 2); // notes 2 and 3

    // Commit
    let report = pipeline.commit(&engine).await.unwrap();
    assert_eq!(report.updated, 1);
    assert_eq!(report.skipped, 2); // notes 2 and 3 skipped
    assert!(report.failed.is_empty());
}

#[tokio::test]
async fn test_pipeline_empty() {
    let server = setup_mock_server().await;

    // Mock findNotes - no matching notes
    mock_action(&server, "findNotes", mock_anki_response(Vec::<i64>::new())).await;

    let engine = engine_for_mock(&server);
    let query = EnrichQuery {
        search: "deck:Empty".to_string(),
        empty_fields: vec!["Example".to_string()],
    };

    let pipeline = engine.enrich().pipeline(&query).await.unwrap();

    assert!(pipeline.is_empty());
    assert_eq!(pipeline.len(), 0);
    assert!(pipeline.by_missing_field().is_empty());

    // Commit on empty pipeline
    let report = pipeline.commit(&engine).await.unwrap();
    assert_eq!(report.updated, 0);
    assert_eq!(report.skipped, 0);
    assert!(report.failed.is_empty());
}

#[tokio::test]
async fn test_pipeline_merge_updates() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action(&server, "findNotes", mock_anki_response(vec![1_i64])).await;

    // Mock notesInfo
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![serde_json::json!({
            "noteId": 1_i64,
            "modelName": "Basic",
            "tags": [],
            "fields": {
                "Front": {"value": "test", "order": 0},
                "Example": {"value": "", "order": 1},
                "Audio": {"value": "", "order": 2}
            }
        })]),
    )
    .await;

    // Mock updateNoteFields - called once with merged fields
    mock_action(
        &server,
        "updateNoteFields",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let engine = engine_for_mock(&server);
    let query = EnrichQuery {
        search: "deck:Test".to_string(),
        empty_fields: vec!["Example".to_string(), "Audio".to_string()],
    };

    let mut pipeline = engine.enrich().pipeline(&query).await.unwrap();

    // Multiple updates to same note get merged
    let mut fields1 = HashMap::new();
    fields1.insert("Example".to_string(), "An example".to_string());
    pipeline.update(1, fields1);

    let mut fields2 = HashMap::new();
    fields2.insert("Audio".to_string(), "[sound:audio.mp3]".to_string());
    pipeline.update(1, fields2);

    // Still only 1 pending update (merged)
    assert_eq!(pipeline.pending_updates(), 1);

    let report = pipeline.commit(&engine).await.unwrap();
    assert_eq!(report.updated, 1);
}
