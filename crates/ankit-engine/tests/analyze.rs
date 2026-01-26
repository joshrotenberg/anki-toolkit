//! Tests for analyze workflow operations.

mod common;

use ankit_engine::analyze::{CompareOptions, ProblemCriteria};
use common::{
    engine_for_mock, mock_action, mock_action_times, mock_anki_response, setup_mock_server,
};

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

#[tokio::test]
async fn test_study_report() {
    let server = setup_mock_server().await;

    // Mock getNumCardsReviewedByDay - 3 consecutive days with reviews
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

    // Mock findCards - called 4 times (review, rated, due tomorrow, due week)
    // All return the same card IDs for simplicity
    mock_action_times(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3]),
        4,
    )
    .await;

    // Mock cardsInfo - called 2 times (review cards, rated cards)
    mock_action_times(
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
                "interval": 10,
                "factor": 2500,
                "reps": 20,
                "lapses": 1,
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
                "type": 0, // new - for new_cards_studied count
                "queue": 0,
                "due": 0,
                "interval": 5,
                "factor": 1800, // Low ease - problem card
                "reps": 15,
                "lapses": 10, // High lapses - leech
                "left": 0,
                "mod": 0
            }),
            serde_json::json!({
                "cardId": 3_i64,
                "noteId": 103_i64,
                "deckName": "Japanese",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 3, // relearning
                "queue": 1,
                "due": 0,
                "interval": 1,
                "factor": 2200,
                "reps": 10,
                "lapses": 3,
                "left": 0,
                "mod": 0
            }),
        ]),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine.analyze().study_report("Japanese", 7).await.unwrap();

    // Activity metrics
    assert_eq!(report.deck, "Japanese");
    assert_eq!(report.period_days, 7);
    assert_eq!(report.total_reviews, 125); // 50+30+45
    assert_eq!(report.study_streak, 3); // 3 consecutive days
    assert!((report.average_reviews_per_day - 41.67).abs() < 0.1);

    // Daily stats
    assert_eq!(report.daily_stats.len(), 3);
    assert_eq!(report.daily_stats[0].date, "2024-01-15");
    assert_eq!(report.daily_stats[0].reviews, 50);

    // Performance metrics
    // retention = 1 - (14/45) = 0.689 (lapses: 1+10+3=14, reps: 20+15+10=45)
    assert!(report.retention_rate > 0.68 && report.retention_rate < 0.70);
    // avg_ease = (2500+1800+2200)/3 = 2166.67
    assert!(report.average_ease > 2160.0 && report.average_ease < 2170.0);

    // Problem cards
    assert_eq!(report.leeches.len(), 1);
    assert_eq!(report.leeches[0], 2); // card with 10 lapses
    assert_eq!(report.low_ease_cards.len(), 1);
    assert_eq!(report.low_ease_cards[0], 2); // card with 1800 ease

    // Cards studied breakdown from cardsInfo mock:
    // card 1: type=2 (review), card 2: type=0 (new), card 3: type=3 (relearning)
    assert_eq!(report.new_cards_studied, 1);
    assert_eq!(report.review_cards_studied, 1);
    assert_eq!(report.relearning_cards, 1);

    // Upcoming workload (same 3 cards returned for both queries)
    assert_eq!(report.due_tomorrow, 3);
    assert_eq!(report.due_this_week, 3);
}

#[tokio::test]
async fn test_study_report_all_decks() {
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

    // Mock findCards - called 3 times for all decks (review, due tomorrow, due week)
    // No rated query for "*" deck
    mock_action_times(&server, "findCards", mock_anki_response(vec![1_i64, 2]), 3).await;

    // Mock cardsInfo for the review cards
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "cardId": 1_i64,
                "noteId": 101_i64,
                "deckName": "Default",
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
                "deckName": "Default",
                "modelName": "Basic",
                "question": "",
                "answer": "",
                "fields": {},
                "type": 2,
                "queue": 2,
                "due": 0,
                "interval": 20,
                "factor": 2500,
                "reps": 15,
                "lapses": 2,
                "left": 0,
                "mod": 0
            }),
        ]),
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine.analyze().study_report("*", 7).await.unwrap();

    assert_eq!(report.deck, "*");
    assert_eq!(report.total_reviews, 100);
    // Same cards returned for all queries
    assert_eq!(report.due_tomorrow, 2);
    assert_eq!(report.due_this_week, 2);
    // No rated query for all decks, so these stay 0
    assert_eq!(report.new_cards_studied, 0);
    assert_eq!(report.review_cards_studied, 0);
    // Retention from the 2 review cards
    assert!(report.retention_rate > 0.8);
}

