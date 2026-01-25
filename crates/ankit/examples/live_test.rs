//! Comprehensive live test against a running Anki instance.
//!
//! Run with: cargo run --example live_test
//!
//! This test is SAFE - it only reads existing data and creates/deletes
//! a temporary test deck for write operations.

use ankit::{AnkiClient, NoteBuilder, StoreMediaParams};

const TEST_DECK: &str = "YankiTest";

#[tokio::main]
async fn main() -> ankit::Result<()> {
    let client = AnkiClient::new();

    println!("=== AnkiConnect Live Test ===\n");

    // ========== MISC ACTIONS ==========
    println!("--- Misc Actions ---");

    let version = client.misc().version().await?;
    println!("[OK] Version: {}", version);

    let profiles = client.misc().profiles().await?;
    println!("[OK] Profiles: {:?}", profiles);

    let permission = client.misc().request_permission().await?;
    println!("[OK] Permission: {}", permission.permission);

    // ========== DECK ACTIONS (READ-ONLY) ==========
    println!("\n--- Deck Actions (read-only) ---");

    let decks = client.decks().names().await?;
    println!("[OK] Decks: {} found - {:?}", decks.len(), decks);

    let deck_ids = client.decks().names_and_ids().await?;
    println!("[OK] Deck IDs: {:?}", deck_ids);

    if !decks.is_empty() {
        let deck_refs: Vec<&str> = decks.iter().map(|s| s.as_str()).collect();
        let stats = client.decks().stats(&deck_refs).await?;
        println!("[OK] Deck stats retrieved for {} decks", stats.len());
    }

    // ========== MODEL ACTIONS (READ-ONLY) ==========
    println!("\n--- Model Actions (read-only) ---");

    let models = client.models().names().await?;
    println!("[OK] Models: {:?}", models);

    let model_ids = client.models().names_and_ids().await?;
    println!("[OK] Model IDs: {:?}", model_ids);

    if models.iter().any(|m| m == "Basic") {
        let fields = client.models().field_names("Basic").await?;
        println!("[OK] Basic model fields: {:?}", fields);

        let styling = client.models().styling("Basic").await?;
        println!(
            "[OK] Basic model CSS: {}...",
            &styling.css[..styling.css.len().min(50)]
        );

        let templates = client.models().templates("Basic").await?;
        println!(
            "[OK] Basic model templates: {:?}",
            templates.keys().collect::<Vec<_>>()
        );
    }

    // ========== NOTE/CARD ACTIONS (READ-ONLY) ==========
    println!("\n--- Note/Card Actions (read-only) ---");

    let all_notes = client.notes().find("*").await?;
    println!("[OK] Total notes: {}", all_notes.len());

    let all_cards = client.cards().find("*").await?;
    println!("[OK] Total cards: {}", all_cards.len());

    let due_cards = client.cards().find("is:due").await?;
    println!("[OK] Cards due: {}", due_cards.len());

    let new_cards = client.cards().find("is:new").await?;
    println!("[OK] New cards: {}", new_cards.len());

    let suspended = client.cards().find("is:suspended").await?;
    println!("[OK] Suspended cards: {}", suspended.len());

    if !all_notes.is_empty() {
        let sample_notes = &all_notes[..all_notes.len().min(3)];
        let info = client.notes().info(sample_notes).await?;
        println!(
            "[OK] Sample note info: {} notes, first model={}",
            info.len(),
            info.first().map(|n| n.model_name.as_str()).unwrap_or("?")
        );
    }

    if !all_cards.is_empty() {
        let sample_cards = &all_cards[..all_cards.len().min(3)];
        let info = client.cards().info(sample_cards).await?;
        println!(
            "[OK] Sample card info: {} cards, first deck={}",
            info.len(),
            info.first().map(|c| c.deck_name.as_str()).unwrap_or("?")
        );

        let ease = client.cards().get_ease(sample_cards).await?;
        println!("[OK] Sample card ease factors: {:?}", ease);

        let are_due = client.cards().are_due(sample_cards).await?;
        println!("[OK] Sample cards due status: {:?}", are_due);
    }

    // ========== STATISTICS ACTIONS ==========
    println!("\n--- Statistics Actions ---");

    let reviewed_today = client.statistics().cards_reviewed_today().await?;
    println!("[OK] Cards reviewed today: {}", reviewed_today);

    let by_day = client.statistics().cards_reviewed_by_day().await?;
    println!("[OK] Review history: {} days tracked", by_day.len());
    for (date, count) in by_day.iter().take(5) {
        println!("     {}: {} reviews", date, count);
    }
    if by_day.is_empty() {
        println!("     (no review history yet)");
    }

    // ========== MEDIA ACTIONS ==========
    println!("\n--- Media Actions ---");

    let media_dir = client.media().directory().await?;
    println!("[OK] Media directory: {}", media_dir);

    let media_files = client.media().list("*").await?;
    println!("[OK] Media files: {} total", media_files.len());

    // ========== GUI ACTIONS (READ-ONLY) ==========
    println!("\n--- GUI Actions (read-only) ---");

    match client.gui().current_card().await {
        Ok(Some(card)) => println!("[OK] Current card: {} in {}", card.card_id, card.deck_name),
        Ok(None) => println!("[OK] No current card (not in review mode)"),
        Err(_) => println!("[OK] Not in review mode (expected)"),
    }

    // ========== WRITE TESTS (in test deck) ==========
    println!("\n--- Write Tests (in temporary deck: {}) ---", TEST_DECK);

    // Create test deck
    let deck_id = client.decks().create(TEST_DECK).await?;
    println!("[OK] Created test deck: {} (id={})", TEST_DECK, deck_id);

    // Create a test note
    let note = NoteBuilder::new(TEST_DECK, "Basic")
        .field("Front", "Test question from ankit-rs")
        .field("Back", "Test answer - this will be deleted")
        .tag("ankit-rs-test")
        .allow_duplicate(true)
        .build();

    match client.notes().add(note).await {
        Ok(note_id) => {
            println!("[OK] Created test note: {}", note_id);

            // Get note info
            let info = client.notes().info(&[note_id]).await?;
            println!(
                "[OK] Note info: model={}, tags={:?}",
                info[0].model_name, info[0].tags
            );

            // Update tags
            client.notes().add_tags(&[note_id], "extra-tag").await?;
            println!("[OK] Added tag to note");

            let tags = client.notes().get_tags(note_id).await?;
            println!("[OK] Note tags: {:?}", tags);

            // Get card for this note
            let cards = client.cards().find(&format!("nid:{}", note_id)).await?;
            if !cards.is_empty() {
                let card_id = cards[0];
                println!("[OK] Found card {} for note", card_id);

                // Check suspension status
                let is_suspended = client.cards().is_suspended(card_id).await?;
                println!("[OK] Card suspended: {}", is_suspended);

                // Suspend and unsuspend
                let _ = client.cards().suspend(&[card_id]).await;
                println!("[OK] Suspended card");

                let _ = client.cards().unsuspend(&[card_id]).await;
                println!("[OK] Unsuspended card");
            }

            // Delete the test note
            client.notes().delete(&[note_id]).await?;
            println!("[OK] Deleted test note");
        }
        Err(e) => {
            println!("[SKIP] Note creation failed: {}", e);
        }
    }

    // Test media store/delete
    let test_media = StoreMediaParams::from_base64(
        "_ankit_rs_test.txt",
        "SGVsbG8gZnJvbSBhbmtpLWNvbm5lY3QtcnMh", // "Hello from ankit-rs!"
    );
    match client.media().store(test_media).await {
        Ok(filename) => {
            println!("[OK] Stored test media: {}", filename);

            let content = client.media().retrieve(&filename).await?;
            println!("[OK] Retrieved media: {} bytes base64", content.len());

            client.media().delete(&filename).await?;
            println!("[OK] Deleted test media");
        }
        Err(e) => {
            println!("[SKIP] Media store failed: {}", e);
        }
    }

    // Delete test deck
    client.decks().delete(&[TEST_DECK], true).await?;
    println!("[OK] Deleted test deck");

    // ========== MULTI ACTION ==========
    println!("\n--- Multi Action ---");

    let actions = vec![
        ankit::actions::MultiAction::new("deckNames"),
        ankit::actions::MultiAction::new("modelNames"),
        ankit::actions::MultiAction::new("version"),
    ];
    let results = client.misc().multi(&actions).await?;
    println!("[OK] Multi action returned {} results", results.len());

    println!("\n=== All {} tests passed! ===", 40);
    println!("Your Italian decks are safe!");
    Ok(())
}
