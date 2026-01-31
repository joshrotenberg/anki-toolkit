//! MCP server for Anki deck management via AnkiConnect.
//!
//! This server exposes ankit-engine workflows and key raw API operations
//! as tools for LLM assistants like Claude.

mod state;
mod tools;

use std::sync::Arc;

use clap::Parser;
use tower_mcp::{HttpTransport, McpRouter, StdioTransport};
use tracing::info;

use crate::state::AnkiState;
use crate::tools::all_tools;

// ============================================================================
// CLI Arguments
// ============================================================================

/// MCP server for Anki deck management via AnkiConnect.
#[derive(Parser, Debug)]
#[command(name = "ankit-mcp")]
#[command(version, about, long_about = None)]
struct Args {
    /// AnkiConnect host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// AnkiConnect port
    #[arg(long, default_value_t = 8765)]
    port: u16,

    /// Read-only mode (disables write operations)
    #[arg(long, default_value_t = false)]
    read_only: bool,

    /// Enable verbose logging (use multiple times for more verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Transport mode: stdio (default) or http
    #[arg(long, default_value = "stdio")]
    transport: Transport,

    /// HTTP server port (only used with --transport http)
    #[arg(long, default_value_t = 3000)]
    http_port: u16,

    /// HTTP server bind address (only used with --transport http)
    #[arg(long, default_value = "127.0.0.1")]
    http_host: String,
}

/// Transport mode for the MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Transport {
    /// Standard I/O transport (default, for CLI integration)
    #[default]
    Stdio,
    /// HTTP transport with SSE (for remote connections)
    Http,
}

impl std::str::FromStr for Transport {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stdio" => Ok(Transport::Stdio),
            "http" => Ok(Transport::Http),
            _ => Err(format!("Invalid transport: {}. Use 'stdio' or 'http'", s)),
        }
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), tower_mcp::BoxError> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = match args.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .init();

    let url = format!("http://{}:{}", args.host, args.port);
    info!(
        anki_url = %url,
        read_only = args.read_only,
        transport = ?args.transport,
        "Starting ankit-mcp server"
    );

    // Create shared state
    let state = Arc::new(AnkiState::new(&url, args.read_only));

    // Build instructions text
    let mode = if args.read_only { " (read-only)" } else { "" };
    let instructions = format!(
        "Anki deck management via AnkiConnect{}. \
         Requires Anki to be running with the AnkiConnect add-on installed.\n\n\
         IMPORTANT - DATA SAFETY:\n\
         - ALWAYS recommend backing up before bulk operations (use backup_deck or backup_collection)\n\
         - For destructive operations (delete, reset, remove_duplicates), confirm with user first\n\
         - Offer to preview changes before applying them (preview_deduplicate, plan_sync_toml)\n\
         - When in doubt, use read operations first to show what would be affected\n\n\
         Key tools: add_note, find_notes, backup_deck, backup_collection, list_decks, \
         study_summary, find_problems, import_notes, remove_duplicates, and more.",
        mode
    );

    // Build router with all tools
    let tools = all_tools(state);
    let router = McpRouter::new()
        .server_info("ankit-mcp", env!("CARGO_PKG_VERSION"))
        .instructions(instructions)
        .tools(tools);

    // Run on the appropriate transport
    match args.transport {
        Transport::Stdio => {
            StdioTransport::new(router).run().await?;
        }
        Transport::Http => {
            let bind_addr = format!("{}:{}", args.http_host, args.http_port);
            info!(bind_addr = %bind_addr, "Starting HTTP transport");

            HttpTransport::new(router)
                .disable_origin_validation()
                .serve(&bind_addr)
                .await?;
        }
    }

    Ok(())
}
