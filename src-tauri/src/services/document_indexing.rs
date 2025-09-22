// src-tauri/src/services/document_indexing.rs
// Service for automatic document indexing when files are added/modified

use crate::commands::document_commands::parse_document;
use crate::commands::vector_commands::{DocumentIndexRequest, VectorState};
use crate::filesystem::watcher::FileEvent;
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Document indexing service that processes file events
pub struct DocumentIndexingService {
    /// Channel for receiving file events
    event_receiver: mpsc::UnboundedReceiver<FileEvent>,
    /// Vector store state for indexing
    vector_state: VectorState,
}

impl DocumentIndexingService {
    /// Create a new document indexing service
    pub fn new(
        event_receiver: mpsc::UnboundedReceiver<FileEvent>,
        vector_state: VectorState,
    ) -> Self {
        Self {
            event_receiver,
            vector_state,
        }
    }

    /// Start the document indexing service
    pub async fn start(mut self) {
        info!("Starting document indexing service");

        while let Some(file_event) = self.event_receiver.recv().await {
            if let Err(e) = self.process_file_event(file_event).await {
                error!("Failed to process file event for indexing: {}", e);
            }
        }

        info!("Document indexing service stopped");
    }

    /// Process a file event and index if it's a supported document
    async fn process_file_event(
        &self,
        file_event: FileEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = file_event.path();

        // Only process created and modified events for document files
        match file_event {
            FileEvent::Created(_) | FileEvent::Modified(_) => {
                if self.is_indexable_document(path) {
                    self.index_document(path).await?;
                }
            }
            FileEvent::Deleted(_) => {
                // Remove from index if it was indexed
                if self.is_indexable_document(path) {
                    self.remove_from_index(path).await?;
                }
            }
            FileEvent::Renamed { from, to } | FileEvent::Moved { from, to } => {
                // Remove old path and index new path if applicable
                if self.is_indexable_document(&from) {
                    self.remove_from_index(&from).await?;
                }
                if self.is_indexable_document(&to) {
                    self.index_document(&to).await?;
                }
            }
        }

        Ok(())
    }

    /// Check if a file should be indexed based on its extension
    fn is_indexable_document(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.ends_with(".docx")
            || path_str.ends_with(".pdf")
            || path_str.ends_with(".txt")
            || path_str.ends_with(".md")
            || path_str.ends_with(".markdown")
    }

    /// Index a document into the vector store
    async fn index_document(
        &self,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path_string = path.to_string_lossy().to_string();
        info!("Indexing document: {}", path_string);

        // Parse the document to extract content
        let parsed_content = match parse_document(path_string.clone()).await {
            Ok(response) => match response {
                crate::commands::document_commands::DocumentParseResponse::Docx {
                    content, ..
                } => {
                    // Extract text content from DOCX
                    content.text
                }
                crate::commands::document_commands::DocumentParseResponse::Pdf {
                    content, ..
                } => {
                    // Extract text content from PDF
                    content.text
                }
                crate::commands::document_commands::DocumentParseResponse::Error {
                    message,
                    ..
                } => {
                    warn!("Failed to parse document {}: {}", path_string, message);
                    return Ok(());
                }
            },
            Err(e) => {
                // For text files and other formats, try reading directly
                if path_string.ends_with(".txt")
                    || path_string.ends_with(".md")
                    || path_string.ends_with(".markdown")
                {
                    match std::fs::read_to_string(path) {
                        Ok(content) => content,
                        Err(e) => {
                            warn!("Failed to read text file {}: {}", path_string, e);
                            return Ok(());
                        }
                    }
                } else {
                    warn!("Failed to parse document {}: {}", path_string, e);
                    return Ok(());
                }
            }
        };

        // Create document ID from file path
        let document_id = path_string.clone();

        // Create indexing request
        let index_request = DocumentIndexRequest {
            document_id: document_id.clone(),
            content: parsed_content,
            metadata: Some({
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("file_path".to_string(), path_string.clone());
                metadata.insert(
                    "file_name".to_string(),
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                );
                if let Some(ext) = path.extension() {
                    metadata.insert("file_type".to_string(), ext.to_string_lossy().to_string());
                }
                metadata.insert("indexed_at".to_string(), chrono::Utc::now().to_rfc3339());
                metadata
            }),
        };

        // Index the document directly using vector state
        let start_time = std::time::Instant::now();

        let engine_lock = self.vector_state.embedding_engine.lock().await;
        let engine = match engine_lock.as_ref() {
            Some(engine) => engine,
            None => {
                warn!(
                    "Vector system not initialized, skipping indexing for: {}",
                    document_id
                );
                return Ok(());
            }
        };

        // Chunk the document
        let chunks = engine.chunk_text(&index_request.content, &index_request.document_id);
        let chunks_count = chunks.len();

        // Generate embeddings
        match engine.embed_chunks(&chunks).await {
            Ok(embeddings) => {
                // Add to vector store
                match self
                    .vector_state
                    .vector_store
                    .add_document_chunks(chunks, embeddings)
                    .await
                {
                    Ok(()) => {
                        let processing_time = start_time.elapsed().as_millis();
                        info!(
                            "Successfully indexed document: {} ({} chunks in {}ms)",
                            document_id, chunks_count, processing_time
                        );
                    }
                    Err(e) => {
                        error!("Failed to store document chunks for {}: {}", document_id, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to generate embeddings for {}: {}", document_id, e);
            }
        }

        Ok(())
    }

    /// Remove a document from the vector index
    async fn remove_from_index(
        &self,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let document_id = path.to_string_lossy().to_string();
        info!("Removing document from index: {}", document_id);

        // Remove document directly from vector store
        match self
            .vector_state
            .vector_store
            .remove_document(&document_id)
            .await
        {
            Ok(_) => {
                info!("Successfully removed document from index: {}", document_id);
            }
            Err(e) => {
                warn!(
                    "Failed to remove document from index {}: {}",
                    document_id, e
                );
            }
        }

        Ok(())
    }
}

/// Create a document indexing service channel
pub fn create_indexing_channel() -> (
    mpsc::UnboundedSender<FileEvent>,
    mpsc::UnboundedReceiver<FileEvent>,
) {
    mpsc::unbounded_channel()
}
