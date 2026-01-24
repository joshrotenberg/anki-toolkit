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

#[tokio::test]
async fn test_get_decks_for_cards() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "getDecks",
        mock_anki_response(serde_json::json!({
            "Default": [1502298033753_i64, 1502298033754_i64],
            "Japanese": [1502298033755_i64]
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let decks = client
        .decks()
        .get_for_cards(&[1502298033753, 1502298033754, 1502298033755])
        .await
        .unwrap();

    assert_eq!(decks.len(), 2);
    assert!(decks.contains_key("Default"));
    assert!(decks.contains_key("Japanese"));
    assert_eq!(decks.get("Default").unwrap().len(), 2);
}

#[tokio::test]
async fn test_move_cards() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "changeDeck",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client
        .decks()
        .move_cards(&[1502298033753], "New Deck")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_deck_config() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "getDeckConfig",
        mock_anki_response(serde_json::json!({
            "id": 1,
            "name": "Default",
            "new": {
                "perDay": 20,
                "bury": true
            },
            "rev": {
                "perDay": 200,
                "bury": true
            },
            "lapse": {
                "leechFails": 8
            }
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let config = client.decks().config("Default").await.unwrap();

    assert_eq!(config.id, 1);
    assert_eq!(config.name, "Default");
    assert_eq!(config.new.per_day, 20);
}

#[tokio::test]
async fn test_save_deck_config() {
    let server = setup_mock_server().await;
    mock_action(&server, "saveDeckConfig", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();

    // Create a minimal config for testing
    let config = yanki::DeckConfig {
        id: 1,
        name: "Default".to_string(),
        max_taken: 60,
        replayq: true,
        autoplay: true,
        timer: 0,
        new: yanki::NewCardConfig {
            delays: vec![1.0, 10.0],
            order: 1,
            initial_factor: 2500,
            separate: true,
            ints: vec![1, 4],
            per_day: 50,
        },
        rev: yanki::ReviewConfig {
            per_day: 200,
            ease4: 1.3,
            fuzz: 0.05,
            min_space: 1,
            max_ivl: 36500,
            bury: true,
            hard_factor: 1.2,
        },
        lapse: yanki::LapseConfig {
            delays: vec![10.0],
            leech_fails: 8,
            leech_action: 0,
            min_int: 1,
            mult: 0.0,
        },
    };

    let result = client.decks().save_config(&config).await.unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_set_config_id() {
    let server = setup_mock_server().await;
    mock_action(&server, "setDeckConfigId", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client
        .decks()
        .set_config_id(&["Japanese", "Korean"], 1)
        .await
        .unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_clone_config() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "cloneDeckConfigId",
        mock_anki_response(1234567890_i64),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let new_id = client.decks().clone_config("My Config", 1).await.unwrap();

    assert_eq!(new_id, 1234567890);
}

#[tokio::test]
async fn test_remove_config() {
    let server = setup_mock_server().await;
    mock_action(&server, "removeDeckConfigId", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.decks().remove_config(1234567890).await.unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_deck_stats() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "getDeckStats",
        mock_anki_response(serde_json::json!({
            "1": {
                "deckId": 1,
                "name": "Default",
                "newCount": 10,
                "learnCount": 5,
                "reviewCount": 20,
                "totalInDeck": 100
            }
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let stats = client.decks().stats(&["Default"]).await.unwrap();

    assert_eq!(stats.len(), 1);
    let stat = stats.get("1").unwrap();
    assert_eq!(stat.name, "Default");
    assert_eq!(stat.new_count, 10);
    assert_eq!(stat.review_count, 20);
}
