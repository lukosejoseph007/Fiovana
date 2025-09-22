// src-tauri/src/ai/ollama.rs

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_seconds: 120,
            retry_attempts: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub digest: String,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
    base_url: Url,
}

impl OllamaClient {
    pub async fn new(config: OllamaConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        let base_url =
            Url::parse(&config.base_url).map_err(|e| anyhow!("Invalid Ollama base URL: {}", e))?;

        let ollama_client = Self {
            client,
            config,
            base_url,
        };

        // Test connection
        if !ollama_client.is_available().await {
            tracing::warn!(
                "Ollama server not available at {}",
                ollama_client.config.base_url
            );
        }

        Ok(ollama_client)
    }

    pub async fn is_available(&self) -> bool {
        let url = self.base_url.join("/api/tags").unwrap();

        match self.client.get(url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = self.base_url.join("/api/tags")?;

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list models: HTTP {}", response.status()));
        }

        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        options: Option<ChatOptions>,
    ) -> Result<ChatResponse> {
        let url = self.base_url.join("/api/chat")?;

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
            options,
        };

        let mut attempts = 0;
        let max_attempts = self.config.retry_attempts;

        while attempts < max_attempts {
            match self.send_chat_request(&url, &request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow!("Failed after {} attempts: {}", max_attempts, e));
                    }
                    tracing::warn!(
                        "Attempt {}/{} failed: {}. Retrying...",
                        attempts,
                        max_attempts,
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
                }
            }
        }

        unreachable!()
    }

    async fn send_chat_request(&self, url: &Url, request: &ChatRequest) -> Result<ChatResponse> {
        let response = self.client.post(url.clone()).json(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Chat request failed: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }

    #[allow(dead_code)]
    pub async fn simple_chat(&self, model: &str, message: &str) -> Result<String> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
        }];

        let response = self.chat(model, messages, None).await?;
        Ok(response.message.content)
    }

    pub async fn system_chat(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
        temperature: Option<f32>,
    ) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ];

        let options = temperature.map(|temp| ChatOptions {
            temperature: Some(temp),
            num_predict: None,
            top_p: None,
            top_k: None,
        });

        let response = self.chat(model, messages, options).await?;
        Ok(response.message.content)
    }

    #[allow(dead_code)]
    pub async fn check_model_availability(&self, model: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.contains(&model.to_string()))
    }

    pub async fn pull_model(&self, model: &str) -> Result<()> {
        let url = self.base_url.join("/api/pull")?;

        #[derive(Serialize)]
        struct PullRequest {
            name: String,
        }

        let request = PullRequest {
            name: model.to_string(),
        };

        let response = self.client.post(url).json(&request).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to pull model {}: HTTP {}",
                model,
                response.status()
            ));
        }

        tracing::info!("Successfully pulled model: {}", model);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_config_default() {
        let config = OllamaConfig::default();
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.timeout_seconds, 120);
        assert_eq!(config.retry_attempts, 3);
    }

    #[tokio::test]
    async fn test_chat_message_serialization() {
        let message = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }

    #[tokio::test]
    async fn test_ollama_client_creation() {
        let config = OllamaConfig::default();
        let result = OllamaClient::new(config).await;

        // Should succeed even if Ollama is not running
        assert!(result.is_ok());
    }
}
