//! Demonstrates the main workflow operations provided by ankit-engine.
//!
//! Run with: `cargo run --example workflow_demo --all-features`
//!
//! Prerequisites:
//! - Anki running with AnkiConnect installed
//! - At least one deck with some cards

use ankit_engine::analyze::ProblemCriteria;
use ankit_engine::{Engine, NoteBuilder};

#[tokio::main]
async fn main() -> ankit_engine::Result<()> {
    let engine = Engine::new();

    // Check connection first
    match engine.client().misc().version().await {
        Ok(v) => println!("Connected to AnkiConnect v{}", v),
        Err(ankit::Error::ConnectionRefused) => {
            eprintln!("Could not connect to Anki. Is it running with AnkiConnect?");
            std::process::exit(1);
        }
        Err(e) => return Err(e.into()),
    }

    // List available decks
    let decks = engine.client().decks().names().await?;
    println!("\nAvailable decks:");
    for deck in &decks {
        println!("  - {}", deck);
    }

    // Get the first deck for demo purposes
    let deck_name = decks.first().map(|s| s.as_str()).unwrap_or("Default");
    println!("\nUsing deck: {}", deck_name);

    // Study Summary (analyze module)
    println!("\n--- Study Summary (last 7 days) ---");
    let summary = engine.analyze().study_summary(deck_name, 7).await?;
    println!("  Total reviews: {}", summary.total_reviews);
    println!("  Average per day: {:.1}", summary.avg_reviews_per_day);

    // Retention Stats (analyze module)
    println!("\n--- Retention Stats ---");
    let retention = engine.analyze().retention_stats(deck_name).await?;
    println!("  Total cards: {}", retention.total_cards);
    println!("  Average ease: {:.0}%", retention.avg_ease as f64 / 10.0);

    // Find Problem Cards (analyze module)
    println!("\n--- Problem Cards (leeches) ---");
    let criteria = ProblemCriteria {
        min_lapses: 3,
        ..Default::default()
    };
    let problems = engine
        .analyze()
        .find_problems(&format!("deck:\"{}\"", deck_name), criteria)
        .await?;
    if problems.is_empty() {
        println!("  No problem cards found (great!)");
    } else {
        println!("  Found {} cards with 3+ lapses", problems.len());
    }

    // Deck Health Report (progress module)
    println!("\n--- Deck Health Report ---");
    let health = engine.progress().deck_health(deck_name).await?;
    println!("  New cards: {}", health.new_cards);
    println!("  Learning: {}", health.learning_cards);
    println!("  Review: {}", health.review_cards);
    println!("  Suspended: {}", health.suspended_cards);
    if health.leech_count > 0 {
        println!("  Leeches: {}", health.leech_count);
    }

    // Media Audit (media module)
    println!("\n--- Media Audit ---");
    let media_report = engine.media().audit().await?;
    println!("  Orphaned files: {}", media_report.orphaned.len());
    println!("  Missing references: {}", media_report.missing.len());

    // Import Example (import module)
    println!("\n--- Import Example ---");
    let notes = vec![
        NoteBuilder::new(deck_name, "Basic")
            .field("Front", "Demo question")
            .field("Back", "Demo answer")
            .tag("demo")
            .build(),
    ];

    // Check if we can add before actually adding
    let can_add = engine.client().notes().can_add(&notes).await?;
    if can_add.first().copied().unwrap_or(false) {
        println!("  Note can be added (but skipping for demo)");
    } else {
        println!("  Note already exists or model not found");
    }

    println!("\nDemo complete!");
    Ok(())
}
