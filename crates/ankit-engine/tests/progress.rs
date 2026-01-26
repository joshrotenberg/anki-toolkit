//! Tests for progress workflow operations.

mod common;

use ankit_engine::progress::{
    KeepStrategy, PerformanceCriteria, SimilarityCriteria, SuspendCriteria, TagOperation,
};
use common::{
    engine_for_mock, mock_action, mock_action_times, mock_anki_response, setup_mock_server,
};

#[tokio::test]
async fn test_reset_deck_with_cards() {
    let server = setup_mock_server().await;

    // Mock findCards returning some card IDs
    mock_action(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3, 4, 5]),
    )
    .await;

    // Mock forgetCards (no return value needed)
    mock_action(
        &server,
        "forgetCards",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine.progress().reset_deck("Test Deck").await.unwrap();

    assert_eq!(report.cards_reset, 5);
    assert_eq!(report.deck, "Test Deck");
}

#[tokio::test]
async fn test_reset_deck_empty() {
    let server = setup_mock_server().await;

    // Mock findCards returning empty
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    // forgetCards should NOT be called when no cards found

    let engine = engine_for_mock(&server);
    let report = engine.progress().reset_deck("Empty Deck").await.unwrap();

    assert_eq!(report.cards_reset, 0);
    assert_eq!(report.deck, "Empty Deck");
}

#[tokio::test]
async fn test_tag_by_performance() {
    let server = setup_mock_server().await;

    // Mock findCards
    mock_action(&server, "findCards", mock_anki_response(vec![1_i64, 2, 3])).await;

    // Mock cardsInfo - one struggling (low ease), one mastered (high ease, many reps), one neutral
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 1,
                "factor": 1800, // Low ease - struggling
                "reps": 10,
                "lapses": 5,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 2_i64,
                "noteId": 102_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 30,
                "factor": 2700, // High ease - mastered
                "reps": 20,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 3_i64,
                "noteId": 103_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 7,
                "factor": 2300, // Middle ease - neither
                "reps": 3,
                "lapses": 1,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    // Mock addTags - called twice (once for struggling, once for mastered)
    mock_action_times(
        &server,
        "addTags",
        mock_anki_response(serde_json::Value::Null),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .tag_by_performance(
            "deck:Test",
            PerformanceCriteria::default(),
            "struggling",
            "mastered",
        )
        .await
        .unwrap();

    assert_eq!(report.struggling_count, 1);
    assert_eq!(report.mastered_count, 1);
    assert_eq!(report.struggling_tag, "struggling");
    assert_eq!(report.mastered_tag, "mastered");
}

#[tokio::test]
async fn test_tag_by_performance_no_cards() {
    let server = setup_mock_server().await;

    // Mock findCards returning empty
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .tag_by_performance(
            "deck:Empty",
            PerformanceCriteria::default(),
            "struggling",
            "mastered",
        )
        .await
        .unwrap();

    assert_eq!(report.struggling_count, 0);
    assert_eq!(report.mastered_count, 0);
}

#[tokio::test]
async fn test_suspend_by_criteria() {
    let server = setup_mock_server().await;

    // Mock findCards
    mock_action(&server, "findCards", mock_anki_response(vec![1_i64, 2, 3])).await;

    // Mock cardsInfo - one meets criteria (low ease AND high lapses), one doesn't
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2, // Not suspended
                "due": 0,
                "interval": 1,
                "factor": 1500, // Very low ease
                "reps": 20,
                "lapses": 10, // High lapses
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 2_i64,
                "noteId": 102_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2, // Not suspended
                "due": 0,
                "interval": 30,
                "factor": 2500, // Good ease - doesn't meet criteria
                "reps": 20,
                "lapses": 1,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 3_i64,
                "noteId": 103_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": -1, // Already suspended
                "due": 0,
                "interval": 1,
                "factor": 1500,
                "reps": 20,
                "lapses": 10,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    // Mock suspend
    mock_action(&server, "suspend", mock_anki_response(true)).await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .suspend_by_criteria("deck:Test", SuspendCriteria::default())
        .await
        .unwrap();

    assert_eq!(report.cards_suspended, 1);
    assert_eq!(report.suspended_ids, vec![1]);
}

#[tokio::test]
async fn test_deck_health_report() {
    let server = setup_mock_server().await;

    // Mock findCards
    mock_action(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3, 4]),
    )
    .await;

    // Mock cardsInfo with various card states
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 0,
                "queue": 0, // New
                "due": 0,
                "interval": 0,
                "factor": 0,
                "reps": 0,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 2_i64,
                "noteId": 102_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2, // Review
                "due": 0,
                "interval": 10,
                "factor": 2500,
                "reps": 5,
                "lapses": 1,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 3_i64,
                "noteId": 103_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": -1, // Suspended
                "due": 0,
                "interval": 5,
                "factor": 2000,
                "reps": 10,
                "lapses": 8, // Leech!
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 4_i64,
                "noteId": 104_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 1,
                "queue": 1, // Learning
                "due": 0,
                "interval": 0,
                "factor": 2500,
                "reps": 2,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine.progress().deck_health("Test").await.unwrap();

    assert_eq!(report.deck, "Test");
    assert_eq!(report.total_cards, 4);
    assert_eq!(report.new_cards, 1);
    assert_eq!(report.learning_cards, 1);
    assert_eq!(report.review_cards, 1);
    assert_eq!(report.suspended_cards, 1);
    assert_eq!(report.leech_count, 1);
    assert_eq!(report.total_reps, 17); // 0+5+10+2
    assert_eq!(report.total_lapses, 9); // 0+1+8+0
}