#[tokio::test]
async fn test_study_report_empty() {
    let server = setup_mock_server().await;

    // Mock getNumCardsReviewedByDay - no reviews
    mock_action(
        &server,
        "getNumCardsReviewedByDay",
        mock_anki_response(Vec::<(String, i64)>::new()),
    )
    .await;

    // Mock findCards - called 4 times (review, rated, due tomorrow, due week)
    // All return empty
    mock_action_times(
        &server,
        "findCards",
        mock_anki_response(Vec::<i64>::new()),
        4,
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine.analyze().study_report("Empty", 7).await.unwrap();

    assert_eq!(report.deck, "Empty");
    assert_eq!(report.total_reviews, 0);
    assert_eq!(report.study_streak, 0);
    assert_eq!(report.retention_rate, 0.0);
    assert_eq!(report.average_ease, 0.0);
    assert!(report.leeches.is_empty());
    assert!(report.low_ease_cards.is_empty());
    assert!(report.daily_stats.is_empty());
}

#[tokio::test]
async fn test_compare_decks_exact_matches() {
    let server = setup_mock_server().await;

    // Mock findNotes - called twice (deck A, deck B)
    // Both return the same IDs for simplicity - the notesInfo will differentiate
    mock_action_times(&server, "findNotes", mock_anki_response(vec![1_i64, 2]), 2).await;

    // Mock notesInfo - called twice (deck A notes, deck B notes)
    // Return all 4 notes each time, implementation handles by requested IDs
    mock_action_times(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 1_i64,
                "modelName": "Basic",
                "tags": ["deck-a"],
                "fields": {
                    "Front": {"value": "hello", "order": 0},
                    "Back": {"value": "world", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 2_i64,
                "modelName": "Basic",
                "tags": ["deck-a"],
                "fields": {
                    "Front": {"value": "goodbye", "order": 0},
                    "Back": {"value": "farewell", "order": 1}
                }
            }),
        ]),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let comparison = engine
        .analyze()
        .compare_decks(
            "Deck A",
            "Deck B",
            CompareOptions {
                key_field: "Front".to_string(),
                similarity_threshold: 1.0, // Exact matches only
            },
        )
        .await
        .unwrap();

    assert_eq!(comparison.deck_a, "Deck A");
    assert_eq!(comparison.deck_b, "Deck B");
    assert_eq!(comparison.key_field, "Front");

    // Both decks have same notes (mocked), so all should be exact matches
    assert_eq!(comparison.exact_matches.len(), 2);

    // No unique notes (both decks identical)
    assert!(comparison.only_in_a.is_empty());
    assert!(comparison.only_in_b.is_empty());
}

