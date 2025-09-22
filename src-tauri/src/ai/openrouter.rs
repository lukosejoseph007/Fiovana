// src-tauri/src/ai/openrouter.rs

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout_seconds: u64,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            timeout_seconds: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterChoice {
    pub message: OpenRouterMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterResponse {
    pub choices: Vec<OpenRouterChoice>,
}

pub struct OpenRouterClient {
    client: Client,
    config: OpenRouterConfig,
}

impl OpenRouterClient {
    pub fn new(config: OpenRouterConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(anyhow!("OpenRouter API key is required"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self { client, config })
    }

    pub async fn is_available(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<OpenRouterMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let request = OpenRouterRequest {
            model: model.to_string(),
            messages,
            temperature,
            max_tokens,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "OpenRouter request failed: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let openrouter_response: OpenRouterResponse = response.json().await?;

        if openrouter_response.choices.is_empty() {
            return Err(anyhow!("No response choices from OpenRouter"));
        }

        Ok(openrouter_response.choices[0].message.content.clone())
    }

    pub async fn simple_chat(&self, model: &str, message: &str) -> Result<String> {
        let messages = vec![OpenRouterMessage {
            role: "user".to_string(),
            content: message.to_string(),
        }];

        self.chat(model, messages, None, None).await
    }

    #[allow(dead_code)]
    pub async fn system_chat(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
        temperature: Option<f32>,
    ) -> Result<String> {
        let messages = vec![
            OpenRouterMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OpenRouterMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ];

        self.chat(model, messages, temperature, None).await
    }
}