#[tokio::test]
async fn test_bulk_tag_add() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action(
        &server,
        "findNotes",
        mock_anki_response(vec![101_i64, 102, 103]),
    )
    .await;

    // Mock addTags
    mock_action(
        &server,
        "addTags",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .bulk_tag("deck:Test", TagOperation::Add("new-tag".to_string()))
        .await
        .unwrap();

    assert_eq!(report.notes_affected, 3);
    assert!(report.operation.contains("Added"));
}

#[tokio::test]
async fn test_bulk_tag_remove() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action(&server, "findNotes", mock_anki_response(vec![101_i64, 102])).await;

    // Mock removeTags
    mock_action(
        &server,
        "removeTags",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .bulk_tag("deck:Test", TagOperation::Remove("old-tag".to_string()))
        .await
        .unwrap();

    assert_eq!(report.notes_affected, 2);
    assert!(report.operation.contains("Removed"));
}

#[tokio::test]
async fn test_bulk_tag_replace() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action(&server, "findNotes", mock_anki_response(vec![101_i64])).await;

    // Mock replaceTags
    mock_action(
        &server,
        "replaceTags",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .bulk_tag(
            "tag:old",
            TagOperation::Replace {
                old: "old".to_string(),
                new: "new".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(report.notes_affected, 1);
    assert!(report.operation.contains("Replaced"));
}

#[tokio::test]
async fn test_smart_suspend_finds_similar() {
    let server = setup_mock_server().await;

    // Mock findCards
    mock_action(&server, "findCards", mock_anki_response(vec![1_i64, 2, 3])).await;

    // Mock cardsInfo with similar content
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 30, // Most mature
                "factor": 2500,
                "reps": 20,
                "lapses": 1,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 2_i64,
                "noteId": 102_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 5, // Less mature
                "factor": 2300,
                "reps": 10,
                "lapses": 2,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 3_i64,
                "noteId": 103_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 60, // Different content - won't be grouped
                "factor": 2500,
                "reps": 30,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    // Mock notesInfo with field values
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 101_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "to eat", "order": 0},
                    "Back": {"value": "eating", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 102_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "to eat!", "order": 0}, // Very similar (one char diff)
                    "Back": {"value": "eating food", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 103_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "completely different word", "order": 0},
                    "Back": {"value": "different meaning", "order": 1}
                }
            }),
        ]),
    )
    .await;

    // Mock suspend
    mock_action(&server, "suspend", mock_anki_response(true)).await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .smart_suspend(
            "deck:Test",
            SimilarityCriteria {
                threshold: 0.7,
                field: "Front".to_string(),
                keep_strategy: KeepStrategy::MostMature,
                dry_run: false,
            },
        )
        .await
        .unwrap();

    assert_eq!(report.cards_analyzed, 3);
    assert_eq!(report.groups_found, 1); // "running" and "runner" grouped
    assert_eq!(report.cards_suspended, 1); // One card suspended
    assert_eq!(report.cards_kept, 1); // One card kept

    // The kept card should be card 1 (most mature with interval 30)
    assert_eq!(report.groups[0].keep, 1);
    assert_eq!(report.groups[0].suspend, vec![2]);
}

