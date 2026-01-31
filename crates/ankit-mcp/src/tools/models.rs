//! Model (note type) tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::debug;

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetModelFieldsParams {
    /// Model (note type) name
    pub model: String,
}

/// List all note type (model) names in Anki.
pub fn list_models(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("list_models")
        .description("List all note type (model) names in Anki.")
        .read_only()
        .handler_with_state_no_params(state, |state: Arc<AnkiState>| async move {
            debug!("Listing models");

            let models = state
                .engine
                .client()
                .models()
                .names()
                .await
                .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

            debug!(count = models.len(), "Listed models");
            Ok(CallToolResult::text(
                serde_json::to_string_pretty(&models).unwrap(),
            ))
        })
        .expect("valid tool")
}

/// Get the field names for a note type (model).
pub fn get_model_fields(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("get_model_fields")
        .description("Get the field names for a note type (model).")
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: GetModelFieldsParams| async move {
                debug!(model = %params.model, "Getting model fields");

                let fields = state
                    .engine
                    .client()
                    .models()
                    .field_names(&params.model)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&fields).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}
