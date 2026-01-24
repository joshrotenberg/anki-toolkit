//! Tests for statistics actions.

mod common;

use common::{mock_action, mock_anki_response, setup_mock_server};
use yanki::AnkiClient;

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
