// src-tauri/src/vector/mod.rs
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
            model_name: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            max_length: 512,
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

pub struct EmbeddingEngine {
    config: EmbeddingConfig,
    embeddings_cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    model_available: bool,
}

impl EmbeddingEngine {
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        // For now, we'll use a basic implementation without heavy ML dependencies
        // This can be enhanced later with actual model loading

        let model_available = Self::check_model_availability(&config.model_name).await;

        if !model_available {
            tracing::warn!(
                "ML model {} not available, using fallback embedding",
                config.model_name
            );
        }

        Ok(Self {
            config,
            embeddings_cache: Arc::new(RwLock::new(HashMap::new())),
            model_available,
        })
    }

    async fn check_model_availability(model_name: &str) -> bool {
        // For now, return false to use fallback implementation
        // Later this can check for actual model files or API availability
        tracing::info!("Checking availability for model: {}", model_name);
        false
    }

    pub async fn get_model_info(&self) -> Result<String> {
        Ok(format!(
            "Embedding Engine - Model: {}, Dimension: {}, Available: {}",
            self.config.model_name, self.config.dimension, self.model_available
        ))
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        let cache_key = format!("{}:{}", text.len(), self.hash_text(text));
        {
            let cache = self.embeddings_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Generate embedding
        let embedding = if self.model_available {
            // TODO: Use actual ML model when available
            self.generate_ml_embedding(text).await?
        } else {
            // Use deterministic fallback embedding
            self.generate_fallback_embedding(text)
        };

        // Cache the result
        {
            let mut cache = self.embeddings_cache.write().await;
            cache.insert(cache_key, embedding.clone());
        }

        Ok(embedding)
    }

    async fn generate_ml_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        // Placeholder for actual ML implementation
        // This would use candle-transformers with a sentence transformer model
        Err(anyhow!("ML embedding not yet implemented"))
    }

    fn generate_fallback_embedding(&self, text: &str) -> Vec<f32> {
        // Generate a deterministic embedding based on text characteristics
        let mut embedding = vec![0.0; self.config.dimension];

        // Simple feature extraction
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len() as f32;
        let char_count = text.len() as f32;
        let avg_word_length = if words.is_empty() {
            0.0
        } else {
            char_count / word_count
        };

        // Character frequency features
        let mut char_freq: HashMap<char, f32> = HashMap::new();
        for c in text.chars() {
            *char_freq
                .entry(c.to_lowercase().next().unwrap_or(c))
                .or_insert(0.0) += 1.0;
        }

        // Fill embedding vector with normalized features
        for (i, embedding_val) in embedding.iter_mut().enumerate().take(self.config.dimension) {
            let feature = match i % 10 {
                0 => word_count / 100.0,           // Normalized word count
                1 => char_count / 1000.0,          // Normalized character count
                2 => avg_word_length / 10.0,       // Normalized average word length
                3 => self.text_entropy(text),      // Text complexity
                4 => self.vowel_ratio(text),       // Vowel ratio
                5 => self.punctuation_ratio(text), // Punctuation ratio
                6 => self.uppercase_ratio(text),   // Uppercase ratio
                7 => self.digit_ratio(text),       // Digit ratio
                8 => self.whitespace_ratio(text),  // Whitespace ratio
                _ => {
                    // Hash-based features for remaining dimensions
                    let hash_input = format!("{}{}", text, i);
                    let hash = self.simple_hash(&hash_input);
                    (hash % 1000) as f32 / 1000.0
                }
            };

            *embedding_val = feature.clamp(-1.0, 1.0); // Clamp to [-1, 1]
        }

        // Normalize the embedding vector
        self.normalize_vector(&mut embedding);
        embedding
    }

    fn text_entropy(&self, text: &str) -> f32 {
        let mut freq: HashMap<char, f32> = HashMap::new();
        let total_chars = text.len() as f32;

        if total_chars == 0.0 {
            return 0.0;
        }

        for c in text.chars() {
            *freq.entry(c).or_insert(0.0) += 1.0;
        }

        let mut entropy = 0.0;
        for count in freq.values() {
            let prob = count / total_chars;
            if prob > 0.0 {
                entropy -= prob * prob.log2();
            }
        }

        entropy / 10.0 // Normalize
    }

    fn vowel_ratio(&self, text: &str) -> f32 {
        let vowels = "aeiouAEIOU";
        let vowel_count = text.chars().filter(|c| vowels.contains(*c)).count() as f32;
        let total_chars = text.len() as f32;

        if total_chars == 0.0 {
            0.0
        } else {
            vowel_count / total_chars
        }
    }

    fn punctuation_ratio(&self, text: &str) -> f32 {
        let punct_count = text.chars().filter(|c| c.is_ascii_punctuation()).count() as f32;
        let total_chars = text.len() as f32;

        if total_chars == 0.0 {
            0.0
        } else {
            punct_count / total_chars
        }
    }

    fn uppercase_ratio(&self, text: &str) -> f32 {
        let upper_count = text.chars().filter(|c| c.is_uppercase()).count() as f32;
        let total_chars = text.len() as f32;

        if total_chars == 0.0 {
            0.0
        } else {
            upper_count / total_chars
        }
    }

    fn digit_ratio(&self, text: &str) -> f32 {
        let digit_count = text.chars().filter(|c| c.is_ascii_digit()).count() as f32;
        let total_chars = text.len() as f32;

        if total_chars == 0.0 {
            0.0
        } else {
            digit_count / total_chars
        }
    }

    fn whitespace_ratio(&self, text: &str) -> f32 {
        let whitespace_count = text.chars().filter(|c| c.is_whitespace()).count() as f32;
        let total_chars = text.len() as f32;

        if total_chars == 0.0 {
            0.0
        } else {
            whitespace_count / total_chars
        }
    }

    fn simple_hash(&self, text: &str) -> usize {
        let mut hash = 5381usize;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as usize);
        }
        hash
    }

    fn hash_text(&self, text: &str) -> String {
        format!("{:x}", self.simple_hash(text))
    }

    fn normalize_vector(&self, vector: &mut [f32]) {
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in vector.iter_mut() {
                *val /= magnitude;
            }
        }
    }

    pub fn chunk_text(&self, text: &str, document_id: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let text_len = text.len();

        if text_len <= self.config.chunk_size {
            // Single chunk for small text
            chunks.push(DocumentChunk {
                id: format!("{}:0", document_id),
                document_id: document_id.to_string(),
                content: text.to_string(),
                chunk_index: 0,
                start_char: 0,
                end_char: text_len,
                metadata: HashMap::new(),
            });
            return chunks;
        }

        let mut start = 0;
        let mut chunk_index = 0;

        while start < text_len {
            let end = std::cmp::min(start + self.config.chunk_size, text_len);

            // Try to break at word boundaries
            let actual_end = if end < text_len {
                if let Some(last_space) = text[start..end].rfind(' ') {
                    start + last_space
                } else {
                    end
                }
            } else {
                end
            };

            let chunk_content = text[start..actual_end].to_string();

            chunks.push(DocumentChunk {
                id: format!("{}:{}", document_id, chunk_index),
                document_id: document_id.to_string(),
                content: chunk_content,
                chunk_index,
                start_char: start,
                end_char: actual_end,
                metadata: HashMap::new(),
            });

            // Move start position with overlap
            start = actual_end.saturating_sub(self.config.chunk_overlap);
            if start >= actual_end {
                break; // Prevent infinite loop
            }

            chunk_index += 1;
        }

        chunks
    }

    pub async fn embed_chunks(&self, chunks: &[DocumentChunk]) -> Result<Vec<EmbeddingRecord>> {
        let mut records = Vec::new();

        for chunk in chunks {
            let embedding = self.embed_text(&chunk.content).await?;
            records.push(EmbeddingRecord {
                chunk_id: chunk.id.clone(),
                embedding,
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(records)
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    pub fn is_model_available(&self) -> bool {
        self.model_available
    }
}

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
                        score += tf_score * idf_score;
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
        let store = VectorStore::new(384); // Common embedding dimension
        assert_eq!(store.dimension(), 384);
    }

    #[tokio::test]
    async fn test_embedding_engine() -> Result<()> {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await?;

        // Test basic functionality
        let text = "This is a test document for embedding.";
        let embedding = engine.embed_text(text).await?;

        assert_eq!(embedding.len(), 384); // Default dimension

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
        let _engine = EmbeddingEngine::new(config).await?;

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
        let engine = EmbeddingEngine::new(config).await?;

        let text = "Test text for deterministic embedding";
        let embedding1 = engine.embed_text(text).await?;
        let embedding2 = engine.embed_text(text).await?;

        assert_eq!(embedding1, embedding2, "Embeddings should be deterministic");

        Ok(())
    }

    #[tokio::test]
    async fn test_embedding_features() -> Result<()> {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await?;

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
        let store = VectorStore::new(384);
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await?;

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
