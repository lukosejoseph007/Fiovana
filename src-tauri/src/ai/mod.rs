// src-tauri/src/ai/mod.rs

pub mod intent;
pub mod ollama;
pub mod response;

use anyhow::Result;
use serde::{Deserialize, Serialize};

// Re-export key types
pub use intent::IntentClassifier;
pub use ollama::{OllamaClient, OllamaConfig};
pub use response::{AIResponse, ResponseGenerator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub ollama: OllamaConfig,
    pub default_model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            ollama: OllamaConfig::default(),
            default_model: "llama3.2-3b".to_string(),
            temperature: 0.7,
            max_tokens: Some(4096),
        }
    }
}

pub struct AIOrchestrator {
    ollama_client: OllamaClient,
    intent_classifier: IntentClassifier,
    response_generator: ResponseGenerator,
    config: AIConfig,
}

impl AIOrchestrator {
    pub async fn new(config: AIConfig) -> Result<Self> {
        let ollama_client = OllamaClient::new(config.ollama.clone()).await?;
        let intent_classifier = IntentClassifier::new();
        let response_generator = ResponseGenerator::new();

        Ok(Self {
            ollama_client,
            intent_classifier,
            response_generator,
            config,
        })
    }

    pub async fn process_conversation(
        &self,
        input: &str,
        context: Option<&str>,
    ) -> Result<AIResponse> {
        // Step 1: Classify intent
        let intent = self.intent_classifier.classify(input).await?;

        // Step 2: Generate response based on intent
        let response = self
            .response_generator
            .generate(&self.ollama_client, input, &intent, context, &self.config)
            .await?;

        Ok(response)
    }

    pub async fn is_available(&self) -> bool {
        self.ollama_client.is_available().await
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        self.ollama_client.list_models().await
    }
}

pub fn init() {
    tracing::info!("AI module initialized");
}
