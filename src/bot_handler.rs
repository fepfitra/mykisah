use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use qrcode::render::unicode;
use qrcode::QrCode;

use wacore::types::events::Event;
use wacore::types::message::MessageInfo;
use waproto::whatsapp;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust::Client;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

use crate::openrouter_api::{ChatMessage, MessageRole};
use crate::openrouter_client::OpenRouterClient;
use tracing::{error, info, warn};

pub struct WhatsAppBot {
    bot: Bot,
    #[allow(dead_code)]
    openrouter_client: Arc<OpenRouterClient>,
}

impl WhatsAppBot {
    pub async fn new(db_path: &str, kisah_path: Option<PathBuf>) -> Result<Self> {
        let backend = Arc::new(
            SqliteStore::new(db_path)
                .await
                .context("Failed to create SQLite store")?,
        );

        let openrouter_api_key = env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY environment variable not set")?;
        let openrouter_model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "nvidia/nemotron-nano-12b-v2-vl:free".to_string());

        let openrouter_client_for_bot_struct = Arc::new(
            OpenRouterClient::new(openrouter_api_key, openrouter_model, kisah_path).await?,
        );

        let openrouter_client_for_closure = Arc::clone(&openrouter_client_for_bot_struct);

        let bot = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, client| {
                let openrouter_client_cloned_for_async_block =
                    Arc::clone(&openrouter_client_for_closure);
                async move {
                    match event {
                        Event::PairingQrCode { code, .. } => {
                            Self::handle_pairing_qr_code(code);
                        }
                        Event::Connected(_) => {
                            info!("✓ Successfully connected to WhatsApp!");
                        }
                        Event::Message(msg, info) => {
                            Self::handle_message(
                                client,
                                *msg,
                                info,
                                openrouter_client_cloned_for_async_block,
                            )
                            .await;
                        }
                        _ => {}
                    }
                }
            })
            .build()
            .await
            .context("Failed to build bot")?;

        Ok(Self {
            bot,
            openrouter_client: openrouter_client_for_bot_struct,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Connecting to WhatsApp...");
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
                error!("Failed to generate QR code: {}", e);
                error!("Raw code: {}", code);
            }
        }
    }

    async fn handle_message(
        client: Arc<Client>,
        msg: whatsapp::Message,
        info: MessageInfo,
        openrouter_client: Arc<OpenRouterClient>,
    ) {
        info!("Message from {}: {:?}", info.source.sender, msg);

        let message_text: Option<String> = if let Some(conversation_text) = &msg.conversation {
            Some(conversation_text.clone())
        } else if let Some(extended_text_message) = &msg.extended_text_message {
            extended_text_message.text.clone()
        } else {
            None
        };

        if let Some(text) = message_text {
            if text.starts_with("!") {
                let cmd_str = &text[1..];
                let current_dir = openrouter_client.get_kisah_path().unwrap().clone();

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(cmd_str)
                    .current_dir(&current_dir)
                    .output();

                let response_text = match output {
                    Ok(output) => {
                        if output.status.success() {
                            String::from_utf8_lossy(&output.stdout).to_string()
                        } else {
                            format!("Error: {}", String::from_utf8_lossy(&output.stderr))
                        }
                    },
                    Err(e) => format!("Failed to execute command: {}", e),
                };

                if let Err(e) = client
                    .send_message(
                        info.source.chat.clone(),
                        whatsapp::Message {
                            conversation: Some(response_text),
                            ..Default::default()
                        },
                    )
                    .await
                {
                    error!("Failed to send shell command response: {}", e);
                }
                return;
            }
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
                    error!("Failed to send pong: {}", e);
                }
            } else {
                let chat_messages = vec![ChatMessage {
                    role: MessageRole::User,
                    content: text.clone(),
                }];

                match openrouter_client.get_chat_completion(chat_messages).await {
                    Ok(response) => {
                        if let Some(choice) = response.choices.first() {
                            let ai_response = choice.message.content.clone();
                            if let Err(e) = client
                                .send_message(
                                    info.source.chat.clone(),
                                    whatsapp::Message {
                                        conversation: Some(ai_response),
                                        ..Default::default()
                                    },
                                )
                                .await
                            {
                                error!("Failed to send AI response: {}", e);
                            }
                        } else {
                            warn!(
                                "OpenRouter returned no choices for message from {}.",
                                info.source.sender
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to get OpenRouter completion for message from {}: {}",
                            info.source.sender, e
                        );
                    }
                }
            }
        }
    }
}
