//! Tests for GUI actions.

mod common;

use ankit::AnkiClient;
use common::{mock_action, mock_anki_response, setup_mock_server};

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

#[tokio::test]
async fn test_gui_add_cards() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiAddCards", mock_anki_response(1234567890_i64)).await;

    let note = ankit::NoteBuilder::new("Default", "Basic")
        .field("Front", "Question")
        .field("Back", "Answer")
        .build();

    let result = client.gui().add_cards(note).await.unwrap();
    assert_eq!(result, Some(1234567890));
}

#[tokio::test]
async fn test_gui_edit_note() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "guiEditNote",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.gui().edit_note(1234567890).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_gui_start_timer() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiStartCardTimer", mock_anki_response(true)).await;

    let result = client.gui().start_timer().await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_gui_show_question() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiShowQuestion", mock_anki_response(true)).await;

    let result = client.gui().show_question().await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_gui_show_answer() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiShowAnswer", mock_anki_response(true)).await;

    let result = client.gui().show_answer().await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_gui_answer_card() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiAnswerCard", mock_anki_response(true)).await;

    let result = client.gui().answer_card(ankit::Ease::Good).await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_gui_deck_review() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "guiDeckReview", mock_anki_response(true)).await;

    let result = client.gui().deck_review("Default").await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_gui_import_file() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "guiImportFile",
        mock_anki_response(serde_json::json!({
            "found_notes": 10,
            "imported_notes": 8
        })),
    )
    .await;

    let result = client
        .gui()
        .import_file("/path/to/deck.apkg")
        .await
        .unwrap();
    assert_eq!(result.found_notes, 10);
    assert_eq!(result.imported_notes, 8);
}

#[tokio::test]
async fn test_gui_exit_anki() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "guiExitAnki",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.gui().exit_anki().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_gui_undo() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "guiUndo",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.gui().undo().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_gui_current_card_with_data() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "guiCurrentCard",
        mock_anki_response(serde_json::json!({
            "cardId": 1234567890_i64,
            "noteId": 9876543210_i64,
            "deckId": 1,
            "modelId": 2,
            "fields": {},
            "question": "<div>Front</div>",
            "answer": "<div>Back</div>",
            "deckName": "Default",
            "modelName": "Basic",
            "templateName": "Card 1",
            "buttons": [1, 2, 3, 4],
            "nextReviews": ["<1m", "1d", "3d", "7d"]
        })),
    )
    .await;

    let result = client.gui().current_card().await.unwrap();
    assert!(result.is_some());
    let card = result.unwrap();
    assert_eq!(card.card_id, 1234567890);
    assert_eq!(card.deck_name, "Default");
}
