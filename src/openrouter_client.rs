use crate::openrouter_api::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, MessageRole};
use anyhow::{anyhow, Result};
use reqwest::Client as ReqwestClient;
use tracing::{error, warn};
use std::path::PathBuf;
use tokio::fs;

pub struct OpenRouterClient {
    client: ReqwestClient,
    api_key: String,
    model: String,
    kisah_path: Option<PathBuf>,
    kisah_context: Vec<ChatMessage>,
}

impl OpenRouterClient {
    pub async fn new(api_key: String, model: String, kisah_path: Option<PathBuf>) -> Result<Self> {
        let client = ReqwestClient::new();
        let mut new_self = OpenRouterClient {
            client,
            api_key,
            model,
            kisah_path,
            kisah_context: Vec::new(),
        };
        new_self.kisah_context = new_self.load_kisah_context_internal().await?;
        Ok(new_self)
    }

    async fn load_kisah_context_internal(&self) -> Result<Vec<ChatMessage>> {
        let mut context_messages = Vec::new();

        if let Some(kisah_dir) = &self.kisah_path {
            let files_to_load = vec![
                "SOUL.md",
                "IDENTITY.md",
                "BOOTSTRAP.md",
                "AGENTS.md",
                "USER.md",
            ];

            for file_name in files_to_load {
                let file_path = kisah_dir.join(file_name);
                if file_path.exists() {
                    match fs::read_to_string(&file_path).await {
                        Ok(content) => {
                            context_messages.push(ChatMessage {
                                role: MessageRole::System,
                                content,
                            });
                        }
                        Err(e) => {
                            warn!("Failed to read kisah file {}: {}", file_path.display(), e);
                        }
                    }
                } else {
                    warn!("Kisah file not found: {}", file_path.display());
                }
            }
        }

        Ok(context_messages)
    }

    pub async fn get_chat_completion(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatCompletionResponse> {
        let mut messages_with_context = self.kisah_context.clone();
        messages_with_context.extend(messages);

        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: messages_with_context,
            temperature: None,
            max_tokens: None,
        };

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/fepfitra/mykisah")
            .header("X-Title", "MyKisah")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to OpenRouter: {}", e))?;

        if response.status().is_success() {
            response
                .json::<ChatCompletionResponse>()
                .await
                .map_err(|e| anyhow!("Failed to parse OpenRouter response: {}", e))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!(
                "OpenRouter API returned an error: Status: {}, Response: {}",
                status,
                text
            );
            Err(anyhow!(
                "OpenRouter API returned an error: Status: {}, Response: {}",
                status,
                text
            ))
        }
    }
}
