//! Tests for card AnkiConnect actions.

mod common;

use ankit::AnkiClient;
use common::{mock_action, mock_anki_response, setup_mock_server};

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

#[tokio::test]
async fn test_cards_to_notes() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "cardsToNotes",
        mock_anki_response(vec![1000_i64, 1001, 1002]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let notes = client.cards().to_notes(&[1, 2, 3]).await.unwrap();

    assert_eq!(notes, vec![1000, 1001, 1002]);
}

#[tokio::test]
async fn test_cards_mod_time() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "cardsModTime",
        mock_anki_response(vec![
            serde_json::json!({"cardId": 123, "mod": 1705330000}),
            serde_json::json!({"cardId": 456, "mod": 1705330100}),
        ]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let times = client.cards().mod_time(&[123, 456]).await.unwrap();

    assert_eq!(times.len(), 2);
    assert_eq!(times[0].card_id, 123);
    assert_eq!(times[0].mod_time, 1705330000);
}

#[tokio::test]
async fn test_suspend_cards() {
    let server = setup_mock_server().await;
    mock_action(&server, "suspend", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().suspend(&[1234567890]).await.unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_unsuspend_cards() {
    let server = setup_mock_server().await;
    mock_action(&server, "unsuspend", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().unsuspend(&[1234567890]).await.unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_is_suspended() {
    let server = setup_mock_server().await;
    mock_action(&server, "suspended", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().is_suspended(1234567890).await.unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_are_suspended() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "areSuspended",
        mock_anki_response(vec![Some(true), Some(false), Option::<bool>::None]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().are_suspended(&[1, 2, 3]).await.unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Some(true));
    assert_eq!(result[1], Some(false));
    assert_eq!(result[2], None);
}

#[tokio::test]
async fn test_are_due() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "areDue",
        mock_anki_response(vec![true, false, true]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().are_due(&[1, 2, 3]).await.unwrap();

    assert_eq!(result, vec![true, false, true]);
}

#[tokio::test]
async fn test_get_intervals() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "getIntervals",
        mock_anki_response(vec![10_i64, 20, 30]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().intervals(&[1, 2, 3], false).await.unwrap();

    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn test_get_ease_factors() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "getEaseFactors",
        mock_anki_response(vec![2500_i64, 2300, 2700]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().get_ease(&[1, 2, 3]).await.unwrap();

    assert_eq!(result, vec![2500, 2300, 2700]);
}

#[tokio::test]
async fn test_set_ease_factors() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "setEaseFactors",
        mock_anki_response(vec![true, true]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client
        .cards()
        .set_ease(&[1, 2], &[2500, 2600])
        .await
        .unwrap();

    assert_eq!(result, vec![true, true]);
}

#[tokio::test]
async fn test_forget_cards() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "forgetCards",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().forget(&[1234567890]).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_relearn_cards() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "relearnCards",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().relearn(&[1234567890]).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_answer_cards() {
    let server = setup_mock_server().await;
    mock_action(&server, "answerCards", mock_anki_response(vec![true, true])).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let answers = vec![
        ankit::CardAnswer::new(1, ankit::Ease::Good),
        ankit::CardAnswer::new(2, ankit::Ease::Easy),
    ];
    let result = client.cards().answer(&answers).await.unwrap();

    assert_eq!(result, vec![true, true]);
}

#[tokio::test]
async fn test_set_due_date() {
    let server = setup_mock_server().await;
    mock_action(&server, "setDueDate", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client.cards().set_due_date(&[1, 2, 3], "0").await.unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_set_due_date_range() {
    let server = setup_mock_server().await;
    mock_action(&server, "setDueDate", mock_anki_response(true)).await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client
        .cards()
        .set_due_date(&[1, 2, 3], "1-7")
        .await
        .unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_set_specific_value() {
    let server = setup_mock_server().await;
    mock_action(
        &server,
        "setSpecificValueOfCard",
        mock_anki_response(vec![true]),
    )
    .await;

    let client = AnkiClient::builder().url(server.uri()).build();
    let result = client
        .cards()
        .set_specific_value(1234567890, &["ivl"], &["30"], true)
        .await
        .unwrap();

    assert_eq!(result, vec![true]);
}
