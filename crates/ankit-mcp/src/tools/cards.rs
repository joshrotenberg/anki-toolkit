//! Card management tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindCardsParams {
    /// Anki search query (e.g., "deck:Japanese is:due")
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCardsInfoParams {
    /// Card IDs to get info for
    pub card_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuspendCardsParams {
    /// Card IDs to suspend
    pub card_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnsuspendCardsParams {
    /// Card IDs to unsuspend
    pub card_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ForgetCardsParams {
    /// Card IDs to forget (reset to new state)
    pub card_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetEaseParams {
    /// Card IDs to set ease for
    pub card_ids: Vec<i64>,
    /// Ease factors as integers (e.g., 2500 = 250%)
    pub ease_factors: Vec<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDueDateParams {
    /// Card IDs to set due date for
    pub card_ids: Vec<i64>,
    /// Days specification: "0" (today), "1" (tomorrow), "-1" (yesterday), "1-7" (random range), "0!" (today and reset interval)
    pub days: String,
}

/// Search for cards using Anki query syntax. Returns card IDs.
pub fn find_cards(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("find_cards")
        .description(
            "Search for cards using Anki query syntax (e.g., 'deck:Japanese is:due'). Returns card IDs.",
        )
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: FindCardsParams| async move {
                debug!(query = %params.query, "Finding cards");

                let card_ids = state
                    .engine
                    .client()
                    .cards()
                    .find(&params.query)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                debug!(count = card_ids.len(), "Found cards");
                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&card_ids).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Get detailed information about cards including reps, lapses, ease factor, and interval.
pub fn get_cards_info(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("get_cards_info")
        .description(
            "Get detailed information about cards including reps, lapses, ease factor, and interval.",
        )
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: GetCardsInfoParams| async move {
                debug!(count = params.card_ids.len(), "Getting cards info");

                let cards = state
                    .engine
                    .client()
                    .cards()
                    .info(&params.card_ids)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                Ok(CallToolResult::text(
                    serde_json::to_string_pretty(&cards).unwrap(),
                ))
            },
        )
        .build()
        .expect("valid tool")
}

/// Suspend cards to prevent them from appearing in reviews.
pub fn suspend_cards(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("suspend_cards")
        .description("Suspend cards to prevent them from appearing in reviews.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: SuspendCardsParams| async move {
                state.check_write("suspend_cards")?;
                debug!(count = params.card_ids.len(), "Suspending cards");

                state
                    .engine
                    .client()
                    .cards()
                    .suspend(&params.card_ids)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = params.card_ids.len(), "Cards suspended");
                Ok(CallToolResult::text(format!(
                    "Suspended {} cards",
                    params.card_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Unsuspend previously suspended cards.
pub fn unsuspend_cards(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("unsuspend_cards")
        .description("Unsuspend previously suspended cards.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: UnsuspendCardsParams| async move {
                state.check_write("unsuspend_cards")?;
                debug!(count = params.card_ids.len(), "Unsuspending cards");

                state
                    .engine
                    .client()
                    .cards()
                    .unsuspend(&params.card_ids)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = params.card_ids.len(), "Cards unsuspended");
                Ok(CallToolResult::text(format!(
                    "Unsuspended {} cards",
                    params.card_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Reset cards to new state, clearing all learning progress.
pub fn forget_cards(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("forget_cards")
        .description("Reset cards to new state, clearing all learning progress.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ForgetCardsParams| async move {
                state.check_write("forget_cards")?;
                debug!(count = params.card_ids.len(), "Forgetting cards");

                state
                    .engine
                    .client()
                    .cards()
                    .forget(&params.card_ids)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = params.card_ids.len(), "Cards reset to new");
                Ok(CallToolResult::text(format!(
                    "Reset {} cards to new state",
                    params.card_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Set ease factors for cards. Ease factors are integers (e.g., 2500 = 250%).
pub fn set_ease(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("set_ease")
        .description("Set ease factors for cards. Ease factors are integers (e.g., 2500 = 250%).")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: SetEaseParams| async move {
                state.check_write("set_ease")?;
                debug!(count = params.card_ids.len(), "Setting ease factors");

                let results = state
                    .engine
                    .client()
                    .cards()
                    .set_ease(&params.card_ids, &params.ease_factors)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                let success_count = results.iter().filter(|&&r| r).count();
                info!(success_count, "Ease factors set");
                Ok(CallToolResult::text(format!(
                    "Set ease for {} of {} cards",
                    success_count,
                    params.card_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Set due date for cards.
pub fn set_due_date(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("set_due_date")
        .description("Set due date for cards. Days can be: '0' (today), '1' (tomorrow), '-1' (yesterday), '1-7' (random range), '0!' (today and reset interval).")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: SetDueDateParams| async move {
                state.check_write("set_due_date")?;
                debug!(count = params.card_ids.len(), days = %params.days, "Setting due date");

                state
                    .engine
                    .client()
                    .cards()
                    .set_due_date(&params.card_ids, &params.days)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = params.card_ids.len(), days = %params.days, "Due date set");
                Ok(CallToolResult::text(format!(
                    "Set due date to '{}' for {} cards",
                    params.days,
                    params.card_ids.len()
                )))
            },
        )
        .build()
        .expect("valid tool")
}
