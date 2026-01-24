//! Media-related AnkiConnect actions.
//!
//! This module provides operations for managing media files in Anki's media folder.
//!
//! # Example
//!
//! ```no_run
//! use yanki::{AnkiClient, StoreMediaParams};
//!
//! # async fn example() -> yanki::Result<()> {
//! let client = AnkiClient::new();
//!
//! // Store a file from URL
//! let params = StoreMediaParams::from_url(
//!     "audio.mp3",
//!     "https://example.com/audio.mp3"
//! );
//! let filename = client.media().store(params).await?;
//!
//! // List media files
//! let files = client.media().list("*.mp3").await?;
//! # Ok(())
//! # }
//! ```

use serde::Serialize;

use crate::client::AnkiClient;
use crate::error::Result;
use crate::types::StoreMediaParams;

/// Provides access to media-related AnkiConnect operations.
///
/// Obtained via [`AnkiClient::media()`].
#[derive(Debug)]
pub struct MediaActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

#[derive(Serialize)]
struct RetrieveParams<'a> {
    filename: &'a str,
}

#[derive(Serialize)]
struct ListParams<'a> {
    pattern: &'a str,
}

#[derive(Serialize)]
struct DeleteParams<'a> {
    filename: &'a str,
}

impl<'a> MediaActions<'a> {
    /// Store a media file.
    ///
    /// Returns the filename that was used (may differ from requested if a file
    /// with the same name already exists).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use yanki::{AnkiClient, StoreMediaParams};
    ///
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// // Store from base64
    /// let params = StoreMediaParams::from_base64("test.txt", "SGVsbG8gV29ybGQ=");
    /// let filename = client.media().store(params).await?;
    ///
    /// // Store from URL
    /// let params = StoreMediaParams::from_url("image.png", "https://example.com/image.png");
    /// let filename = client.media().store(params).await?;
    ///
    /// // Store from local path
    /// let params = StoreMediaParams::from_path("doc.pdf", "/path/to/file.pdf");
    /// let filename = client.media().store(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store(&self, params: StoreMediaParams) -> Result<String> {
        self.client.invoke("storeMediaFile", params).await
    }

    /// Retrieve a media file's contents as base64.
    ///
    /// Returns the base64-encoded file contents, or an error if the file
    /// doesn't exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let base64_data = client.media().retrieve("audio.mp3").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve(&self, filename: &str) -> Result<String> {
        self.client
            .invoke("retrieveMediaFile", RetrieveParams { filename })
            .await
    }

    /// List media files matching a pattern.
    ///
    /// The pattern uses glob syntax (e.g., `*.mp3`, `image_*`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// // List all MP3 files
    /// let mp3_files = client.media().list("*.mp3").await?;
    ///
    /// // List all files
    /// let all_files = client.media().list("*").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, pattern: &str) -> Result<Vec<String>> {
        self.client
            .invoke("getMediaFilesNames", ListParams { pattern })
            .await
    }

    /// Get the path to Anki's media directory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let path = client.media().directory().await?;
    /// println!("Media directory: {}", path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn directory(&self) -> Result<String> {
        self.client.invoke_without_params("getMediaDirPath").await
    }

    /// Delete a media file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// client.media().delete("old_audio.mp3").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, filename: &str) -> Result<()> {
        self.client
            .invoke_void("deleteMediaFile", DeleteParams { filename })
            .await
    }
}
