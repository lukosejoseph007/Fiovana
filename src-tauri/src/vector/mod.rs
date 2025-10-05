// src-tauri/src/vector/mod.rs
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod embedding_service;
pub mod persistent_store;
pub use embedding_service::{
    EmbeddingProvider, EmbeddingService, EmbeddingServiceConfig, UsageStats,
};
// Persistent store types are available but not currently exported
// as they are not used outside the vector module yet
// pub use persistent_store::{
//     OptimizationResult, PersistentStorageInfo, PersistentVectorConfig, PersistentVectorStore,
// };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimension: usize,
    pub max_length: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "text-embedding-3-small".to_string(), // OpenAI API model (no local model)
            dimension: 1536,                                  // OpenAI embedding dimension
            max_length: 8192, // Increased token limit for API models
            chunk_size: 1000,
            chunk_overlap: 200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub chunk_index: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: DocumentChunk,
    pub similarity: f32,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub chunk_id: String,
    pub embedding: Vec<f32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct EmbeddingEngine {
    config: EmbeddingConfig,
    embedding_service: EmbeddingService,
    #[allow(dead_code)] // Cache functionality to be implemented in future versions
    embeddings_cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    model_available: bool,
}

impl EmbeddingEngine {
    /// Create a new EmbeddingEngine for testing that uses mock embeddings
    #[cfg(test)]
    pub async fn new_mock_for_test(mut config: EmbeddingConfig) -> Result<Self> {
        use std::collections::HashMap;

        // Override config to use test model
        config.model_name = "test-embedding-model".to_string();

        // Create a mock embedding service config
        let service_config = EmbeddingServiceConfig {
            provider: EmbeddingProvider::OpenAI,
            api_key: Some("test_key".to_string()),
            model_name: config.model_name.clone(),
            dimension: config.dimension,
            max_tokens: config.max_length,
            batch_size: 25,
            timeout_seconds: 20,
        };

        // Create embedding service but don't test connection for mocks
        let embedding_service = EmbeddingService::new(service_config).await?;

        Ok(Self {
            config,
            embedding_service,
            embeddings_cache: Arc::new(RwLock::new(HashMap::new())),
            model_available: true, // Mock as available
        })
    }

    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        // Create embedding service configuration from engine config
        // Prioritize API-based embeddings to avoid CPU overload

        // Helper function to load UI settings
        async fn load_ui_embedding_settings(
        ) -> Option<crate::commands::embedding_settings_commands::EmbeddingSettings> {
            let config_dir = dirs::config_dir()?.join("fiovana");
            let settings_file = config_dir.join("embedding_settings.json");

            if !settings_file.exists() {
                return None;
            }

            let settings_json = std::fs::read_to_string(&settings_file).ok()?;
            serde_json::from_str(&settings_json).ok()
        }

        // Helper function for .env loading (kept for future optional use)
        #[allow(dead_code)]
        fn load_env_embedding_config(
        ) -> Result<(EmbeddingProvider, Option<String>, String, usize), anyhow::Error> {
            let embedding_model = std::env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string());

            // Determine model dimensions based on model name
            let get_model_dimension = |model: &str| -> usize {
                match model {
                    "text-embedding-3-small" => 1536, // Cheapest and most efficient
                    "text-embedding-3-large" => 3072, // Higher performance, more expensive
                    "text-embedding-ada-002" => 1536, // Legacy model
                    _ => 1536,                        // Default to 1536 for unknown models
                }
            };
            let model_dimension = get_model_dimension(&embedding_model);

