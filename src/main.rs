mod bot_handler;
mod openrouter_api;
mod openrouter_client;
mod tui_handler;

use anyhow::Result;
use bot_handler::WhatsAppBot;
use clap::Parser;
use once_cell::sync::Lazy;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use std::path::PathBuf;

static TRACING_SETUP: Lazy<()> = Lazy::new(|| {
    let filter_layer = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer())
        .with(console_subscriber::spawn());

    registry.init();

    info!("Tracing initialized.");
});

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    tui: bool,
    #[arg(long, value_name = "PATH", help = "Path to the kisah directory (e.g., ~/kisah)")]
    kisah_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    Lazy::force(&TRACING_SETUP);

    let cli = Cli::parse();

    if cli.tui {
        info!("Starting in TUI mode.");
        tui_handler::run_tui(cli.kisah_path).await?;
    } else {
        info!("Starting WhatsApp QR Code Pairing...");
        
        let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
        let kisah_dir = std::path::Path::new(&home_dir).join(".kisah");
        
        if !kisah_dir.exists() {
            std::fs::create_dir_all(&kisah_dir).expect("Failed to create .kisah directory");
            info!("Created directory: {:?}", kisah_dir);
        }

        let db_path = kisah_dir.join("whatsapp.db");
        let current_db_path = std::path::Path::new("whatsapp.db");

        if current_db_path.exists() && !db_path.exists() {
            info!("Migrating whatsapp.db to ~/.kisah/whatsapp.db...");
            std::fs::rename(current_db_path, &db_path).expect("Failed to move whatsapp.db");
            info!("Migration complete.");
        }

        let db_path_str = db_path.to_str().expect("Invalid database path");
        let bot = WhatsAppBot::new(db_path_str, cli.kisah_path).await?;
        bot.run().await?;
    }

    Ok(())
}
