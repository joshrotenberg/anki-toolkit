//! Internal request and response types for the AnkiConnect protocol.

use serde::{Deserialize, Serialize};

/// The request format expected by AnkiConnect.
#[derive(Debug, Serialize)]
pub(crate) struct AnkiRequest<'a, T> {
    /// The action to perform.
    pub action: &'a str,
    /// The API version (always 6).
    pub version: u8,
    /// Optional API key for authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<&'a str>,
    /// Optional parameters for the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

impl<'a, T> AnkiRequest<'a, T> {
    /// Create a new request with parameters.
    pub fn new(action: &'a str, params: T, key: Option<&'a str>) -> Self {
        Self {
            action,
            version: 6,
            key,
            params: Some(params),
        }
    }

    /// Create a new request without parameters.
    pub fn without_params(action: &'a str, key: Option<&'a str>) -> AnkiRequest<'a, ()> {
        AnkiRequest {
            action,
            version: 6,
            key,
            params: None,
        }
    }
}

/// The response format returned by AnkiConnect.
#[derive(Debug, Deserialize)]
pub(crate) struct AnkiResponse<T> {
    /// The result of the action, if successful.
    pub result: Option<T>,
    /// The error message, if the action failed.
    pub error: Option<String>,
}
