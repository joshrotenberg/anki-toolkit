//! Tests for GUI actions.

mod common;

use common::{mock_action, mock_anki_response, setup_mock_server};
use yanki::AnkiClient;

#[tokio::test]
async fn test_gui_browse() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "guiBrowse",
        mock_anki_response(vec![1234567890_i64, 1234567891, 1234567892]),
    )
    .await;

    let result = client.gui().browse("deck:Default").await.unwrap();
    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn test_gui_selected_notes() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "guiSelectedNotes",
        mock_anki_response(vec![1234567890_i64, 1234567891]),
    )
    .await;

    let result = client.gui().selected_notes().await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_gui_deck_browser() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiDeckBrowser", mock_anki_response(true)).await;

    let result = client.gui().deck_browser().await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_gui_deck_overview() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiDeckOverview", mock_anki_response(true)).await;

    let result = client.gui().deck_overview("Default").await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_gui_current_card_none() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    // When not in review, current_card returns null
    mock_action(
        &server,
        "guiCurrentCard",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.gui().current_card().await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_gui_check_database() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiCheckDatabase", mock_anki_response(true)).await;

    let result = client.gui().check_database().await.unwrap();
    assert!(result);
}
