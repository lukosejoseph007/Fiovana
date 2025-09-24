// src-tauri/src/commands/embedding_commands.rs
use crate::vector::{
    EmbeddingConfig, EmbeddingEngine, EmbeddingProvider, EmbeddingServiceConfig, UsageStats,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub text: String,
    pub provider: Option<EmbeddingProvider>,
    pub model_name: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEmbeddingRequest {
    pub texts: Vec<String>,
    pub provider: Option<EmbeddingProvider>,
    pub model_name: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub dimension: usize,
    pub provider: String,
    pub model_name: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub dimension: usize,
    pub provider: String,
    pub model_name: String,
    pub success: bool,
    pub error: Option<String>,
    pub processed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingServiceStatus {
    pub provider: EmbeddingProvider,
    pub model_name: String,
    pub dimension: usize,
    pub available: bool,
    pub usage_stats: UsageStats,
    pub cache_size: usize,
}

// Global embedding engine state type alias
type EmbeddingEngineState<'a> = State<'a, tokio::sync::RwLock<Option<EmbeddingEngine>>>;

#[tauri::command]
pub async fn configure_embedding_service(
    provider: EmbeddingProvider,
    model_name: String,
    api_key: Option<String>,
    dimension: Option<usize>,
    engine_state: EmbeddingEngineState<'_>,
) -> Result<bool, String> {
    info!(
        "Configuring embedding service: provider={:?}, model={}",
        provider, model_name
    );

    let service_config = EmbeddingServiceConfig {
        provider: provider.clone(),
        api_key,
        model_name: model_name.clone(),
        dimension: dimension.unwrap_or(1536), // API model default dimension
        max_tokens: 8192,
        batch_size: 20,
        timeout_seconds: 30,
    };

    let embedding_config = EmbeddingConfig {
        model_name: model_name.clone(),
        dimension: service_config.dimension,
        max_length: service_config.max_tokens,
        chunk_size: 1000,
        chunk_overlap: 200,
    };

    match EmbeddingEngine::new_with_service(embedding_config, service_config).await {
        Ok(engine) => {
            let mut state = engine_state.write().await;
            *state = Some(engine);

            info!("Successfully configured embedding service");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to configure embedding service: {}", e);
            Err(format!("Failed to configure embedding service: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_embedding(
    request: EmbeddingRequest,
    engine_state: EmbeddingEngineState<'_>,
) -> Result<EmbeddingResponse, String> {
    debug!(
        "Getting embedding for text of length: {}",
        request.text.len()
    );

    // If specific configuration is provided, create a temporary engine
    if let Some(provider) = request.provider {
        let service_config = EmbeddingServiceConfig {
            provider: provider.clone(),
            api_key: request.api_key,
            model_name: request
                .model_name
                .unwrap_or_else(|| "text-embedding-ada-002".to_string()),
            dimension: 1536, // Default OpenAI dimension
            max_tokens: 8192,
            batch_size: 1,
            timeout_seconds: 30,
        };

        let embedding_config = EmbeddingConfig {
            model_name: service_config.model_name.clone(),
            dimension: service_config.dimension,
            max_length: service_config.max_tokens,
            chunk_size: 1000,
            chunk_overlap: 200,
        };

        match EmbeddingEngine::new_with_service(embedding_config, service_config).await {
            Ok(engine) => match engine.embed_text(&request.text).await {
                Ok(embedding) => {
                    return Ok(EmbeddingResponse {
                        embedding,
                        dimension: engine.config().dimension,
                        provider: format!("{:?}", provider),
                        model_name: engine.config().model_name.clone(),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    return Err(format!("Failed to generate embedding: {}", e));
                }
            },
            Err(e) => {
                return Err(format!("Failed to create embedding engine: {}", e));
            }
        }
    }

    // Use existing configured engine
    let state = engine_state.read().await;
    match state.as_ref() {
        Some(engine) => match engine.embed_text(&request.text).await {
            Ok(embedding) => Ok(EmbeddingResponse {
                embedding,
                dimension: engine.config().dimension,
                provider: "Configured".to_string(),
                model_name: engine.config().model_name.clone(),
                success: true,
                error: None,
            }),
            Err(e) => {
                error!("Failed to generate embedding: {}", e);
                Err(format!("Failed to generate embedding: {}", e))
            }
        },
        None => {
            // No engine configured, require explicit configuration
            error!("No embedding engine configured. Local fallback disabled for stability.");
            Err("No embedding engine configured. Please configure API-based embeddings through Settings > Embeddings. Local embeddings are disabled to prevent system crashes.".to_string())
        }
    }
}

#[tauri::command]
pub async fn get_batch_embeddings(
    request: BatchEmbeddingRequest,
    engine_state: EmbeddingEngineState<'_>,
) -> Result<BatchEmbeddingResponse, String> {
    debug!("Getting batch embeddings for {} texts", request.texts.len());

    // If specific configuration is provided, create a temporary engine
    if let Some(provider) = request.provider {
        let service_config = EmbeddingServiceConfig {
            provider: provider.clone(),
            api_key: request.api_key,
            model_name: request
                .model_name
                .unwrap_or_else(|| "text-embedding-ada-002".to_string()),
            dimension: 1536, // Default OpenAI dimension
            max_tokens: 8192,
            batch_size: 100,     // Larger batch size for bulk operations
            timeout_seconds: 60, // Longer timeout for batches
        };

        let embedding_config = EmbeddingConfig {
            model_name: service_config.model_name.clone(),
            dimension: service_config.dimension,
            max_length: service_config.max_tokens,
            chunk_size: 1000,
            chunk_overlap: 200,
        };

        match EmbeddingEngine::new_with_service(embedding_config, service_config).await {
            Ok(engine) => {
                let mut embeddings = Vec::new();
                let mut processed_count = 0;

                for text in &request.texts {
                    match engine.embed_text(text).await {
                        Ok(embedding) => {
                            embeddings.push(embedding);
                            processed_count += 1;
                        }
                        Err(e) => {
                            error!("Failed to generate embedding for text: {}", e);
                            // Continue processing other texts
                            embeddings.push(vec![0.0; engine.config().dimension]);
                            // Placeholder
                        }
                    }
                }

                return Ok(BatchEmbeddingResponse {
                    embeddings,
                    dimension: engine.config().dimension,
                    provider: format!("{:?}", provider),
                    model_name: engine.config().model_name.clone(),
                    success: processed_count == request.texts.len(),
                    error: if processed_count < request.texts.len() {
                        Some(format!(
                            "Only processed {} out of {} texts",
                            processed_count,
                            request.texts.len()
                        ))
                    } else {
                        None
                    },
                    processed_count,
                });
            }
            Err(e) => {
                return Err(format!("Failed to create embedding engine: {}", e));
            }
        }
    }

    // Use existing configured engine
    let state = engine_state.read().await;
    match state.as_ref() {
        Some(engine) => {
            let mut embeddings = Vec::new();
            let mut processed_count = 0;

            for text in &request.texts {
                match engine.embed_text(text).await {
                    Ok(embedding) => {
                        embeddings.push(embedding);
                        processed_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to generate embedding for text: {}", e);
                        // Continue processing other texts
                        embeddings.push(vec![0.0; engine.config().dimension]); // Placeholder
                    }
                }
            }

            Ok(BatchEmbeddingResponse {
                embeddings,
                dimension: engine.config().dimension,
                provider: "Configured".to_string(),
                model_name: engine.config().model_name.clone(),
                success: processed_count == request.texts.len(),
                error: if processed_count < request.texts.len() {
                    Some(format!(
                        "Only processed {} out of {} texts",
                        processed_count,
                        request.texts.len()
                    ))
                } else {
                    None
                },
                processed_count,
            })
        }
        None => Err("No embedding engine configured".to_string()),
    }
}

#[tauri::command]
pub async fn get_embedding_service_status(
    engine_state: EmbeddingEngineState<'_>,
) -> Result<EmbeddingServiceStatus, String> {
    let state = engine_state.read().await;
    match state.as_ref() {
        Some(engine) => {
            let usage_stats = engine.get_embedding_service_stats().await;
            // Note: cache_size method not available, use 0 as placeholder
            let cache_size = 0;

            Ok(EmbeddingServiceStatus {
                provider: EmbeddingProvider::OpenAI, // Default to OpenAI (local fallback removed)
                model_name: engine.config().model_name.clone(),
                dimension: engine.config().dimension,
                available: engine.is_model_available(),
                usage_stats,
                cache_size,
            })
        }
        None => Err("No embedding engine configured".to_string()),
    }
}

#[tauri::command]
pub async fn clear_embedding_cache(engine_state: EmbeddingEngineState<'_>) -> Result<bool, String> {
    let state = engine_state.read().await;
    match state.as_ref() {
        Some(engine) => {
            engine.clear_embedding_cache().await;
            info!("Embedding cache cleared");
            Ok(true)
        }
        None => Err("No embedding engine configured".to_string()),
    }
}

#[tauri::command]
pub async fn test_embedding_connection(
    provider: EmbeddingProvider,
    model_name: String,
    api_key: Option<String>,
) -> Result<bool, String> {
    debug!("Testing embedding connection: provider={:?}", provider);

    let service_config = EmbeddingServiceConfig {
        provider,
        api_key,
        model_name,
        dimension: 1536, // API model dimension for testing
        max_tokens: 100, // Small token limit for testing
        batch_size: 1,
        timeout_seconds: 10,
    };

    let embedding_config = EmbeddingConfig {
        model_name: service_config.model_name.clone(),
        dimension: service_config.dimension,
        max_length: service_config.max_tokens,
        chunk_size: 1000,
        chunk_overlap: 200,
    };

    match EmbeddingEngine::new_with_service(embedding_config, service_config).await {
        Ok(_engine) => {
            // For now, just return true since we can create the engine
            // TODO: Add method to EmbeddingEngine to test connection
            info!("Embedding connection test successful - engine created");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to create embedding engine for testing: {}", e);
            Err(format!("Failed to create embedding engine: {}", e))
        }
    }
}

// Helper function to get available embedding providers
#[tauri::command]
pub async fn get_available_embedding_providers() -> Result<Vec<String>, String> {
    Ok(vec![
        "OpenAI".to_string(),
        "OpenRouter".to_string(),
        // "Local" and "Fallback" removed for system stability
    ])
}

// Helper function to get recommended models for each provider
#[tauri::command]
pub async fn get_recommended_models(
    provider: EmbeddingProvider,
) -> Result<Vec<HashMap<String, String>>, String> {
    let models = match provider {
        EmbeddingProvider::OpenAI => vec![
            HashMap::from([
                ("name".to_string(), "text-embedding-ada-002".to_string()),
                (
                    "description".to_string(),
                    "Most capable embedding model".to_string(),
                ),
                ("dimension".to_string(), "1536".to_string()),
            ]),
            HashMap::from([
                ("name".to_string(), "text-embedding-3-small".to_string()),
                (
                    "description".to_string(),
                    "Smaller, faster embedding model".to_string(),
                ),
                ("dimension".to_string(), "1536".to_string()),
            ]),
            HashMap::from([
                ("name".to_string(), "text-embedding-3-large".to_string()),
                (
                    "description".to_string(),
                    "Largest embedding model".to_string(),
                ),
                ("dimension".to_string(), "3072".to_string()),
            ]),
        ],
        EmbeddingProvider::OpenRouter => vec![HashMap::from([
            ("name".to_string(), "text-embedding-ada-002".to_string()),
            (
                "description".to_string(),
                "OpenAI embedding via OpenRouter".to_string(),
            ),
            ("dimension".to_string(), "1536".to_string()),
        ])],
        // Local and Fallback providers removed for safety
    };

    Ok(models)
}
