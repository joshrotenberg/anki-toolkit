//! Tests for analyze workflow operations.

mod common;

use ankit_engine::analyze::ProblemCriteria;
use common::{engine_for_mock, mock_action, mock_anki_response, setup_mock_server};

#[tokio::test]
async fn test_study_summary() {
    let server = setup_mock_server().await;

    // Mock getNumCardsReviewedByDay
    mock_action(
        &server,
        "getNumCardsReviewedByDay",
        mock_anki_response(vec![
            vec![serde_json::json!("2024-01-15"), serde_json::json!(50)],
            vec![serde_json::json!("2024-01-14"), serde_json::json!(30)],
            vec![serde_json::json!("2024-01-13"), serde_json::json!(45)],
        ]),
    )
    .await;

    // Mock findCards for unique cards count
    mock_action(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let summary = engine.analyze().study_summary("Japanese", 7).await.unwrap();

    assert_eq!(summary.total_reviews, 125); // 50+30+45
    assert_eq!(summary.unique_cards, 10);
    assert_eq!(summary.daily.len(), 3);
    assert_eq!(summary.daily[0].reviews, 50);
}

#[tokio::test]
async fn test_study_summary_all_decks() {
    let server = setup_mock_server().await;

    // Mock getNumCardsReviewedByDay
    mock_action(
        &server,
        "getNumCardsReviewedByDay",
        mock_anki_response(vec![vec![
            serde_json::json!("2024-01-15"),
            serde_json::json!(100),
        ]]),
    )
    .await;

    // No findCards call when deck is "*"

    let engine = engine_for_mock(&server);
    let summary = engine.analyze().study_summary("*", 7).await.unwrap();

    assert_eq!(summary.total_reviews, 100);
    assert_eq!(summary.unique_cards, 0); // Not calculated for all decks
}

#[tokio::test]
async fn test_find_problems_high_lapses() {
    let server = setup_mock_server().await;

    // Mock findCards
    mock_action(&server, "findCards", mock_anki_response(vec![1_i64, 2])).await;

    // Mock cardsInfo - one with high lapses, one normal
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Japanese",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 5,
                "factor": 2500,
                "reps": 20,
                "lapses": 10, // High lapses - problem!
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 2_i64,
                "noteId": 102_i64,
                "deckName": "Japanese",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 30,
                "factor": 2500,
                "reps": 10,
                "lapses": 1, // Normal
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    // Mock notesInfo for getting front field
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![serde_json::json!({
            "noteId": 101_i64,
            "modelName": "Basic",
            "tags": [],
            "fields": {
                "Front": {"value": "Problem card", "order": 0},
                "Back": {"value": "Answer", "order": 1}
            }
        })]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let criteria = ProblemCriteria {
        min_lapses: 5,
        ..Default::default()
    };
    let problems = engine
        .analyze()
        .find_problems("deck:Japanese", criteria)
        .await
        .unwrap();

    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].card_id, 1);
    assert_eq!(problems[0].lapses, 10);
}

#[tokio::test]
async fn test_find_problems_empty() {
    let server = setup_mock_server().await;

    // Mock findCards returning empty
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    let engine = engine_for_mock(&server);
    let problems = engine
        .analyze()
        .find_problems("deck:Empty", ProblemCriteria::default())
        .await
        .unwrap();

    assert!(problems.is_empty());
}

