//! Tests for card AnkiConnect actions.

mod common;

use common::{mock_action, mock_anki_response, setup_mock_server};
use yanki::AnkiClient;

#[tokio::test]
async fn test_find_cards() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "findCards",
        mock_anki_response(vec![1_i64, 2, 3, 4, 5]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let cards = client.cards().find("is:due").await.unwrap();

    assert_eq!(cards, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_cards_info() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "cardsInfo",
        mock_anki_response(vec![serde_json::json!({
            "cardId": 1234567890_i64,
            "noteId": 9876543210_i64,
            "deckName": "Default",
            "modelName": "Basic",
            "question": "<div>Front</div>",
            "answer": "<div>Back</div>",
            "fields": {
                "Front": {"value": "Hello", "order": 0},
                "Back": {"value": "World", "order": 1}
            },
            "type": 2,
            "queue": 2,
            "due": 100,
            "interval": 10,
            "factor": 2500,
            "reps": 5,
            "lapses": 1,
            "left": 0,
            "mod": 1234567890
        })]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let cards = client.cards().info(&[1234567890]).await.unwrap();

    assert_eq!(cards.len(), 1);
    let card = &cards[0];
    assert_eq!(card.card_id, 1234567890);
    assert_eq!(card.note_id, 9876543210);
    assert_eq!(card.deck_name, "Default");
    assert_eq!(card.model_name, "Basic");
    assert_eq!(card.card_type, 2); // review
    assert_eq!(card.interval, 10);
    assert_eq!(card.reps, 5);
    assert_eq!(card.lapses, 1);
}

#[tokio::test]
async fn test_find_cards_empty() {
    let server = setup_mock_server().await;
    mock_action(&server, "findCards", mock_anki_response(Vec::<i64>::new())).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let cards = client.cards().find("deck:NonExistent").await.unwrap();

    assert!(cards.is_empty());
}
