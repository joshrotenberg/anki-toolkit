//! Tests for deck AnkiConnect actions.

mod common;

use common::{mock_action, mock_anki_error, mock_anki_response, setup_mock_server};
use yanki::AnkiClient;

#[tokio::test]
async fn test_deck_names() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "deckNames",
        mock_anki_response(vec!["Default", "Japanese"]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let decks = client.decks().names().await.unwrap();

    assert_eq!(decks, vec!["Default", "Japanese"]);
}

#[tokio::test]
async fn test_deck_names_and_ids() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "deckNamesAndIds",
        mock_anki_response(serde_json::json!({
            "Default": 1,
            "Japanese": 1234567890
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let decks = client.decks().names_and_ids().await.unwrap();

    assert_eq!(decks.get("Default"), Some(&1));
    assert_eq!(decks.get("Japanese"), Some(&1234567890));
}

#[tokio::test]
async fn test_create_deck() {
    let server = setup_mock_server().await;
    mock_action(&server, "createDeck", mock_anki_response(1234567890_i64)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let deck_id = client.decks().create("New Deck").await.unwrap();

    assert_eq!(deck_id, 1234567890);
}

#[tokio::test]
async fn test_delete_deck() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "deleteDecks",
        mock_anki_response(serde_json::Value::Null),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.decks().delete(&["Old Deck"], true).await;

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn test_deck_error() {
    let server = setup_mock_server().await;
    mock_action(&server, "deckNames", mock_anki_error("deck not found")).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.decks().names().await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deck not found"));
}
