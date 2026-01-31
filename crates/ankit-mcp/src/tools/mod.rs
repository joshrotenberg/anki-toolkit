//! Tool definitions for the Anki MCP server.
//!
//! Each submodule contains tools for a specific domain (notes, cards, decks, etc.).
//! Tools are created using tower-mcp's `ToolBuilder` API.

pub mod analyze;
pub mod backup;
pub mod cards;
pub mod decks;
pub mod deduplicate;
pub mod enrich;
pub mod export;
pub mod import;
pub mod media;
pub mod misc;
pub mod models;
pub mod notes;
pub mod organize;
pub mod progress;
pub mod tags;
pub mod toml;

use std::sync::Arc;

use tower_mcp::Tool;

use crate::state::AnkiState;

/// Create all tools for the Anki MCP server.
pub fn all_tools(state: Arc<AnkiState>) -> Vec<Tool> {
    vec![
        // Misc tools
        misc::version(state.clone()),
        misc::sync(state.clone()),
        // Model tools
        models::list_models(state.clone()),
        models::get_model_fields(state.clone()),
        // Deck tools
        decks::list_decks(state.clone()),
        decks::create_deck(state.clone()),
        decks::delete_deck(state.clone()),
        decks::clone_deck(state.clone()),
        decks::merge_decks(state.clone()),
        // Note tools
        notes::add_note(state.clone()),
        notes::find_notes(state.clone()),
        notes::get_notes_info(state.clone()),
        notes::update_note(state.clone()),
        notes::delete_notes(state.clone()),
        // Card tools
        cards::find_cards(state.clone()),
        cards::get_cards_info(state.clone()),
        cards::suspend_cards(state.clone()),
        cards::unsuspend_cards(state.clone()),
        cards::forget_cards(state.clone()),
        cards::set_ease(state.clone()),
        cards::set_due_date(state.clone()),
        // Tag tools
        tags::add_tags(state.clone()),
        tags::remove_tags(state.clone()),
        tags::replace_tags_all(state.clone()),
        tags::clear_unused_tags(state.clone()),
        // Import tools
        import::import_notes(state.clone()),
        import::validate_notes(state.clone()),
        // Export tools
        export::export_deck(state.clone()),
        export::export_reviews(state.clone()),
        // Organize tools
        organize::move_by_tag(state.clone()),
        // Analyze tools
        analyze::study_summary(state.clone()),
        analyze::find_problems(state.clone()),
        analyze::retention_stats(state.clone()),
        // Media tools
        media::audit_media(state.clone()),
        media::cleanup_media(state.clone()),
        // Backup tools
        backup::backup_deck(state.clone()),
        backup::backup_collection(state.clone()),
        backup::restore_deck(state.clone()),
        backup::list_backups(state.clone()),
        // Progress tools
        progress::reset_deck_progress(state.clone()),
        progress::tag_by_performance(state.clone()),
        progress::suspend_by_criteria(state.clone()),
        progress::deck_health_report(state.clone()),
        progress::bulk_tag_operation(state.clone()),
        // Enrich tools
        enrich::find_enrich_candidates(state.clone()),
        enrich::enrich_note(state.clone()),
        enrich::enrich_notes(state.clone()),
        // Deduplicate tools
        deduplicate::find_duplicates(state.clone()),
        deduplicate::preview_deduplicate(state.clone()),
        deduplicate::remove_duplicates(state.clone()),
        // TOML tools
        toml::export_deck_toml(state.clone()),
        toml::diff_deck_toml(state.clone()),
        toml::plan_sync_toml(state.clone()),
        toml::sync_deck_toml(state.clone()),
        toml::import_deck_toml(state),
    ]
}
