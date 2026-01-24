//! Tests for miscellaneous AnkiConnect actions.

mod common;

use common::{mock_action, mock_anki_error, mock_anki_response, setup_mock_server};
use yanki::AnkiClient;

#[tokio::test]
async fn test_version() {
    let server = setup_mock_server().await;
    mock_action(&server, "version", mock_anki_response(6)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let version = client.misc().version().await.unwrap();

    assert_eq!(version, 6);
}

#[tokio::test]
async fn test_version_error() {
    let server = setup_mock_server().await;
    mock_action(&server, "version", mock_anki_error("Internal error")).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.misc().version().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Internal error"));
}

#[tokio::test]
async fn test_connection_refused() {
    // Use a port that's almost certainly not in use
    let client = AnkiClient::builder().url("http://127.0.0.1:59999").build();

    let result = client.misc().version().await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Could not connect to Anki"),
        "Expected connection refused error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_profiles() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "getProfiles",
        mock_anki_response(vec!["User 1", "Test Profile"]),
    )
    .await;

    let result = client.misc().profiles().await.unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.contains(&"User 1".to_string()));
}

#[tokio::test]
async fn test_load_profile() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "loadProfile", mock_anki_response(true)).await;

    let result = client.misc().load_profile("User 1").await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_request_permission() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "requestPermission",
        mock_anki_response(serde_json::json!({
            "permission": "granted",
            "requireApiKey": false,
            "version": 6
        })),
    )
    .await;

    let result = client.misc().request_permission().await.unwrap();
    assert_eq!(result.permission, "granted");
}

#[tokio::test]
async fn test_multi() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "multi",
        mock_anki_response(vec![
            serde_json::json!(["Default", "Test"]),
            serde_json::json!(["Basic", "Cloze"]),
        ]),
    )
    .await;

    let actions = vec![
        yanki::actions::MultiAction::new("deckNames"),
        yanki::actions::MultiAction::new("modelNames"),
    ];

    let result = client.misc().multi(&actions).await.unwrap();
    assert_eq!(result.len(), 2);
}
