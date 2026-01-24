//! Miscellaneous AnkiConnect actions.
//!
//! This module provides access to general-purpose actions like version checking,
//! syncing, and profile management.

use serde::{Deserialize, Serialize};

use crate::client::AnkiClient;
use crate::error::Result;

/// Provides access to miscellaneous AnkiConnect operations.
///
/// Obtained via [`AnkiClient::misc()`].
#[derive(Debug)]
pub struct MiscActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

#[derive(Serialize)]
struct LoadProfileParams<'a> {
    name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportPackageParams<'a> {
    deck: &'a str,
    path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_sched_data: Option<bool>,
}

#[derive(Serialize)]
struct ImportPackageParams<'a> {
    path: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiReflectParams<'a> {
    scopes: &'a [&'a str],
    actions: Option<&'a [&'a str]>,
}

#[derive(Serialize)]
struct MultiParams<'a> {
    actions: &'a [MultiAction<'a>],
}

/// A single action for the multi endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct MultiAction<'a> {
    /// The action name.
    pub action: &'a str,
    /// Optional parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl<'a> MultiAction<'a> {
    /// Create a new action without parameters.
    pub fn new(action: &'a str) -> Self {
        Self {
            action,
            params: None,
        }
    }

    /// Create a new action with parameters.
    pub fn with_params(action: &'a str, params: serde_json::Value) -> Self {
        Self {
            action,
            params: Some(params),
        }
    }
}

/// Result of requesting permission.
#[derive(Debug, Clone, Deserialize)]
pub struct PermissionResult {
    /// The permission status.
    pub permission: String,
    /// Whether permission was granted.
    #[serde(default)]
    pub require_api_key: bool,
    /// API version if granted.
    #[serde(default)]
    pub version: Option<u8>,
}

/// Result of API reflection.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiReflectResult {
    /// List of scopes.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// List of actions.
    #[serde(default)]
    pub actions: Vec<String>,
}

impl<'a> MiscActions<'a> {
    /// Get the AnkiConnect API version.
    ///
    /// This is useful for verifying that AnkiConnect is running and accessible.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use yanki::AnkiClient;
    ///
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let version = client.misc().version().await?;
    /// assert_eq!(version, 6);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn version(&self) -> Result<u8> {
        self.client.invoke_without_params("version").await
    }

    /// Request permission to use AnkiConnect.
    ///
    /// This will show a dialog in Anki asking the user to grant permission.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let result = client.misc().request_permission().await?;
    /// if result.permission == "granted" {
    ///     println!("Permission granted!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request_permission(&self) -> Result<PermissionResult> {
        self.client.invoke_without_params("requestPermission").await
    }

    /// Trigger a sync with AnkiWeb.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// client.misc().sync().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sync(&self) -> Result<()> {
        self.client.invoke_void("sync", ()).await
    }

    /// Get list of available profiles.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let profiles = client.misc().profiles().await?;
    /// for profile in profiles {
    ///     println!("Profile: {}", profile);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn profiles(&self) -> Result<Vec<String>> {
        self.client.invoke_without_params("getProfiles").await
    }

    /// Load a profile by name.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// client.misc().load_profile("User 1").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_profile(&self, name: &str) -> Result<bool> {
        self.client
            .invoke("loadProfile", LoadProfileParams { name })
            .await
    }

    /// Export a deck to an .apkg file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// client.misc().export_package("Default", "/tmp/deck.apkg", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_package(
        &self,
        deck: &str,
        path: &str,
        include_sched_data: Option<bool>,
    ) -> Result<bool> {
        self.client
            .invoke(
                "exportPackage",
                ExportPackageParams {
                    deck,
                    path,
                    include_sched_data,
                },
            )
            .await
    }

    /// Import an .apkg file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// client.misc().import_package("/tmp/deck.apkg").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn import_package(&self, path: &str) -> Result<bool> {
        self.client
            .invoke("importPackage", ImportPackageParams { path })
            .await
    }

    /// Reload the collection from disk.
    ///
    /// This is useful after making changes to the database externally.
    pub async fn reload_collection(&self) -> Result<()> {
        self.client.invoke_void("reloadCollection", ()).await
    }

    /// Query available AnkiConnect actions and their parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use yanki::AnkiClient;
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    /// let result = client.misc().api_reflect(&["actions"], None).await?;
    /// println!("Available actions: {:?}", result.actions);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn api_reflect(
        &self,
        scopes: &[&str],
        actions: Option<&[&str]>,
    ) -> Result<ApiReflectResult> {
        self.client
            .invoke("apiReflect", ApiReflectParams { scopes, actions })
            .await
    }

    /// Execute multiple actions in a single request.
    ///
    /// This is useful for batching multiple operations to reduce latency.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use yanki::{AnkiClient, actions::MultiAction};
    ///
    /// # async fn example() -> yanki::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let actions = vec![
    ///     MultiAction::new("deckNames"),
    ///     MultiAction::new("modelNames"),
    /// ];
    ///
    /// let results = client.misc().multi(&actions).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn multi(&self, actions: &[MultiAction<'_>]) -> Result<Vec<serde_json::Value>> {
        self.client.invoke("multi", MultiParams { actions }).await
    }
}
