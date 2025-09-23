// src-tauri/src/commands/ai_commands.rs

use crate::ai::{AIConfig, AIOrchestrator, AIResponse};
use crate::commands::document_indexing_commands::{
    get_relevant_documents_for_context, DocumentIndexerState,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context: Option<String>,
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
    _vector_state: State<'_, crate::commands::vector_commands::VectorState>,
    indexer_state: State<'_, DocumentIndexerState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let state = ai_state.lock().await;

    match state.as_ref() {
        Some(orchestrator) => {
            // Perform document search to provide context
            let enhanced_context = if let Some(context) = request.context.as_deref() {
                format!("User Context: {}", context)
            } else {
                // Search indexed documents for relevant context
                perform_indexed_document_search(&indexer_state, &request.message).await
            };

            match orchestrator
                .process_conversation(&request.message, Some(&enhanced_context))
                .await
            {
                Ok(response) => Ok(ChatResponse {
                    success: true,
                    response: Some(response),
                    error: None,
                }),
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
    // Try to load from storage first
    let stored_settings = AI_SETTINGS_STORAGE.lock().unwrap().clone();

    if let Some(settings) = stored_settings {
        tracing::info!("Loaded stored AI settings: {:?}", settings);
        Ok(settings)
    } else {
        tracing::info!("No stored settings found, using defaults");
        // Return default settings
        let default_settings = serde_json::json!({
            "provider": "local",
            "openrouterApiKey": "",
            "anthropicApiKey": "",
            "selectedModel": "llama3.2-3b",
            "preferLocalModels": true,
            "recentModels": []
        });
        Ok(default_settings)
    }
}

#[tauri::command]
pub async fn save_ai_settings(settings: serde_json::Value) -> Result<bool, String> {
    tracing::info!("Saving AI settings: {:?}", settings);

    // Store the settings
    {
        let mut storage = AI_SETTINGS_STORAGE.lock().unwrap();
        *storage = Some(settings.clone());
    }

    tracing::info!("AI settings saved successfully");
    Ok(true)
}

#[tauri::command]
pub async fn test_ai_conversation(
    _ai_state: State<'_, AIState>,
    _vector_state: State<'_, crate::commands::vector_commands::VectorState>,
    test_message: String,
) -> Result<String, String> {
    let _request = ChatRequest {
        message: test_message,
        context: Some("This is a test conversation".to_string()),
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
