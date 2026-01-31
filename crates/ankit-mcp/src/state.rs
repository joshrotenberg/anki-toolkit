//! Shared state for the Anki MCP server.

use std::sync::Arc;

use ankit_engine::Engine;
use tower_mcp::Error;
use tracing::warn;

/// Shared state containing the Anki engine and configuration.
#[derive(Clone)]
pub struct AnkiState {
    /// The Anki engine for API operations.
    pub engine: Arc<Engine>,
    /// Whether the server is in read-only mode.
    pub read_only: bool,
}

impl AnkiState {
    /// Create a new AnkiState.
    pub fn new(url: &str, read_only: bool) -> Self {
        let client = ankit_engine::ClientBuilder::new().url(url).build();
        let engine = Engine::from_client(client);
        Self {
            engine: Arc::new(engine),
            read_only,
        }
    }

    /// Check if a write operation is allowed.
    ///
    /// Returns an error if the server is in read-only mode.
    pub fn check_write(&self, operation: &str) -> Result<(), Error> {
        if self.read_only {
            warn!("Blocked write operation in read-only mode: {}", operation);
            Err(Error::tool(format!(
                "Write operation '{}' is not allowed in read-only mode",
                operation
            )))
        } else {
            Ok(())
        }
    }
}
