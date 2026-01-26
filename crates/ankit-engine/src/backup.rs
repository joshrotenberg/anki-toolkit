//! Backup and restore workflows for Anki decks.
//!
//! This module provides high-level operations for backing up and restoring
//! Anki decks to/from .apkg files.
//!
//! # Example
//!
//! ```no_run
//! use ankit_engine::Engine;
//!
//! # async fn example() -> ankit_engine::Result<()> {
//! let engine = Engine::new();
//!
//! // Backup a deck
//! let result = engine.backup()
//!     .backup_deck("Japanese", "/tmp/backups")
//!     .await?;
//! println!("Backed up to: {}", result.path.display());
//!
//! // Restore from backup
//! let result = engine.backup()
//!     .restore_deck(&result.path)
//!     .await?;
//! println!("Restored: {}", if result.success { "yes" } else { "no" });
//! # Ok(())
//! # }
//! ```

use crate::{Error, Result};
use ankit::AnkiClient;
use std::path::{Path, PathBuf};

/// Engine for backup and restore operations.
pub struct BackupEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> BackupEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Backup a deck to an .apkg file.
    ///
    /// Creates a backup file in the specified directory with a timestamped filename.
    /// The backup includes all notes, cards, scheduling data, and media.
    ///
    /// # Arguments
    ///
    /// * `deck` - Name of the deck to backup
    /// * `backup_dir` - Directory where the backup file will be created
    ///
    /// # Returns
    ///
    /// Returns [`BackupResult`] with the path to the created backup file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_engine::Engine;
    ///
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let result = engine.backup()
    ///     .backup_deck("Japanese::Vocabulary", "/home/user/anki-backups")
    ///     .await?;
    /// println!("Backup created: {}", result.path.display());
    /// println!("Size: {} bytes", result.size_bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn backup_deck(
        &self,
        deck: &str,
        backup_dir: impl AsRef<Path>,
    ) -> Result<BackupResult> {
        self.backup_deck_with_options(deck, backup_dir, BackupOptions::default())
            .await
    }

    /// Backup a deck with custom options.
    ///
    /// # Arguments
    ///
    /// * `deck` - Name of the deck to backup
    /// * `backup_dir` - Directory where the backup file will be created
    /// * `options` - Backup options (scheduling data, filename format)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_engine::Engine;
    /// use ankit_engine::backup::BackupOptions;
    ///
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let options = BackupOptions {
    ///     include_scheduling: false,  // Don't include review history
    ///     ..Default::default()
    /// };
    /// let result = engine.backup()
    ///     .backup_deck_with_options("Japanese", "/tmp/backups", options)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn backup_deck_with_options(
        &self,
        deck: &str,
        backup_dir: impl AsRef<Path>,
        options: BackupOptions,
    ) -> Result<BackupResult> {
        let backup_dir = backup_dir.as_ref();

        // Ensure backup directory exists
        if !backup_dir.exists() {
            std::fs::create_dir_all(backup_dir).map_err(|e| {
                Error::Backup(format!(
                    "Failed to create backup directory '{}': {}",
                    backup_dir.display(),
                    e
                ))
            })?;
        }

        // Generate filename with timestamp
        let timestamp = chrono_lite_timestamp();
        let safe_deck_name = sanitize_filename(deck);
        let filename = format!("{}-{}.apkg", safe_deck_name, timestamp);
        let backup_path = backup_dir.join(&filename);

        // Export the deck
        let path_str = backup_path.to_string_lossy();
        self.client
            .misc()
            .export_package(deck, &path_str, Some(options.include_scheduling))
            .await
            .map_err(|e| Error::Backup(format!("Failed to export deck '{}': {}", deck, e)))?;

        // Get file size
        let size_bytes = std::fs::metadata(&backup_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(BackupResult {
            path: backup_path,
            deck_name: deck.to_string(),
            size_bytes,
            include_scheduling: options.include_scheduling,
        })
    }

    /// Restore a deck from an .apkg backup file.
    ///
    /// Imports the backup file into Anki. If the deck already exists,
    /// Anki's default duplicate handling will apply.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the .apkg file to restore
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_engine::Engine;
    ///
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let result = engine.backup()
    ///     .restore_deck("/home/user/backups/Japanese-2024-01-15.apkg")
    ///     .await?;
    /// if result.success {
    ///     println!("Restore completed successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restore_deck(&self, backup_path: impl AsRef<Path>) -> Result<RestoreResult> {
        let backup_path = backup_path.as_ref();

        if !backup_path.exists() {
            return Err(Error::Backup(format!(
                "Backup file not found: {}",
                backup_path.display()
            )));
        }

        let path_str = backup_path.to_string_lossy();
        let success = self
            .client
            .misc()
            .import_package(&path_str)
            .await
            .map_err(|e| Error::Backup(format!("Failed to import backup: {}", e)))?;

        Ok(RestoreResult {
            path: backup_path.to_path_buf(),
            success,
        })
    }

    /// Backup all decks to separate .apkg files.
    ///
    /// Creates individual backup files for each deck in the collection.
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory where backup files will be created
    ///
    /// # Returns
    ///
    /// Returns a [`CollectionBackupResult`] with results for each deck.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_engine::Engine;
    ///
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let result = engine.backup()
    ///     .backup_collection("/home/user/anki-backups")
    ///     .await?;
    /// println!("Backed up {} decks", result.successful.len());
    /// if !result.failed.is_empty() {
    ///     println!("Failed: {:?}", result.failed);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn backup_collection(
        &self,
        backup_dir: impl AsRef<Path>,
    ) -> Result<CollectionBackupResult> {
        let backup_dir = backup_dir.as_ref();

        // Create a timestamped subdirectory for this backup
        let timestamp = chrono_lite_timestamp();
        let collection_dir = backup_dir.join(format!("collection-{}", timestamp));
        std::fs::create_dir_all(&collection_dir).map_err(|e| {
            Error::Backup(format!(
                "Failed to create backup directory '{}': {}",
                collection_dir.display(),
                e
            ))
        })?;

        // Get all deck names
        let decks = self
            .client
            .decks()
            .names()
            .await
            .map_err(|e| Error::Backup(format!("Failed to list decks: {}", e)))?;

        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for deck in decks {
            // Skip the Default deck if it's empty (common case)
            match self.backup_deck(&deck, &collection_dir).await {
                Ok(result) => successful.push(result),
                Err(e) => failed.push((deck, e.to_string())),
            }
        }

        Ok(CollectionBackupResult {
            backup_dir: collection_dir,
            successful,
            failed,
        })
    }

    /// List backup files in a directory.
    ///
    /// Scans the directory for .apkg files and returns information about each.
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory to scan for backup files
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_engine::Engine;
    ///
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let backups = engine.backup()
    ///     .list_backups("/home/user/anki-backups")
    ///     .await?;
    /// for backup in backups {
    ///     println!("{}: {} bytes", backup.path.display(), backup.size_bytes);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_backups(&self, backup_dir: impl AsRef<Path>) -> Result<Vec<BackupInfo>> {
        let backup_dir = backup_dir.as_ref();

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        // Recursively find all .apkg files
        collect_apkg_files(backup_dir, &mut backups)?;

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.modified.cmp(&a.modified));

        Ok(backups)
    }

    /// Delete old backups, keeping the most recent N.
    ///
    /// Useful for implementing backup rotation.
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory containing backup files
    /// * `keep` - Number of most recent backups to keep
    ///
    /// # Returns
    ///
    /// Returns the paths of deleted backup files.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit_engine::Engine;
    ///
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// // Keep only the 5 most recent backups
    /// let deleted = engine.backup()
    ///     .rotate_backups("/home/user/anki-backups", 5)
    ///     .await?;
    /// println!("Deleted {} old backups", deleted.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rotate_backups(
        &self,
        backup_dir: impl AsRef<Path>,
        keep: usize,
    ) -> Result<Vec<PathBuf>> {
        let backups = self.list_backups(&backup_dir).await?;

        if backups.len() <= keep {
            return Ok(Vec::new());
        }

        let mut deleted = Vec::new();
        for backup in backups.into_iter().skip(keep) {
            if std::fs::remove_file(&backup.path).is_ok() {
                deleted.push(backup.path);
            }
        }

        Ok(deleted)
    }
}

