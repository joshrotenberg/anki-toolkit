//! Tests for organize workflow operations.

mod common;

use common::{
    engine_for_mock, mock_action, mock_action_times, mock_anki_response, setup_mock_server,
};

#[tokio::test]
async fn test_clone_deck() {
    let server = setup_mock_server().await;

    // Mock getDeckNames to verify source exists
    mock_action(
        &server,
        "deckNames",
        mock_anki_response(vec!["Source Deck", "Other Deck"]),
    )
    .await;

    // Mock createDeck for destination
    mock_action(&server, "createDeck", mock_anki_response(123456789_i64)).await;

    // Mock findNotes for source deck
    mock_action(&server, "findNotes", mock_anki_response(vec![101_i64, 102])).await;

    // Mock notesInfo for source notes
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(vec![
            serde_json::json!({
                "noteId": 101_i64,
                "modelName": "Basic",
                "tags": ["tag1"],
                "fields": {
                    "Front": {"value": "Hello", "order": 0},
                    "Back": {"value": "World", "order": 1}
                }
            }),
            serde_json::json!({
                "noteId": 102_i64,
                "modelName": "Basic",
                "tags": [],
                "fields": {
                    "Front": {"value": "Foo", "order": 0},
                    "Back": {"value": "Bar", "order": 1}
                }
            }),
        ]),
    )
    .await;

    // Mock addNote for each cloned note
    mock_action_times(&server, "addNote", mock_anki_response(201_i64), 2).await;

    let engine = engine_for_mock(&server);
    let report = engine
        .organize()
        .clone_deck("Source Deck", "Dest Deck")
        .await
        .unwrap();

    assert_eq!(report.notes_cloned, 2);
    assert_eq!(report.notes_failed, 0);
    assert_eq!(report.destination, "Dest Deck");
}

#[tokio::test]
async fn test_clone_deck_empty() {
    let server = setup_mock_server().await;

    // Mock getDeckNames
    mock_action(
        &server,
        "deckNames",
        mock_anki_response(vec!["Source Deck"]),
    )
    .await;

    // Mock createDeck
    mock_action(&server, "createDeck", mock_anki_response(123456789_i64)).await;

    // Mock findNotes returning empty
    mock_action(&server, "findNotes", mock_anki_response(Vec::<i64>::new())).await;

    // Mock notesInfo (called even with empty array)
    mock_action(
        &server,
        "notesInfo",
        mock_anki_response(Vec::<serde_json::Value>::new()),
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine
        .organize()
        .clone_deck("Source Deck", "Dest Deck")
        .await
        .unwrap();

    assert_eq!(report.notes_cloned, 0);
    assert_eq!(report.notes_failed, 0);
}

#[tokio::test]
async fn test_merge_decks() {
    let server = setup_mock_server().await;

    // Mock createDeck for destination
    mock_action(&server, "createDeck", mock_anki_response(999_i64)).await;

    // Mock findCards - called twice, same mock used for both (returns 3 cards each time)
    mock_action_times(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3]),
        2,
    )
    .await;

    // Mock changeDeck - called twice
    mock_action_times(
        &server,
        "changeDeck",
        mock_anki_response(serde_json::Value::Null),
        2,
    )
    .await;

    let engine = engine_for_mock(&server);
    let report = engine
        .organize()
        .merge_decks(&["Deck A", "Deck B"], "Merged Deck")
        .await
        .unwrap();

    assert_eq!(report.cards_moved, 6); // 3 + 3 (same mock returns 3 each time)
    assert_eq!(report.destination, "Merged Deck");
}

#[tokio::test]
async fn test_move_by_tag() {
    let server = setup_mock_server().await;

    // Mock createDeck
    mock_action(&server, "createDeck", mock_anki_response(123_i64)).await;

    // Mock findCards with tag
    mock_action(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3, 4]),
    )
    .await;

    // Mock changeDeck
    mock_action(
        &server,
        "changeDeck",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let engine = engine_for_mock(&server);
    let count = engine
        .organize()
        .move_by_tag("important", "Important Deck")
        .await
        .unwrap();

    assert_eq!(count, 4);
}

#[tokio::test]
async fn test_move_by_tag_no_matches() {
    let server = setup_mock_server().await;

    // Mock createDeck
    mock_action(&server, "createDeck", mock_anki_response(123_i64)).await;

    // Mock findCards returning empty
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    // changeDeck should NOT be called

    let engine = engine_for_mock(&server);
    let count = engine
        .organize()
        .move_by_tag("nonexistent", "Some Deck")
        .await
        .unwrap();

    assert_eq!(count, 0);
}