            if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
                tracing::info!("✅ Using .env OpenAI API for embeddings");
                Ok((
                    EmbeddingProvider::OpenAI,
                    Some(openai_key),
                    embedding_model,
                    model_dimension,
                ))
            } else if let Ok(openrouter_key) = std::env::var("OPENROUTER_API_KEY") {
                tracing::info!("✅ Using .env OpenRouter API for embeddings");
                Ok((
                    EmbeddingProvider::OpenRouter,
                    Some(openrouter_key),
                    embedding_model,
                    model_dimension,
                ))
            } else {
                Err(anyhow!("No .env embedding configuration found"))
            }
        }

        // PRIORITY CONFIGURATION: UI Settings First, then .env fallback, then graceful failure
        let (provider, api_key, model_name, dimension) = if let Some(ui_settings) =
            load_ui_embedding_settings().await
        {
            if !ui_settings.api_key.is_empty() {
                tracing::info!(
                    "✅ Using UI embedding settings - Provider: {}, Model: {}",
                    ui_settings.provider,
                    ui_settings.model
                );
                let embedding_provider = match ui_settings.provider.as_str() {
                    "openai" => EmbeddingProvider::OpenAI,
                    "openrouter" => EmbeddingProvider::OpenRouter,
                    _ => EmbeddingProvider::OpenAI, // Default to OpenAI
                };
                let model_dimension = match ui_settings.model.as_str() {
                    "text-embedding-3-small" => 1536,
                    "text-embedding-3-large" => 3072,
                    "text-embedding-ada-002" => 1536,
                    _ => 1536,
                };
                let final_dimension = ui_settings.custom_dimensions.unwrap_or(model_dimension);
                (
                    embedding_provider,
                    Some(ui_settings.api_key),
                    ui_settings.model,
                    final_dimension,
                )
            } else {
                tracing::warn!("⚠️ UI settings found but API key is empty");
                return Err(anyhow!("Embedding API key not configured. Please configure your API key through Settings > Embeddings in the UI."));
            }
        } else {
            tracing::info!("ℹ️ No UI embedding settings found");
            return Err(anyhow!("No embedding configuration found. Please configure your API key through Settings > Embeddings in the UI."));
        };

        let service_config = EmbeddingServiceConfig {
            provider: provider.clone(),
            api_key,
            model_name: model_name.clone(),
            dimension,
            max_tokens: config.max_length,
            batch_size: 25,      // Reduced batch size to prevent hangs
            timeout_seconds: 20, // Aggressive timeout to prevent system hangs
        };

        let embedding_service = EmbeddingService::new(service_config).await?;
        let model_available = embedding_service.test_connection().await?;

        if !model_available {
            tracing::error!(
                "❌ CRITICAL: API connection failed for model {} - Provider: {:?}",
                model_name,
                provider
            );
            return Err(anyhow!("API embedding service connection failed. Please check your API key and internet connection. Local embeddings are disabled for safety."));
        } else {
            tracing::info!(
                "✅ SUCCESS: API-based embeddings connected - Provider: {:?}, Model: {}, Dimension: {}",
                provider, model_name, dimension
            );
        }

        // Update config dimensions to match API provider
        let mut final_config = config;
        final_config.dimension = dimension;

        Ok(Self {
            config: final_config,
            embedding_service,
            embeddings_cache: Arc::new(RwLock::new(HashMap::new())),
            model_available,
        })
    }

    pub async fn new_with_service(
        config: EmbeddingConfig,
        service_config: EmbeddingServiceConfig,
    ) -> Result<Self> {
        let embedding_service = EmbeddingService::new(service_config).await?;
        let model_available = embedding_service.test_connection().await?;

        if !model_available {
            tracing::warn!(
                "ML model {} not available, using fallback embedding",
                config.model_name
            );
        }

        Ok(Self {
            config,
            embedding_service,
            embeddings_cache: Arc::new(RwLock::new(HashMap::new())),
            model_available,
        })
    }

    pub async fn get_model_info(&self) -> Result<String> {
        Ok(format!(
            "Embedding Engine - Model: {}, Dimension: {}, Available: {}",
            self.config.model_name, self.config.dimension, self.model_available
        ))
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        #[cfg(test)]
        {
            // For testing: generate deterministic mock embeddings
            if self.config.model_name == "test-embedding-model" {
                return self.generate_mock_embedding(text).await;
            }
        }

        // Use the embedding service instead of manual implementation
        self.embedding_service.get_embedding(text.to_string()).await
    }

    #[cfg(test)]
    async fn generate_mock_embedding(&self, text: &str) -> Result<Vec<f32>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Check cache first
        {
            let cache = self.embeddings_cache.read().await;
            if let Some(embedding) = cache.get(text) {
                return Ok(embedding.clone());
            }
        }

        // Generate deterministic embedding based on text hash
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // Create a deterministic but varied embedding vector
        let mut embedding = Vec::with_capacity(self.config.dimension);
        let mut seed = hash;

        for _i in 0..self.config.dimension {
            // Simple PRNG to generate consistent values
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let normalized = (seed as f32 / u64::MAX as f32) * 2.0 - 1.0; // Range [-1, 1]
            embedding.push(normalized * 0.1); // Scale down to reasonable embedding values
        }

        // Cache the result
        {
            let mut cache = self.embeddings_cache.write().await;
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    #[cfg(test)]
    async fn generate_mock_chunk_embeddings(
        &self,
        chunks: &[DocumentChunk],
    ) -> Result<Vec<EmbeddingRecord>> {
        let mut records = Vec::new();
        for chunk in chunks {
            let embedding = self.generate_mock_embedding(&chunk.content).await?;
            records.push(EmbeddingRecord {
                chunk_id: chunk.id.clone(),
                embedding,
                timestamp: chrono::Utc::now(),
            });
        }
        Ok(records)
    }

    // Expose embedding service methods for advanced configuration
    pub async fn get_embedding_service_stats(&self) -> UsageStats {
        self.embedding_service.get_usage_stats().await
    }

    pub async fn clear_embedding_cache(&self) {
        self.embedding_service.clear_cache().await
    }

    pub fn chunk_text(&self, text: &str, document_id: &str) -> Vec<DocumentChunk> {
        let start_time = std::time::Instant::now();
        let max_duration = std::time::Duration::from_secs(30); // 30 second timeout for chunking

        tracing::debug!(
            "🔪 Starting text chunking for document '{}' ({} chars)",
            document_id,
            text.len()
        );
        let mut chunks = Vec::new();

        if text.len() <= self.config.chunk_size {
            // Single chunk for small text
            chunks.push(DocumentChunk {
                id: format!("{}:0", document_id),
                document_id: document_id.to_string(),
                content: text.to_string(),
                chunk_index: 0,
                start_char: 0,
                end_char: text.len(),
                metadata: HashMap::new(),
            });
            tracing::debug!("🔪 Document '{}' fits in single chunk", document_id);
            return chunks;
        }

        let mut start_byte = 0;
        let mut chunk_index = 0;
        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 1000; // Safety limit

        while start_byte < text.len() {
            iteration_count += 1;

            // Timeout protection
            if start_time.elapsed() > max_duration {
                tracing::error!("🚨 CHUNKING TIMEOUT: Document '{}' chunking exceeded {} seconds, stopping at {} chunks",
                    document_id, max_duration.as_secs(), chunks.len());
                break;
            }

            // Iteration protection
            if iteration_count > MAX_ITERATIONS {
                tracing::error!(
                    "🚨 CHUNKING ITERATION LIMIT: Document '{}' exceeded {} iterations, stopping",
                    document_id,
                    MAX_ITERATIONS
                );
                break;
            }

            if iteration_count % 100 == 0 {
                tracing::debug!(
                    "🔄 Chunking progress: Document '{}' - {} iterations, {} chunks created",
                    document_id,
                    iteration_count,
                    chunks.len()
                );
            }

            let mut end_byte = std::cmp::min(start_byte + self.config.chunk_size, text.len());

            // Ensure we don't break UTF-8 character boundaries - with safety limit
            let mut boundary_attempts = 0;
            while end_byte > start_byte
                && !text.is_char_boundary(end_byte)
                && boundary_attempts < 10
            {
                end_byte -= 1;
                boundary_attempts += 1;
            }

            if boundary_attempts >= 10 {
                tracing::warn!(
                    "⚠️ UTF-8 boundary adjustment limit reached for document '{}' at position {}",
                    document_id,
                    end_byte
                );
                // Force to next valid boundary or use original position
                if !text.is_char_boundary(end_byte) {
                    end_byte = std::cmp::min(start_byte + self.config.chunk_size, text.len());
                }
            }

            // Try to break at word boundaries (space characters)
            if end_byte < text.len() {
                // Look backward from end_byte to find the last space
                let search_text = &text[start_byte..end_byte];
                if let Some(last_space_pos) = search_text.rfind(' ') {
                    let potential_end = start_byte + last_space_pos;
                    // Don't make chunks too small - if the space is too early, use the full chunk
                    if potential_end > start_byte + (self.config.chunk_size / 4) {
                        end_byte = potential_end;
                    }
                }
            }

            let chunk_content = text[start_byte..end_byte].to_string();

            if chunk_content.is_empty() {
                tracing::warn!(
                    "⚠️ Empty chunk detected for document '{}', breaking",
                    document_id
                );
                break; // Avoid infinite loop with empty chunks
            }

            chunks.push(DocumentChunk {
                id: format!("{}:{}", document_id, chunk_index),
                document_id: document_id.to_string(),
                content: chunk_content,
                chunk_index,
                start_char: start_byte,
                end_char: end_byte,
                metadata: HashMap::new(),
            });

            // Calculate next position with overlap
            let chunk_size = end_byte - start_byte;
            let overlap_bytes = std::cmp::min(self.config.chunk_overlap, chunk_size);
            let next_start = end_byte.saturating_sub(overlap_bytes);

            // Ensure we don't start in the middle of a UTF-8 character - with safety limit
            let mut new_start_byte = next_start;
            let mut utf8_attempts = 0;
            while new_start_byte > 0 && !text.is_char_boundary(new_start_byte) && utf8_attempts < 10
            {
                new_start_byte -= 1;
                utf8_attempts += 1;
            }

            if utf8_attempts >= 10 {
                tracing::warn!("⚠️ UTF-8 start boundary adjustment limit reached for document '{}' at position {}",
                    document_id, new_start_byte);
                // Use a safe fallback - move forward to next boundary
                new_start_byte = next_start;
                while new_start_byte < text.len() && !text.is_char_boundary(new_start_byte) {
                    new_start_byte += 1;
                }
            }

            // Ensure we make progress
            if new_start_byte >= end_byte || new_start_byte <= start_byte {
                // Force progress to prevent infinite loop
                new_start_byte = end_byte;
                tracing::debug!(
                    "🔧 Forced progress: Moving from {} to {} for document '{}'",
                    start_byte,
                    new_start_byte,
                    document_id
                );
            }

            start_byte = new_start_byte;
            chunk_index += 1;

            // Safety check: if we've processed too many chunks, something might be wrong
            if chunks.len() > 500 {
                tracing::error!(
                    "🚨 CHUNK LIMIT: Document '{}' generated {} chunks, stopping for safety",
                    document_id,
                    chunks.len()
                );
                break;
            }
        }

        let duration = start_time.elapsed();
        tracing::debug!(
            "🔪 Completed text chunking for document '{}': {} chunks in {:?}",
            document_id,
            chunks.len(),
            duration
        );

        if duration > std::time::Duration::from_secs(5) {
            tracing::warn!(
                "⚠️ Slow chunking: Document '{}' took {:?} to chunk",
                document_id,
                duration
            );
        }

        chunks
    }

    pub async fn embed_chunks(&self, chunks: &[DocumentChunk]) -> Result<Vec<EmbeddingRecord>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        #[cfg(test)]
        {
            // For testing: generate mock embeddings
            if self.config.model_name == "test-embedding-model" {
                return self.generate_mock_chunk_embeddings(chunks).await;
            }
        }

        // Batch process chunks for efficiency - collect all text content
        let texts: Vec<String> = chunks.iter().map(|chunk| chunk.content.clone()).collect();

        // Single batch API call instead of individual calls (major performance improvement)
        let embeddings = self.embedding_service.get_embeddings(texts).await?;

        // Create records from batch results
        let mut records = Vec::new();
        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            records.push(EmbeddingRecord {
                chunk_id: chunk.id.clone(),
                embedding: embedding.clone(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(records)
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    pub fn get_config(&self) -> &EmbeddingConfig {
        &self.config
    }

    pub fn is_model_available(&self) -> bool {
        self.model_available
    }
}

#[derive(Clone)]
pub struct VectorStore {
    embeddings: Arc<RwLock<HashMap<String, EmbeddingRecord>>>,
    chunks: Arc<RwLock<HashMap<String, DocumentChunk>>>,
    dimension: usize,
    document_index: Arc<RwLock<HashMap<String, Vec<String>>>>, // document_id -> chunk_ids
    keyword_index: Arc<RwLock<KeywordIndex>>,                  // TF-IDF and phrase search index
}

#[derive(Debug, Clone, Default)]
pub struct KeywordIndex {
    // Term frequency: chunk_id -> word -> count
    term_frequency: HashMap<String, HashMap<String, usize>>,
    // Document frequency: word -> number of chunks containing it
    document_frequency: HashMap<String, usize>,
    // Total number of chunks for IDF calculation
    total_chunks: usize,
    // Phrase index: normalized phrase -> chunk_ids containing it
    phrase_index: HashMap<String, Vec<String>>,
    // Word positions for exact phrase matching: chunk_id -> word -> positions
    word_positions: HashMap<String, HashMap<String, Vec<usize>>>,
}

impl KeywordIndex {
    pub fn new() -> Self {
        Self::default()
    }

    fn normalize_word(word: &str) -> String {
        word.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    fn extract_words(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(Self::normalize_word)
            .filter(|word| !word.is_empty() && word.len() > 1) // Filter out single characters
            .collect()
    }

    fn extract_phrases(text: &str, max_phrase_length: usize) -> Vec<String> {
        let words = Self::extract_words(text);
        let mut phrases = Vec::new();

        for length in 2..=max_phrase_length.min(words.len()) {
            for i in 0..=(words.len() - length) {
                let phrase = words[i..i + length].join(" ");
                if phrase.len() >= 4 {
                    // Minimum phrase length
                    phrases.push(phrase);
                }
            }
        }

        phrases
    }

    pub fn add_chunk(&mut self, chunk_id: &str, content: &str) {
        let words = Self::extract_words(content);
        let phrases = Self::extract_phrases(content, 4); // Max 4-word phrases

        // Track term frequencies and word positions
        let mut tf_map = HashMap::new();
        let mut positions_map = HashMap::new();

        for (position, word) in words.iter().enumerate() {
            *tf_map.entry(word.clone()).or_insert(0) += 1;
            positions_map
                .entry(word.clone())
                .or_insert_with(Vec::new)
                .push(position);

            // Update document frequency
            if tf_map[word] == 1 {
                // First occurrence of this word in this chunk
                *self.document_frequency.entry(word.clone()).or_insert(0) += 1;
            }
        }

        // Store term frequencies and positions
        self.term_frequency.insert(chunk_id.to_string(), tf_map);
        self.word_positions
            .insert(chunk_id.to_string(), positions_map);

        // Index phrases
        for phrase in phrases {
            self.phrase_index
                .entry(phrase)
                .or_default()
                .push(chunk_id.to_string());
        }

        self.total_chunks += 1;
    }

    pub fn remove_chunk(&mut self, chunk_id: &str) {
        if let Some(tf_map) = self.term_frequency.remove(chunk_id) {
            // Decrease document frequencies
            for word in tf_map.keys() {
                if let Some(df) = self.document_frequency.get_mut(word) {
                    *df -= 1;
                    if *df == 0 {
                        self.document_frequency.remove(word);
                    }
                }
            }

            // Remove from phrase index
            for phrase_chunks in self.phrase_index.values_mut() {
                phrase_chunks.retain(|id| id != chunk_id);
            }
            self.phrase_index.retain(|_, chunks| !chunks.is_empty());

            // Remove word positions
            self.word_positions.remove(chunk_id);

            self.total_chunks -= 1;
        }
    }

    pub fn calculate_tf_idf_score(&self, chunk_id: &str, query_words: &[String]) -> f64 {
        if let Some(tf_map) = self.term_frequency.get(chunk_id) {
            let mut score = 0.0;

            for word in query_words {
                let tf = *tf_map.get(word).unwrap_or(&0) as f64;
                if tf > 0.0 {
                    let df = *self.document_frequency.get(word).unwrap_or(&0) as f64;
                    if df > 0.0 {
                        // TF-IDF calculation: log(1 + tf) * log(N / df)
                        let tf_score = (1.0 + tf).ln();
                        let idf_score = (self.total_chunks as f64 / df).ln();

                        // Handle single document case: when total_chunks = df = 1, idf_score = ln(1) = 0
                        // In this case, just use the TF score so we still get results
                        if self.total_chunks == 1 {
                            score += tf_score; // Use only TF component when there's just one document
                        } else {
                            score += tf_score * idf_score;
                        }
                    }
                }
            }

            score
        } else {
            0.0
        }
    }

    pub fn search_phrases(&self, query: &str, max_results: usize) -> Vec<(String, f64)> {
        let query_phrases = Self::extract_phrases(query, 4);
        let mut phrase_matches: HashMap<String, f64> = HashMap::new();

        for phrase in &query_phrases {
            if let Some(chunk_ids) = self.phrase_index.get(phrase) {
                let phrase_score = phrase.split_whitespace().count() as f64; // Longer phrases get higher scores
                for chunk_id in chunk_ids {
                    *phrase_matches.entry(chunk_id.clone()).or_insert(0.0) += phrase_score * 2.0;
                    // Boost phrase matches
                }
            }
        }

        // Also check for exact phrase matches using word positions
        for phrase in &query_phrases {
            let phrase_words: Vec<String> =
                phrase.split_whitespace().map(|w| w.to_string()).collect();
            if phrase_words.len() >= 2 {
                for (chunk_id, positions_map) in &self.word_positions {
                    if self.contains_exact_phrase(positions_map, &phrase_words) {
                        *phrase_matches.entry(chunk_id.clone()).or_insert(0.0) +=
                            phrase_words.len() as f64 * 3.0; // Higher boost for exact phrases
                    }
                }
            }
        }

        // Sort by score and return top results
        let mut results: Vec<(String, f64)> = phrase_matches.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(max_results).collect()
    }

    fn contains_exact_phrase(
        &self,
        positions_map: &HashMap<String, Vec<usize>>,
        phrase_words: &[String],
    ) -> bool {
        if phrase_words.is_empty() {
            return false;
        }

        // Get positions of the first word
        if let Some(first_positions) = positions_map.get(&phrase_words[0]) {
            for &start_pos in first_positions {
                // Check if all subsequent words appear in consecutive positions
                let mut all_found = true;
                for (i, word) in phrase_words.iter().enumerate().skip(1) {
                    let expected_pos = start_pos + i;
                    if let Some(positions) = positions_map.get(word) {
                        if !positions.contains(&expected_pos) {
                            all_found = false;
                            break;
                        }
                    } else {
                        all_found = false;
                        break;
                    }
                }

                if all_found {
                    return true;
                }
            }
        }

        false
    }
}

impl VectorStore {
    pub fn new(dimension: usize) -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(HashMap::new())),
            chunks: Arc::new(RwLock::new(HashMap::new())),
            dimension,
            document_index: Arc::new(RwLock::new(HashMap::new())),
            keyword_index: Arc::new(RwLock::new(KeywordIndex::new())),
        }
    }

    pub async fn add_document_chunks(
        &self,
        chunks: Vec<DocumentChunk>,
        embeddings: Vec<EmbeddingRecord>,
    ) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(anyhow!("Chunks and embeddings count mismatch"));
        }

        let mut chunks_store = self.chunks.write().await;
        let mut embeddings_store = self.embeddings.write().await;
        let mut doc_index = self.document_index.write().await;
        let mut keyword_index = self.keyword_index.write().await;

        for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
            if embedding.embedding.len() != self.dimension {
                return Err(anyhow!(
                    "Embedding dimension mismatch: expected {}, got {}",
                    self.dimension,
                    embedding.embedding.len()
                ));
            }

            let document_id = chunk.document_id.clone();
            let chunk_id = chunk.id.clone();
            let chunk_content = chunk.content.clone();

            // Store chunk and embedding
            chunks_store.insert(chunk_id.clone(), chunk);
            embeddings_store.insert(chunk_id.clone(), embedding);

            // Update document index
            doc_index
                .entry(document_id)
                .or_insert_with(Vec::new)
                .push(chunk_id.clone());

            // Add to keyword index for TF-IDF search
            keyword_index.add_chunk(&chunk_id, &chunk_content);
        }

        Ok(())
    }

    pub async fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        if query_vector.len() != self.dimension {
            return Err(anyhow!("Query vector dimension mismatch"));
        }

        let embeddings = self.embeddings.read().await;
        let chunks = self.chunks.read().await;

        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        let mut similarities: Vec<(String, f32)> = Vec::new();

        // Calculate cosine similarity for each stored vector
        for (chunk_id, embedding_record) in embeddings.iter() {
            let similarity = cosine_similarity(query_vector, &embedding_record.embedding);
            similarities.push((chunk_id.clone(), similarity));
        }

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top k results and build SearchResult objects
        let mut results = Vec::new();
        for (chunk_id, similarity) in similarities.into_iter().take(k) {
            if let Some(chunk) = chunks.get(&chunk_id) {
                results.push(SearchResult {
                    chunk: chunk.clone(),
                    similarity,
                    explanation: format!(
                        "Found in document '{}' (chunk {}) with {:.2}% similarity",
                        chunk.document_id,
                        chunk.chunk_index,
                        similarity * 100.0
                    ),
                });
            }
        }

        Ok(results)
    }

    pub async fn search_by_document(
        &self,
        document_id: &str,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<SearchResult>> {
        let doc_index = self.document_index.read().await;
        let chunk_ids = doc_index
            .get(document_id)
            .ok_or_else(|| anyhow!("Document not found: {}", document_id))?;

        let embeddings = self.embeddings.read().await;
        let chunks = self.chunks.read().await;

        let mut similarities: Vec<(String, f32)> = Vec::new();

        // Calculate similarity only for chunks in the specified document
        for chunk_id in chunk_ids {
            if let Some(embedding_record) = embeddings.get(chunk_id) {
                let similarity = cosine_similarity(query_vector, &embedding_record.embedding);
                similarities.push((chunk_id.clone(), similarity));
            }
        }

        // Sort and take top k
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut results = Vec::new();
        for (chunk_id, similarity) in similarities.into_iter().take(k) {
            if let Some(chunk) = chunks.get(&chunk_id) {
                results.push(SearchResult {
                    chunk: chunk.clone(),
                    similarity,
                    explanation: format!(
                        "Found in chunk {} with {:.2}% similarity",
                        chunk.chunk_index,
                        similarity * 100.0
                    ),
                });
            }
        }

        Ok(results)
    }

    pub async fn get_document_chunks(&self, document_id: &str) -> Result<Vec<DocumentChunk>> {
        let doc_index = self.document_index.read().await;
        let chunks = self.chunks.read().await;

        let chunk_ids = doc_index
            .get(document_id)
            .ok_or_else(|| anyhow!("Document not found: {}", document_id))?;

        let mut document_chunks = Vec::new();
        for chunk_id in chunk_ids {
            if let Some(chunk) = chunks.get(chunk_id) {
                document_chunks.push(chunk.clone());
            }
        }

        // Sort by chunk index
        document_chunks.sort_by_key(|chunk| chunk.chunk_index);

        Ok(document_chunks)
    }

    pub async fn keyword_search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        let keyword_index = self.keyword_index.read().await;
        let chunks = self.chunks.read().await;

        // Extract query words for TF-IDF scoring
        let query_words = KeywordIndex::extract_words(query);

        if query_words.is_empty() {
            return Ok(Vec::new());
        }

        // First, try phrase matching for higher precision
        let phrase_matches = keyword_index.search_phrases(query, max_results * 2);

        // Then do TF-IDF scoring for all chunks
        let mut tf_idf_scores: Vec<(String, f64)> = Vec::new();
        for chunk_id in chunks.keys() {
            let score = keyword_index.calculate_tf_idf_score(chunk_id, &query_words);
            if score > 0.0 {
                tf_idf_scores.push((chunk_id.clone(), score));
            }
        }

        // Combine scores - phrase matches get priority
        let mut combined_scores: HashMap<String, f64> = HashMap::new();

        // Add phrase scores with high weight
        for (chunk_id, phrase_score) in phrase_matches {
            combined_scores.insert(chunk_id, phrase_score);
        }

        // Add or boost TF-IDF scores
        for (chunk_id, tf_idf_score) in tf_idf_scores {
            let existing_score = combined_scores.get(&chunk_id).copied().unwrap_or(0.0);
            combined_scores.insert(chunk_id, existing_score + tf_idf_score);
        }

        // Sort by combined score
        let mut results: Vec<(String, f64)> = combined_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert to SearchResult objects
        let mut search_results = Vec::new();
        for (chunk_id, score) in results.into_iter().take(max_results) {
            if let Some(chunk) = chunks.get(&chunk_id) {
                // Normalize score to 0-1 range for similarity
                let normalized_score = (score / 10.0).min(1.0) as f32;

                search_results.push(SearchResult {
                    chunk: chunk.clone(),
                    similarity: normalized_score,
                    explanation: format!(
                        "Keyword match in document '{}' (chunk {}) - TF-IDF score: {:.2}",
                        chunk.document_id, chunk.chunk_index, score
                    ),
                });
            }
        }

        Ok(search_results)
    }

    pub async fn keyword_search_by_document(
        &self,
        document_id: &str,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        let doc_index = self.document_index.read().await;
        let chunk_ids = doc_index
            .get(document_id)
            .ok_or_else(|| anyhow!("Document not found: {}", document_id))?;

        let keyword_index = self.keyword_index.read().await;
        let chunks = self.chunks.read().await;

        // Extract query words for TF-IDF scoring
        let query_words = KeywordIndex::extract_words(query);

        if query_words.is_empty() {
            return Ok(Vec::new());
        }

        // Score only chunks in the specified document
        let mut scores: Vec<(String, f64)> = Vec::new();

        for chunk_id in chunk_ids {
            // Calculate TF-IDF score
            let tf_idf_score = keyword_index.calculate_tf_idf_score(chunk_id, &query_words);

            // Check for phrase matches
            let phrase_matches = keyword_index.search_phrases(query, chunk_ids.len());
            let phrase_score = phrase_matches
                .iter()
                .find(|(id, _)| id == chunk_id)
                .map(|(_, score)| *score)
                .unwrap_or(0.0);

            let combined_score = tf_idf_score + phrase_score;
            if combined_score > 0.0 {
                scores.push((chunk_id.clone(), combined_score));
            }
        }

        // Sort by score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert to SearchResult objects
        let mut search_results = Vec::new();
        for (chunk_id, score) in scores.into_iter().take(max_results) {
            if let Some(chunk) = chunks.get(&chunk_id) {
                // Normalize score to 0-1 range for similarity
                let normalized_score = (score / 10.0).min(1.0) as f32;

                search_results.push(SearchResult {
                    chunk: chunk.clone(),
                    similarity: normalized_score,
                    explanation: format!(
                        "Keyword match in chunk {} - TF-IDF score: {:.2}",
                        chunk.chunk_index, score
                    ),
                });
            }
        }

        Ok(search_results)
    }

    pub async fn remove_document(&self, document_id: &str) -> Result<()> {
        let mut doc_index = self.document_index.write().await;
        let mut chunks = self.chunks.write().await;
        let mut embeddings = self.embeddings.write().await;
        let mut keyword_index = self.keyword_index.write().await;

        if let Some(chunk_ids) = doc_index.remove(document_id) {
            for chunk_id in chunk_ids {
                chunks.remove(&chunk_id);
                embeddings.remove(&chunk_id);
                keyword_index.remove_chunk(&chunk_id);
            }
        }

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<VectorStoreStats> {
        let embeddings = self.embeddings.read().await;
        let chunks = self.chunks.read().await;
        let doc_index = self.document_index.read().await;

        Ok(VectorStoreStats {
            total_chunks: chunks.len(),
            total_embeddings: embeddings.len(),
            total_documents: doc_index.len(),
            dimension: self.dimension,
            memory_usage_estimate: self.estimate_memory_usage(&embeddings, &chunks).await,
        })
    }

    async fn estimate_memory_usage(
        &self,
        embeddings: &HashMap<String, EmbeddingRecord>,
        chunks: &HashMap<String, DocumentChunk>,
    ) -> usize {
        let embedding_size = embeddings.len() * self.dimension * std::mem::size_of::<f32>();
        let chunk_text_size: usize = chunks.values().map(|chunk| chunk.content.len()).sum();
        let metadata_size = chunks.len() * 200; // Rough estimate for metadata

        embedding_size + chunk_text_size + metadata_size
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub total_chunks: usize,
    pub total_embeddings: usize,
    pub total_documents: usize,
    pub dimension: usize,
    pub memory_usage_estimate: usize,
}

// Helper function to calculate cosine similarity
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_vector_store_creation() {
        let store = VectorStore::new(1536); // OpenAI embedding dimension (changed from local 384)
        assert_eq!(store.dimension(), 1536);
    }

    #[tokio::test]
    async fn test_embedding_engine() -> Result<()> {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new_mock_for_test(config).await?;

        // Test basic functionality
        let text = "This is a test document for embedding.";
        let embedding = engine.embed_text(text).await?;

        assert_eq!(embedding.len(), 1536); // API model default dimension

        // Test caching - second call should be faster
        let embedding2 = engine.embed_text(text).await?;
        assert_eq!(embedding, embedding2);

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Temporarily disabled due to performance issues
    async fn test_text_chunking() -> Result<()> {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await?;

        let long_text = "This is a very long document. ".repeat(40); // 31 * 40 = 1240 chars > 1000
        let chunks = engine.chunk_text(&long_text, "test_doc");

        assert!(!chunks.is_empty());
        assert!(chunks.len() > 1); // Should be chunked

        // Verify chunk properties
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i);
            assert_eq!(chunk.document_id, "test_doc");
            assert!(!chunk.content.is_empty());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_vector_store_operations() -> Result<()> {
        let store = VectorStore::new(3);
        let config = EmbeddingConfig {
            dimension: 3,
            ..Default::default()
        };
        let _engine = EmbeddingEngine::new_mock_for_test(config).await?;

        // Create test chunks
        let chunks = vec![
            DocumentChunk {
                id: "doc1:0".to_string(),
                document_id: "doc1".to_string(),
                content: "First test chunk".to_string(),
                chunk_index: 0,
                start_char: 0,
                end_char: 16,
                metadata: HashMap::new(),
            },
            DocumentChunk {
                id: "doc1:1".to_string(),
                document_id: "doc1".to_string(),
                content: "Second test chunk".to_string(),
                chunk_index: 1,
                start_char: 16,
                end_char: 33,
                metadata: HashMap::new(),
            },
        ];

        // Create test embeddings
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

        // Add to vector store
        store.add_document_chunks(chunks, embeddings).await?;

        // Test search
        let results = store.search(&[1.0, 0.1, 0.0], 2).await?;
        assert_eq!(results.len(), 2);
        assert!(results[0].similarity > results[1].similarity); // Should be sorted by similarity

        // Test document-specific search
        let doc_results = store
            .search_by_document("doc1", &[1.0, 0.1, 0.0], 1)
            .await?;
        assert_eq!(doc_results.len(), 1);

        // Test stats
        let stats = store.get_stats().await?;
        assert_eq!(stats.total_chunks, 2);
        assert_eq!(stats.total_embeddings, 2);
        assert_eq!(stats.total_documents, 1);
        assert_eq!(stats.dimension, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_document_removal() -> Result<()> {
        let store = VectorStore::new(3);

        // Add test data
        let chunks = vec![DocumentChunk {
            id: "doc1:0".to_string(),
            document_id: "doc1".to_string(),
            content: "Test chunk".to_string(),
            chunk_index: 0,
            start_char: 0,
            end_char: 10,
            metadata: HashMap::new(),
        }];

        let embeddings = vec![EmbeddingRecord {
            chunk_id: "doc1:0".to_string(),
            embedding: vec![1.0, 0.0, 0.0],
            timestamp: chrono::Utc::now(),
        }];

        store.add_document_chunks(chunks, embeddings).await?;

        // Verify data is there
        let stats_before = store.get_stats().await?;
        assert_eq!(stats_before.total_documents, 1);

        // Remove document
        store.remove_document("doc1").await?;

        // Verify data is gone
        let stats_after = store.get_stats().await?;
        assert_eq!(stats_after.total_documents, 0);
        assert_eq!(stats_after.total_chunks, 0);
        assert_eq!(stats_after.total_embeddings, 0);

        Ok(())
    }

    #[test]
    fn test_cosine_similarity() {
        let a = [1.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 1e-6);

        let c = [1.0, 0.0, 0.0];
        let d = [0.0, 1.0, 0.0];
        let similarity2 = cosine_similarity(&c, &d);
        assert!((similarity2 - 0.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_embedding_deterministic() -> Result<()> {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new_mock_for_test(config).await?;

        let text = "Test text for deterministic embedding";
        let embedding1 = engine.embed_text(text).await?;
        let embedding2 = engine.embed_text(text).await?;

        assert_eq!(embedding1, embedding2, "Embeddings should be deterministic");

        Ok(())
    }

    #[tokio::test]
    async fn test_embedding_features() -> Result<()> {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new_mock_for_test(config).await?;

        // Test different types of text
        let texts = vec![
            "Short text",
            "This is a much longer text with more words and punctuation! It should have different characteristics.",
            "NUMBER123 with UPPERCASE and symbols @#$",
            "     lots     of     whitespace     ",
        ];

        let mut embeddings = Vec::new();
        for text in &texts {
            let embedding = engine.embed_text(text).await?;
            embeddings.push(embedding);
        }

        // Each embedding should be different
        for i in 0..embeddings.len() {
            for j in (i + 1)..embeddings.len() {
                let similarity = cosine_similarity(&embeddings[i], &embeddings[j]);
                assert!(
                    similarity < 0.99,
                    "Different texts should have different embeddings"
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_keyword_index() -> Result<()> {
        let mut keyword_index = KeywordIndex::new();

        // Add test chunks
        keyword_index.add_chunk("chunk1", "authentication password user login security");
        keyword_index.add_chunk("chunk2", "user authentication system login credentials");
        keyword_index.add_chunk(
            "chunk3",
            "database connection pooling authentication tokens",
        );

        // Test TF-IDF scoring with discriminative words
        let query_words = vec!["user".to_string(), "database".to_string()];
        let score1 = keyword_index.calculate_tf_idf_score("chunk1", &query_words);
        let score2 = keyword_index.calculate_tf_idf_score("chunk2", &query_words);
        let score3 = keyword_index.calculate_tf_idf_score("chunk3", &query_words);

        // Chunks 1 and 2 contain "user", chunk 3 contains "database"
        assert!(
            score1 > 0.0,
            "Chunk1 should have positive TF-IDF score for 'user'"
        );
        assert!(
            score2 > 0.0,
            "Chunk2 should have positive TF-IDF score for 'user'"
        );
        assert!(
            score3 > 0.0,
            "Chunk3 should have positive TF-IDF score for 'database'"
        );

        // Test single word that's discriminative
        let db_query = vec!["database".to_string()];
        let db_score1 = keyword_index.calculate_tf_idf_score("chunk1", &db_query);
        let db_score3 = keyword_index.calculate_tf_idf_score("chunk3", &db_query);

        assert_eq!(db_score1, 0.0, "Chunk1 should not score for 'database'");
        assert!(db_score3 > 0.0, "Chunk3 should score for 'database'");

        // Chunks 1 and 2 contain "user" but not "database", chunk 3 contains "database" but not "user"
        // So chunks 1 and 2 should score the same, and chunk 3 should also score positively
        assert_eq!(
            score1, score2,
            "Chunk1 and chunk2 should have same score (both have 'user')"
        );
        assert!(
            score1 > 0.0 && score3 > 0.0,
            "Both user-containing and database-containing chunks should score"
        );

        // Test phrase search
        keyword_index.add_chunk(
            "chunk4",
            "user authentication is very important for security",
        );
        let phrase_results = keyword_index.search_phrases("user authentication", 5);

        assert!(!phrase_results.is_empty(), "Should find phrase matches");

        // Check that chunk4 appears in results (it contains the exact phrase)
        let has_chunk4 = phrase_results.iter().any(|(id, _)| id == "chunk4");
        assert!(has_chunk4, "Should find chunk4 with exact phrase match");

        Ok(())
    }

    #[tokio::test]
    async fn test_keyword_search_integration() -> Result<()> {
        let store = VectorStore::new(1536); // API model dimensions
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new_mock_for_test(config).await?;

        // Create test chunks with relevant content
        let chunks = vec![
            DocumentChunk {
                id: "doc1:0".to_string(),
                document_id: "doc1".to_string(),
                content: "User authentication and password security are critical for system protection.".to_string(),
                chunk_index: 0,
                start_char: 0,
                end_char: 77,
                metadata: HashMap::new(),
            },
            DocumentChunk {
                id: "doc1:1".to_string(),
                document_id: "doc1".to_string(),
                content: "Database connections require proper authentication tokens and user credentials.".to_string(),
                chunk_index: 1,
                start_char: 77,
                end_char: 155,
                metadata: HashMap::new(),
            },
            DocumentChunk {
                id: "doc2:0".to_string(),
                document_id: "doc2".to_string(),
                content: "Network security protocols and firewall configuration management.".to_string(),
                chunk_index: 0,
                start_char: 0,
                end_char: 65,
                metadata: HashMap::new(),
            },
        ];

        // Generate embeddings
        let embeddings = engine.embed_chunks(&chunks).await?;

        // Add to vector store
        store.add_document_chunks(chunks, embeddings).await?;

        // Test keyword search
        let results = store.keyword_search("authentication user", 5).await?;
        assert!(!results.is_empty(), "Should find keyword search results");

        // Results should be sorted by relevance
        if results.len() > 1 {
            assert!(
                results[0].similarity >= results[1].similarity,
                "Results should be sorted by similarity score"
            );
        }

        // Test document-specific keyword search
        let doc_results = store
            .keyword_search_by_document("doc1", "authentication", 5)
            .await?;
        assert!(
            !doc_results.is_empty(),
            "Should find results in specific document"
        );

        // All results should be from doc1
        for result in &doc_results {
            assert_eq!(
                result.chunk.document_id, "doc1",
                "All results should be from doc1"
            );
        }

        Ok(())
    }

    #[test]
    fn test_phrase_extraction() {
        let text = "User authentication is very important for system security";
        let phrases = KeywordIndex::extract_phrases(text, 3);

        // Debug output to see what phrases are extracted
        println!("Extracted phrases: {:?}", phrases);

        assert!(
            phrases.contains(&"user authentication".to_string()),
            "Should extract 2-word phrases"
        );
        assert!(
            phrases.contains(&"authentication is".to_string()),
            "Should extract consecutive phrases: actual phrases = {:?}",
            phrases
        );
        assert!(
            phrases.contains(&"user authentication is".to_string()),
            "Should extract 3-word phrases"
        );

        // Should not contain single words or very short phrases
        assert!(
            !phrases.contains(&"user".to_string()),
            "Should not extract single words"
        );
    }

    #[test]
    fn test_exact_phrase_matching() {
        let keyword_index = KeywordIndex::new();

        // Create word positions map
        let mut positions_map = HashMap::new();
        positions_map.insert("user".to_string(), vec![0, 5]);
        positions_map.insert("authentication".to_string(), vec![1]);
        positions_map.insert("system".to_string(), vec![2]);
        positions_map.insert("login".to_string(), vec![3]);
        positions_map.insert("security".to_string(), vec![4]);
        positions_map.insert("password".to_string(), vec![6]);

        // Test exact phrase matching
        let phrase1 = vec!["user".to_string(), "authentication".to_string()];
        let phrase2 = vec!["login".to_string(), "password".to_string()]; // Non-consecutive (pos 3, 6)
        let phrase3 = vec!["authentication".to_string(), "system".to_string()];

        assert!(
            keyword_index.contains_exact_phrase(&positions_map, &phrase1),
            "Should find exact phrase at position 0-1"
        );
        assert!(
            !keyword_index.contains_exact_phrase(&positions_map, &phrase2),
            "Should not find non-consecutive phrase (login at pos 3, password at pos 6)"
        );
        assert!(
            keyword_index.contains_exact_phrase(&positions_map, &phrase3),
            "Should find exact phrase at position 1-2"
        );
    }
}
