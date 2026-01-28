mod bot_handler;
mod openrouter_api;
mod openrouter_client;
mod tui_handler;

use anyhow::Result;
use bot_handler::WhatsAppBot;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    tui: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

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
