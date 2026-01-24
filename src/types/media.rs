//! Media-related types.

use serde::Serialize;

/// Data source for storing media files.
#[derive(Debug, Clone)]
pub enum MediaData {
    /// Base64-encoded file content.
    Base64(String),
    /// URL to download the file from.
    Url(String),
    /// Local file path to read from.
    Path(String),
}

/// Parameters for storing a media file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreMediaParams {
    /// Filename to save as.
    pub filename: String,
    /// Base64-encoded file data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// URL to download from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Local file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Delete existing file with same name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_existing: Option<bool>,
}

impl StoreMediaParams {
    /// Create params for storing base64-encoded data.
    pub fn from_base64(filename: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            data: Some(data.into()),
            url: None,
            path: None,
            delete_existing: None,
        }
    }

    /// Create params for storing from a URL.
    pub fn from_url(filename: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            data: None,
            url: Some(url.into()),
            path: None,
            delete_existing: None,
        }
    }

    /// Create params for storing from a local file path.
    pub fn from_path(filename: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            data: None,
            url: None,
            path: Some(path.into()),
            delete_existing: None,
        }
    }

    /// Set whether to delete existing file with same name.
    pub fn delete_existing(mut self, delete: bool) -> Self {
        self.delete_existing = Some(delete);
        self
    }
}
