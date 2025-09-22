// src-tauri/src/ai/anthropic.rs

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout_seconds: u64,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            timeout_seconds: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicContent {
    pub text: String,
    #[serde(rename = "type")]
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicContent>,
}

pub struct AnthropicClient {
    client: Client,
    config: AnthropicConfig,
}

impl AnthropicClient {
    pub fn new(config: AnthropicConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(anyhow!("Anthropic API key is required"));
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
        messages: Vec<AnthropicMessage>,
        system: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let url = format!("{}/messages", self.config.base_url);

        let request = AnthropicRequest {
            model: model.to_string(),
            max_tokens: max_tokens.unwrap_or(4096),
            messages,
            temperature,
            system,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
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
                "Anthropic request failed: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        if anthropic_response.content.is_empty() {
            return Err(anyhow!("No content in Anthropic response"));
        }

        Ok(anthropic_response.content[0].text.clone())
    }

    pub async fn simple_chat(&self, model: &str, message: &str) -> Result<String> {
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: message.to_string(),
        }];

        self.chat(model, messages, None, None, None).await
    }

    #[allow(dead_code)]
    pub async fn system_chat(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
        temperature: Option<f32>,
    ) -> Result<String> {
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
        }];

        self.chat(
            model,
            messages,
            Some(system_prompt.to_string()),
            temperature,
            None,
        )
        .await
    }
}
