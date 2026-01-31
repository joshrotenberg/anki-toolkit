//! Deck management tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDeckParams {
    /// Name of the deck to create
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDeckParams {
    /// Name of the deck to delete
    pub name: String,
    /// If true, also delete all cards in the deck. If false, cards are moved to Default deck.
    #[serde(default)]
    pub cards_too: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CloneDeckParams {
    /// Source deck name
    pub source: String,
    /// Destination deck name
    pub destination: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MergeDecksParams {
    /// Source deck names to merge
    pub sources: Vec<String>,
    /// Destination deck name
    pub destination: String,
}

/// List all deck names in Anki.
pub fn list_decks(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("list_decks")
        .description("List all deck names in Anki.")
        .read_only()
        .handler_no_params_with_state(state, |state: Arc<AnkiState>| async move {
            debug!("Listing decks");

            let decks = state
                .engine
                .client()
                .decks()
                .names()
                .await
                .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

            debug!(count = decks.len(), "Listed decks");
            Ok(CallToolResult::text(
                serde_json::to_string_pretty(&decks).unwrap(),
            ))
        })
        .expect("valid tool")
}

/// Create a new deck. Returns the deck ID.
pub fn create_deck(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("create_deck")
        .description("Create a new deck. Returns the deck ID.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: CreateDeckParams| async move {
                state.check_write("create_deck")?;
                debug!(name = %params.name, "Creating deck");

                let deck_id = state
                    .engine
                    .client()
                    .decks()
                    .create(&params.name)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(deck_id, name = %params.name, "Deck created");
                Ok(CallToolResult::text(format!(
                    "Created deck '{}' with ID: {}",
                    params.name, deck_id
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Delete a deck. If cards_too is false, cards are moved to Default deck.
pub fn delete_deck(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("delete_deck")
        .description("Delete a deck. If cards_too is false, cards are moved to Default deck.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: DeleteDeckParams| async move {
                state.check_write("delete_deck")?;
                debug!(name = %params.name, cards_too = params.cards_too, "Deleting deck");

                state
                    .engine
                    .client()
                    .decks()
                    .delete(&[params.name.as_str()], params.cards_too)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                let action = if params.cards_too {
                    "and its cards"
                } else {
                    "(cards moved to Default)"
                };

                info!(name = %params.name, "Deck deleted");
                Ok(CallToolResult::text(format!(
                    "Deleted deck '{}' {}",
                    params.name, action
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Clone a deck with all its notes. Cards start as new.
pub fn clone_deck(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("clone_deck")
        .description("Clone a deck with all its notes. Cards start as new.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: CloneDeckParams| async move {
                state.check_write("clone_deck")?;
                debug!(source = %params.source, destination = %params.destination, "Cloning deck");

                let report = state
                    .engine
                    .organize()
                    .clone_deck(&params.source, &params.destination)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    notes_cloned = report.notes_cloned,
                    notes_failed = report.notes_failed,
                    destination = %report.destination,
                    "Deck cloned"
                );
                Ok(CallToolResult::text(format!(
                    "Cloned {} notes to '{}' ({} failed)",
                    report.notes_cloned, report.destination, report.notes_failed
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Merge multiple decks into one destination deck.
pub fn merge_decks(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("merge_decks")
        .description("Merge multiple decks into one destination deck.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: MergeDecksParams| async move {
                state.check_write("merge_decks")?;
                debug!(
                    sources = ?params.sources,
                    destination = %params.destination,
                    "Merging decks"
                );

                let sources: Vec<&str> = params.sources.iter().map(|s| s.as_str()).collect();
                let report = state
                    .engine
                    .organize()
                    .merge_decks(&sources, &params.destination)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    cards_moved = report.cards_moved,
                    destination = %report.destination,
                    "Decks merged"
                );
                Ok(CallToolResult::text(format!(
                    "Moved {} cards to '{}'",
                    report.cards_moved, report.destination
                )))
            },
        )
        .build()
        .expect("valid tool")
}
