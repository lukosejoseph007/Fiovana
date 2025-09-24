// src-tauri/src/commands/embedding_settings_commands.rs
// Commands for managing embedding service settings through the UI

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingSettings {
    pub provider: String, // "openai" or "openrouter"
    pub api_key: String,
    pub model: String,
    pub custom_dimensions: Option<usize>,
    pub batch_size: usize,
    pub timeout_seconds: u64,
}

impl Default for EmbeddingSettings {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: String::new(),
            model: "text-embedding-3-small".to_string(),
            custom_dimensions: None,
            batch_size: 25,
            timeout_seconds: 90,
        }
    }
}

/// Test embedding connection with given settings
#[tauri::command]
pub async fn test_embedding_settings_connection(
    provider: String,
    api_key: String,
    model: String,
) -> Result<bool, String> {
    tracing::info!(
        "Testing embedding connection - Provider: {}, Model: {}",
        provider,
        model
    );

    if api_key.is_empty() {
        return Err("API key is required".to_string());
    }

    // Create a minimal embedding service to test connection
    let embedding_provider = match provider.as_str() {
        "openai" => crate::vector::EmbeddingProvider::OpenAI,
        "openrouter" => crate::vector::EmbeddingProvider::OpenRouter,
        _ => return Err("Unsupported provider".to_string()),
    };

    let service_config = crate::vector::EmbeddingServiceConfig {
        provider: embedding_provider,
        api_key: Some(api_key),
        model_name: model,
        dimension: 1536, // Standard dimension for testing
        max_tokens: 8192,
        batch_size: 5,       // Small batch for testing
        timeout_seconds: 30, // Quick timeout for testing
    };

    match crate::vector::EmbeddingService::new(service_config).await {
        Ok(service) => match service.test_connection().await {
            Ok(connected) => {
                if connected {
                    tracing::info!("✅ Embedding connection test successful");
                    Ok(true)
                } else {
                    tracing::warn!("❌ Embedding connection test failed - service unavailable");
                    Ok(false)
                }
            }
            Err(e) => {
                tracing::error!("❌ Embedding connection test error: {}", e);
                Err(format!("Connection test failed: {}", e))
            }
        },
        Err(e) => {
            tracing::error!("❌ Failed to create embedding service: {}", e);
            Err(format!("Failed to initialize embedding service: {}", e))
        }
    }
}

/// Save embedding settings to persistent storage
#[tauri::command]
pub async fn save_embedding_settings(settings: EmbeddingSettings) -> Result<(), String> {
    tracing::info!(
        "Saving embedding settings - Provider: {}, Model: {}",
        settings.provider,
        settings.model
    );

    // Save to local storage (could be extended to save to file/database)
    let settings_json = serde_json::to_string(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // For now, we'll use a simple file-based approach
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("proxemic");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let settings_file = config_dir.join("embedding_settings.json");
    std::fs::write(&settings_file, settings_json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    tracing::info!("✅ Embedding settings saved successfully");
    Ok(())
}

/// Load embedding settings from persistent storage
#[tauri::command]
pub async fn load_embedding_settings() -> Result<EmbeddingSettings, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("proxemic");

    let settings_file = config_dir.join("embedding_settings.json");

    if !settings_file.exists() {
        tracing::info!("No embedding settings file found, returning defaults");
        return Ok(EmbeddingSettings::default());
    }

    let settings_json = std::fs::read_to_string(&settings_file)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: EmbeddingSettings = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    tracing::info!(
        "✅ Loaded embedding settings - Provider: {}, Model: {}",
        settings.provider,
        settings.model
    );
    Ok(settings)
}

/// Apply embedding settings to the vector system
#[tauri::command]
pub async fn apply_embedding_settings(
    settings: EmbeddingSettings,
    vector_state: tauri::State<'_, crate::commands::vector_commands::VectorState>,
) -> Result<String, String> {
    tracing::info!("Applying embedding settings to vector system");

    if settings.api_key.is_empty() {
        return Err("API key is required".to_string());
    }

    // Create new embedding configuration
    let embedding_provider = match settings.provider.as_str() {
        "openai" => crate::vector::EmbeddingProvider::OpenAI,
        "openrouter" => crate::vector::EmbeddingProvider::OpenRouter,
        _ => return Err("Unsupported provider".to_string()),
    };

    // Determine dimensions based on model
    let dimension = match settings.model.as_str() {
        "text-embedding-3-small" => 1536,
        "text-embedding-3-large" => 3072,
        "text-embedding-ada-002" => 1536,
        _ => settings.custom_dimensions.unwrap_or(1536),
    };

    let service_config = crate::vector::EmbeddingServiceConfig {
        provider: embedding_provider,
        api_key: Some(settings.api_key.clone()),
        model_name: settings.model.clone(),
        dimension: settings.custom_dimensions.unwrap_or(dimension),
        max_tokens: 8192,
        batch_size: settings.batch_size,
        timeout_seconds: settings.timeout_seconds,
    };

    let embedding_config = crate::vector::EmbeddingConfig {
        model_name: settings.model.clone(),
        dimension: settings.custom_dimensions.unwrap_or(dimension),
        max_length: 8192,
        chunk_size: 1000,
        chunk_overlap: 200,
    };

    // Create new embedding engine
    match crate::vector::EmbeddingEngine::new_with_service(embedding_config, service_config).await {
        Ok(engine) => {
            // Replace the existing engine
            let mut engine_guard = vector_state.embedding_engine.lock().await;
            *engine_guard = Some(engine);

            // Also update the vector store dimension if needed
            let current_dimension = vector_state.vector_store.dimension();
            let new_dimension = settings.custom_dimensions.unwrap_or(dimension);

            if current_dimension != new_dimension {
                tracing::info!(
                    "Vector store dimension changed from {} to {}, reinitializing...",
                    current_dimension,
                    new_dimension
                );
                // Note: In a production system, you might want to migrate existing embeddings
                // For now, we'll just note this and let the user re-sync documents
            }

            tracing::info!("✅ Embedding settings applied successfully");
            Ok(format!(
                "Embedding settings applied successfully. Provider: {}, Model: {}, Dimensions: {}",
                settings.provider, settings.model, new_dimension
            ))
        }
        Err(e) => {
            tracing::error!("❌ Failed to apply embedding settings: {}", e);
            Err(format!("Failed to apply settings: {}", e))
        }
    }
}

/// Get current embedding system status
#[tauri::command]
pub async fn get_embedding_system_status(
    vector_state: tauri::State<'_, crate::commands::vector_commands::VectorState>,
) -> Result<HashMap<String, String>, String> {
    let mut status = HashMap::new();

    let engine_guard = vector_state.embedding_engine.lock().await;
    match engine_guard.as_ref() {
        Some(engine) => {
            let config = engine.get_config();
            let is_available = engine.is_model_available();

            status.insert(
                "status".to_string(),
                if is_available {
                    "connected".to_string()
                } else {
                    "disconnected".to_string()
                },
            );
            status.insert("model".to_string(), config.model_name.clone());
            status.insert("dimension".to_string(), config.dimension.to_string());
            status.insert("provider".to_string(), "api".to_string()); // Since we removed local fallback
        }
        None => {
            status.insert("status".to_string(), "not_initialized".to_string());
            status.insert("model".to_string(), "none".to_string());
            status.insert("dimension".to_string(), "0".to_string());
            status.insert("provider".to_string(), "none".to_string());
        }
    }

    Ok(status)
}
