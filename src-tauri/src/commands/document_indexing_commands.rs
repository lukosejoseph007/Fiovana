// src-tauri/src/commands/document_indexing_commands.rs
// Commands for document indexing and search operations

use crate::document::{DocumentIndexEntry, DocumentIndexer, SearchFilter, SearchResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentRequest {
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocumentsRequest {
    pub query: String,
    pub filter: Option<SearchFilter>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSearchResponse {
    pub success: bool,
    pub results: Option<Vec<SearchResult>>,
    pub total_found: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatsResponse {
    pub total_documents: usize,
    pub total_keywords: usize,
    pub total_content_size: usize,
    pub index_version: u32,
}

// Global document indexer state
pub type DocumentIndexerState = Arc<Mutex<Option<DocumentIndexer>>>;

/// Initialize the document indexer
#[tauri::command]
pub async fn init_document_indexer(
    indexer_state: State<'_, DocumentIndexerState>,
    index_dir: Option<String>,
) -> Result<bool, String> {
    tracing::info!(
        "Initializing document indexer with index_dir: {:?}",
        index_dir
    );

    let index_path = if let Some(dir) = index_dir {
        PathBuf::from(dir)
    } else {
        // Default to workspace/.proxemic/index
        let mut default_path = PathBuf::from(".");
        default_path.push(".proxemic");
        default_path.push("index");
        default_path
    };

    tracing::info!("Using index path: {:?}", index_path);

    match DocumentIndexer::new(index_path.clone()) {
        Ok(indexer) => {
            let mut state = indexer_state.lock().await;
            *state = Some(indexer);
            tracing::info!(
                "Document indexer successfully initialized at {:?}",
                index_path
            );
            Ok(true)
        }
        Err(e) => {
            tracing::error!(
                "Failed to initialize document indexer at {:?}: {}",
                index_path,
                e
            );
            Err(format!("Failed to initialize document indexer: {}", e))
        }
    }
}

/// Index a document
#[tauri::command]
pub async fn index_document(
    indexer_state: State<'_, DocumentIndexerState>,
    request: IndexDocumentRequest,
) -> Result<DocumentIndexEntry, String> {
    tracing::info!("Indexing document: {}", request.file_path);

    let mut state = indexer_state.lock().await;
    if let Some(ref mut indexer) = *state {
        let path = PathBuf::from(&request.file_path);
        tracing::info!("Document indexer is available, indexing path: {:?}", path);
        match indexer.index_document(&path).await {
            Ok(entry) => {
                tracing::info!(
                    "Successfully indexed document: {} (title: {})",
                    request.file_path,
                    entry.title
                );
                Ok(entry)
            }
            Err(e) => {
                tracing::error!("Failed to index document {}: {}", request.file_path, e);
                Err(format!("Failed to index document: {}", e))
            }
        }
    } else {
        tracing::error!(
            "Document indexer not initialized when trying to index: {}",
            request.file_path
        );
        Err("Document indexer not initialized".to_string())
    }
}

/// Search documents
#[tauri::command]
pub async fn search_documents(
    indexer_state: State<'_, DocumentIndexerState>,
    request: SearchDocumentsRequest,
) -> Result<DocumentSearchResponse, String> {
    let state = indexer_state.lock().await;
    if let Some(ref indexer) = *state {
        match indexer.search(&request.query, request.filter) {
            Ok(mut results) => {
                // Apply limit if specified
                if let Some(limit) = request.limit {
                    results.truncate(limit);
                }

                let total_found = results.len();
                Ok(DocumentSearchResponse {
                    success: true,
                    results: Some(results),
                    total_found,
                    error: None,
                })
            }
            Err(e) => Ok(DocumentSearchResponse {
                success: false,
                results: None,
                total_found: 0,
                error: Some(format!("Search failed: {}", e)),
            }),
        }
    } else {
        Ok(DocumentSearchResponse {
            success: false,
            results: None,
            total_found: 0,
            error: Some("Document indexer not initialized".to_string()),
        })
    }
}

/// Get index statistics
#[tauri::command]
pub async fn get_index_stats(
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<IndexStatsResponse, String> {
    tracing::info!("Getting index statistics");

    let state = indexer_state.lock().await;
    if let Some(ref indexer) = *state {
        let stats = indexer.get_stats();
        tracing::info!(
            "Index stats: {} documents, {} keywords, {} bytes",
            stats.total_documents,
            stats.total_keywords,
            stats.total_content_size
        );
        Ok(IndexStatsResponse {
            total_documents: stats.total_documents,
            total_keywords: stats.total_keywords,
            total_content_size: stats.total_content_size,
            index_version: stats.index_version,
        })
    } else {
        tracing::error!("Document indexer not initialized when getting stats");
        Err("Document indexer not initialized".to_string())
    }
}

/// Get all indexed documents
#[tauri::command]
pub async fn get_all_documents(
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<Vec<DocumentIndexEntry>, String> {
    let state = indexer_state.lock().await;
    if let Some(ref indexer) = *state {
        let documents = indexer.get_all_documents();
        Ok(documents.into_iter().cloned().collect())
    } else {
        Err("Document indexer not initialized".to_string())
    }
}

/// Get detailed document information by path
#[tauri::command]
pub async fn get_document_details(
    indexer_state: State<'_, DocumentIndexerState>,
    file_path: String,
) -> Result<Option<DocumentIndexEntry>, String> {
    tracing::info!("Getting document details for: {}", file_path);

    let state = indexer_state.lock().await;
    if let Some(ref indexer) = *state {
        let documents = indexer.get_all_documents();
        let found_doc = documents
            .iter()
            .find(|doc| doc.path.to_string_lossy() == file_path);

        if let Some(doc) = found_doc {
            tracing::info!(
                "Found document details for: {} (title: {})",
                file_path,
                doc.title
            );
            Ok(Some((*doc).clone()))
        } else {
            tracing::warn!("Document not found in index: {}", file_path);
            Ok(None)
        }
    } else {
        tracing::error!(
            "Document indexer not initialized when getting details for: {}",
            file_path
        );
        Err("Document indexer not initialized".to_string())
    }
}

/// Remove a document from the index
#[tauri::command]
pub async fn remove_document_from_indexer(
    indexer_state: State<'_, DocumentIndexerState>,
    document_id: String,
) -> Result<bool, String> {
    tracing::info!("Removing document from indexer: {}", document_id);

    let mut state = indexer_state.lock().await;
    if let Some(ref mut indexer) = *state {
        match indexer.remove_document(&document_id) {
            Ok(removed) => {
                if removed {
                    tracing::info!("Successfully removed document: {}", document_id);
                    // Persist the changes
                    if let Err(e) = indexer.save_index() {
                        tracing::error!("Failed to save index after removal: {}", e);
                        return Err(format!("Failed to save index after removal: {}", e));
                    }
                    Ok(true)
                } else {
                    tracing::warn!("Document not found for removal: {}", document_id);
                    Ok(false)
                }
            }
            Err(e) => {
                tracing::error!("Failed to remove document {}: {}", document_id, e);
                Err(format!("Failed to remove document: {}", e))
            }
        }
    } else {
        tracing::error!(
            "Document indexer not initialized when trying to remove: {}",
            document_id
        );
        Err("Document indexer not initialized".to_string())
    }
}

/// Clear all documents from the index
#[tauri::command]
pub async fn clear_document_index(
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<usize, String> {
    tracing::info!("Clearing all documents from index");

    let mut state = indexer_state.lock().await;
    if let Some(ref mut indexer) = *state {
        let documents = indexer.get_all_documents();
        let total_count = documents.len();
        let document_ids: Vec<String> = documents.iter().map(|doc| doc.id.clone()).collect();

        let mut removed_count = 0;
        for doc_id in document_ids {
            match indexer.remove_document(&doc_id) {
                Ok(true) => removed_count += 1,
                Ok(false) => {
                    tracing::warn!("Document {} was not found during bulk removal", doc_id)
                }
                Err(e) => tracing::error!("Failed to remove document {}: {}", doc_id, e),
            }
        }

        // Persist the changes
        if let Err(e) = indexer.save_index() {
            tracing::error!("Failed to save index after clearing: {}", e);
            return Err(format!("Failed to save index after clearing: {}", e));
        }

        tracing::info!(
            "Successfully cleared {} of {} documents from index",
            removed_count,
            total_count
        );
        Ok(removed_count)
    } else {
        tracing::error!("Document indexer not initialized when trying to clear");
        Err("Document indexer not initialized".to_string())
    }
}

/// Get relevant documents for AI context (internal function)
pub async fn get_relevant_documents_for_context(
    indexer_state: &DocumentIndexerState,
    query: &str,
    limit: usize,
) -> Result<Vec<DocumentIndexEntry>, String> {
    tracing::info!(
        "Getting relevant documents for context with query: '{}'",
        query
    );

    let state = indexer_state.lock().await;
    if let Some(ref indexer) = *state {
        tracing::info!("Document indexer is initialized, performing search");
        match indexer.search(query, None) {
            Ok(mut results) => {
                tracing::info!("Search returned {} results", results.len());
                results.truncate(limit);
                let documents: Vec<DocumentIndexEntry> =
                    results.into_iter().map(|r| r.document).collect();
                tracing::info!("Returning {} documents for AI context", documents.len());
                Ok(documents)
            }
            Err(e) => {
                tracing::error!("Search failed with error: {}", e);
                Err(format!("Failed to search for relevant documents: {}", e))
            }
        }
    } else {
        tracing::error!("Document indexer not initialized");
        Err("Document indexer not initialized".to_string())
    }
}
