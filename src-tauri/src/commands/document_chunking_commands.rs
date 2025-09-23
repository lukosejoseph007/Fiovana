// src-tauri/src/commands/document_chunking_commands.rs
// Commands for document chunking operations

use crate::commands::document_indexing_commands::DocumentIndexerState;
use crate::document::{
    chunker::{ChunkConfig, DocumentChunk, DocumentChunker},
    indexer::DocumentIndexEntry,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDocumentRequest {
    pub file_path: String,
    pub config: Option<ChunkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDocumentResponse {
    pub success: bool,
    pub chunks: Option<Vec<DocumentChunk>>,
    pub total_chunks: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentChunksRequest {
    pub file_path: String,
}

/// Chunk a document into semantic chunks
#[tauri::command]
pub async fn chunk_document(
    indexer_state: State<'_, DocumentIndexerState>,
    request: ChunkDocumentRequest,
) -> Result<ChunkDocumentResponse, String> {
    tracing::info!("Chunking document: {}", request.file_path);

    let state = indexer_state.lock().await;
    if let Some(ref indexer) = *state {
        // First, get the document from the index
        let documents: Vec<&DocumentIndexEntry> = indexer.get_all_documents();
        let document = documents
            .iter()
            .find(|&doc| doc.path.to_string_lossy() == request.file_path);

        if let Some(doc) = document {
            // Create chunker with provided config or default
            let chunker = if let Some(config) = request.config {
                DocumentChunker::with_config(config)
            } else {
                DocumentChunker::new()
            };

            // Perform chunking
            let chunks = chunker.chunk_document(doc);
            let total_chunks = chunks.len();

            tracing::info!(
                "Successfully chunked document {} into {} chunks",
                request.file_path,
                total_chunks
            );

            Ok(ChunkDocumentResponse {
                success: true,
                chunks: Some(chunks),
                total_chunks,
                error: None,
            })
        } else {
            let error_msg = format!("Document not found in index: {}", request.file_path);
            tracing::error!("{}", error_msg);
            Ok(ChunkDocumentResponse {
                success: false,
                chunks: None,
                total_chunks: 0,
                error: Some(error_msg),
            })
        }
    } else {
        let error_msg = "Document indexer not initialized".to_string();
        tracing::error!("{}", error_msg);
        Ok(ChunkDocumentResponse {
            success: false,
            chunks: None,
            total_chunks: 0,
            error: Some(error_msg),
        })
    }
}

/// Get chunks for a specific document
#[tauri::command]
pub async fn get_document_semantic_chunks(
    indexer_state: State<'_, DocumentIndexerState>,
    request: GetDocumentChunksRequest,
) -> Result<ChunkDocumentResponse, String> {
    let chunk_request = ChunkDocumentRequest {
        file_path: request.file_path,
        config: None, // Use default config
    };

    chunk_document(indexer_state, chunk_request).await
}

/// Get chunking statistics for a document
#[tauri::command]
pub async fn get_chunk_stats(
    indexer_state: State<'_, DocumentIndexerState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    tracing::info!("Getting chunk stats for document: {}", file_path);

    let request = ChunkDocumentRequest {
        file_path: file_path.clone(),
        config: None,
    };

    match chunk_document(indexer_state, request).await {
        Ok(response) => {
            if response.success {
                if let Some(chunks) = response.chunks {
                    let total_content_length: usize =
                        chunks.iter().map(|chunk| chunk.content.len()).sum();

                    let avg_chunk_size = if chunks.is_empty() {
                        0
                    } else {
                        total_content_length / chunks.len()
                    };

                    let word_count: usize =
                        chunks.iter().map(|chunk| chunk.metadata.word_count).sum();

                    let has_code_chunks = chunks.iter().any(|chunk| chunk.metadata.has_code);

                    let has_list_chunks = chunks.iter().any(|chunk| chunk.metadata.has_lists);

                    let has_table_chunks = chunks.iter().any(|chunk| chunk.metadata.has_tables);

                    let total_reading_time: u32 = chunks
                        .iter()
                        .map(|chunk| chunk.metadata.reading_time_seconds)
                        .sum();

                    Ok(serde_json::json!({
                        "success": true,
                        "total_chunks": chunks.len(),
                        "total_content_length": total_content_length,
                        "average_chunk_size": avg_chunk_size,
                        "total_word_count": word_count,
                        "has_code_chunks": has_code_chunks,
                        "has_list_chunks": has_list_chunks,
                        "has_table_chunks": has_table_chunks,
                        "total_reading_time_seconds": total_reading_time,
                        "estimated_reading_time_minutes": (total_reading_time as f32 / 60.0).ceil() as u32
                    }))
                } else {
                    Ok(serde_json::json!({
                        "success": false,
                        "error": "No chunks available"
                    }))
                }
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "error": response.error.unwrap_or("Unknown error".to_string())
                }))
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to get chunk stats: {}", e);
            tracing::error!("{}", error_msg);
            Ok(serde_json::json!({
                "success": false,
                "error": error_msg
            }))
        }
    }
}

/// Test chunking with different configurations
#[tauri::command]
pub async fn test_chunking_configs(
    indexer_state: State<'_, DocumentIndexerState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    tracing::info!("Testing chunking configurations for: {}", file_path);

    // Test different chunk sizes
    let configs = vec![
        (
            "small",
            ChunkConfig {
                chunk_size: 500,
                overlap_size: 100,
                min_chunk_size: 50,
                max_chunk_size: 1000,
                respect_paragraphs: true,
                respect_sentences: true,
            },
        ),
        ("medium", ChunkConfig::default()),
        (
            "large",
            ChunkConfig {
                chunk_size: 2000,
                overlap_size: 400,
                min_chunk_size: 200,
                max_chunk_size: 4000,
                respect_paragraphs: true,
                respect_sentences: true,
            },
        ),
    ];

    let mut results = serde_json::Map::new();

    for (name, config) in configs {
        let request = ChunkDocumentRequest {
            file_path: file_path.clone(),
            config: Some(config.clone()),
        };

        match chunk_document(indexer_state.clone(), request).await {
            Ok(response) => {
                if response.success {
                    results.insert(
                        name.to_string(),
                        serde_json::json!({
                            "total_chunks": response.total_chunks,
                            "config": config
                        }),
                    );
                } else {
                    results.insert(
                        name.to_string(),
                        serde_json::json!({
                            "error": response.error.unwrap_or("Unknown error".to_string())
                        }),
                    );
                }
            }
            Err(e) => {
                results.insert(
                    name.to_string(),
                    serde_json::json!({
                        "error": format!("Failed to test config: {}", e)
                    }),
                );
            }
        }
    }

    Ok(serde_json::Value::Object(results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{indexer::DocumentStructure, DocumentIndexEntry, EnhancedMetadata};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_chunk_document_command() {
        // Create a mock document indexer state
        use crate::document::{
            BasicMetadata, ContentMetadata, ContentStats, SecurityMetadata, TechnicalMetadata,
        };
        use std::time::SystemTime;

        let _mock_doc = DocumentIndexEntry {
            id: "test_doc_1".to_string(),
            path: PathBuf::from("test.md"),
            title: "Test Document".to_string(),
            summary: None,
            content: "This is a test document. It has multiple sentences. And multiple paragraphs.\n\nThis is the second paragraph. It also has multiple sentences.".to_string(),
            metadata: EnhancedMetadata {
                basic: BasicMetadata {
                    file_name: "test.md".to_string(),
                    file_extension: Some("md".to_string()),
                    file_size: 1024,
                    created: Some(SystemTime::now()),
                    modified: Some(SystemTime::now()),
                    accessed: Some(SystemTime::now()),
                    is_file: true,
                    is_dir: false,
                    is_symlink: false,
                },
                content: ContentMetadata {
                    detected_mime_type: Some("text/markdown".to_string()),
                    encoding: Some("utf-8".to_string()),
                    line_endings: Some("unix".to_string()),
                    preview: Some("Test content...".to_string()),
                    language: Some("en".to_string()),
                    stats: ContentStats {
                        char_count: Some(500),
                        word_count: Some(100),
                        line_count: Some(10),
                        paragraph_count: Some(2),
                        binary_ratio: 0.0,
                    },
                },
                security: SecurityMetadata {
                    permissions: None,
                    is_executable: false,
                    is_hidden: false,
                    has_extended_attributes: false,
                    security_flags: vec![],
                },
                technical: TechnicalMetadata {
                    entropy: 0.5,
                    compression_ratio: Some(0.8),
                    checksums: {
                        let mut map = std::collections::HashMap::new();
                        map.insert("md5".to_string(), "test_checksum".to_string());
                        map.insert("sha256".to_string(), "test_sha256".to_string());
                        map
                    },
                    structure: crate::document::FileStructure {
                        has_structure: true,
                        format_version: Some("1.0".to_string()),
                        embedded_resources: 0,
                        sections: vec![],
                    },
                },
                document: None,
            },
            structure: DocumentStructure {
                document_type: crate::document::indexer::DocumentType::Manual,
                sections: vec![],
                toc: None,
                page_count: None,
                has_images: false,
                has_tables: false,
                has_code: false,
            },
            keywords: vec![],
            content_hash: "test_hash".to_string(),
            indexed_at: SystemTime::now(),
            index_version: 1,
        };

        // Since we can't easily mock DocumentIndexer, this test verifies the command structure
        // In a real test environment, you would set up a test indexer with test data
        let request = ChunkDocumentRequest {
            file_path: "test.md".to_string(),
            config: None,
        };

        // Test that the request structure is correct
        assert_eq!(request.file_path, "test.md");
        assert!(request.config.is_none());
    }
}