#[tokio::test]
async fn test_retention_stats() {
    let server = setup_mock_server().await;

    // Mock findCards
    mock_action(&server, "findCards", mock_anki_response(vec![1_i64, 2, 3])).await;

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
                "interval": 20,
                "factor": 2700,
                "reps": 15,
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
                "interval": 30,
                "factor": 2300,
                "reps": 20,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    // Mock getEaseFactors
    mock_action(
        &server,
        "getEaseFactors",
        mock_anki_response(vec![2500_i64, 2700, 2300]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let stats = engine.analyze().retention_stats("Test").await.unwrap();

    assert_eq!(stats.total_cards, 3);
    assert_eq!(stats.total_reviews, 45); // 10+15+20
    assert_eq!(stats.total_lapses, 3); // 1+2+0
    assert_eq!(stats.avg_ease, 2500); // (2500+2700+2300)/3
    assert_eq!(stats.avg_interval, 20); // (10+20+30)/3
    // retention_rate = 1 - (3/45) = 0.933...
    assert!(stats.retention_rate > 0.9 && stats.retention_rate < 0.95);
}

#[tokio::test]
async fn test_retention_stats_empty() {
    let server = setup_mock_server().await;

    // Mock findCards returning empty
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    let engine = engine_for_mock(&server);
    let stats = engine.analyze().retention_stats("Empty").await.unwrap();

    assert_eq!(stats.total_cards, 0);
    assert_eq!(stats.retention_rate, 0.0);
}

#[tokio::test]
async fn test_deck_audit() {
    let server = setup_mock_server().await;

    // Mock findCards for card count
    mock_action(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3, 4]),
    )
    .await;

    // Mock cardsInfo for scheduling and model analysis
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Japanese",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 0, // new
                "queue": 0,
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
                "deckName": "Japanese",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2, // review
                "queue": 2,
                "due": 0,
                "interval": 30,
                "factor": 2500,
                "reps": 10,
                "lapses": 1,
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 3_i64,
                "noteId": 103_i64,
                "deckName": "Japanese",
                "modelName": "Cloze",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2, // review
                "queue": -1, // suspended
                "due": 0,
                "interval": 10,
                "factor": 2000,
                "reps": 20,
                "lapses": 10, // leech
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 4_i64,
                "noteId": 104_i64,
                "deckName": "Japanese",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 1, // learning
                "queue": 1,
                "due": 0,
                "interval": 1,
                "factor": 2500,
                "reps": 5,
                "lapses": 0,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    // Mock findNotes for note count
    mock_action(
        &server,
        "findNotes",
        mock_anki_response(vec![101_i64, 102, 103, 104]),
    )
    .await;

    // Mock notesInfo for tag and field analysis
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 101_i64,
                "modelName": "Basic",
                "tags": ["vocabulary", "n5"],
                "fields": {
                    "Front": {"value": "hello", "order": 0},
                    "Back": {"value": "world", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 102_i64,
                "modelName": "Basic",
                "tags": ["vocabulary"],
                "fields": {
                    "Front": {"value": "goodbye", "order": 0},
                    "Back": {"value": "", "order": 1} // empty field
                }
            }),
            serde_json::json!({
                "noteId": 103_i64,
                "modelName": "Cloze",
                "tags": [],  // untagged
                "fields": {
                    "Text": {"value": "test", "order": 0},
                    "Extra": {"value": "", "order": 1} // empty field
                }
            }),
            serde_json::json!({
                "noteId": 104_i64,
                "modelName": "Basic",
                "tags": ["grammar"],
                "fields": {
                    "Front": {"value": "hello", "order": 0}, // duplicate of note 101
                    "Back": {"value": "different", "order": 1}
                }
            }),
        ]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let audit = engine.analyze().deck_audit("Japanese").await.unwrap();

    assert_eq!(audit.deck, "Japanese");
    assert_eq!(audit.total_cards, 4);
    assert_eq!(audit.total_notes, 4);

    // Cards by model
    assert_eq!(audit.cards_by_model.get("Basic"), Some(&3));
    assert_eq!(audit.cards_by_model.get("Cloze"), Some(&1));

    // Scheduling
    assert_eq!(audit.new_cards, 1);
    assert_eq!(audit.learning_cards, 1);
    assert_eq!(audit.review_cards, 2);
    assert_eq!(audit.suspended_count, 1);
    assert_eq!(audit.leech_count, 1);

    // Tags
    assert_eq!(audit.tag_distribution.get("vocabulary"), Some(&2));
    assert_eq!(audit.tag_distribution.get("n5"), Some(&1));
    assert_eq!(audit.tag_distribution.get("grammar"), Some(&1));
    assert_eq!(audit.untagged_notes, 1);

    // Empty fields
    assert_eq!(audit.empty_field_counts.get("Back"), Some(&1));
    assert_eq!(audit.empty_field_counts.get("Extra"), Some(&1));

    // Duplicates (note 101 and 104 have same "hello" in first field)
    assert_eq!(audit.duplicate_count, 1);

    // Average ease (only cards with ease > 0: 2500, 2000, 2500 = 2333.33)
    assert!(audit.average_ease > 2300.0 && audit.average_ease < 2400.0);
}

#[tokio::test]
async fn test_deck_audit_empty() {
    let server = setup_mock_server().await;

    // Mock findCards returning empty
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    let engine = engine_for_mock(&server);
    let audit = engine.analyze().deck_audit("Empty").await.unwrap();

    assert_eq!(audit.deck, "Empty");
    assert_eq!(audit.total_cards, 0);
    assert_eq!(audit.total_notes, 0);
    assert!(audit.cards_by_model.is_empty());
    assert!(audit.tag_distribution.is_empty());
}
