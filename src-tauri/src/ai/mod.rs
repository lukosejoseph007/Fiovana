// src-tauri/src/ai/mod.rs

pub mod actions;
pub mod anthropic;
pub mod context;
pub mod conversation_context;
pub mod document_commands;
pub mod intent;
pub mod ollama;
pub mod openrouter;
pub mod prompts;
pub mod response;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Re-export key types
pub use anthropic::{AnthropicClient, AnthropicConfig};
pub use context::{DocumentContextManager, DocumentRef, UserPreferences};
pub use conversation_context::ConversationContextManager;
#[allow(unused_imports)]
pub use conversation_context::{EnrichedConversationContext, TaskStatus};
pub use intent::IntentClassifier;
pub use ollama::{OllamaClient, OllamaConfig};
pub use openrouter::{OpenRouterClient, OpenRouterConfig};
#[allow(unused_imports)]
pub use prompts::{PromptContext, PromptTemplates};
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
    context_manager: Arc<Mutex<DocumentContextManager>>,
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
            context_manager: Arc::new(Mutex::new(DocumentContextManager::new())),
            config,
            vector_search_enabled: true, // Enable vector search for document-aware AI
        })
    }

    pub async fn process_conversation(
        &self,
        input: &str,
        context: Option<&str>,
    ) -> Result<AIResponse> {
        self.process_conversation_with_session(input, context, "default")
            .await
    }

    /// Process conversation with session-specific context management
    pub async fn process_conversation_with_session(
        &self,
        input: &str,
        context: Option<&str>,
        session_id: &str,
    ) -> Result<AIResponse> {
        use prompts::{PromptContext, PromptTemplates};

        // Step 1: Classify intent
        let intent = self.intent_classifier.classify(input).await?;

        // Step 2: Get enhanced context from context manager and existing context
        let enhanced_context = if let Some(ctx) = context {
            if ctx.starts_with("No relevant documents")
                || ctx.starts_with("Vector search unavailable")
            {
                self.get_enhanced_context(session_id, input).await
            } else {
                // Combine external context with session context
                let session_context = self.get_enhanced_context(session_id, input).await;
                if session_context.contains("No document context available") {
                    ctx.to_string()
                } else {
                    format!("{}\n\n{}", ctx, session_context)
                }
            }
        } else {
            self.get_enhanced_context(session_id, input).await
        };

        // Step 3: Detect appropriate prompt type and build context-aware prompt
        let prompt_type = PromptTemplates::detect_prompt_type(input);

        let enhanced_input = if enhanced_context.contains("No document context available")
            || enhanced_context.starts_with("No relevant documents")
            || enhanced_context.starts_with("Vector search unavailable")
        {
            // Use original input when no document context is available
            input.to_string()
        } else {
            // Extract conversation history from context manager
            let conversation_history = {
                let context_manager = self.context_manager.lock().await;
                context_manager
                    .contexts
                    .get(session_id)
                    .map(|ctx| ctx.conversation_history.clone())
                    .unwrap_or_default()
            };

            // Build document-aware prompt using templates
            let prompt_context = PromptContext {
                document_content: Some(enhanced_context),
                document_metadata: {
                    let context_manager = self.context_manager.lock().await;
                    context_manager.extract_document_metadata(session_id)
                },
                user_query: input.to_string(),
                conversation_history,
            };

            match prompt_type {
                prompts::PromptType::DocumentAnalysis => {
                    PromptTemplates::build_document_analysis_prompt(&prompt_context)
                }
                prompts::PromptType::QuestionAnswering => {
                    PromptTemplates::build_qa_prompt(&prompt_context)
                }
                prompts::PromptType::StructureAnalysis => {
                    PromptTemplates::build_structure_analysis_prompt(
                        &prompt_context.document_content.unwrap_or_default(),
                        input,
                    )
                }
                prompts::PromptType::ProcedureAnalysis => {
                    PromptTemplates::build_procedure_analysis_prompt(
                        &prompt_context.document_content.unwrap_or_default(),
                        input,
                    )
                }
                prompts::PromptType::StyleAnalysis => PromptTemplates::build_style_analysis_prompt(
                    &prompt_context.document_content.unwrap_or_default(),
                    input,
                ),
                _ => {
                    // Default to question-answering for other types
                    PromptTemplates::build_qa_prompt(&prompt_context)
                }
            }
        };

        // Step 4: Generate response based on provider
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

        // Step 5: Record conversation turn
        let _ = self.add_conversation_turn(session_id, "user", input).await;
        let _ = self
            .add_conversation_turn(session_id, "assistant", &response_content)
            .await;

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
                confidence_explanation:
                    "Document-aware response using context manager and prompt templates".to_string(),
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

    /// Add documents to the context for a session
    #[allow(dead_code)]
    pub async fn add_documents_to_context(
        &self,
        session_id: &str,
        documents: Vec<DocumentRef>,
    ) -> Result<()> {
        let mut context_manager = self.context_manager.lock().await;
        context_manager.add_relevant_documents(session_id, documents)
    }

    /// Add a conversation turn to the history
    pub async fn add_conversation_turn(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<()> {
        let mut context_manager = self.context_manager.lock().await;
        context_manager.add_conversation_turn(session_id, role, content)
    }

    /// Get enhanced context for AI processing
    pub async fn get_enhanced_context(&self, session_id: &str, query: &str) -> String {
        let context_manager = self.context_manager.lock().await;
        context_manager.get_relevant_context(session_id, query)
    }

    /// Update user preferences for a session
    #[allow(dead_code)]
    pub async fn update_user_preferences(
        &self,
        session_id: &str,
        preferences: UserPreferences,
    ) -> Result<()> {
        let mut context_manager = self.context_manager.lock().await;
        context_manager.update_preferences(session_id, preferences)
    }

    /// Clear session context
    #[allow(dead_code)]
    pub async fn clear_session_context(&self, session_id: &str) {
        let mut context_manager = self.context_manager.lock().await;
        context_manager.clear_session(session_id);
    }
}

pub fn init() {
    tracing::info!("AI module initialized");
}
