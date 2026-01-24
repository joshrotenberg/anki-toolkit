//! Example: Searching and querying Anki.
//!
//! This example demonstrates how to search for notes and cards using
//! Anki's query syntax.
//!
//! Run with: cargo run --example search

use yanki::AnkiClient;

#[tokio::main]
async fn main() -> yanki::Result<()> {
    let client = AnkiClient::new();

    println!("=== Anki Search Examples ===\n");

    // ========== FIND ALL NOTES IN A DECK ==========
    println!("--- Finding notes by deck ---");
    let note_ids = client.notes().find("deck:Default").await?;
    println!("Found {} notes in 'Default' deck", note_ids.len());

    // ========== FIND NOTES BY TAG ==========
    println!("\n--- Finding notes by tag ---");
    let tagged = client.notes().find("tag:marked").await?;
    println!("Found {} notes with 'marked' tag", tagged.len());

    // ========== FIND CARDS BY STATUS ==========
    println!("\n--- Finding cards by status ---");

    let due = client.cards().find("is:due").await?;
    println!("Cards due for review: {}", due.len());

    let new = client.cards().find("is:new").await?;
    println!("New cards: {}", new.len());

    let suspended = client.cards().find("is:suspended").await?;
    println!("Suspended cards: {}", suspended.len());

    // ========== COMPOUND QUERIES ==========
    println!("\n--- Compound queries ---");

    // Due cards in Default deck that are not suspended
    let query = "deck:Default is:due -is:suspended";
    let results = client.cards().find(query).await?;
    println!("Due cards in Default (not suspended): {}", results.len());

    // Notes added today
    let today = client.notes().find("added:1").await?;
    println!("Notes added today: {}", today.len());

    // Notes rated today
    let rated = client.cards().find("rated:1").await?;
    println!("Cards rated today: {}", rated.len());

    // ========== GET DETAILED INFO ==========
    println!("\n--- Getting detailed information ---");

    if !due.is_empty() {
        // Get info for up to 5 due cards
        let card_ids: Vec<i64> = due.into_iter().take(5).collect();
        let cards = client.cards().info(&card_ids).await?;

        println!("Sample of due cards:");
        for card in &cards {
            println!(
                "  - Card {} in '{}': {} reviews, {} lapses",
                card.card_id, card.deck_name, card.reps, card.lapses
            );
        }
    }

    if !note_ids.is_empty() {
        // Get info for up to 5 notes
        let ids: Vec<i64> = note_ids.into_iter().take(5).collect();
        let notes = client.notes().info(&ids).await?;

        println!("\nSample notes:");
        for note in &notes {
            let front = note
                .fields
                .get("Front")
                .map(|f| f.value.as_str())
                .unwrap_or("");
            let preview = if front.len() > 50 {
                format!("{}...", &front[..50])
            } else {
                front.to_string()
            };
            println!(
                "  - Note {} ({}): {}",
                note.note_id, note.model_name, preview
            );
        }
    }

    // ========== SEARCH BY FIELD CONTENT ==========
    println!("\n--- Field content search ---");

    // Search for specific text in the Front field
    let query = "Front:*capital*";
    let results = client.notes().find(query).await?;
    println!("Notes with 'capital' in Front field: {}", results.len());

    // ========== INTERVAL-BASED SEARCH ==========
    println!("\n--- Interval-based search ---");

    let mature = client.cards().find("prop:ivl>=21").await?;
    println!("Mature cards (interval >= 21 days): {}", mature.len());

    let young = client.cards().find("prop:ivl<21 -is:new").await?;
    println!("Young cards (interval < 21 days): {}", young.len());

    println!("\nDone!");
    Ok(())
}
