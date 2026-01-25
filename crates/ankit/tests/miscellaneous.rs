//! Tests for miscellaneous AnkiConnect actions.

mod common;

use ankit::AnkiClient;
use common::{mock_action, mock_anki_error, mock_anki_response, setup_mock_server};

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
        ankit::actions::MultiAction::new("deckNames"),
        ankit::actions::MultiAction::new("modelNames"),
    ];

    let result = client.misc().multi(&actions).await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_sync() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "sync",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.misc().sync().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_export_package() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "exportPackage", mock_anki_response(true)).await;

    let result = client
        .misc()
        .export_package("Default", "/tmp/deck.apkg", Some(true))
        .await
        .unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_import_package() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "importPackage", mock_anki_response(true)).await;

    let result = client
        .misc()
        .import_package("/tmp/deck.apkg")
        .await
        .unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_reload_collection() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "reloadCollection",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.misc().reload_collection().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_api_reflect() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "apiReflect",
        mock_anki_response(serde_json::json!({
            "scopes": ["actions", "scopes"],
            "actions": ["deckNames", "modelNames", "addNote"]
        })),
    )
    .await;

    let result = client.misc().api_reflect(&["actions"], None).await.unwrap();
    assert!(!result.actions.is_empty());
    assert!(result.actions.contains(&"deckNames".to_string()));
}

#[tokio::test]
async fn test_multi_with_params() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "multi",
        mock_anki_response(vec![
            serde_json::json!(["Default"]),
            serde_json::json!([1234567890_i64]),
        ]),
    )
    .await;

    let actions = vec![
        ankit::actions::MultiAction::new("deckNames"),
        ankit::actions::MultiAction::with_params(
            "findNotes",
            serde_json::json!({"query": "deck:Default"}),
        ),
    ];

    let result = client.misc().multi(&actions).await.unwrap();
    assert_eq!(result.len(), 2);
}
