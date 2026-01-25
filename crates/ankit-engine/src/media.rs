//! Media audit and cleanup operations.
//!
//! This module provides workflows for auditing media files
//! and cleaning up orphaned or missing references.

use crate::Result;
use ankit::AnkiClient;
use serde::Serialize;
use std::collections::HashSet;

/// Result of a media audit.
#[derive(Debug, Clone, Default, Serialize)]
pub struct MediaAudit {
    /// Total number of media files.
    pub total_files: usize,
    /// Total size of media files in bytes.
    pub total_size_bytes: u64,
    /// Media files not referenced by any note.
    pub orphaned: Vec<String>,
    /// Media references in notes that don't exist.
    pub missing: Vec<MissingMedia>,
    /// Media files by type.
    pub by_type: MediaByType,
}

/// Missing media reference.
#[derive(Debug, Clone, Serialize)]
pub struct MissingMedia {
    /// The note ID referencing this media.
    pub note_id: i64,
    /// The missing filename.
    pub filename: String,
}

/// Media file counts by type.
#[derive(Debug, Clone, Default, Serialize)]
pub struct MediaByType {
    /// Number of image files.
    pub images: usize,
    /// Number of audio files.
    pub audio: usize,
    /// Number of video files.
    pub video: usize,
    /// Number of other files.
    pub other: usize,
}

/// Result of a cleanup operation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CleanupReport {
    /// Number of files deleted.
    pub files_deleted: usize,
    /// Bytes freed.
    pub bytes_freed: u64,
    /// Files that failed to delete.
    pub failed: Vec<String>,
}

/// Media workflow engine.
#[derive(Debug)]
pub struct MediaEngine<'a> {
    client: &'a AnkiClient,
}

impl<'a> MediaEngine<'a> {
    pub(crate) fn new(client: &'a AnkiClient) -> Self {
        Self { client }
    }

    /// Audit media files in the collection.
    ///
    /// Identifies orphaned files (not referenced by notes) and
    /// missing references (notes referencing non-existent files).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let audit = engine.media().audit().await?;
    /// println!("Found {} orphaned files", audit.orphaned.len());
    /// println!("Found {} missing references", audit.missing.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn audit(&self) -> Result<MediaAudit> {
        // Get all media files
        let all_files = self.client.media().list("*").await?;

        let mut audit = MediaAudit {
            total_files: all_files.len(),
            ..Default::default()
        };

        // Categorize by type
        for file in &all_files {
            let lower = file.to_lowercase();
            if lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
                || lower.ends_with(".png")
                || lower.ends_with(".gif")
                || lower.ends_with(".webp")
                || lower.ends_with(".svg")
            {
                audit.by_type.images += 1;
            } else if lower.ends_with(".mp3")
                || lower.ends_with(".wav")
                || lower.ends_with(".ogg")
                || lower.ends_with(".m4a")
                || lower.ends_with(".flac")
            {
                audit.by_type.audio += 1;
            } else if lower.ends_with(".mp4")
                || lower.ends_with(".webm")
                || lower.ends_with(".mkv")
                || lower.ends_with(".avi")
            {
                audit.by_type.video += 1;
            } else {
                audit.by_type.other += 1;
            }
        }

        // Get all notes and check for media references
        let all_notes = self.client.notes().find("*").await?;

        if all_notes.is_empty() {
            // No notes, all media is orphaned
            audit.orphaned = all_files;
            return Ok(audit);
        }

        // Get note info in batches
        let mut referenced_files: HashSet<String> = HashSet::new();
        let batch_size = 100;

        for chunk in all_notes.chunks(batch_size) {
            let infos = self.client.notes().info(chunk).await?;
            for info in infos {
                for field in info.fields.values() {
                    // Extract media references from field content
                    // Matches [sound:filename] and <img src="filename">
                    for filename in extract_media_references(&field.value) {
                        referenced_files.insert(filename);
                    }
                }
            }
        }

        // Find orphaned files
        let file_set: HashSet<_> = all_files.iter().cloned().collect();
        audit.orphaned = all_files
            .iter()
            .filter(|f| !referenced_files.contains(*f))
            .cloned()
            .collect();

        // Find missing references
        for filename in &referenced_files {
            if !file_set.contains(filename) {
                // Find which note references this
                // For now, just record the filename without the note ID
                audit.missing.push(MissingMedia {
                    note_id: 0, // Would need to track this during extraction
                    filename: filename.clone(),
                });
            }
        }

        Ok(audit)
    }

    /// Delete orphaned media files.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - If true, only report what would be deleted
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    ///
    /// // Preview what would be deleted
    /// let preview = engine.media().cleanup_orphaned(true).await?;
    /// println!("Would delete {} files", preview.files_deleted);
    ///
    /// // Actually delete
    /// let report = engine.media().cleanup_orphaned(false).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cleanup_orphaned(&self, dry_run: bool) -> Result<CleanupReport> {
        let audit = self.audit().await?;

        if dry_run || audit.orphaned.is_empty() {
            return Ok(CleanupReport {
                files_deleted: audit.orphaned.len(),
                ..Default::default()
            });
        }

        let mut report = CleanupReport::default();

        for filename in audit.orphaned {
            match self.client.media().delete(&filename).await {
                Ok(_) => report.files_deleted += 1,
                Err(_) => report.failed.push(filename),
            }
        }

        Ok(report)
    }

    /// List media files matching a pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Glob pattern (e.g., "*.mp3", "image_*")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit_engine::Engine;
    /// # async fn example() -> ankit_engine::Result<()> {
    /// let engine = Engine::new();
    /// let audio_files = engine.media().list("*.mp3").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, pattern: &str) -> Result<Vec<String>> {
        Ok(self.client.media().list(pattern).await?)
    }
}

/// Extract media filenames from HTML field content.
fn extract_media_references(html: &str) -> Vec<String> {
    let mut files = Vec::new();

    // Match [sound:filename]
    let sound_pattern = regex_lite::Regex::new(r"\[sound:([^\]]+)\]").unwrap();
    for cap in sound_pattern.captures_iter(html) {
        if let Some(m) = cap.get(1) {
            files.push(m.as_str().to_string());
        }
    }

    // Match <img src="filename">
    let img_pattern = regex_lite::Regex::new(r#"<img[^>]+src="([^"]+)"[^>]*>"#).unwrap();
    for cap in img_pattern.captures_iter(html) {
        if let Some(m) = cap.get(1) {
            let src = m.as_str();
            // Skip external URLs
            if !src.starts_with("http://") && !src.starts_with("https://") {
                files.push(src.to_string());
            }
        }
    }

    files
}
