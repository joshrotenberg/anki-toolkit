//! Tests for model actions.

mod common;

use common::{mock_action, mock_anki_response, setup_mock_server};
use std::collections::HashMap;
use yanki::{AnkiClient, CreateModelParams};

#[tokio::test]
async fn test_model_names() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelNames",
        mock_anki_response(vec!["Basic", "Basic (and reversed card)", "Cloze"]),
    )
    .await;

    let result = client.models().names().await.unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"Basic".to_string()));
}

#[tokio::test]
async fn test_model_names_and_ids() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    let mut expected = HashMap::new();
    expected.insert("Basic", 1234567890_i64);
    expected.insert("Cloze", 9876543210_i64);

    mock_action(&server, "modelNamesAndIds", mock_anki_response(expected)).await;

    let result = client.models().names_and_ids().await.unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result.get("Basic"), Some(&1234567890));
}

#[tokio::test]
async fn test_model_field_names() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldNames",
        mock_anki_response(vec!["Front", "Back"]),
    )
    .await;

    let result = client.models().field_names("Basic").await.unwrap();
    assert_eq!(result, vec!["Front", "Back"]);
}

#[tokio::test]
async fn test_create_model() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "createModel",
        mock_anki_response(serde_json::json!({
            "id": 1234567890,
            "name": "My Model"
        })),
    )
    .await;

    let params = CreateModelParams::new("My Model")
        .field("Front")
        .field("Back")
        .css(".card { font-family: arial; }")
        .template("Card 1", "{{Front}}", "{{FrontSide}}<hr>{{Back}}");

    let result = client.models().create(params).await.unwrap();
    assert!(result.get("id").is_some());
}

#[tokio::test]
async fn test_model_styling() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelStyling",
        mock_anki_response(serde_json::json!({
            "css": ".card { font-family: arial; font-size: 20px; }"
        })),
    )
    .await;

    let result = client.models().styling("Basic").await.unwrap();
    assert!(result.css.contains("font-family"));
}
