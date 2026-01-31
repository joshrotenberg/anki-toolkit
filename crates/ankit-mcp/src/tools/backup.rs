//! Backup and restore tools.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder};
use tracing::{debug, info};

use crate::state::AnkiState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BackupDeckParams {
    /// Deck name to backup
    pub deck: String,
    /// Directory to save the backup file
    pub backup_dir: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BackupCollectionParams {
    /// Directory to save backup files
    pub backup_dir: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RestoreDeckParams {
    /// Path to the .apkg backup file
    pub backup_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListBackupsParams {
    /// Directory to scan for backup files
    pub backup_dir: String,
}

/// Backup a deck to an .apkg file.
pub fn backup_deck(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("backup_deck")
        .description("Backup a deck to an .apkg file. Creates a timestamped backup file. IMPORTANT: Always backup before making bulk changes.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: BackupDeckParams| async move {
                // Backup is a write operation because it creates files
                state.check_write("backup_deck")?;
                debug!(deck = %params.deck, backup_dir = %params.backup_dir, "Backing up deck");

                let result = state
                    .engine
                    .backup()
                    .backup_deck(&params.deck, &params.backup_dir)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    deck = %result.deck_name,
                    path = %result.path.display(),
                    size = result.size_bytes,
                    "Deck backed up"
                );

                Ok(CallToolResult::text(format!(
                    "Backed up deck '{}' to {} ({} bytes)",
                    result.deck_name,
                    result.path.display(),
                    result.size_bytes
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Backup all decks in the collection to separate .apkg files.
pub fn backup_collection(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("backup_collection")
        .description("Backup all decks in the collection to separate .apkg files. Creates a timestamped directory with one file per deck.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: BackupCollectionParams| async move {
                state.check_write("backup_collection")?;
                debug!(backup_dir = %params.backup_dir, "Backing up collection");

                let result = state
                    .engine
                    .backup()
                    .backup_collection(&params.backup_dir)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    successful = result.successful.len(),
                    failed = result.failed.len(),
                    dir = %result.backup_dir.display(),
                    "Collection backed up"
                );

                let mut msg = format!(
                    "Backed up {} decks to {}",
                    result.successful.len(),
                    result.backup_dir.display()
                );
                if !result.failed.is_empty() {
                    msg.push_str(&format!(
                        ". {} failed: {:?}",
                        result.failed.len(),
                        result.failed
                    ));
                }

                Ok(CallToolResult::text(msg))
            },
        )
        .build()
        .expect("valid tool")
}

/// Restore a deck from an .apkg backup file.
pub fn restore_deck(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("restore_deck")
        .description("Restore a deck from an .apkg backup file.")
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: RestoreDeckParams| async move {
                state.check_write("restore_deck")?;
                debug!(backup_path = %params.backup_path, "Restoring deck");

                let result = state
                    .engine
                    .backup()
                    .restore_deck(&params.backup_path)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(
                    path = %result.path.display(),
                    success = result.success,
                    "Deck restored"
                );

                let status = if result.success {
                    "successfully"
                } else {
                    "with warnings"
                };
                Ok(CallToolResult::text(format!(
                    "Restored {} {}",
                    result.path.display(),
                    status
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// List backup files in a directory.
pub fn list_backups(state: Arc<AnkiState>) -> Tool {
    ToolBuilder::new("list_backups")
        .description(
            "List backup files in a directory. Returns .apkg files sorted by date (newest first).",
        )
        .read_only()
        .handler_with_state(
            state,
            |state: Arc<AnkiState>, params: ListBackupsParams| async move {
                debug!(backup_dir = %params.backup_dir, "Listing backups");

                let backups = state
                    .engine
                    .backup()
                    .list_backups(&params.backup_dir)
                    .await
                    .map_err(|e| tower_mcp::Error::tool(e.to_string()))?;

                info!(count = backups.len(), "Listed backups");

                if backups.is_empty() {
                    return Ok(CallToolResult::text("No backup files found"));
                }

                let backup_list: Vec<String> = backups
                    .iter()
                    .map(|b| format!("{} ({} bytes)", b.path.display(), b.size_bytes))
                    .collect();

                Ok(CallToolResult::text(format!(
                    "Found {} backup(s):\n{}",
                    backups.len(),
                    backup_list.join("\n")
                )))
            },
        )
        .build()
        .expect("valid tool")
}
