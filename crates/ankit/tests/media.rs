//! Tests for media actions.

mod common;

use ankit::{AnkiClient, StoreMediaParams};
use common::{mock_action, mock_anki_response, setup_mock_server};

#[tokio::test]
async fn test_store_media_from_base64() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "storeMediaFile",
        mock_anki_response("test_audio.mp3"),
    )
    .await;

    let params = StoreMediaParams::from_base64("test_audio.mp3", "SGVsbG8gV29ybGQ=");
    let result = client.media().store(params).await.unwrap();
    assert_eq!(result, "test_audio.mp3");
}

#[tokio::test]
async fn test_store_media_from_url() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "storeMediaFile", mock_anki_response("image.png")).await;

    let params = StoreMediaParams::from_url("image.png", "https://example.com/image.png");
    let result = client.media().store(params).await.unwrap();
    assert_eq!(result, "image.png");
}

#[tokio::test]
async fn test_retrieve_media() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "retrieveMediaFile",
        mock_anki_response("SGVsbG8gV29ybGQ="),
    )
    .await;

    let result = client.media().retrieve("test.txt").await.unwrap();
    assert_eq!(result, "SGVsbG8gV29ybGQ=");
}

#[tokio::test]
async fn test_list_media() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "getMediaFilesNames",
        mock_anki_response(vec!["audio1.mp3", "audio2.mp3", "audio3.mp3"]),
    )
    .await;

    let result = client.media().list("*.mp3").await.unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"audio1.mp3".to_string()));
}

#[tokio::test]
async fn test_media_directory() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "getMediaDirPath",
        mock_anki_response("/Users/test/.local/share/Anki2/User 1/collection.media"),
    )
    .await;

    let result = client.media().directory().await.unwrap();
    assert!(result.contains("collection.media"));
}

#[tokio::test]
async fn test_delete_media() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    // Delete returns null on success
    mock_action(
        &server,
        "deleteMediaFile",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.media().delete("old_file.mp3").await;
    assert!(result.is_ok());
}
