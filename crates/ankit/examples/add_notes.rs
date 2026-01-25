//! Example: Adding notes to Anki.
//!
//! This example demonstrates various ways to add notes, including:
//! - Simple note creation with NoteBuilder
//! - Adding notes with tags
//! - Bulk adding multiple notes
//! - Adding notes with duplicate handling
//!
//! Run with: cargo run --example add_notes

use ankit::{AnkiClient, NoteBuilder};

#[tokio::main]
async fn main() -> ankit::Result<()> {
    let client = AnkiClient::new();

    // First, ensure we have a deck to work with
    let test_deck = "Example Deck";
    let _deck_id = client.decks().create(test_deck).await?;
    println!("Using deck: {}", test_deck);

    // ========== SIMPLE NOTE ==========
    println!("\n--- Adding a simple note ---");

    let note = NoteBuilder::new(test_deck, "Basic")
        .field("Front", "What is the capital of France?")
        .field("Back", "Paris")
        .build();

    match client.notes().add(note).await {
        Ok(id) => println!("Created note with ID: {}", id),
        Err(e) => println!("Note creation failed (might be duplicate): {}", e),
    }

    // ========== NOTE WITH TAGS ==========
    println!("\n--- Adding a note with tags ---");

    let note = NoteBuilder::new(test_deck, "Basic")
        .field("Front", "What is the capital of Japan?")
        .field("Back", "Tokyo")
        .tag("geography")
        .tag("capitals")
        .tags(["asia", "countries"])
        .build();

    match client.notes().add(note).await {
        Ok(id) => println!("Created tagged note with ID: {}", id),
        Err(e) => println!("Note creation failed: {}", e),
    }

    // ========== ALLOWING DUPLICATES ==========
    println!("\n--- Adding a note with duplicate allowed ---");

    let note = NoteBuilder::new(test_deck, "Basic")
        .field("Front", "What is 2 + 2?")
        .field("Back", "4")
        .allow_duplicate(true) // Allow this even if a duplicate exists
        .build();

    match client.notes().add(note).await {
        Ok(id) => println!("Created note (duplicates allowed) with ID: {}", id),
        Err(e) => println!("Note creation failed: {}", e),
    }

    // ========== BULK ADDING NOTES ==========
    println!("\n--- Adding multiple notes at once ---");

    let notes = vec![
        NoteBuilder::new(test_deck, "Basic")
            .field("Front", "What is the capital of Germany?")
            .field("Back", "Berlin")
            .tag("geography")
            .build(),
        NoteBuilder::new(test_deck, "Basic")
            .field("Front", "What is the capital of Italy?")
            .field("Back", "Rome")
            .tag("geography")
            .build(),
        NoteBuilder::new(test_deck, "Basic")
            .field("Front", "What is the capital of Spain?")
            .field("Back", "Madrid")
            .tag("geography")
            .build(),
    ];

    let results = client.notes().add_many(&notes).await?;
    let successful = results.iter().filter(|r| r.is_some()).count();
    let failed = results.iter().filter(|r| r.is_none()).count();
    println!(
        "Bulk add results: {} successful, {} failed (duplicates)",
        successful, failed
    );

    // ========== CHECK BEFORE ADDING ==========
    println!("\n--- Checking if notes can be added ---");

    let notes_to_check = vec![
        NoteBuilder::new(test_deck, "Basic")
            .field("Front", "New unique question")
            .field("Back", "New answer")
            .build(),
        NoteBuilder::new(test_deck, "Basic")
            .field("Front", "What is the capital of France?") // Likely duplicate
            .field("Back", "Paris")
            .build(),
    ];

    let can_add = client.notes().can_add(&notes_to_check).await?;
    for (i, result) in can_add.iter().enumerate() {
        println!(
            "  Note {}: {}",
            i + 1,
            if *result { "can add" } else { "duplicate" }
        );
    }

    // ========== DETAILED DUPLICATE CHECK ==========
    println!("\n--- Detailed duplicate check ---");

    let detailed = client.notes().can_add_detailed(&notes_to_check).await?;
    for (i, result) in detailed.iter().enumerate() {
        if result.can_add {
            println!("  Note {}: can add", i + 1);
        } else {
            println!(
                "  Note {}: {}",
                i + 1,
                result.error.as_deref().unwrap_or("unknown error")
            );
        }
    }

    println!("\nDone! Check Anki to see the new notes in '{}'", test_deck);
    Ok(())
}
