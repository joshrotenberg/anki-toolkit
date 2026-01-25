//! Common test utilities for ankit-engine workflow tests.

use ankit_engine::Engine;
use serde::Serialize;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate, Times};

/// Start a new mock server for testing.
pub async fn setup_mock_server() -> MockServer {
    MockServer::start().await
}

/// Create an Engine connected to the mock server.
pub fn engine_for_mock(server: &MockServer) -> Engine {
    let client = ankit_engine::ClientBuilder::new().url(server.uri()).build();
    Engine::from_client(client)
}

/// Create a successful AnkiConnect response.
pub fn mock_anki_response<T: Serialize>(result: T) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "result": result,
        "error": null
    }))
}

/// Create an error AnkiConnect response.
#[allow(dead_code)]
pub fn mock_anki_error(error: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "result": null,
        "error": error
    }))
}

/// Mount a mock for a specific action (expect exactly 1 call).
pub async fn mock_action(server: &MockServer, action: &str, response: ResponseTemplate) {
    mock_action_times(server, action, response, 1).await;
}

/// Mount a mock for a specific action with expected call count.
pub async fn mock_action_times(
    server: &MockServer,
    action: &str,
    response: ResponseTemplate,
    times: u64,
) {
    Mock::given(method("POST"))
        .and(body_partial_json(serde_json::json!({
            "action": action,
            "version": 6
        })))
        .respond_with(response)
        .expect(Times::from(times))
        .mount(server)
        .await;
}

/// Mount a mock that can be called any number of times (for optional calls).
#[allow(dead_code)]
pub async fn mock_action_any(server: &MockServer, action: &str, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(body_partial_json(serde_json::json!({
            "action": action,
            "version": 6
        })))
        .respond_with(response)
        .expect(Times::from(0..))
        .mount(server)
        .await;
}