#[tokio::test]
async fn test_compare_decks_similar_matches() {
    let server = setup_mock_server().await;

    // Mock findNotes - called twice (deck A, deck B)
    mock_action_times(&server, "findNotes", mock_anki_response(vec![1_i64]), 2).await;

    // Mock notesInfo - called twice, same response
    // Both decks have same note with "running"
    mock_action_times(
        &server,
        "notesInfo",
        mock_anki_response(vec![serde_json::json!({
            "noteId": 1_i64,
            "modelName": "Basic",
            "tags": [],
            "fields": {
                "Front": {"value": "running", "order": 0},
                "Back": {"value": "to run", "order": 1}
            }
        })]),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let comparison = engine
        .analyze()
        .compare_decks(
            "Deck A",
            "Deck B",
            CompareOptions {
                key_field: "Front".to_string(),
                similarity_threshold: 0.7,
            },
        )
        .await
        .unwrap();

    // Both decks have identical "running" note, so it's an exact match
    assert_eq!(comparison.exact_matches.len(), 1);
    assert_eq!(comparison.exact_matches[0].0.key_value, "running");

    // No similar matches needed (exact match found)
    assert!(comparison.similar.is_empty());

    // No unique notes
    assert!(comparison.only_in_a.is_empty());
    assert!(comparison.only_in_b.is_empty());
}

#[tokio::test]
async fn test_compare_decks_empty() {
    let server = setup_mock_server().await;

    // Mock findNotes for both decks - empty
    mock_action_times(
        &server,
        "findNotes",
        mock_anki_response(Vec::<i64>::new()),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let comparison = engine
        .analyze()
        .compare_decks("Empty A", "Empty B", CompareOptions::default())
        .await
        .unwrap();

    assert!(comparison.exact_matches.is_empty());
    assert!(comparison.similar.is_empty());
    assert!(comparison.only_in_a.is_empty());
    assert!(comparison.only_in_b.is_empty());
}

#[tokio::test]
async fn test_compare_decks_unique_notes() {
    let server = setup_mock_server().await;

    // Mock findNotes - called twice, returns different note IDs that result in unique entries
    // Note: wiremock returns same response for all calls to same action
    // So we'll have both decks get [1, 2, 3] and test that identical decks have no unique notes
    mock_action_times(
        &server,
        "findNotes",
        mock_anki_response(vec![1_i64, 2, 3]),
        2,
    )
    .await;

    // Mock notesInfo - returns 3 notes, one unique to "A", one unique to "B" conceptually
    // but since mocks return same data, we test the matching logic
    mock_action_times(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 1_i64,
                "modelName": "Basic",
                "tags": ["common"],
                "fields": {
                    "Front": {"value": "shared word", "order": 0},
                    "Back": {"value": "definition", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 2_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "another shared", "order": 0},
                    "Back": {"value": "another def", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 3_i64,
                "modelName": "Basic",
                "tags": ["extra"],
                "fields": {
                    "Front": {"value": "third word", "order": 0},
                    "Back": {"value": "third def", "order": 1}
                }
            }),
        ]),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let comparison = engine
        .analyze()
        .compare_decks("Deck A", "Deck B", CompareOptions::default())
        .await
        .unwrap();

    // All 3 notes should be exact matches (same content in both decks)
    assert_eq!(comparison.exact_matches.len(), 3);

    // No unique notes (identical decks)
    assert!(comparison.only_in_a.is_empty());
    assert!(comparison.only_in_b.is_empty());
    assert!(comparison.similar.is_empty());
}

#[tokio::test]
async fn test_compare_decks_preserves_tags() {
    let server = setup_mock_server().await;

    // Mock findNotes
    mock_action_times(&server, "findNotes", mock_anki_response(vec![1_i64]), 2).await;

    // Mock notesInfo with tagged note
    mock_action_times(
        &server,
        "notesInfo",
        mock_anki_response(vec![serde_json::json!({
            "noteId": 1_i64,
            "modelName": "Basic",
            "tags": ["vocabulary", "n5", "verb"],
            "fields": {
                "Front": {"value": "to run", "order": 0},
                "Back": {"value": "to move quickly", "order": 1}
            }
        })]),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let comparison = engine
        .analyze()
        .compare_decks("Deck A", "Deck B", CompareOptions::default())
        .await
        .unwrap();

    // Check that tags are preserved in the comparison result
    assert_eq!(comparison.exact_matches.len(), 1);
    assert_eq!(
        comparison.exact_matches[0].0.tags,
        vec!["vocabulary", "n5", "verb"]
    );
    assert_eq!(
        comparison.exact_matches[0].1.tags,
        vec!["vocabulary", "n5", "verb"]
    );
}