#[tokio::test]
async fn test_smart_suspend_dry_run() {
    let server = setup_mock_server().await;

    // Mock findCards
    mock_action(&server, "findCards", mock_anki_response(vec![1_i64, 2])).await;

    // Mock cardsInfo
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 10,
                "factor": 2500,
                "reps": 10,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 2_i64,
                "noteId": 102_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 5,
                "factor": 2300,
                "reps": 5,
                "lapses": 1,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    // Mock notesInfo with identical content
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 101_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "hello", "order": 0},
                    "Back": {"value": "world", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 102_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "hello", "order": 0}, // Exact match
                    "Back": {"value": "different", "order": 1}
                }
            }),
        ]),
    )
    .await;

    // NO suspend mock - it shouldn't be called in dry_run mode

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .smart_suspend(
            "deck:Test",
            SimilarityCriteria {
                threshold: 1.0, // Exact match
                field: "Front".to_string(),
                keep_strategy: KeepStrategy::MostMature,
                dry_run: true, // Dry run!
            },
        )
        .await
        .unwrap();

    assert!(report.dry_run);
    assert_eq!(report.groups_found, 1);
    assert_eq!(report.cards_suspended, 1);
    // Suspend was NOT called because dry_run=true
}

#[tokio::test]
async fn test_smart_suspend_empty() {
    let server = setup_mock_server().await;

    // Mock findCards returning empty
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    let engine = engine_for_mock(&server);
    let report = engine
        .progress()
        .smart_suspend("deck:Empty", SimilarityCriteria::default())
        .await
        .unwrap();

    assert_eq!(report.cards_analyzed, 0);
    assert_eq!(report.groups_found, 0);
    assert_eq!(report.cards_suspended, 0);
}

#[tokio::test]
async fn test_smart_suspend_keep_strategies() {
    let server = setup_mock_server().await;

    // Mock findCards - called multiple times
    mock_action_times(&server, "findCards", mock_anki_response(vec![1_i64, 2]), 4).await;

    // Mock cardsInfo - called multiple times
    mock_action_times(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 30, // Most mature
                "factor": 2000, // Lower ease
                "reps": 10,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 2_i64,
                "noteId": 102_i64,
                "deckName": "Test",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 5, // Least mature
                "factor": 2500, // Higher ease
                "reps": 20, // Most reviews
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
        ]),
        4,
    )
    .await;

    // Mock notesInfo - called multiple times
    mock_action_times(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 101_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "test", "order": 0},
                    "Back": {"value": "answer", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 102_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "test", "order": 0}, // Identical
                    "Back": {"value": "different", "order": 1}
                }
            }),
        ]),
        4,
    )
    .await;

    let engine = engine_for_mock(&server);

    // Test MostMature - keeps card 1 (interval 30)
    let report = engine
        .progress()
        .smart_suspend(
            "deck:Test",
            SimilarityCriteria {
                keep_strategy: KeepStrategy::MostMature,
                dry_run: true,
                ..SimilarityCriteria::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.groups[0].keep, 1);

    // Test LeastMature - keeps card 2 (interval 5)
    let report = engine
        .progress()
        .smart_suspend(
            "deck:Test",
            SimilarityCriteria {
                keep_strategy: KeepStrategy::LeastMature,
                dry_run: true,
                ..SimilarityCriteria::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.groups[0].keep, 2);

    // Test HighestEase - keeps card 2 (ease 2500)
    let report = engine
        .progress()
        .smart_suspend(
            "deck:Test",
            SimilarityCriteria {
                keep_strategy: KeepStrategy::HighestEase,
                dry_run: true,
                ..SimilarityCriteria::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.groups[0].keep, 2);

    // Test MostReviewed - keeps card 2 (reps 20)
    let report = engine
        .progress()
        .smart_suspend(
            "deck:Test",
            SimilarityCriteria {
                keep_strategy: KeepStrategy::MostReviewed,
                dry_run: true,
                ..SimilarityCriteria::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.groups[0].keep, 2);
}
