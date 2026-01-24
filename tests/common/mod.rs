//! Common test utilities for AnkiConnect tests.

use serde::Serialize;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Start a new mock server for testing.
pub async fn setup_mock_server() -> MockServer {
    MockServer::start().await
}

/// Create a successful AnkiConnect response.
pub fn mock_anki_response<T: Serialize>(result: T) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "result": result,
        "error": null
    }))
}

/// Create an error AnkiConnect response.
#[allow(dead_code)] // Not all test files use this
pub fn mock_anki_error(error: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "result": null,
        "error": error
    }))
}

/// Mount a mock for a specific action.
pub async fn mock_action(server: &MockServer, action: &str, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(body_partial_json(serde_json::json!({
            "action": action,
            "version": 6
        })))
        .respond_with(response)
        .expect(1)
        .mount(server)
        .await;
}
