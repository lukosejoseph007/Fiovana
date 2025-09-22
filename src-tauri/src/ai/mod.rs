// src-tauri/src/ai/mod.rs

pub mod anthropic;
pub mod intent;
pub mod ollama;
pub mod openrouter;
pub mod response;

use anyhow::Result;
use serde::{Deserialize, Serialize};

// Re-export key types
pub use anthropic::{AnthropicClient, AnthropicConfig};
pub use intent::IntentClassifier;
pub use ollama::{OllamaClient, OllamaConfig};
pub use openrouter::{OpenRouterClient, OpenRouterConfig};
pub use response::{AIResponse, ResponseGenerator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String, // "local", "openrouter", "anthropic"
    pub ollama: OllamaConfig,
    pub openrouter_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub default_model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            ollama: OllamaConfig::default(),
            openrouter_api_key: None,
            anthropic_api_key: None,
            default_model: "llama3.2-3b".to_string(),
            temperature: 0.7,
            max_tokens: Some(4096),
        }
    }
}

pub struct AIOrchestrator {
    ollama_client: Option<OllamaClient>,
    openrouter_client: Option<OpenRouterClient>,
    anthropic_client: Option<AnthropicClient>,
    intent_classifier: IntentClassifier,
    #[allow(dead_code)]
    response_generator: ResponseGenerator,
    config: AIConfig,
    #[allow(dead_code)]
    vector_search_enabled: bool,
}

impl AIOrchestrator {
    pub async fn new(config: AIConfig) -> Result<Self> {
        let intent_classifier = IntentClassifier::new();
        let response_generator = ResponseGenerator::new();

        // Initialize clients based on provider
        let mut ollama_client = None;
        let mut openrouter_client = None;
        let mut anthropic_client = None;

        match config.provider.as_str() {
            "local" => {
                ollama_client = match OllamaClient::new(config.ollama.clone()).await {
                    Ok(client) => Some(client),
                    Err(e) => {
                        tracing::warn!("Failed to initialize Ollama client: {}", e);
                        None
                    }
                };
            }
            "openrouter" => {
                if let Some(api_key) = &config.openrouter_api_key {
                    let openrouter_config = OpenRouterConfig {
                        api_key: api_key.clone(),
                        ..Default::default()
                    };
                    openrouter_client = match OpenRouterClient::new(openrouter_config) {
                        Ok(client) => Some(client),
                        Err(e) => {
                            tracing::warn!("Failed to initialize OpenRouter client: {}", e);
                            None
                        }
                    };
                }
            }
            "anthropic" => {
                if let Some(api_key) = &config.anthropic_api_key {
                    let anthropic_config = AnthropicConfig {
                        api_key: api_key.clone(),
                        ..Default::default()
                    };
                    anthropic_client = match AnthropicClient::new(anthropic_config) {
                        Ok(client) => Some(client),
                        Err(e) => {
                            tracing::warn!("Failed to initialize Anthropic client: {}", e);
                            None
                        }
                    };
                }
            }
            _ => {
                tracing::warn!("Unknown AI provider: {}", config.provider);
            }
        }

        Ok(Self {
            ollama_client,
            openrouter_client,
            anthropic_client,
            intent_classifier,
            response_generator,
            config,
            vector_search_enabled: true, // Enable vector search for document-aware AI
        })
    }

    pub async fn process_conversation(
        &self,
        input: &str,
        context: Option<&str>,
    ) -> Result<AIResponse> {
        // Step 1: Classify intent
        let intent = self.intent_classifier.classify(input).await?;

        // Step 2: Prepare input with context if available
        let enhanced_input = if let Some(ctx) = context {
            if ctx.starts_with("No relevant documents")
                || ctx.starts_with("Vector search unavailable")
            {
                // Use original input when no document context is available
                input.to_string()
            } else {
                // Include document context in the prompt
                format!("{}\n\nUser Question: {}", ctx, input)
            }
        } else {
            input.to_string()
        };

        // Step 3: Generate response based on provider
        let response_content = match self.config.provider.as_str() {
            "local" => {
                if let Some(client) = &self.ollama_client {
                    client
                        .simple_chat(&self.config.default_model, &enhanced_input)
                        .await?
                } else {
                    return Err(anyhow::anyhow!("Ollama client not available"));
                }
            }
            "openrouter" => {
                if let Some(client) = &self.openrouter_client {
                    client
                        .simple_chat(&self.config.default_model, &enhanced_input)
                        .await?
                } else {
                    return Err(anyhow::anyhow!("OpenRouter client not available"));
                }
            }
            "anthropic" => {
                if let Some(client) = &self.anthropic_client {
                    client
                        .simple_chat(&self.config.default_model, &enhanced_input)
                        .await?
                } else {
                    return Err(anyhow::anyhow!("Anthropic client not available"));
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown provider: {}",
                    self.config.provider
                ));
            }
        };

        // Create AI response
        let response = AIResponse {
            response_type: response::ResponseType::Information,
            content: response_content,
            intent: intent.intent.clone(),
            confidence: 0.95, // Default confidence
            suggested_actions: vec![],
            metadata: response::ResponseMetadata {
                processing_time_ms: 0,
                model_used: self.config.default_model.clone(),
                tokens_used: None,
                confidence_explanation: "Direct provider response".to_string(),
            },
        };

        Ok(response)
    }

    pub async fn is_available(&self) -> bool {
        match self.config.provider.as_str() {
            "local" => {
                if let Some(client) = &self.ollama_client {
                    client.is_available().await
                } else {
                    false
                }
            }
            "openrouter" => {
                if let Some(client) = &self.openrouter_client {
                    client.is_available().await
                } else {
                    false
                }
            }
            "anthropic" => {
                if let Some(client) = &self.anthropic_client {
                    client.is_available().await
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        match self.config.provider.as_str() {
            "local" => {
                if let Some(client) = &self.ollama_client {
                    client.list_models().await
                } else {
                    Ok(vec![])
                }
            }
            "openrouter" => {
                // Return common OpenRouter models
                Ok(vec![
                    "deepseek/deepseek-chat-v3-0324:free".to_string(),
                    "openai/gpt-4o-mini".to_string(),
                    "anthropic/claude-3-haiku".to_string(),
                    "meta-llama/llama-3.1-8b-instruct:free".to_string(),
                ])
            }
            "anthropic" => {
                // Return Anthropic models
                Ok(vec![
                    "claude-3-haiku-20240307".to_string(),
                    "claude-3-sonnet-20240229".to_string(),
                    "claude-3-opus-20240229".to_string(),
                ])
            }
            _ => Ok(vec![]),
        }
    }
}

pub fn init() {
    tracing::info!("AI module initialized");
}
