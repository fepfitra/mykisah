use anyhow::{Result, Context};
use qrcode::render::unicode;
use qrcode::QrCode;
use std::default::Default;
use std::sync::Arc;
use wacore::types::events::Event;
use wacore::types::message::MessageInfo;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use whatsapp_rust::Client;
use waproto::whatsapp;

pub struct WhatsAppBot {
    bot: Bot,
}

impl WhatsAppBot {
    pub async fn new(db_path: &str) -> Result<Self> {
        let backend = Arc::new(
            SqliteStore::new(db_path)
                .await
                .context("Failed to create SQLite store")?,
        );

        let bot = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(|event, client| async move {
                match event {
                    Event::PairingQrCode { code, .. } => {
                        Self::handle_pairing_qr_code(code);
                    }
                    Event::Connected(_) => {
                        println!("✓ Successfully connected to WhatsApp!");
                    }
                    Event::Message(msg, info) => {
                        Self::handle_message(client, *msg, info).await;
                    }
                    _ => {}
                }
            })
            .build()
            .await
            .context("Failed to build bot")?;

        Ok(Self { bot })
    }

    pub async fn run(mut self) -> Result<()> {
        println!("Connecting to WhatsApp...\n");
        self.bot
            .run()
            .await
            .context("Failed to start bot")?
            .await
            .context("Bot error")?;
        Ok(())
    }

    fn handle_pairing_qr_code(code: String) {
        match QrCode::new(&code) {
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
        }
    }

    async fn handle_message(
        client: Arc<Client>,
        msg: whatsapp::Message,
        info: MessageInfo,
    ) {
        println!("Message from {}: {:?}", info.source.sender, msg);

        let message_text: Option<String> = if let Some(conversation_text) = &msg.conversation {
            Some(conversation_text.clone())
        } else if let Some(extended_text_message) = &msg.extended_text_message {
            extended_text_message.text.clone()
        } else {
            None
        };

        if let Some(text) = message_text {
            if text.to_lowercase().trim() == "ping" {
                if let Err(e) = client
                    .send_message(
                        info.source.chat.clone(),
                        whatsapp::Message {
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
}