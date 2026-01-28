mod bot_handler;

use bot_handler::WhatsAppBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting WhatsApp QR Code Pairing...\n");

    let bot = WhatsAppBot::new("whatsapp.db").await?;
    bot.run().await?;

    Ok(())
}