/// Options for backup operations.
#[derive(Debug, Clone)]
pub struct BackupOptions {
    /// Include scheduling data (review history, due dates).
    /// Default: true
    pub include_scheduling: bool,
}

impl Default for BackupOptions {
    fn default() -> Self {
        Self {
            include_scheduling: true,
        }
    }
}

/// Result of a deck backup operation.
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// Path to the created backup file.
    pub path: PathBuf,
    /// Name of the backed up deck.
    pub deck_name: String,
    /// Size of the backup file in bytes.
    pub size_bytes: u64,
    /// Whether scheduling data was included.
    pub include_scheduling: bool,
}

/// Result of a restore operation.
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Path to the restored backup file.
    pub path: PathBuf,
    /// Whether the restore was successful.
    pub success: bool,
}

/// Result of a collection backup operation.
#[derive(Debug, Clone)]
pub struct CollectionBackupResult {
    /// Directory containing the backup files.
    pub backup_dir: PathBuf,
    /// Successfully backed up decks.
    pub successful: Vec<BackupResult>,
    /// Decks that failed to backup (deck name, error message).
    pub failed: Vec<(String, String)>,
}

/// Information about a backup file.
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Path to the backup file.
    pub path: PathBuf,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Last modification time (Unix timestamp).
    pub modified: u64,
}

