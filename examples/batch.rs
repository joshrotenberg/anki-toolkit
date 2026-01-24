//! Example: Batch operations with the multi endpoint.
//!
//! This example demonstrates how to execute multiple actions in a single
//! request using the `multi` endpoint, which can significantly reduce
//! latency when you need to perform many operations.
//!
//! Run with: cargo run --example batch

use yanki::{AnkiClient, actions::MultiAction};

#[tokio::main]
async fn main() -> yanki::Result<()> {
    let client = AnkiClient::new();

    println!("=== Batch Operations Example ===\n");

    // ========== SIMPLE BATCH: MULTIPLE QUERIES ==========
    println!("--- Executing multiple actions at once ---");

    let actions = vec![
        MultiAction::new("deckNames"),
        MultiAction::new("modelNames"),
        MultiAction::new("getProfiles"),
        MultiAction::new("version"),
    ];

    let results = client.misc().multi(&actions).await?;

    println!("Results from batch operation:");
    println!("  Decks: {}", results[0]);
    println!("  Models: {}", results[1]);
    println!("  Profiles: {}", results[2]);
    println!("  Version: {}", results[3]);

    // ========== BATCH WITH PARAMETERS ==========
    println!("\n--- Batch with parameters ---");

    let actions = vec![
        // Find notes in Default deck
        MultiAction::with_params("findNotes", serde_json::json!({"query": "deck:Default"})),
        // Find due cards
        MultiAction::with_params("findCards", serde_json::json!({"query": "is:due"})),
        // Get deck stats
        MultiAction::with_params("getDeckStats", serde_json::json!({"decks": ["Default"]})),
    ];

    let results = client.misc().multi(&actions).await?;

    let note_count = results[0].as_array().map(|a| a.len()).unwrap_or(0);
    let card_count = results[1].as_array().map(|a| a.len()).unwrap_or(0);

    println!("Notes in Default deck: {}", note_count);
    println!("Due cards: {}", card_count);
    println!("Deck stats: {}", results[2]);

    // ========== USE CASE: DASHBOARD DATA ==========
    println!("\n--- Dashboard data in one request ---");

    let actions = vec![
        MultiAction::new("getNumCardsReviewedToday"),
        MultiAction::with_params("findCards", serde_json::json!({"query": "is:due"})),
        MultiAction::with_params("findCards", serde_json::json!({"query": "is:new"})),
        MultiAction::with_params("findCards", serde_json::json!({"query": "is:suspended"})),
        MultiAction::new("deckNamesAndIds"),
    ];

    let results = client.misc().multi(&actions).await?;

    println!("Dashboard summary:");
    println!("  Reviewed today: {}", results[0].as_i64().unwrap_or(0));
    println!(
        "  Due cards: {}",
        results[1].as_array().map(|a| a.len()).unwrap_or(0)
    );
    println!(
        "  New cards: {}",
        results[2].as_array().map(|a| a.len()).unwrap_or(0)
    );
    println!(
        "  Suspended cards: {}",
        results[3].as_array().map(|a| a.len()).unwrap_or(0)
    );
    println!(
        "  Total decks: {}",
        results[4].as_object().map(|o| o.len()).unwrap_or(0)
    );

    println!("\nDone!");
    Ok(())
}
