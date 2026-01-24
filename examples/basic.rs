//! Basic usage example for yanki.
//!
//! This example demonstrates how to connect to AnkiConnect and perform
//! common operations like listing decks and checking the API version.
//!
//! Run with: cargo run --example basic

use yanki::AnkiClient;

#[tokio::main]
async fn main() -> yanki::Result<()> {
    // Create a client with default settings (localhost:8765)
    let client = AnkiClient::new();

    // Verify AnkiConnect is running
    let version = client.misc().version().await?;
    println!("Connected to AnkiConnect version {}", version);

    // List all decks
    let decks = client.decks().names().await?;
    println!("\nAvailable decks:");
    for deck in &decks {
        println!("  - {}", deck);
    }

    // Get deck names with their IDs
    let deck_ids = client.decks().names_and_ids().await?;
    println!("\nDeck IDs:");
    for (name, id) in &deck_ids {
        println!("  {} (ID: {})", name, id);
    }

    // List all note types (models)
    let models = client.models().names().await?;
    println!("\nAvailable note types:");
    for model in &models {
        println!("  - {}", model);
    }

    // Get field names for the Basic model
    if models.contains(&"Basic".to_string()) {
        let fields = client.models().field_names("Basic").await?;
        println!("\nFields in 'Basic' note type: {:?}", fields);
    }

    // Get some statistics
    let reviewed_today = client.statistics().cards_reviewed_today().await?;
    println!("\nCards reviewed today: {}", reviewed_today);

    println!("\nDone!");
    Ok(())
}
