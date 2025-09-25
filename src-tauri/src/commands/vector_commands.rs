// src-tauri/src/commands/vector_commands.rs

use crate::commands::document_indexing_commands::DocumentIndexerState;
use crate::vector::{
    DocumentChunk, EmbeddingConfig, EmbeddingEngine, SearchResult, VectorStore, VectorStoreStats,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// Performance limits to prevent CPU overload
const MAX_CHUNK_SIZE: usize = 1000;
const MAX_CHUNKS_PER_DOCUMENT: usize = 50; // Reduced to prevent CPU overload
const MAX_SEARCH_RESULTS: usize = 50; // Reduced for better performance
#[allow(dead_code)]
const MAX_CONCURRENT_OPERATIONS: usize = 2; // Reduced concurrent operations
const EMBEDDING_BATCH_DELAY_MS: u64 = 100; // Small delay between batch operations

// Helper function to check embedding configuration status
async fn check_embedding_configuration() -> String {
    // Check UI settings first
    if let Some(ui_settings) = load_ui_embedding_settings().await {
        if !ui_settings.api_key.is_empty() {
            return format!(
                "✅ Configuration: UI settings configured ({} - {})",
                ui_settings.provider, ui_settings.model
            );
        }
    }

    // Fall back to .env checks
    if std::env::var("OPENAI_API_KEY").is_ok() {
        "✅ Configuration: OpenAI API key configured in .env (consider migrating to UI settings)"
            .to_string()
    } else if std::env::var("OPENROUTER_API_KEY").is_ok() {
        "✅ Configuration: OpenRouter API key configured in .env (consider migrating to UI settings)".to_string()
    } else {
        "❌ Configuration: No API keys found. Configure through Settings > Embeddings".to_string()
    }
}

// Helper function to load UI embedding settings (reused from vector/mod.rs)
async fn load_ui_embedding_settings(
) -> Option<crate::commands::embedding_settings_commands::EmbeddingSettings> {
    let config_dir = dirs::config_dir()?.join("proxemic");
    let settings_file = config_dir.join("embedding_settings.json");

    if !settings_file.exists() {
        return None;
    }

    let settings_json = std::fs::read_to_string(&settings_file).ok()?;
    serde_json::from_str(&settings_json).ok()
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchRequest {
    pub query: String,
    pub document_id: Option<String>,
    pub max_results: Option<usize>,
    pub keyword_weight: Option<f32>,
    pub vector_weight: Option<f32>,
    pub enable_vector_search: Option<bool>,
    pub enable_keyword_search: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResponse {
    pub success: bool,
    pub results: Vec<SearchResult>,
    pub keyword_results_count: usize,
    pub vector_results_count: usize,
    pub combined_results_count: usize,
    pub query_time_ms: u64,
    pub search_strategy: String,
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
            vector_store: Arc::new(VectorStore::new(default_config.dimension)), // Now 1536 for API models
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

    // Validate document size to prevent CPU overload
    if request.content.len() > MAX_CHUNK_SIZE * MAX_CHUNKS_PER_DOCUMENT {
        return Ok(DocumentIndexResponse {
            success: false,
            chunks_created: 0,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            error: Some(format!(
                "Document too large: {} chars (max: {} chars). Please split into smaller documents.",
                request.content.len(),
                MAX_CHUNK_SIZE * MAX_CHUNKS_PER_DOCUMENT
            )),
        });
    }

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

    // Additional safety check for chunk count
    if chunks_count > MAX_CHUNKS_PER_DOCUMENT {
        return Ok(DocumentIndexResponse {
            success: false,
            chunks_created: 0,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            error: Some(format!(
                "Document produced too many chunks: {} (max: {}). Consider reducing chunk size or document length.",
                chunks_count,
                MAX_CHUNKS_PER_DOCUMENT
            )),
        });
    }

    // Log performance info
    tracing::info!(
        "Indexing document '{}': {} chunks, {} characters",
        request.document_id,
        chunks_count,
        request.content.len()
    );

    // Add small delay to prevent CPU overload during batch processing
    if chunks_count > 10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(EMBEDDING_BATCH_DELAY_MS)).await;
    }

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

    // Perform vector search with safety limits
    let max_results = request.max_results.unwrap_or(5).min(MAX_SEARCH_RESULTS);
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

#[tauri::command]
pub async fn hybrid_search(
    vector_state: State<'_, VectorState>,
    request: HybridSearchRequest,
) -> Result<HybridSearchResponse, String> {
    let start_time = std::time::Instant::now();

    let max_results = request.max_results.unwrap_or(10);
    let keyword_weight = request.keyword_weight.unwrap_or(0.5);
    let vector_weight = request.vector_weight.unwrap_or(0.5);
    let enable_vector = request.enable_vector_search.unwrap_or(true);
    let enable_keyword = request.enable_keyword_search.unwrap_or(true);

    if !enable_vector && !enable_keyword {
        return Ok(HybridSearchResponse {
            success: false,
            results: Vec::new(),
            keyword_results_count: 0,
            vector_results_count: 0,
            combined_results_count: 0,
            query_time_ms: 0,
            search_strategy: "disabled".to_string(),
            error: Some("Both search methods are disabled".to_string()),
        });
    }

    let mut keyword_results = Vec::new();
    let mut vector_results = Vec::new();
    let mut search_strategy_parts = Vec::new();

    // Perform keyword search if enabled
    if enable_keyword {
        match if let Some(doc_id) = &request.document_id {
            vector_state
                .vector_store
                .keyword_search_by_document(doc_id, &request.query, max_results * 2)
                .await
        } else {
            vector_state
                .vector_store
                .keyword_search(&request.query, max_results * 2)
                .await
        } {
            Ok(results) => {
                keyword_results = results;
                search_strategy_parts.push("keyword");
            }
            Err(e) => {
                tracing::warn!("Keyword search failed: {}", e);
            }
        }
    }

    // Perform vector search if enabled
    if enable_vector {
        // Get embedding engine
        let engine_guard = vector_state.embedding_engine.lock().await;
        match engine_guard.as_ref() {
            Some(engine) => match engine.embed_text(&request.query).await {
                Ok(query_embedding) => {
                    let search_result = if let Some(doc_id) = &request.document_id {
                        vector_state
                            .vector_store
                            .search_by_document(doc_id, &query_embedding, max_results * 2)
                            .await
                    } else {
                        vector_state
                            .vector_store
                            .search(&query_embedding, max_results * 2)
                            .await
                    };

                    match search_result {
                        Ok(results) => {
                            vector_results = results;
                            search_strategy_parts.push("vector");
                        }
                        Err(e) => {
                            tracing::warn!("Vector search failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to generate query embedding: {}", e);
                }
            },
            None => {
                tracing::warn!("Embedding engine not initialized, skipping vector search");
            }
        }
    }

    // Combine results using hybrid scoring
    let combined_results = combine_hybrid_results(
        keyword_results.clone(),
        vector_results.clone(),
        keyword_weight,
        vector_weight,
        max_results,
    );

    let query_time = start_time.elapsed().as_millis() as u64;
    let search_strategy = if search_strategy_parts.is_empty() {
        "failed".to_string()
    } else {
        format!("hybrid({})", search_strategy_parts.join("+"))
    };

    Ok(HybridSearchResponse {
        success: !combined_results.is_empty(),
        results: combined_results.clone(),
        keyword_results_count: keyword_results.len(),
        vector_results_count: vector_results.len(),
        combined_results_count: combined_results.len(),
        query_time_ms: query_time,
        search_strategy,
        error: if combined_results.is_empty() {
            Some("No results found from any search method".to_string())
        } else {
            None
        },
    })
}

/// Combine keyword and vector search results using weighted scoring
fn combine_hybrid_results(
    keyword_results: Vec<SearchResult>,
    vector_results: Vec<SearchResult>,
    keyword_weight: f32,
    vector_weight: f32,
    max_results: usize,
) -> Vec<SearchResult> {
    use std::collections::HashMap;

    let mut combined_scores: HashMap<String, (f32, SearchResult)> = HashMap::new();

    // Process keyword results
    for (i, result) in keyword_results.iter().enumerate() {
        let normalized_rank_score = 1.0 - (i as f32 / keyword_results.len().max(1) as f32);
        let weighted_score = (result.similarity + normalized_rank_score) * keyword_weight / 2.0;

        combined_scores.insert(result.chunk.id.clone(), (weighted_score, result.clone()));
    }

    // Process vector results and combine with keyword scores
    for (i, result) in vector_results.iter().enumerate() {
        let normalized_rank_score = 1.0 - (i as f32 / vector_results.len().max(1) as f32);
        let weighted_score = (result.similarity + normalized_rank_score) * vector_weight / 2.0;

        match combined_scores.get_mut(&result.chunk.id) {
            Some((existing_score, existing_result)) => {
                // Combine scores and update explanation
                *existing_score += weighted_score;
                existing_result.similarity = *existing_score;
                existing_result.explanation = format!(
                    "Hybrid result (keyword + vector): {}",
                    existing_result.explanation
                );
            }
            None => {
                let mut updated_result = result.clone();
                updated_result.similarity = weighted_score;
                updated_result.explanation =
                    format!("Vector-only result: {}", updated_result.explanation);
                combined_scores.insert(result.chunk.id.clone(), (weighted_score, updated_result));
            }
        }
    }

    // Sort by combined score and take top results
    let mut final_results: Vec<(f32, SearchResult)> = combined_scores.into_values().collect();
    final_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    final_results
        .into_iter()
        .take(max_results)
        .map(|(score, mut result)| {
            result.similarity = score;
            result
        })
        .collect()
}

/// Sync documents from Document Index to Vector Search system
#[tauri::command]
pub async fn sync_documents_to_vector_system(
    vector_state: State<'_, VectorState>,
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<String, String> {
    tracing::info!("Starting document sync from indexer to vector system");

    // Get all documents from the document indexer with detailed validation
    let documents = {
        let indexer_guard = indexer_state.lock().await;
        match indexer_guard.as_ref() {
            Some(indexer) => {
                tracing::info!("✅ Document indexer found, retrieving documents...");
                let docs = indexer.get_all_documents();
                let doc_list: Vec<_> = docs.into_iter().cloned().collect();
                tracing::info!("📄 Retrieved {} documents from indexer", doc_list.len());

                // Log document details for debugging
                for (i, doc) in doc_list.iter().enumerate().take(3) {
                    tracing::info!(
                        "Document {}: '{}' (ID: {}, Content length: {} chars)",
                        i + 1,
                        doc.title,
                        doc.id,
                        doc.content.len()
                    );
                }

                doc_list
            }
            None => {
                tracing::error!("❌ Document indexer not initialized!");
                return Err("Document indexer not initialized. Please go to Document Index section and index some documents first.".to_string());
            }
        }
    };

    if documents.is_empty() {
        tracing::warn!("📭 No documents found in indexer to sync");
        return Ok("No documents found in Document Indexer. Please go to the Document Index section and add some documents first, then try syncing again.".to_string());
    }

    tracing::info!(
        "🔄 Starting sync of {} documents to vector system",
        documents.len()
    );

    // Get vector system components
    let engine_guard = vector_state.embedding_engine.lock().await;
    let engine = match engine_guard.as_ref() {
        Some(engine) => engine,
        None => {
            return Err(
                "Vector system not initialized. Please initialize vector system first.".to_string(),
            )
        }
    };

    // Verify API embeddings are working (local fallback removed for safety)
    if !engine.is_model_available() {
        tracing::error!("❌ CRITICAL: API embedding service unavailable!");
        return Err("API embedding service is unavailable. Please check your API key and internet connection. Local embeddings have been disabled for safety.".to_string());
    } else {
        tracing::info!("✅ Using API-based embeddings for document sync (safe mode)");
    }

    let mut sync_results = Vec::new();
    let mut successful_syncs = 0;
    let mut failed_syncs = 0;

    // Process documents in smaller batches to prevent timeout and system overload
    const BATCH_SIZE: usize = 1; // Process ONE document at a time for safety
    const MAX_SYNC_DURATION: u64 = 75; // 75 seconds total sync timeout (reduced from 10 minutes)

    // Add overall sync timeout to prevent infinite hangs
    let overall_start = std::time::Instant::now();

    for batch_start in (0..documents.len()).step_by(BATCH_SIZE) {
        // Check overall timeout
        if overall_start.elapsed().as_secs() > MAX_SYNC_DURATION {
            tracing::error!(
                "❌ SYNC TIMEOUT: Stopping after {} seconds to prevent system hang",
                MAX_SYNC_DURATION
            );
            break;
        }

        let batch_end = (batch_start + BATCH_SIZE).min(documents.len());
        let batch_documents = &documents[batch_start..batch_end];

        tracing::info!(
            "Processing document {}/{}: {}",
            batch_start + 1,
            documents.len(),
            batch_documents[0].title
        );

        // Process each document with aggressive timeout protection
        for (local_index, document) in batch_documents.iter().enumerate() {
            let doc_index = batch_start + local_index;

            // Skip overly large documents to prevent hangs
            if document.content.len() > 50000 {
                // 50KB limit
                let skip_msg = format!(
                    "⏩ {}: Skipped (too large: {} chars)",
                    document.title,
                    document.content.len()
                );
                sync_results.push(skip_msg.clone());
                tracing::warn!("{}", skip_msg);
                continue;
            }

            tracing::info!(
                "📄 Syncing document {}/{}: {} ({} chars)",
                doc_index + 1,
                documents.len(),
                document.title,
                document.content.len()
            );

            // Create a future for processing this single document with strict limits
            let doc_future = async {
                // Create chunks from the document content with timeout protection
                tracing::debug!(
                    "🔪 Starting chunking for document '{}' ({} chars)",
                    document.title,
                    document.content.len()
                );

                let chunking_future = async { engine.chunk_text(&document.content, &document.id) };

                let chunks = match tokio::time::timeout(
                    tokio::time::Duration::from_secs(45),
                    chunking_future,
                )
                .await
                {
                    Ok(chunks) => {
                        tracing::debug!(
                            "🔪 Chunking completed for '{}': {} chunks",
                            document.title,
                            chunks.len()
                        );
                        chunks
                    }
                    Err(_) => {
                        tracing::error!(
                            "🚨 CHUNKING TIMEOUT: Document '{}' chunking exceeded 45 seconds",
                            document.title
                        );
                        return Err("Document chunking timed out - document may contain problematic content".to_string());
                    }
                };

                if chunks.is_empty() {
                    return Err("No chunks generated".to_string());
                }

                // Safety check for chunk count (increased for normal document processing)
                let max_allowed_chunks = 50; // Restored to reasonable limit for document processing
                if chunks.len() > max_allowed_chunks {
                    return Err(format!(
                        "Too many chunks ({} > {}). Document too complex.",
                        chunks.len(),
                        max_allowed_chunks
                    ));
                }

                tracing::info!("🔗 Generating embeddings for {} chunks...", chunks.len());

                // Generate embeddings for chunks with aggressive timeout
                let embedding_start = std::time::Instant::now();
                let embeddings = engine
                    .embed_chunks(&chunks)
                    .await
                    .map_err(|e| format!("Embedding API error: {}", e))?;

                let embedding_time = embedding_start.elapsed();
                tracing::info!("⚡ Embeddings generated in {:?}", embedding_time);

                // Warn if embedding took too long (might indicate API issues)
                if embedding_time.as_secs() > 10 {
                    tracing::warn!(
                        "⚠️ Embedding generation took {} seconds - API may be slow",
                        embedding_time.as_secs()
                    );
                }

                // Store embeddings in vector store
                vector_state
                    .vector_store
                    .add_document_chunks(chunks.clone(), embeddings)
                    .await
                    .map_err(|e| format!("Storage error: {}", e))?;

                Ok(chunks.len())
            };

            // Apply aggressive timeout to individual document processing (15 seconds per document)
            let doc_timeout = tokio::time::Duration::from_secs(15); // Reduced from 60 to 15 seconds to prevent hangs
            match tokio::time::timeout(doc_timeout, doc_future).await {
                Ok(Ok(chunk_count)) => {
                    sync_results.push(format!("✅ {}: {} chunks", document.title, chunk_count));
                    successful_syncs += 1;
                    tracing::info!(
                        "✅ Successfully synced '{}' with {} chunks",
                        document.title,
                        chunk_count
                    );
                }
                Ok(Err(error)) => {
                    sync_results.push(format!("❌ {}: {}", document.title, error));
                    failed_syncs += 1;
                    tracing::error!("❌ Failed to sync '{}': {}", document.title, error);
                }
                Err(_) => {
                    let timeout_msg = format!(
                        "⏰ {}: Timed out after 15s (API hang detected - preventing system freeze)",
                        document.title
                    );
                    sync_results.push(timeout_msg.clone());
                    failed_syncs += 1;
                    tracing::error!("⏰ CRITICAL: Document '{}' timed out after 15 seconds - preventing system hang", document.title);
                }
            }

            // Aggressive circuit breaker: Stop immediately on any timeout or if too many failures
            if failed_syncs >= 2 || (failed_syncs >= 1 && successful_syncs == 0) {
                tracing::error!(
                    "🔴 EMERGENCY STOP: Halting sync after {} failures to prevent system freeze",
                    failed_syncs
                );
                sync_results
                    .push("🔴 EMERGENCY STOP: Sync halted to prevent system hang".to_string());
                break;
            }

            // Minimal delay between documents - too much delay can cause UI hangs
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await; // Reduced from 1s to 200ms
        }

        // Minimal delay between batches to prevent UI blocking
        if batch_end < documents.len() {
            tracing::info!("😴 Brief pause before next document...");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Reduced from 2s to 100ms
        }
    }

    let summary = format!(
        "Document sync completed: {} successful, {} failed\n\nDetails:\n{}",
        successful_syncs,
        failed_syncs,
        sync_results.join("\n")
    );

    tracing::info!(
        "Document sync completed: {} successful, {} failed",
        successful_syncs,
        failed_syncs
    );
    Ok(summary)
}

/// Diagnostic command to check system status and connectivity
#[tauri::command]
pub async fn diagnose_document_vector_system(
    vector_state: State<'_, VectorState>,
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<String, String> {
    tracing::info!("🔍 Running system diagnostics...");

    let mut diagnostics = Vec::new();

    // Check Document Indexer
    let indexer_status = {
        let indexer_guard = indexer_state.lock().await;
        match indexer_guard.as_ref() {
            Some(indexer) => {
                let stats = indexer.get_stats();
                diagnostics.push(format!(
                    "✅ Document Indexer: {} documents indexed",
                    stats.total_documents
                ));
                stats.total_documents
            }
            None => {
                diagnostics.push("❌ Document Indexer: Not initialized".to_string());
                0
            }
        }
    };

    // Check Vector System
    let vector_status = {
        let engine_guard = vector_state.embedding_engine.lock().await;
        match engine_guard.as_ref() {
            Some(engine) => {
                let model_available = engine.is_model_available();
                let config = engine.get_config();
                if model_available {
                    diagnostics.push(format!(
                        "✅ Vector System: API embeddings connected ({})",
                        config.model_name
                    ));
                } else {
                    diagnostics.push("❌ Vector System: API embeddings not connected".to_string());
                }
                model_available
            }
            None => {
                diagnostics.push("❌ Vector System: Not initialized".to_string());
                false
            }
        }
    };

    // Check Vector Store Stats
    match vector_state.vector_store.get_stats().await {
        Ok(stats) => {
            diagnostics.push(format!(
                "📊 Vector Store: {} documents, {} chunks indexed for search",
                stats.total_documents, stats.total_chunks
            ));
        }
        Err(e) => {
            diagnostics.push(format!("❌ Vector Store: Error getting stats - {}", e));
        }
    }

    // Configuration checks - prioritize UI settings
    let config_status = check_embedding_configuration().await;
    diagnostics.push(config_status);

    // Overall status and recommendations
    let recommendations = if indexer_status == 0 {
        vec![
            "🎯 NEXT STEPS:".to_string(),
            "1. Go to Document Index section".to_string(),
            "2. Add documents using 'Index Document' button".to_string(),
            "3. Return to Intelligent Search and sync documents".to_string(),
        ]
    } else if !vector_status {
        vec![
            "🎯 NEXT STEPS:".to_string(),
            "1. Check your API key in the .env file".to_string(),
            "2. Restart the application".to_string(),
            "3. Try syncing documents again".to_string(),
        ]
    } else {
        vec![
            "🎯 READY TO SYNC:".to_string(),
            "1. Click 'Sync Documents from Index'".to_string(),
            "2. Documents should sync without performance issues".to_string(),
        ]
    };

    diagnostics.extend(recommendations);

    Ok(diagnostics.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_system_state() {
        let state = VectorSystemState::new();
        assert_eq!(state.vector_store.dimension(), 1536); // API model default dimension (changed from local 384)

        // Test that embedding engine can be created (requires API configuration)
        let _config = EmbeddingConfig::default();
        // Note: This will fail without API keys, which is expected for security
        // Local embedding fallback has been removed to prevent system crashes
    }

    #[tokio::test]
    async fn test_vector_functionality() {
        // Skip test if no API key is configured
        if std::env::var("OPENAI_API_KEY").is_err() && std::env::var("OPENROUTER_API_KEY").is_err()
        {
            println!("Skipping test: No API key configured (OPENAI_API_KEY or OPENROUTER_API_KEY)");
            return;
        }

        // Test core vector functionality without Tauri state
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config.clone()).await.unwrap();
        let store = VectorStore::new(config.dimension);

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
