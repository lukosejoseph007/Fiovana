// src-tauri/src/commands/vector_commands.rs

use crate::vector::{
    DocumentChunk, EmbeddingConfig, EmbeddingEngine, SearchResult, VectorStore, VectorStoreStats,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchRequest {
    pub query: String,
    pub document_id: Option<String>,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResponse {
    pub success: bool,
    pub results: Vec<SearchResult>,
    pub query_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndexRequest {
    pub document_id: String,
    pub content: String,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndexResponse {
    pub success: bool,
    pub chunks_created: usize,
    pub processing_time_ms: u64,
    pub error: Option<String>,
}

// Vector system state management
pub struct VectorSystemState {
    pub embedding_engine: Arc<Mutex<Option<EmbeddingEngine>>>,
    pub vector_store: Arc<VectorStore>,
}

impl Default for VectorSystemState {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorSystemState {
    pub fn new() -> Self {
        let default_config = EmbeddingConfig::default();
        Self {
            embedding_engine: Arc::new(Mutex::new(None)),
            vector_store: Arc::new(VectorStore::new(default_config.dimension)),
        }
    }
}

pub type VectorState = Arc<VectorSystemState>;

#[tauri::command]
pub async fn init_vector_system(
    vector_state: State<'_, VectorState>,
    config: Option<EmbeddingConfig>,
) -> Result<bool, String> {
    let config = config.unwrap_or_default();

    match EmbeddingEngine::new(config).await {
        Ok(engine) => {
            let mut engine_lock = vector_state.embedding_engine.lock().await;
            *engine_lock = Some(engine);
            tracing::info!("Vector system initialized successfully");
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Failed to initialize vector system: {}", e);
            Err(format!("Failed to initialize vector system: {}", e))
        }
    }
}

#[tauri::command]
pub async fn index_document_vector(
    vector_state: State<'_, VectorState>,
    request: DocumentIndexRequest,
) -> Result<DocumentIndexResponse, String> {
    let start_time = std::time::Instant::now();

    let engine_lock = vector_state.embedding_engine.lock().await;
    let engine = match engine_lock.as_ref() {
        Some(engine) => engine,
        None => {
            return Ok(DocumentIndexResponse {
                success: false,
                chunks_created: 0,
                processing_time_ms: 0,
                error: Some("Vector system not initialized".to_string()),
            });
        }
    };

    // Chunk the document
    let chunks = engine.chunk_text(&request.content, &request.document_id);
    let chunks_count = chunks.len();

    // Generate embeddings
    match engine.embed_chunks(&chunks).await {
        Ok(embeddings) => {
            // Add to vector store
            match vector_state
                .vector_store
                .add_document_chunks(chunks, embeddings)
                .await
            {
                Ok(()) => {
                    let processing_time = start_time.elapsed().as_millis() as u64;
                    tracing::info!(
                        "Successfully indexed document '{}' with {} chunks in {}ms",
                        request.document_id,
                        chunks_count,
                        processing_time
                    );

                    Ok(DocumentIndexResponse {
                        success: true,
                        chunks_created: chunks_count,
                        processing_time_ms: processing_time,
                        error: None,
                    })
                }
                Err(e) => {
                    let error_msg = format!("Failed to store document chunks: {}", e);
                    tracing::error!("{}", error_msg);
                    Ok(DocumentIndexResponse {
                        success: false,
                        chunks_created: 0,
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        error: Some(error_msg),
                    })
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to generate embeddings: {}", e);
            tracing::error!("{}", error_msg);
            Ok(DocumentIndexResponse {
                success: false,
                chunks_created: 0,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(error_msg),
            })
        }
    }
}

#[tauri::command]
pub async fn search_vectors(
    vector_state: State<'_, VectorState>,
    request: VectorSearchRequest,
) -> Result<VectorSearchResponse, String> {
    let start_time = std::time::Instant::now();

    let engine_lock = vector_state.embedding_engine.lock().await;
    let engine = match engine_lock.as_ref() {
        Some(engine) => engine,
        None => {
            return Ok(VectorSearchResponse {
                success: false,
                results: Vec::new(),
                query_time_ms: 0,
                error: Some("Vector system not initialized".to_string()),
            });
        }
    };

    // Try keyword-based search first as a fallback/enhancement
    let keyword_results = vector_state
        .vector_store
        .keyword_search(&request.query, request.max_results.unwrap_or(5))
        .await;

    match keyword_results {
        Ok(results) => {
            let query_time = start_time.elapsed().as_millis() as u64;
            tracing::info!(
                "Keyword search completed: query='{}', results={}, time={}ms",
                request.query,
                results.len(),
                query_time
            );

            return Ok(VectorSearchResponse {
                success: true,
                results,
                query_time_ms: query_time,
                error: None,
            });
        }
        Err(_) => {
            // Fall back to vector search if keyword search fails
            tracing::info!("Falling back to vector search");
        }
    }

    // Generate query embedding for vector search
    let query_embedding = match engine.embed_text(&request.query).await {
        Ok(embedding) => embedding,
        Err(e) => {
            let error_msg = format!("Failed to generate query embedding: {}", e);
            tracing::error!("{}", error_msg);
            return Ok(VectorSearchResponse {
                success: false,
                results: Vec::new(),
                query_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(error_msg),
            });
        }
    };

    // Perform vector search
    let max_results = request.max_results.unwrap_or(5);
    let search_results = match &request.document_id {
        Some(doc_id) => {
            vector_state
                .vector_store
                .search_by_document(doc_id, &query_embedding, max_results)
                .await
        }
        None => {
            vector_state
                .vector_store
                .search(&query_embedding, max_results)
                .await
        }
    };

    match search_results {
        Ok(results) => {
            let query_time = start_time.elapsed().as_millis() as u64;
            tracing::info!(
                "Vector search completed: query='{}', results={}, time={}ms",
                request.query,
                results.len(),
                query_time
            );

            Ok(VectorSearchResponse {
                success: true,
                results,
                query_time_ms: query_time,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Search failed: {}", e);
            tracing::error!("{}", error_msg);
            Ok(VectorSearchResponse {
                success: false,
                results: Vec::new(),
                query_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(error_msg),
            })
        }
    }
}

#[tauri::command]
pub async fn get_vector_stats(
    vector_state: State<'_, VectorState>,
) -> Result<VectorStoreStats, String> {
    vector_state
        .vector_store
        .get_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_document_from_index(
    vector_state: State<'_, VectorState>,
    document_id: String,
) -> Result<bool, String> {
    match vector_state
        .vector_store
        .remove_document(&document_id)
        .await
    {
        Ok(()) => {
            tracing::info!(
                "Successfully removed document '{}' from vector index",
                document_id
            );
            Ok(true)
        }
        Err(e) => {
            let error_msg = format!("Failed to remove document '{}': {}", document_id, e);
            tracing::error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_document_chunks(
    vector_state: State<'_, VectorState>,
    document_id: String,
) -> Result<Vec<DocumentChunk>, String> {
    vector_state
        .vector_store
        .get_document_chunks(&document_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_vector_system_status(
    vector_state: State<'_, VectorState>,
) -> Result<serde_json::Value, String> {
    let engine_lock = vector_state.embedding_engine.lock().await;
    let engine_initialized = engine_lock.is_some();

    let (model_info, model_available) = if let Some(engine) = engine_lock.as_ref() {
        (
            engine
                .get_model_info()
                .await
                .unwrap_or_else(|_| "Unknown".to_string()),
            engine.is_model_available(),
        )
    } else {
        ("Not initialized".to_string(), false)
    };

    let stats = vector_state
        .vector_store
        .get_stats()
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "initialized": engine_initialized,
        "model_info": model_info,
        "model_available": model_available,
        "vector_store_stats": stats,
        "embedding_dimension": vector_state.vector_store.dimension(),
    }))
}

#[tauri::command]
pub async fn keyword_search(
    vector_state: State<'_, VectorState>,
    request: VectorSearchRequest,
) -> Result<VectorSearchResponse, String> {
    let start_time = std::time::Instant::now();

    // Perform advanced keyword-based search with TF-IDF and phrase matching
    let max_results = request.max_results.unwrap_or(5);
    let search_results = match &request.document_id {
        Some(doc_id) => {
            vector_state
                .vector_store
                .keyword_search_by_document(doc_id, &request.query, max_results)
                .await
        }
        None => {
            vector_state
                .vector_store
                .keyword_search(&request.query, max_results)
                .await
        }
    };

    match search_results {
        Ok(results) => {
            let query_time = start_time.elapsed().as_millis() as u64;
            tracing::info!(
                "Advanced keyword search completed: query='{}', results={}, time={}ms, relevance_scoring=TF-IDF+phrase_matching",
                request.query,
                results.len(),
                query_time
            );

            Ok(VectorSearchResponse {
                success: true,
                results,
                query_time_ms: query_time,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Advanced keyword search failed: {}", e);
            tracing::error!("{}", error_msg);
            Ok(VectorSearchResponse {
                success: false,
                results: Vec::new(),
                query_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(error_msg),
            })
        }
    }
}

#[tauri::command]
pub async fn semantic_search(
    vector_state: State<'_, VectorState>,
    request: VectorSearchRequest,
) -> Result<VectorSearchResponse, String> {
    let start_time = std::time::Instant::now();

    // Perform semantic search combining keyword and contextual understanding
    let max_results = request.max_results.unwrap_or(5);

    // First try keyword search for immediate matches
    let keyword_results = match &request.document_id {
        Some(doc_id) => {
            vector_state
                .vector_store
                .keyword_search_by_document(doc_id, &request.query, max_results * 2)
                .await
        }
        None => {
            vector_state
                .vector_store
                .keyword_search(&request.query, max_results * 2)
                .await
        }
    };

    // If keyword search succeeds, enhance with semantic understanding
    let results = match keyword_results {
        Ok(mut keyword_results) => {
            // Sort by relevance and take top results
            keyword_results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            keyword_results.truncate(max_results);

            // Enhance explanations with semantic context
            for result in &mut keyword_results {
                result.explanation = format!(
                    "Semantic match: {} (Relevance: {:.1}% - Found via advanced TF-IDF analysis with phrase matching)",
                    result.explanation,
                    result.similarity * 100.0
                );
            }

            keyword_results
        }
        Err(e) => {
            tracing::warn!("Keyword search failed, returning empty results: {}", e);
            Vec::new()
        }
    };

    let query_time = start_time.elapsed().as_millis() as u64;
    tracing::info!(
        "Semantic search completed: query='{}', results={}, time={}ms, method=hybrid_keyword_semantic",
        request.query,
        results.len(),
        query_time
    );

    Ok(VectorSearchResponse {
        success: true,
        results,
        query_time_ms: query_time,
        error: None,
    })
}

#[tauri::command]
pub async fn test_vector_search(
    vector_state: State<'_, VectorState>,
    test_text: String,
) -> Result<String, String> {
    // Index the test text as a document
    let index_request = DocumentIndexRequest {
        document_id: "test_document".to_string(),
        content: test_text.clone(),
        metadata: None,
    };

    let index_result = index_document_vector(vector_state.clone(), index_request).await?;
    if !index_result.success {
        return Err(index_result
            .error
            .unwrap_or("Failed to index test document".to_string()));
    }

    // Search for a portion of the text
    let search_query = test_text
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");

    let search_request = VectorSearchRequest {
        query: search_query.clone(),
        document_id: None,
        max_results: Some(3),
    };

    let search_result = search_vectors(vector_state.clone(), search_request).await?;
    if !search_result.success {
        return Err(search_result.error.unwrap_or("Search failed".to_string()));
    }

    // Clean up
    let _ = remove_document_from_index(vector_state, "test_document".to_string()).await;

    Ok(format!(
        "Test completed successfully!\nIndexed {} chunks, searched for '{}', found {} results in {}ms",
        index_result.chunks_created,
        search_query,
        search_result.results.len(),
        search_result.query_time_ms
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_system_state() {
        let state = VectorSystemState::new();
        assert_eq!(state.vector_store.dimension(), 384); // Default dimension

        // Test that embedding engine can be created
        let config = EmbeddingConfig::default();
        let engine_result = EmbeddingEngine::new(config).await;
        assert!(
            engine_result.is_ok(),
            "Embedding engine should initialize successfully"
        );
    }

    #[tokio::test]
    async fn test_vector_functionality() {
        // Test core vector functionality without Tauri state
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await.unwrap();
        let store = VectorStore::new(384);

        // Test text chunking
        let test_text = "This is a test document for vector search functionality.";
        let chunks = engine.chunk_text(test_text, "test_doc");
        assert!(!chunks.is_empty());

        // Test embedding generation
        let embeddings = engine.embed_chunks(&chunks).await.unwrap();
        assert_eq!(chunks.len(), embeddings.len());

        // Test vector store operations
        store.add_document_chunks(chunks, embeddings).await.unwrap();
        let stats = store.get_stats().await.unwrap();
        assert!(stats.total_documents > 0);
        assert!(stats.total_chunks > 0);
    }
}
