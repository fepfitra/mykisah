use crate::openrouter_api::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage};
use anyhow::{anyhow, Result};
use reqwest::Client as ReqwestClient;
use tracing::error;

pub struct OpenRouterClient {
    client: ReqwestClient,
    api_key: String,
    model: String,
}

impl OpenRouterClient {
    pub fn new(api_key: String, model: String) -> Self {
        let client = ReqwestClient::new();
        OpenRouterClient {
            client,
            api_key,
            model,
        }
    }

    pub async fn get_chat_completion(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatCompletionResponse> {
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
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
