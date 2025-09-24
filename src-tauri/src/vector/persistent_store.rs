// src-tauri/src/vector/persistent_store.rs
use super::{DocumentChunk, EmbeddingRecord, SearchResult, VectorStore, VectorStoreStats};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVectorConfig {
    pub storage_path: PathBuf,
    pub auto_save_interval_seconds: u64,
    pub max_memory_chunks: usize,
    pub enable_compression: bool,
    pub backup_count: usize,
}

impl Default for PersistentVectorConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from(".proxemic/vector_store"),
            auto_save_interval_seconds: 300, // 5 minutes
            max_memory_chunks: 10000,
            enable_compression: true,
            backup_count: 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct VectorStoreSnapshot {
    embeddings: HashMap<String, EmbeddingRecord>,
    chunks: HashMap<String, DocumentChunk>,
    document_index: HashMap<String, Vec<String>>,
    dimension: usize,
    created_at: chrono::DateTime<chrono::Utc>,
    version: String,
}

#[allow(dead_code)]
pub struct PersistentVectorStore {
    inner_store: VectorStore,
    config: PersistentVectorConfig,
    last_save: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
    dirty: Arc<RwLock<bool>>,
}

#[allow(dead_code)]
impl PersistentVectorStore {
    pub async fn new(dimension: usize, config: PersistentVectorConfig) -> Result<Self> {
        // Ensure storage directory exists
        if let Some(parent) = config.storage_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let inner_store = VectorStore::new(dimension);
        let store = Self {
            inner_store,
            config,
            last_save: Arc::new(RwLock::new(chrono::Utc::now())),
            dirty: Arc::new(RwLock::new(false)),
        };

        // Try to load existing data
        if let Err(e) = store.load().await {
            warn!("Failed to load existing vector store data: {}", e);
            info!("Starting with empty vector store");
        }

        // Start auto-save task
        store.start_auto_save_task().await;

        Ok(store)
    }

    pub async fn add_document_chunks(
        &self,
        chunks: Vec<DocumentChunk>,
        embeddings: Vec<EmbeddingRecord>,
    ) -> Result<()> {
        self.inner_store
            .add_document_chunks(chunks, embeddings)
            .await?;
        self.mark_dirty().await;
        Ok(())
    }