/// Generate a simple timestamp without external dependencies.
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    // Convert to date components (simplified, doesn't handle leap seconds)
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Calculate year, month, day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to year, month, day.
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Simplified calculation - accurate enough for backup timestamps
    let mut remaining_days = days as i64;
    let mut year = 1970i64;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1i64;
    for days_in_month in days_in_months.iter() {
        if remaining_days < *days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;

    (year as u64, month as u64, day as u64)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Sanitize a deck name for use as a filename.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Recursively collect .apkg files from a directory.
fn collect_apkg_files(dir: &Path, results: &mut Vec<BackupInfo>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        Error::Backup(format!(
            "Failed to read directory '{}': {}",
            dir.display(),
            e
        ))
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_apkg_files(&path, results)?;
        } else if path.extension().map(|e| e == "apkg").unwrap_or(false) {
            if let Ok(metadata) = std::fs::metadata(&path) {
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                results.push(BackupInfo {
                    path,
                    size_bytes: metadata.len(),
                    modified,
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Japanese"), "Japanese");
        assert_eq!(sanitize_filename("Japanese::Vocab"), "Japanese__Vocab");
        assert_eq!(sanitize_filename("Test/Deck"), "Test_Deck");
        assert_eq!(sanitize_filename("A:B*C?D"), "A_B_C_D");
    }

    #[test]
    fn test_chrono_lite_timestamp() {
        let ts = chrono_lite_timestamp();
        // Should be 15 characters: YYYYMMDD-HHMMSS
        assert_eq!(ts.len(), 15);
        assert!(ts.chars().nth(8) == Some('-'));
    }

    #[test]
    fn test_days_to_ymd() {
        // 1970-01-01
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        // 2000-01-01 (10957 days from epoch)
        assert_eq!(days_to_ymd(10957), (2000, 1, 1));
        // 2024-01-01 (19723 days from epoch)
        assert_eq!(days_to_ymd(19723), (2024, 1, 1));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(!is_leap_year(1970));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2024));
    }
}
