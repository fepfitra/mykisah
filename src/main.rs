mod bot_handler;
mod tui_handler;

use bot_handler::WhatsAppBot;
use anyhow::Result;
use clap::Parser; // Add this import

/// A WhatsApp bot and AI chat application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Run the application in TUI mode
    #[arg(long)]
    tui: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse(); // Use clap to parse arguments

    if cli.tui {
        println!("Starting in TUI mode.");
        tui_handler::run_tui().await?;
    } else {
        println!("Starting WhatsApp QR Code Pairing...\n");
        let bot = WhatsAppBot::new("whatsapp.db").await?;
        bot.run().await?;
    }

    Ok(())
}

