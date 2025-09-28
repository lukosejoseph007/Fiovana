// src-tauri/src/commands/ai_commands.rs

use crate::ai::{AIConfig, AIOrchestrator, AIResponse};
use crate::commands::conversation_context_commands::ConversationContextState;
use crate::commands::document_indexing_commands::{
    get_relevant_documents_for_context, DocumentIndexerState,
};
use crate::commands::vector_commands::VectorState;
use crate::vector::{EmbeddingConfig, EmbeddingEngine, SearchResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub success: bool,
    pub response: Option<AIResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStatusResponse {
    pub available: bool,
    pub models: Vec<String>,
    pub current_model: String,
    pub error: Option<String>,
}

// AI state management
pub type AIState = Arc<Mutex<Option<AIOrchestrator>>>;

#[tauri::command]
pub async fn init_ai_system(
    ai_state: State<'_, AIState>,
    config: Option<serde_json::Value>,
) -> Result<bool, String> {
    tracing::info!("Initializing AI system with config: {:?}", config);

    // Load settings from storage if no config provided
    let settings = if let Some(config) = config {
        tracing::info!("Using provided config: {:?}", config);
        config
    } else {
        let loaded_settings = get_ai_settings().await?;
        tracing::info!("Using stored settings: {:?}", loaded_settings);
        loaded_settings
    };

    // Convert JSON settings to AIConfig
    let ai_config = settings_to_ai_config(settings.clone())?;
    tracing::info!(
        "Converted to AIConfig: provider={}, model={}, openrouter_key_present={}",
        ai_config.provider,
        ai_config.default_model,
        ai_config.openrouter_api_key.is_some()
    );

    match AIOrchestrator::new(ai_config).await {
        Ok(orchestrator) => {
            let mut state = ai_state.lock().await;
            *state = Some(orchestrator);
            tracing::info!("AI system initialized successfully");
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Failed to initialize AI system: {}", e);
            Err(format!("Failed to initialize AI system: {}", e))
        }
    }
}

#[tauri::command]
pub async fn chat_with_ai(
    ai_state: State<'_, AIState>,
    vector_state: State<'_, VectorState>,
    indexer_state: State<'_, DocumentIndexerState>,
    conversational_state: State<
        '_,
        crate::commands::conversational_intelligence_commands::ConversationalIntelligenceState,
    >,
    conversation_context_state: State<'_, ConversationContextState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    // Get or create session ID
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("session_{}", chrono::Utc::now().timestamp_millis()));

    // Add user message to conversation context
    {
        let mut context_manager = conversation_context_state.lock().await;
        let _ = context_manager.add_conversation_turn(&session_id, "user", &request.message, None);
    }

    // Get enriched conversation context
    let enriched_context = {
        let context_manager = conversation_context_state.lock().await;
        context_manager.get_enriched_context(&session_id, &request.message)
    };

    // Check if conversational intelligence is available and enabled
    let use_conversational_intelligence = {
        let conv_state = conversational_state.lock().await;
        conv_state.is_some()
    };

    if use_conversational_intelligence {
        // Use the conversational intelligence system for enhanced intent-based processing
        let conversation_request =
            crate::commands::conversational_intelligence_commands::ConversationRequest {
                user_input: request.message.clone(),
                session_id: None, // ChatRequest doesn't have session_id, we'll auto-generate
                context: request.context.as_ref().map(|ctx| {
                    let mut context_map = std::collections::HashMap::new();
                    context_map.insert("user_context".to_string(), ctx.clone());
                    context_map
                }),
            };

        match crate::commands::conversational_intelligence_commands::process_conversation(
            conversational_state,
            conversation_request,
        )
        .await
        {
            Ok(conv_response) => {
                // If the action was executed successfully, return the result
                if conv_response.action_executed {
                    if let Some(action_result) = conv_response.action_result {
                        let response_text = if action_result.success {
                            format!(
                                "I understood your intent ({:?}) and completed the action: {}",
                                conv_response.intent, action_result.message
                            )
                        } else {
                            format!(
                                "I understood your intent ({:?}) but couldn't complete the action: {}",
                                conv_response.intent, action_result.message
                            )
                        };

                        let ai_response = AIResponse {
                            response_type: if action_result.success {
                                crate::ai::response::ResponseType::Action
                            } else {
                                crate::ai::response::ResponseType::Error
                            },
                            content: response_text,
                            intent: conv_response.intent,
                            confidence: conv_response.confidence,
                            confidence_level: crate::ai::response::ConfidenceLevel::from_score(
                                conv_response.confidence,
                            ),
                            suggested_actions: conv_response
                                .suggested_actions
                                .into_iter()
                                .map(|action| crate::ai::response::SuggestedAction {
                                    action_type: action,
                                    description: "Generated from conversational intelligence"
                                        .to_string(),
                                    parameters: None,
                                    priority: crate::ai::response::ActionPriority::Medium,
                                    estimated_duration: Some("Variable".to_string()),
                                    prerequisites: vec![],
                                })
                                .collect(),
                            follow_up_questions: vec![],
                            document_references: vec![],
                            action_items: vec![],
                            style_guidance: None,
                            metadata: crate::ai::response::ResponseMetadata {
                                processing_time_ms: action_result.execution_time_ms,
                                model_used: "intent_classification".to_string(),
                                tokens_used: None,
                                confidence_explanation: conv_response.reasoning,
                                context_used: false,
                                documents_analyzed: 0,
                                session_id: None,
                                turn_id: None,
                                reasoning_chain: vec![
                                    "Used conversational intelligence system".to_string()
                                ],
                            },
                        };

                        return Ok(ChatResponse {
                            success: action_result.success,
                            response: Some(ai_response),
                            error: if action_result.success {
                                None
                            } else {
                                Some(action_result.message)
                            },
                        });
                    }
                } else {
                    // Action wasn't executed, fall back to conversational AI
                    tracing::debug!("Intent classified as {:?} but action not executed, falling back to conversational AI", conv_response.intent);
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Conversational intelligence failed: {}, falling back to AI chat",
                    e
                );
            }
        }
    }

    // Fall back to traditional AI chat for general conversation or when conversational intelligence fails
    let state = ai_state.lock().await;

    match state.as_ref() {
        Some(orchestrator) => {
            // Build comprehensive context combining conversation history, document search, and user context
            let mut context_parts = Vec::new();

            // Add conversation context if available
            if let Some(enriched_ctx) = &enriched_context {
                context_parts.push("=== CONVERSATION CONTEXT ===".to_string());
                context_parts.push(format!("Session: {}", enriched_ctx.session_id));

                if let Some(topic) = &enriched_ctx.current_topic {
                    context_parts.push(format!("Current Topic: {}", topic));
                }

                if !enriched_ctx.active_documents.is_empty() {
                    context_parts.push("Active Documents:".to_string());
                    for doc in &enriched_ctx.active_documents {
                        context_parts
                            .push(format!("- {} (refs: {})", doc.title, doc.reference_count));
                    }
                }

                if !enriched_ctx.reference_resolution_map.is_empty() {
                    context_parts.push("Reference Resolutions:".to_string());
                    for (original, resolved) in &enriched_ctx.reference_resolution_map {
                        context_parts.push(format!("- '{}' refers to '{}'", original, resolved));
                    }
                }

                if !enriched_ctx.recent_conversation.is_empty() {
                    context_parts.push("Recent Conversation:".to_string());
                    for turn in enriched_ctx.recent_conversation.iter().rev().take(3).rev() {
                        let content_preview = if turn.content.len() > 100 {
                            format!("{}...", &turn.content[..100])
                        } else {
                            turn.content.clone()
                        };
                        context_parts.push(format!(
                            "{}: {}",
                            turn.role.to_uppercase(),
                            content_preview
                        ));
                    }
                }

                context_parts.push(format!("Context Summary: {}", enriched_ctx.context_summary));
            }

            // Add user-provided context if available
            if let Some(context) = request.context.as_deref() {
                context_parts.push("=== USER PROVIDED CONTEXT ===".to_string());
                context_parts.push(context.to_string());
            }

            // Add document search results
            context_parts.push("=== DOCUMENT SEARCH RESULTS ===".to_string());
            let document_context =
                perform_enhanced_document_search(&vector_state, &indexer_state, &request.message)
                    .await;
            context_parts.push(document_context);

            let enhanced_context = context_parts.join("\n\n");
            debug!(
                "Enhanced chat context length: {} characters",
                enhanced_context.len()
            );

            // Process with AI orchestrator
            match orchestrator
                .process_conversation(&request.message, Some(&enhanced_context))
                .await
            {
                Ok(response) => {
                    // Add assistant response to conversation context
                    {
                        let mut context_manager = conversation_context_state.lock().await;
                        let intent_str = Some(format!("{:?}", response.intent));
                        let _ = context_manager.add_conversation_turn(
                            &session_id,
                            "assistant",
                            &response.content,
                            intent_str,
                        );
                    }

                    Ok(ChatResponse {
                        success: true,
                        response: Some(response),
                        error: None,
                    })
                }
                Err(e) => {
                    let error_msg = format!("AI processing failed: {}", e);
                    tracing::error!("{}", error_msg);
                    Ok(ChatResponse {
                        success: false,
                        response: None,
                        error: Some(error_msg),
                    })
                }
            }
        }
        None => Ok(ChatResponse {
            success: false,
            response: None,
            error: Some("AI system not initialized".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn get_ai_status(ai_state: State<'_, AIState>) -> Result<AIStatusResponse, String> {
    let state = ai_state.lock().await;

    match state.as_ref() {
        Some(orchestrator) => {
            tracing::info!("AI orchestrator exists, checking availability...");
            let available = orchestrator.is_available().await;
            let models = orchestrator.list_models().await.unwrap_or_else(|_| vec![]);
            tracing::info!("AI status: available={}, models={:?}", available, models);

            Ok(AIStatusResponse {
                available,
                models: models.clone(),
                current_model: "llama3.2-3b".to_string(), // Default model
                error: if !available {
                    Some("Ollama service not available".to_string())
                } else {
                    None
                },
            })
        }
        None => {
            tracing::warn!("AI orchestrator not initialized");
            Ok(AIStatusResponse {
                available: false,
                models: vec![],
                current_model: "".to_string(),
                error: Some("AI system not initialized".to_string()),
            })
        }
    }
}

#[tauri::command]
pub async fn shutdown_ai_system(ai_state: State<'_, AIState>) -> Result<bool, String> {
    let mut state = ai_state.lock().await;
    *state = None;
    tracing::info!("AI system shut down");
    Ok(true)
}

#[tauri::command]
pub async fn restart_ai_system(
    ai_state: State<'_, AIState>,
    config: Option<serde_json::Value>,
) -> Result<bool, String> {
    // Shutdown first
    let _ = shutdown_ai_system(ai_state.clone()).await;

    // Then reinitialize
    init_ai_system(ai_state, config).await
}

#[tauri::command]
pub async fn check_ollama_connection() -> Result<bool, String> {
    use crate::ai::ollama::{OllamaClient, OllamaConfig};

    let config = OllamaConfig::default();
    match OllamaClient::new(config).await {
        Ok(client) => Ok(client.is_available().await),
        Err(e) => {
            tracing::warn!("Ollama connection check failed: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn get_available_models() -> Result<Vec<String>, String> {
    use crate::ai::ollama::{OllamaClient, OllamaConfig};

    let config = OllamaConfig::default();
    match OllamaClient::new(config).await {
        Ok(client) => {
            if client.is_available().await {
                client.list_models().await.map_err(|e| e.to_string())
            } else {
                Ok(vec![])
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to Ollama: {}", e);
            Ok(vec![])
        }
    }
}

#[tauri::command]
pub async fn pull_model(model_name: String) -> Result<bool, String> {
    use crate::ai::ollama::{OllamaClient, OllamaConfig};

    let config = OllamaConfig::default();
    match OllamaClient::new(config).await {
        Ok(client) => {
            if client.is_available().await {
                client
                    .pull_model(&model_name)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(true)
            } else {
                Err("Ollama service not available".to_string())
            }
        }
        Err(e) => Err(format!("Failed to connect to Ollama: {}", e)),
    }
}

// Test command for development
// Global storage for AI settings (in a real app, this would be in a database or config file)
use once_cell::sync::Lazy;
use std::sync::Mutex as StdMutex;

static AI_SETTINGS_STORAGE: Lazy<StdMutex<Option<serde_json::Value>>> =
    Lazy::new(|| StdMutex::new(None));

// Helper function to convert JSON settings to AIConfig
fn settings_to_ai_config(settings: serde_json::Value) -> Result<AIConfig, String> {
    let provider = settings
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("local")
        .to_string();

    let default_model = settings
        .get("selectedModel")
        .and_then(|v| v.as_str())
        .unwrap_or("llama3.2-3b")
        .to_string();

    let openrouter_api_key = settings
        .get("openrouterApiKey")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let anthropic_api_key = settings
        .get("anthropicApiKey")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Ok(AIConfig {
        provider,
        openrouter_api_key,
        anthropic_api_key,
        default_model,
        temperature: 0.7,
        max_tokens: Some(4096),
        ollama: crate::ai::OllamaConfig::default(),
    })
}

#[tauri::command]
pub async fn get_ai_settings() -> Result<serde_json::Value, String> {
    // Try to load from file first
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("proxemic");

    let settings_file = config_dir.join("ai_settings.json");

    if !settings_file.exists() {
        tracing::info!("No AI settings file found, returning defaults");
        let default_settings = serde_json::json!({
            "provider": "local",
            "openrouterApiKey": "",
            "anthropicApiKey": "",
            "selectedModel": "llama3.2-3b",
            "preferLocalModels": true,
            "recentModels": []
        });
        return Ok(default_settings);
    }

    let settings_json = std::fs::read_to_string(&settings_file)
        .map_err(|e| format!("Failed to read AI settings file: {}", e))?;

    let settings: serde_json::Value = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Failed to parse AI settings: {}", e))?;

    tracing::info!("✅ Loaded AI settings from file");
    Ok(settings)
}

#[tauri::command]
pub async fn save_ai_settings(settings: serde_json::Value) -> Result<bool, String> {
    tracing::info!("Saving AI settings to file");

    // Save to file-based storage
    let settings_json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize AI settings: {}", e))?;

    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("proxemic");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let settings_file = config_dir.join("ai_settings.json");
    std::fs::write(&settings_file, settings_json)
        .map_err(|e| format!("Failed to write AI settings file: {}", e))?;

    // Also store in memory for compatibility with existing code
    {
        let mut storage = AI_SETTINGS_STORAGE.lock().unwrap();
        *storage = Some(settings);
    }

    tracing::info!("✅ AI settings saved successfully to file");
    Ok(true)
}

#[tauri::command]
pub async fn test_ai_conversation(
    _ai_state: State<'_, AIState>,
    _vector_state: State<'_, VectorState>,
    test_message: String,
) -> Result<String, String> {
    let _request = ChatRequest {
        message: test_message,
        context: Some("This is a test conversation".to_string()),
        session_id: Some("test_session".to_string()),
    };

    // Note: This test is simplified and doesn't test document indexer integration
    // TODO: Add proper integration test with mocked indexer state
    Ok(
        "Test conversation completed - document indexer integration requires full app context"
            .to_string(),
    )
}

/// Command to manually trigger indexing of a specific document
#[allow(dead_code)]
#[tauri::command]
pub async fn index_document_for_ai(
    app_state: tauri::State<'_, crate::app_state::AppState>,
    file_path: String,
) -> Result<bool, String> {
    use crate::filesystem::watcher::FileEvent;
    use std::path::PathBuf;

    // Create a file event for the document
    let path_buf = PathBuf::from(&file_path);
    let file_event = FileEvent::Created(path_buf);

    // Send to indexing service
    match app_state.document_indexing_sender.send(file_event) {
        Ok(()) => {
            tracing::info!("Successfully queued document for indexing: {}", file_path);
            Ok(true)
        }
        Err(e) => {
            let error_msg = format!("Failed to queue document for indexing: {}", e);
            tracing::error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

/// Command to get information about what documents are currently indexed
#[allow(dead_code)]
#[tauri::command]
pub async fn get_indexed_documents_info(
    vector_state: State<'_, crate::commands::vector_commands::VectorState>,
) -> Result<serde_json::Value, String> {
    // Get vector store statistics to understand what's indexed
    let stats = crate::commands::vector_commands::get_vector_stats(vector_state).await?;

    Ok(serde_json::json!({
        "total_documents": stats.total_documents,
        "total_chunks": stats.total_chunks,
        "total_embeddings": stats.total_embeddings,
        "dimension": stats.dimension,
        "memory_usage_estimate_bytes": stats.memory_usage_estimate
    }))
}

/// Perform document search using the document indexer
async fn perform_indexed_document_search(
    indexer_state: &DocumentIndexerState,
    query: &str,
) -> String {
    tracing::info!("AI searching for documents with query: '{}'", query);

    match get_relevant_documents_for_context(indexer_state, query, 5).await {
        Ok(documents) => {
            tracing::info!("Document search returned {} documents", documents.len());
            if documents.is_empty() {
                tracing::warn!("No documents found in index for query: '{}'", query);
                "No relevant documents found in index. Make sure you have uploaded documents in File Management and they have been successfully indexed.".to_string()
            } else {
                let mut context_parts = vec!["=== RELEVANT DOCUMENTS ===".to_string()];

                for (i, doc) in documents.iter().enumerate() {
                    let preview = if doc.content.len() > 500 {
                        format!("{}...", &doc.content[..500])
                    } else {
                        doc.content.clone()
                    };

                    let doc_section = format!(
                        "Document {}: {}\nPath: {}\nContent: {}\n",
                        i + 1,
                        doc.title,
                        doc.path.display(),
                        preview
                    );
                    context_parts.push(doc_section);
                }

                let result = context_parts.join("\n");
                tracing::info!(
                    "Providing AI with document context: {} characters",
                    result.len()
                );
                result
            }
        }
        Err(e) => {
            tracing::error!("Failed to search documents with error: {}", e);
            format!("Document search unavailable due to error: {}. Please check if documents are indexed.", e)
        }
    }
}

/// Perform enhanced document search combining vector and keyword search
async fn perform_enhanced_document_search(
    vector_state: &VectorState,
    indexer_state: &DocumentIndexerState,
    query: &str,
) -> String {
    info!("AI performing enhanced search for query: '{}'", query);

    // Vector-first approach: Try vector search first, only fall back to document search if needed
    let vector_results = perform_vector_search(vector_state, query).await;

    if !vector_results.is_empty() {
        // Vector search found results - use only these (most precise and efficient)
        tracing::info!(
            "Using vector search results only - {} characters",
            vector_results.len()
        );
        vector_results
    } else {
        // Vector search failed, fall back to document search
        tracing::info!("Vector search empty, attempting document search fallback");
        let keyword_results = perform_indexed_document_search(indexer_state, query).await;

        if !keyword_results.is_empty() {
            tracing::info!(
                "Using document search fallback - {} characters",
                keyword_results.len()
            );
            keyword_results
        } else {
            // No results from either search
            tracing::warn!("Both vector and document search returned empty results");
            "No relevant documents found. Make sure you have uploaded documents in File Management and they have been successfully indexed and processed for vector search.".to_string()
        }
    }
}

/// Perform vector search using embeddings
async fn perform_vector_search(vector_state: &VectorState, query: &str) -> String {
    debug!("Performing vector search for query: '{}'", query);

    // Get the vector store from state
    let vector_store = &vector_state.vector_store;

    // VectorStore is always available, no need for Option check
    // Create a temporary embedding engine to generate query embedding
    match EmbeddingEngine::new(EmbeddingConfig::default()).await {
        Ok(engine) => {
            match engine.embed_text(query).await {
                Ok(query_embedding) => {
                    // Perform vector search
                    match vector_store.search(&query_embedding, 5).await {
                        Ok(results) => {
                            if results.is_empty() {
                                debug!("Vector search returned no results");
                                String::new()
                            } else {
                                debug!("Vector search returned {} results", results.len());
                                format_vector_search_results(results)
                            }
                        }
                        Err(e) => {
                            warn!("Vector search error: {}", e);
                            String::new()
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to generate query embedding: {}", e);
                    String::new()
                }
            }
        }
        Err(e) => {
            warn!("Failed to create embedding engine: {}", e);
            String::new()
        }
    }
}

/// Format vector search results for AI context
fn format_vector_search_results(results: Vec<SearchResult>) -> String {
    let mut context_parts = vec!["=== VECTOR SEARCH RESULTS ===".to_string()];

    for (i, result) in results.iter().enumerate() {
        let preview = if result.chunk.content.len() > 400 {
            format!("{}...", &result.chunk.content[..400])
        } else {
            result.chunk.content.clone()
        };

        context_parts.push(format!(
            "Result {}: Document {} (Similarity: {:.3})\nContent: {}\nExplanation: {}",
            i + 1,
            result.chunk.document_id,
            result.similarity,
            preview,
            result.explanation
        ));
    }

    context_parts.join("\n\n")
}

// Document operation command types and Tauri commands
use crate::ai::document_commands::{
    CommandParser, CommandResult, DocumentCommand, DocumentCommandProcessor,
};

// State for document command processor
pub type DocumentCommandProcessorState = Arc<Mutex<Option<DocumentCommandProcessor>>>;

#[tauri::command]
pub async fn initialize_document_commands(
    indexer_state: State<'_, DocumentIndexerState>,
    vector_state: State<'_, VectorState>,
    command_processor_state: State<'_, DocumentCommandProcessorState>,
) -> Result<String, String> {
    let indexer_guard = indexer_state.lock().await;
    if let Some(indexer) = indexer_guard.as_ref() {
        let mut processor = DocumentCommandProcessor::new(Arc::new(indexer.clone()));

        // Add vector search capabilities if available
        let embedding_engine_guard = vector_state.embedding_engine.lock().await;
        if let Some(embedding_engine) = embedding_engine_guard.as_ref() {
            processor = processor.with_vector_search(
                embedding_engine.clone(),
                (*vector_state.vector_store).clone(),
            );
        }

        let mut command_processor_guard = command_processor_state.lock().await;
        *command_processor_guard = Some(processor);

        Ok("Document command processor initialized successfully".to_string())
    } else {
        Err("Document indexer not initialized".to_string())
    }
}

#[tauri::command]
pub async fn execute_document_command(
    command_text: String,
    command_processor_state: State<'_, DocumentCommandProcessorState>,
) -> Result<CommandResult, String> {
    // Parse natural language command
    let command = CommandParser::parse_command(&command_text).ok_or(
        "Could not parse command. Try commands like 'summarize document X' or 'compare A and B'"
            .to_string(),
    )?;

    // Execute command
    let command_processor_guard = command_processor_state.lock().await;
    if let Some(processor) = command_processor_guard.as_ref() {
        processor
            .execute_command(command)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Document command processor not initialized".to_string())
    }
}

#[tauri::command]
pub async fn parse_document_command(
    command_text: String,
) -> Result<Option<DocumentCommand>, String> {
    Ok(CommandParser::parse_command(&command_text))
}

#[tauri::command]
pub async fn get_available_document_commands() -> Result<Vec<String>, String> {
    let examples = vec![
        "summarize document [document_id]".to_string(),
        "compare [document_a] and [document_b]".to_string(),
        "find similar documents to [document_id]".to_string(),
        "analyze content of [document_id]".to_string(),
        "analyze structure of [document_id]".to_string(),
        "extract key points from [document_id]".to_string(),
        "search for [query]".to_string(),
        "search documents about [topic]".to_string(),
    ];

    Ok(examples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_config_and_orchestrator() {
        // Test that AI config can be created
        let config = AIConfig::default();
        assert_eq!(config.default_model, "llama3.2-3b");
        assert_eq!(config.temperature, 0.7);

        // Test that orchestrator can be initialized (will use fallback if Ollama not available)
        let result = AIOrchestrator::new(config).await;
        assert!(
            result.is_ok(),
            "AI orchestrator should initialize successfully"
        );
    }

    #[tokio::test]
    async fn test_ollama_connection_check() {
        let result = check_ollama_connection().await;
        assert!(result.is_ok());
        // Result depends on whether Ollama is running
    }
}
