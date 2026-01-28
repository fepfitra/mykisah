use qrcode::render::unicode;
use qrcode::QrCode;
use std::default::Default;
use std::sync::Arc;
use wacore::types::events::Event;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

#[tokio::main]
async fn main() {
    println!("Starting WhatsApp QR Code Pairing...\n");

    let backend = Arc::new(
        SqliteStore::new("whatsapp.db")
            .await
            .expect("Failed to create SQLite store"),
    );

    let mut bot = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(|event, _client| async move {
            match event {
                Event::PairingQrCode { code, .. } => match QrCode::new(&code) {
                    Ok(qr) => {
                        let qr_string = qr
                            .render::<unicode::Dense1x2>()
                            .dark_color(unicode::Dense1x2::Light)
                            .light_color(unicode::Dense1x2::Dark)
                            .build();

                        println!("\n╔═══════════════════════════════════════════╗");
                        println!("║     Scan this QR code with WhatsApp       ║");
                        println!("╚═══════════════════════════════════════════╝\n");
                        println!("{}", qr_string);
                        println!("\nOpen WhatsApp → Settings → Linked Devices → Link a Device\n");
                    }
                    Err(e) => {
                        println!("Failed to generate QR code: {}", e);
                        println!("Raw code: {}", code);
                    }
                },
                Event::Connected(_) => {
                    println!("✓ Successfully connected to WhatsApp!");
                }
                Event::Message(msg, info) => {
                    println!("Message from {}: {:?}", info.source.sender, msg);

                    let message_text = if let Some(conversation_text) = &msg.conversation {
                        Some(conversation_text.clone())
                    } else if let Some(extended_text_message) = &msg.extended_text_message {
                        extended_text_message.text.clone()
                    } else {
                        None
                    };

                    if let Some(text) = message_text {
                        if text.to_lowercase().trim() == "ping" {
                            if let Err(e) = _client
                                .send_message(
                                    info.source.chat.clone(),
                                    waproto::whatsapp::Message {
                                        conversation: Some("pong".to_string()),
                                        ..Default::default()
                                    },
                                )
                                .await
                            {
                                println!("Failed to send pong: {}", e);
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .build()
        .await
        .expect("Failed to build bot");

    println!("Connecting to WhatsApp...\n");

    bot.run()
        .await
        .expect("Failed to start bot")
        .await
        .expect("Bot error");
}
