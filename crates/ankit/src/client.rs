//! The AnkiConnect client and builder.

use std::time::Duration;

use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

use crate::actions::{
    CardActions, DeckActions, GuiActions, MediaActions, MiscActions, ModelActions, NoteActions,
    StatisticsActions,
};
use crate::error::{Error, Result};
use crate::request::{AnkiRequest, AnkiResponse};

/// Default URL for AnkiConnect.
const DEFAULT_URL: &str = "http://127.0.0.1:8765";

/// Default timeout for requests.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The main client for interacting with AnkiConnect.
///
/// # Example
///
/// ```no_run
/// use ankit::AnkiClient;
///
/// # async fn example() -> ankit::Result<()> {
/// // Create a client with default settings
/// let client = AnkiClient::new();
///
/// // Check the AnkiConnect version
/// let version = client.misc().version().await?;
/// println!("AnkiConnect version: {}", version);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AnkiClient {
    http_client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl AnkiClient {
    /// Create a new client with default settings.
    ///
    /// Connects to `http://127.0.0.1:8765` with a 30 second timeout.
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Create a builder for custom client configuration.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Access deck operations.
    pub fn decks(&self) -> DeckActions<'_> {
        DeckActions { client: self }
    }

    /// Access miscellaneous operations.
    pub fn misc(&self) -> MiscActions<'_> {
        MiscActions { client: self }
    }

    /// Access note operations.
    pub fn notes(&self) -> NoteActions<'_> {
        NoteActions { client: self }
    }

    /// Access card operations.
    pub fn cards(&self) -> CardActions<'_> {
        CardActions { client: self }
    }

    /// Access media operations.
    pub fn media(&self) -> MediaActions<'_> {
        MediaActions { client: self }
    }

    /// Access model (note type) operations.
    pub fn models(&self) -> ModelActions<'_> {
        ModelActions { client: self }
    }

    /// Access GUI operations.
    pub fn gui(&self) -> GuiActions<'_> {
        GuiActions { client: self }
    }

    /// Access statistics operations.
    pub fn statistics(&self) -> StatisticsActions<'_> {
        StatisticsActions { client: self }
    }

    /// Execute an action without parameters.
    pub(crate) async fn invoke_without_params<R>(&self, action: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let request = AnkiRequest::<()>::without_params(action, self.api_key.as_deref());
        self.send_request(&request).await
    }

    /// Execute an action with parameters.
    pub(crate) async fn invoke<P, R>(&self, action: &str, params: P) -> Result<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let request = AnkiRequest::new(action, params, self.api_key.as_deref());
        self.send_request(&request).await
    }

    /// Execute an action that returns null on success.
    pub(crate) async fn invoke_void<P>(&self, action: &str, params: P) -> Result<()>
    where
        P: Serialize,
    {
        let request = AnkiRequest::new(action, params, self.api_key.as_deref());
        self.send_void_request(&request).await
    }

    /// Execute an action without parameters that returns null on success.
    pub(crate) async fn invoke_void_without_params(&self, action: &str) -> Result<()> {
        let request = AnkiRequest::<()>::without_params(action, self.api_key.as_deref());
        self.send_void_request(&request).await
    }

    /// Execute an action without parameters that may return null.
    pub(crate) async fn invoke_nullable_without_params<R>(&self, action: &str) -> Result<Option<R>>
    where
        R: DeserializeOwned,
    {
        let request = AnkiRequest::<()>::without_params(action, self.api_key.as_deref());
        self.send_nullable_request(&request).await
    }

    /// Send a request to AnkiConnect and process the response.
    async fn send_request<T, R>(&self, request: &AnkiRequest<'_, T>) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let response = self
            .http_client
            .post(&self.base_url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    Error::ConnectionRefused
                } else {
                    Error::Http(e)
                }
            })?;

        let anki_response: AnkiResponse<R> = response.json().await?;

        match (anki_response.result, anki_response.error) {
            (Some(result), None) => Ok(result),
            (None, Some(err)) => {
                if err.contains("permission") {
                    Err(Error::PermissionDenied)
                } else {
                    Err(Error::AnkiConnect(err))
                }
            }
            (None, None) => Err(Error::EmptyResponse),
            (Some(_), Some(err)) => Err(Error::AnkiConnect(err)),
        }
    }

    /// Send a request for an action that returns null on success.
    async fn send_void_request<T>(&self, request: &AnkiRequest<'_, T>) -> Result<()>
    where
        T: Serialize,
    {
        let response = self
            .http_client
            .post(&self.base_url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    Error::ConnectionRefused
                } else {
                    Error::Http(e)
                }
            })?;

        // For void actions, we only check for errors - null result is success
        let anki_response: AnkiResponse<serde_json::Value> = response.json().await?;

        if let Some(err) = anki_response.error {
            if err.contains("permission") {
                Err(Error::PermissionDenied)
            } else {
                Err(Error::AnkiConnect(err))
            }
        } else {
            Ok(())
        }
    }

    /// Send a request for an action where null is a valid response.
    async fn send_nullable_request<T, R>(&self, request: &AnkiRequest<'_, T>) -> Result<Option<R>>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let response = self
            .http_client
            .post(&self.base_url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    Error::ConnectionRefused
                } else {
                    Error::Http(e)
                }
            })?;

        let anki_response: AnkiResponse<R> = response.json().await?;

        match (anki_response.result, anki_response.error) {
            (Some(result), None) => Ok(Some(result)),
            (None, Some(err)) => {
                if err.contains("permission") {
                    Err(Error::PermissionDenied)
                } else {
                    Err(Error::AnkiConnect(err))
                }
            }
            (None, None) => Ok(None),
            (Some(_), Some(err)) => Err(Error::AnkiConnect(err)),
        }
    }
}

impl Default for AnkiClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating a customized [`AnkiClient`].
///
/// # Example
///
/// ```no_run
/// use std::time::Duration;
/// use ankit::AnkiClient;
///
/// let client = AnkiClient::builder()
///     .url("http://localhost:8765")
///     .api_key("my-secret-key")
///     .timeout(Duration::from_secs(60))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
}

impl ClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            base_url: DEFAULT_URL.to_string(),
            api_key: None,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set the AnkiConnect URL.
    ///
    /// Defaults to `http://127.0.0.1:8765`.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the API key for authentication.
    ///
    /// Only required if AnkiConnect is configured to require an API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the request timeout.
    ///
    /// Defaults to 30 seconds.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// Build the client.
    pub fn build(self) -> AnkiClient {
        let http_client = Client::builder()
            .timeout(self.timeout)
            .build()
            .expect("Failed to build HTTP client");

        AnkiClient {
            http_client,
            base_url: self.base_url,
            api_key: self.api_key,
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
