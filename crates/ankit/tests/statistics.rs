//! Tests for statistics actions.

mod common;

use ankit::AnkiClient;
use common::{mock_action, mock_anki_response, setup_mock_server};

#[tokio::test]
async fn test_cards_reviewed_today() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "getNumCardsReviewedToday",
        mock_anki_response(42_i64),
    )
    .await;

    let result = client.statistics().cards_reviewed_today().await.unwrap();
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_cards_reviewed_by_day() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    // AnkiConnect returns an array of [date, count] pairs
    let by_day = vec![("2024-01-15", 30_i64), ("2024-01-16", 25_i64)];

    mock_action(
        &server,
        "getNumCardsReviewedByDay",
        mock_anki_response(by_day),
    )
    .await;

    let result = client.statistics().cards_reviewed_by_day().await.unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "2024-01-15");
    assert_eq!(result[0].1, 30);
}

#[tokio::test]
async fn test_collection_html() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "getCollectionStatsHTML",
        mock_anki_response("<html>stats</html>"),
    )
    .await;

    let result = client.statistics().collection_html(true).await.unwrap();
    assert!(result.contains("stats"));
}

#[tokio::test]
async fn test_latest_review_id() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "getLatestReviewID",
        mock_anki_response(1705330000000_i64),
    )
    .await;

    let result = client
        .statistics()
        .latest_review_id("Default")
        .await
        .unwrap();
    assert_eq!(result, 1705330000000);
}

#[tokio::test]
async fn test_reviews_since() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "cardReviews",
        mock_anki_response(serde_json::json!({
            "1234567890": [[1705330000000_i64, 3, 10]],
            "1234567891": [[1705330100000_i64, 2, 5]]
        })),
    )
    .await;

    let result = client
        .statistics()
        .reviews_since("Default", 0)
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.contains_key("1234567890"));
}

#[tokio::test]
async fn test_reviews_for_cards() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "getReviewsOfCards",
        mock_anki_response(serde_json::json!({
            "1234567890": [{
                "cardId": 1234567890_i64,
                "id": 1705330000000_i64,
                "ease": 3,
                "ivl": 10,
                "lastIvl": 1,
                "factor": 2500,
                "time": 5000,
                "type": 1
            }]
        })),
    )
    .await;

    let result = client
        .statistics()
        .reviews_for_cards(&[1234567890])
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    let reviews = result.get("1234567890").unwrap();
    assert_eq!(reviews[0].card_id, 1234567890);
    assert_eq!(reviews[0].ease, 3);
}

#[tokio::test]
async fn test_insert_reviews() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "insertReviews",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let reviews = vec![
        ankit::ReviewEntry::new(1234567890, 1705330000000)
            .ease(3)
            .interval(10)
            .time(5000),
    ];

    let result = client.statistics().insert(&reviews).await;
    assert!(result.is_ok());
}