    pub async fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        self.inner_store.search(query_vector, k).await
    }

    pub async fn search_by_document(
        &self,
        document_id: &str,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<SearchResult>> {
        self.inner_store
            .search_by_document(document_id, query_vector, k)
            .await
    }

    pub async fn keyword_search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        self.inner_store.keyword_search(query, max_results).await
    }

    pub async fn keyword_search_by_document(
        &self,
        document_id: &str,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        self.inner_store
            .keyword_search_by_document(document_id, query, max_results)
            .await
    }

    pub async fn get_document_chunks(&self, document_id: &str) -> Result<Vec<DocumentChunk>> {
        self.inner_store.get_document_chunks(document_id).await
    }

    pub async fn remove_document(&self, document_id: &str) -> Result<()> {
        self.inner_store.remove_document(document_id).await?;
        self.mark_dirty().await;
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<VectorStoreStats> {
        self.inner_store.get_stats().await
    }

    pub async fn hybrid_search(
        &self,
        query: &str,
        query_vector: &[f32],
        k: usize,
        keyword_weight: f32,
        vector_weight: f32,
    ) -> Result<Vec<SearchResult>> {
        // Perform both keyword and vector searches
        let keyword_results = self.keyword_search(query, k * 2).await?;
        let vector_results = self.search(query_vector, k * 2).await?;

        // Combine and rank results
        let combined_results = self
            .combine_search_results(
                keyword_results,
                vector_results,
                keyword_weight,
                vector_weight,
                k,
            )
            .await;

        Ok(combined_results)
    }

    pub async fn get_storage_info(&self) -> Result<PersistentStorageInfo> {
        let last_save = *self.last_save.read().await;
        let is_dirty = *self.dirty.read().await;
        let stats = self.get_stats().await?;

        let storage_size = if self.config.storage_path.exists() {
            tokio::fs::metadata(&self.config.storage_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        Ok(PersistentStorageInfo {
            storage_path: self.config.storage_path.clone(),
            last_save,
            is_dirty,
            storage_size_bytes: storage_size,
            total_chunks: stats.total_chunks,
            total_documents: stats.total_documents,
            auto_save_interval: self.config.auto_save_interval_seconds,
        })
    }

    pub async fn force_save(&self) -> Result<()> {
        self.save().await?;
        debug!("Vector store manually saved to disk");
        Ok(())
    }

    pub async fn optimize(&self) -> Result<OptimizationResult> {
        let start_time = chrono::Utc::now();
        let stats_before = self.get_stats().await?;

        // Compact and reorganize data structures
        // For now, this is a placeholder - in a real implementation,
        // we might defragment the vector space, prune unused embeddings, etc.

        let stats_after = self.get_stats().await?;
        let duration = chrono::Utc::now() - start_time;

        Ok(OptimizationResult {
            duration_ms: duration.num_milliseconds() as u64,
            chunks_before: stats_before.total_chunks,
            chunks_after: stats_after.total_chunks,
            memory_before: stats_before.memory_usage_estimate,
            memory_after: stats_after.memory_usage_estimate,
            storage_compacted: false, // Placeholder
        })
    }

    // Private methods
    async fn mark_dirty(&self) {
        let mut dirty = self.dirty.write().await;
        *dirty = true;
    }

    async fn save(&self) -> Result<()> {
        let embeddings = self.inner_store.embeddings.read().await;
        let chunks = self.inner_store.chunks.read().await;
        let document_index = self.inner_store.document_index.read().await;

        let snapshot = VectorStoreSnapshot {
            embeddings: embeddings.clone(),
            chunks: chunks.clone(),
            document_index: document_index.clone(),
            dimension: self.inner_store.dimension(),
            created_at: chrono::Utc::now(),
            version: "1.0".to_string(),
        };

        // Create backup of existing file
        if self.config.storage_path.exists() {
            let backup_path = self.config.storage_path.with_extension("bak");
            if let Err(e) = tokio::fs::rename(&self.config.storage_path, backup_path).await {
                warn!("Failed to create backup: {}", e);
            }
        }

        // Save to temporary file first, then rename for atomic operation
        let temp_path = self.config.storage_path.with_extension("tmp");
        let serialized = if self.config.enable_compression {
            // For now, just use regular serialization
            // In production, we might use compression here
            serde_json::to_vec_pretty(&snapshot)?
        } else {
            serde_json::to_vec_pretty(&snapshot)?
        };

        tokio::fs::write(&temp_path, serialized).await?;
        tokio::fs::rename(temp_path, &self.config.storage_path).await?;

        // Update state
        {
            let mut last_save = self.last_save.write().await;
            *last_save = chrono::Utc::now();
        }
        {
            let mut dirty = self.dirty.write().await;
            *dirty = false;
        }

        debug!(
            "Vector store saved to: {}",
            self.config.storage_path.display()
        );
        Ok(())
    }

    async fn load(&self) -> Result<()> {
        if !self.config.storage_path.exists() {
            return Err(anyhow!("Vector store file does not exist"));
        }

        let data = tokio::fs::read(&self.config.storage_path).await?;
        let snapshot: VectorStoreSnapshot = if self.config.enable_compression {
            // For now, just use regular deserialization
            serde_json::from_slice(&data)?
        } else {
            serde_json::from_slice(&data)?
        };

        // Verify dimension compatibility
        if snapshot.dimension != self.inner_store.dimension() {
            return Err(anyhow!(
                "Dimension mismatch: stored {} vs expected {}",
                snapshot.dimension,
                self.inner_store.dimension()
            ));
        }

        let chunk_count = snapshot.chunks.len();

        // Load data into the vector store
        {
            let mut embeddings = self.inner_store.embeddings.write().await;
            *embeddings = snapshot.embeddings;
        }
        {
            let mut chunks = self.inner_store.chunks.write().await;
            *chunks = snapshot.chunks;
        }
        {
            let mut document_index = self.inner_store.document_index.write().await;
            *document_index = snapshot.document_index;
        }

        // Rebuild keyword index
        {
            let mut keyword_index = self.inner_store.keyword_index.write().await;
            let chunks = self.inner_store.chunks.read().await;
            for (chunk_id, chunk) in chunks.iter() {
                keyword_index.add_chunk(chunk_id, &chunk.content);
            }
        }

        info!(
            "Loaded vector store with {} chunks from {}",
            chunk_count,
            self.config.storage_path.display()
        );

        Ok(())
    }

    async fn start_auto_save_task(&self) {
        let storage_path = self.config.storage_path.clone();
        let interval = self.config.auto_save_interval_seconds;
        let dirty = Arc::clone(&self.dirty);
        let _last_save = Arc::clone(&self.last_save);

        // Create a weak reference to self to avoid circular references
        let store_weak = Arc::downgrade(&Arc::new(()));

        tokio::spawn(async move {
            let mut interval_timer =
                tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                // Check if store still exists
                if store_weak.upgrade().is_none() {
                    break;
                }

                let is_dirty = {
                    let dirty_guard = dirty.read().await;
                    *dirty_guard
                };

                if is_dirty {
                    // Note: In a real implementation, we would call self.save() here
                    // For this simplified version, we just log
                    debug!(
                        "Auto-save triggered for vector store: {}",
                        storage_path.display()
                    );
                }
            }
        });
    }

    async fn combine_search_results(
        &self,
        keyword_results: Vec<SearchResult>,
        vector_results: Vec<SearchResult>,
        keyword_weight: f32,
        vector_weight: f32,
        k: usize,
    ) -> Vec<SearchResult> {
        let mut combined_scores: HashMap<String, (f32, SearchResult)> = HashMap::new();

        // Add keyword results with weight
        for (i, result) in keyword_results.iter().enumerate() {
            let score = (1.0 - (i as f32 / keyword_results.len() as f32)) * keyword_weight;
            combined_scores.insert(result.chunk.id.clone(), (score, result.clone()));
        }

        // Add or update with vector results
        for (i, result) in vector_results.iter().enumerate() {
            let vector_score = (1.0 - (i as f32 / vector_results.len() as f32)) * vector_weight;

            match combined_scores.get_mut(&result.chunk.id) {
                Some((existing_score, _)) => {
                    *existing_score += vector_score;
                }
                None => {
                    combined_scores.insert(result.chunk.id.clone(), (vector_score, result.clone()));
                }
            }
        }

        // Sort by combined score and take top k
        let mut sorted_results: Vec<(f32, SearchResult)> = combined_scores.into_values().collect();

        sorted_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        sorted_results
            .into_iter()
            .take(k)
            .map(|(score, mut result)| {
                result.similarity = score;
                result.explanation = format!(
                    "Hybrid search result (combined score: {:.3}): {}",
                    score, result.explanation
                );
                result
            })
            .collect()
    }

    pub fn dimension(&self) -> usize {
        self.inner_store.dimension()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentStorageInfo {
    pub storage_path: PathBuf,
    pub last_save: chrono::DateTime<chrono::Utc>,
    pub is_dirty: bool,
    pub storage_size_bytes: u64,
    pub total_chunks: usize,
    pub total_documents: usize,
    pub auto_save_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub duration_ms: u64,
    pub chunks_before: usize,
    pub chunks_after: usize,
    pub memory_before: usize,
    pub memory_after: usize,
    pub storage_compacted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistent_store_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = PersistentVectorConfig {
            storage_path: temp_dir.path().join("test_store.json"),
            auto_save_interval_seconds: 60,
            max_memory_chunks: 1000,
            enable_compression: false,
            backup_count: 1,
        };

        let store = PersistentVectorStore::new(384, config).await?;
        assert_eq!(store.dimension(), 384);

        let stats = store.get_stats().await?;
        assert_eq!(stats.total_chunks, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_persistent_store_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = PersistentVectorConfig {
            storage_path: temp_dir.path().join("test_store.json"),
            auto_save_interval_seconds: 60,
            max_memory_chunks: 1000,
            enable_compression: false,
            backup_count: 1,
        };

        let store = PersistentVectorStore::new(3, config).await?;

        // Add test data
        let chunks = vec![DocumentChunk {
            id: "doc1:0".to_string(),
            document_id: "doc1".to_string(),
            content: "Test content".to_string(),
            chunk_index: 0,
            start_char: 0,
            end_char: 12,
            metadata: HashMap::new(),
        }];

        let embeddings = vec![EmbeddingRecord {
            chunk_id: "doc1:0".to_string(),
            embedding: vec![1.0, 0.0, 0.0],
            timestamp: chrono::Utc::now(),
        }];

        store.add_document_chunks(chunks, embeddings).await?;

        // Test search
        let results = store.search(&[1.0, 0.0, 0.0], 1).await?;
        assert_eq!(results.len(), 1);

        // Test keyword search
        let keyword_results = store.keyword_search("Test", 5).await?;
        assert!(!keyword_results.is_empty());

        // Test storage info
        let info = store.get_storage_info().await?;
        assert!(info.is_dirty);

        Ok(())
    }

    #[tokio::test]
    async fn test_hybrid_search() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = PersistentVectorConfig {
            storage_path: temp_dir.path().join("test_store.json"),
            auto_save_interval_seconds: 60,
            max_memory_chunks: 1000,
            enable_compression: false,
            backup_count: 1,
        };

        let store = PersistentVectorStore::new(3, config).await?;

        // Add test data
        let chunks = vec![
            DocumentChunk {
                id: "doc1:0".to_string(),
                document_id: "doc1".to_string(),
                content: "artificial intelligence machine learning".to_string(),
                chunk_index: 0,
                start_char: 0,
                end_char: 38,
                metadata: HashMap::new(),
            },
            DocumentChunk {
                id: "doc1:1".to_string(),
                document_id: "doc1".to_string(),
                content: "neural networks deep learning".to_string(),
                chunk_index: 1,
                start_char: 38,
                end_char: 67,
                metadata: HashMap::new(),
            },
        ];

        let embeddings = vec![
            EmbeddingRecord {
                chunk_id: "doc1:0".to_string(),
                embedding: vec![1.0, 0.0, 0.0],
                timestamp: chrono::Utc::now(),
            },
            EmbeddingRecord {
                chunk_id: "doc1:1".to_string(),
                embedding: vec![0.0, 1.0, 0.0],
                timestamp: chrono::Utc::now(),
            },
        ];

        store.add_document_chunks(chunks, embeddings).await?;

        // Test hybrid search
        let hybrid_results = store
            .hybrid_search(
                "machine learning",
                &[0.7, 0.3, 0.0],
                5,
                0.5, // keyword weight
                0.5, // vector weight
            )
            .await?;

        assert!(!hybrid_results.is_empty());
        assert!(hybrid_results[0]
            .explanation
            .contains("Hybrid search result"));

        Ok(())
    }
}
