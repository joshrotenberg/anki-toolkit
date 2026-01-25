//! Deck-related AnkiConnect actions.
//!
//! This module provides operations for managing Anki decks, including
//! creating, deleting, and configuring decks.
//!
//! # Example
//!
//! ```no_run
//! use ankit::AnkiClient;
//!
//! # async fn example() -> ankit::Result<()> {
//! let client = AnkiClient::new();
//!
//! // List all decks
//! let decks = client.decks().names().await?;
//! println!("Decks: {:?}", decks);
//!
//! // Create a new deck
//! let deck_id = client.decks().create("My New Deck").await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use serde::Serialize;

use crate::client::AnkiClient;
use crate::error::Result;
use crate::types::{DeckConfig, DeckStats};

/// Provides access to deck-related AnkiConnect operations.
///
/// Obtained via [`AnkiClient::decks()`].
#[derive(Debug)]
pub struct DeckActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

// Parameter structs for actions that need them
#[derive(Serialize)]
struct CreateDeckParams<'a> {
    deck: &'a str,
}

#[derive(Serialize)]
struct GetDecksParams<'a> {
    cards: &'a [i64],
}

#[derive(Serialize)]
struct ChangeDeckParams<'a> {
    cards: &'a [i64],
    deck: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteDecksParams<'a> {
    decks: &'a [&'a str],
    cards_too: bool,
}

#[derive(Serialize)]
struct GetDeckConfigParams<'a> {
    deck: &'a str,
}

#[derive(Serialize)]
struct SaveDeckConfigParams<'a> {
    config: &'a DeckConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetDeckConfigIdParams<'a> {
    decks: &'a [&'a str],
    config_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CloneDeckConfigParams<'a> {
    name: &'a str,
    clone_from: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoveDeckConfigParams {
    config_id: i64,
}

#[derive(Serialize)]
struct GetDeckStatsParams<'a> {
    decks: &'a [&'a str],
}

impl<'a> DeckActions<'a> {
    /// Get all deck names.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let names = client.decks().names().await?;
    /// for name in names {
    ///     println!("{}", name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn names(&self) -> Result<Vec<String>> {
        self.client.invoke_without_params("deckNames").await
    }

    /// Get all deck names with their IDs.
    ///
    /// Returns a map from deck name to deck ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let decks = client.decks().names_and_ids().await?;
    /// for (name, id) in decks {
    ///     println!("{}: {}", name, id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn names_and_ids(&self) -> Result<HashMap<String, i64>> {
        self.client.invoke_without_params("deckNamesAndIds").await
    }

    /// Get the decks that contain the given cards.
    ///
    /// Returns a map from deck name to the list of card IDs in that deck.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let card_ids = vec![1502298033753, 1502298033754];
    /// let decks = client.decks().get_for_cards(&card_ids).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_for_cards(&self, cards: &[i64]) -> Result<HashMap<String, Vec<i64>>> {
        self.client
            .invoke("getDecks", GetDecksParams { cards })
            .await
    }

    /// Create a new deck.
    ///
    /// Returns the ID of the created deck. If a deck with the same name
    /// already exists, returns the ID of the existing deck.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let deck_id = client.decks().create("Japanese::Vocabulary").await?;
    /// println!("Created deck with ID: {}", deck_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, name: &str) -> Result<i64> {
        self.client
            .invoke("createDeck", CreateDeckParams { deck: name })
            .await
    }

    /// Move cards to a different deck.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let card_ids = vec![1502298033753];
    /// client.decks().move_cards(&card_ids, "New Deck").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn move_cards(&self, cards: &[i64], deck: &str) -> Result<()> {
        self.client
            .invoke_void("changeDeck", ChangeDeckParams { cards, deck })
            .await
    }

    /// Delete decks and optionally their cards.
    ///
    /// # Arguments
    ///
    /// * `decks` - Names of decks to delete
    /// * `cards_too` - If true, also delete all cards in the decks.
    ///   If false, cards are moved to the Default deck.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// // Delete deck and all its cards
    /// client.decks().delete(&["Old Deck"], true).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, decks: &[&str], cards_too: bool) -> Result<()> {
        self.client
            .invoke_void("deleteDecks", DeleteDecksParams { decks, cards_too })
            .await
    }

    /// Get the configuration for a deck.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let config = client.decks().config("Default").await?;
    /// println!("New cards per day: {}", config.new.per_day);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn config(&self, deck: &str) -> Result<DeckConfig> {
        self.client
            .invoke("getDeckConfig", GetDeckConfigParams { deck })
            .await
    }

    /// Save a deck configuration.
    ///
    /// Returns true if successful.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let mut config = client.decks().config("Default").await?;
    /// config.new.per_day = 50;
    /// client.decks().save_config(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_config(&self, config: &DeckConfig) -> Result<bool> {
        self.client
            .invoke("saveDeckConfig", SaveDeckConfigParams { config })
            .await
    }

    /// Assign a configuration to multiple decks.
    ///
    /// Returns true if successful.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let decks = ["Japanese", "Korean"];
    /// client.decks().set_config_id(&decks, 1).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_config_id(&self, decks: &[&str], config_id: i64) -> Result<bool> {
        self.client
            .invoke(
                "setDeckConfigId",
                SetDeckConfigIdParams { decks, config_id },
            )
            .await
    }

    /// Clone a deck configuration.
    ///
    /// Creates a new configuration by cloning an existing one.
    /// Returns the ID of the new configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let new_config_id = client.decks().clone_config("My Config", 1).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clone_config(&self, name: &str, clone_from: i64) -> Result<i64> {
        self.client
            .invoke(
                "cloneDeckConfigId",
                CloneDeckConfigParams { name, clone_from },
            )
            .await
    }

    /// Remove a deck configuration.
    ///
    /// Returns true if successful. Cannot remove the default configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.decks().remove_config(1234567890).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_config(&self, config_id: i64) -> Result<bool> {
        self.client
            .invoke("removeDeckConfigId", RemoveDeckConfigParams { config_id })
            .await
    }

    /// Get statistics for multiple decks.
    ///
    /// Returns a map from deck ID (as string) to deck statistics.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let stats = client.decks().stats(&["Default", "Japanese"]).await?;
    /// for (id, stat) in stats {
    ///     println!("{}: {} cards due", stat.name, stat.review_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stats(&self, decks: &[&str]) -> Result<HashMap<String, DeckStats>> {
        self.client
            .invoke("getDeckStats", GetDeckStatsParams { decks })
            .await
    }
}
