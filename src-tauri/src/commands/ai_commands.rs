// src-tauri/src/commands/ai_commands.rs

use crate::ai::{AIConfig, AIOrchestrator, AIResponse};
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
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let state = ai_state.lock().await;

    match state.as_ref() {
        Some(orchestrator) => {
            match orchestrator
                .process_conversation(&request.message, request.context.as_deref())
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
    ai_state: State<'_, AIState>,
    test_message: String,
) -> Result<String, String> {
    let request = ChatRequest {
        message: test_message,
        context: Some("This is a test conversation".to_string()),
    };

    match chat_with_ai(ai_state, request).await {
        Ok(response) => {
            if response.success {
                if let Some(ai_response) = response.response {
                    Ok(format!(
                        "Intent: {:?}, Response: {}",
                        ai_response.intent, ai_response.content
                    ))
                } else {
                    Ok("No response generated".to_string())
                }
            } else {
                Err(response.error.unwrap_or("Unknown error".to_string()))
            }
        }
        Err(e) => Err(e),
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
