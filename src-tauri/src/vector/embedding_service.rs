// src-tauri/src/vector/embedding_service.rs
use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingServiceConfig {
    pub provider: EmbeddingProvider,
    pub api_key: Option<String>,
    pub model_name: String,
    pub dimension: usize,
    pub max_tokens: usize,
    pub batch_size: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmbeddingProvider {
    OpenAI,
    OpenRouter,
    // Local and Fallback providers removed for safety
}

impl Default for EmbeddingServiceConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::OpenAI, // Default to OpenAI (safer)
            api_key: None,
            model_name: "text-embedding-ada-002".to_string(),
            dimension: 1536, // OpenAI ada-002 dimension
            max_tokens: 8192,
            batch_size: 25,      // Reduced batch size for safety
            timeout_seconds: 30, // Reduced timeout to prevent hangs
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIEmbeddingRequest {
    input: Vec<String>,
    model: String,
    encoding_format: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    #[allow(dead_code)] // Field required for API response deserialization
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Clone)]
pub struct EmbeddingService {
    config: EmbeddingServiceConfig,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    usage_stats: Arc<RwLock<UsageStats>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub cache_hits: u64,
    pub errors: u64,
}

impl EmbeddingService {
    pub async fn new(config: EmbeddingServiceConfig) -> Result<Self> {
        // Create HTTP client with aggressive timeout settings to prevent system hangs
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds.min(20))) // Max 20s total
            .connect_timeout(std::time::Duration::from_secs(5)) // 5s to connect
            .read_timeout(std::time::Duration::from_secs(15)) // 15s to read response
            .tcp_keepalive(std::time::Duration::from_secs(60)) // Keep connections alive
            .pool_idle_timeout(std::time::Duration::from_secs(10)) // Close idle connections quickly
            .pool_max_idle_per_host(2) // Limit connection pool size
            .build()?;

        info!(
            "Initializing embedding service with provider: {:?}, model: {}",
            config.provider, config.model_name
        );

        Ok(Self {
            config,
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(RwLock::new(UsageStats::default())),
        })
    }

    pub async fn get_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Add aggressive timeout wrapper around the entire operation
        let timeout_duration = std::time::Duration::from_secs(self.config.timeout_seconds.min(20)); // Max 20 seconds for API calls

        tracing::info!(
            "🕒 Starting embedding generation for {} texts with {}s timeout",
            texts.len(),
            timeout_duration.as_secs()
        );

        match tokio::time::timeout(timeout_duration, self.get_embeddings_internal(texts)).await {
            Ok(result) => {
                tracing::info!("✅ Embedding generation completed successfully");
                result
            }
            Err(_) => {
                tracing::error!("⏰ CRITICAL: Embedding generation timed out after {} seconds - API may be hanging", timeout_duration.as_secs());
                Err(anyhow!(
                    "CRITICAL: Embedding generation timed out after {} seconds. This may indicate API connectivity issues or system overload.",
                    timeout_duration.as_secs()
                ))
            }
        }
    }

    async fn get_embeddings_internal(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Check cache first
        let mut cache_hits = Vec::new();
        let mut cache_misses = Vec::new();
        let mut cache_miss_indices = Vec::new();

        {
            let cache = self.cache.read().await;
            for (i, text) in texts.iter().enumerate() {
                let cache_key = self.generate_cache_key(text);
                if let Some(embedding) = cache.get(&cache_key) {
                    cache_hits.push((i, embedding.clone()));
                } else {
                    cache_misses.push(text.clone());
                    cache_miss_indices.push(i);
                }
            }
        }

        // Update cache hit stats
        {
            let mut stats = self.usage_stats.write().await;
            stats.cache_hits += cache_hits.len() as u64;
        }

        // If all texts were cached, return cached results
        if cache_misses.is_empty() {
            let mut results = vec![Vec::new(); texts.len()];
            for (i, embedding) in cache_hits {
                results[i] = embedding;
            }
            return Ok(results);
        }

        // Generate embeddings for cache misses - API ONLY
        let new_embeddings = match self.config.provider {
            EmbeddingProvider::OpenAI => self.get_openai_embeddings(cache_misses).await?,
            EmbeddingProvider::OpenRouter => self.get_openrouter_embeddings(cache_misses).await?,
        };

        // Cache new embeddings
        {
            let mut cache = self.cache.write().await;
            for (i, text) in texts.iter().enumerate() {
                if cache_miss_indices.contains(&i) {
                    let cache_key = self.generate_cache_key(text);
                    let embedding_index = cache_miss_indices.iter().position(|&x| x == i).unwrap();
                    cache.insert(cache_key, new_embeddings[embedding_index].clone());
                }
            }
        }

        // Combine cached and new embeddings
        let mut results = vec![Vec::new(); texts.len()];
        for (i, embedding) in cache_hits {
            results[i] = embedding;
        }
        for (local_idx, global_idx) in cache_miss_indices.iter().enumerate() {
            results[*global_idx] = new_embeddings[local_idx].clone();
        }

        // Update usage stats
        {
            let mut stats = self.usage_stats.write().await;
            stats.total_requests += 1;
            stats.total_tokens += texts.iter().map(|t| self.estimate_tokens(t)).sum::<u64>();
        }

        Ok(results)
    }

    pub async fn get_embedding(&self, text: String) -> Result<Vec<f32>> {
        let embeddings = self.get_embeddings(vec![text]).await?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }

    async fn get_openai_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("OpenAI API key not configured"))?;

        debug!(
            "Requesting OpenAI embeddings for {} texts using model: {}",
            texts.len(),
            self.config.model_name
        );

        // Support custom dimensions for cost optimization
        let custom_dimensions = std::env::var("EMBEDDING_DIMENSIONS")
            .ok()
            .and_then(|d| d.parse::<usize>().ok());

        let request = if let Some(dimensions) = custom_dimensions {
            debug!(
                "Using custom embedding dimensions: {} (reduced from {})",
                dimensions, self.config.dimension
            );
            serde_json::json!({
                "input": texts,
                "model": self.config.model_name,
                "encoding_format": "float",
                "dimensions": dimensions
            })
        } else {
            serde_json::json!({
                "input": texts,
                "model": self.config.model_name,
                "encoding_format": "float"
            })
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let embedding_response: OpenAIEmbeddingResponse = response.json().await?;

        // Sort embeddings by index to maintain order
        let mut embeddings: Vec<(usize, Vec<f32>)> = embedding_response
            .data
            .into_iter()
            .map(|data| (data.index, data.embedding))
            .collect();
        embeddings.sort_by_key(|(index, _)| *index);

        let result: Vec<Vec<f32>> = embeddings
            .into_iter()
            .map(|(_, embedding)| embedding)
            .collect();

        info!(
            "Successfully generated {} OpenAI embeddings (tokens used: {})",
            result.len(),
            embedding_response.usage.total_tokens
        );

        Ok(result)
    }

    async fn get_openrouter_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("OpenRouter API key not configured"))?;

        debug!("Requesting OpenRouter embeddings for {} texts", texts.len());

        // OpenRouter uses the same format as OpenAI but different endpoint
        let request = OpenAIEmbeddingRequest {
            input: texts,
            model: self.config.model_name.clone(),
            encoding_format: "float".to_string(),
        };

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://proxemic.ai") // Required by OpenRouter
            .header("X-Title", "Proxemic AI Content System")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenRouter API error: {}", error_text));
        }

        let embedding_response: OpenAIEmbeddingResponse = response.json().await?;

        // Sort embeddings by index to maintain order
        let mut embeddings: Vec<(usize, Vec<f32>)> = embedding_response
            .data
            .into_iter()
            .map(|data| (data.index, data.embedding))
            .collect();
        embeddings.sort_by_key(|(index, _)| *index);

        let result: Vec<Vec<f32>> = embeddings
            .into_iter()
            .map(|(_, embedding)| embedding)
            .collect();

        info!(
            "Successfully generated {} OpenRouter embeddings (tokens used: {})",
            result.len(),
            embedding_response.usage.total_tokens
        );

        Ok(result)
    }

    // Local embedding methods completely removed for safety
    // System now requires API keys to prevent CPU overload and crashes

    fn generate_cache_key(&self, text: &str) -> String {
        format!(
            "{}:{}:{}",
            self.config.model_name,
            text.len(),
            self.simple_hash(text)
        )
    }

    fn simple_hash(&self, text: &str) -> usize {
        let mut hash = 5381usize;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as usize);
        }
        hash
    }

    fn estimate_tokens(&self, text: &str) -> u64 {
        // Rough estimation: 1 token ≈ 4 characters for English text
        (text.len() as f64 / 4.0).ceil() as u64
    }

    pub async fn get_usage_stats(&self) -> UsageStats {
        self.usage_stats.read().await.clone()
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    #[allow(dead_code)]
    pub async fn get_cache_size(&self) -> usize {
        self.cache.read().await.len()
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> &EmbeddingServiceConfig {
        &self.config
    }

    pub async fn test_connection(&self) -> Result<bool> {
        // Test API connection with a small embedding request
        let test_result = self.get_embeddings(vec!["test".to_string()]).await;
        match test_result {
            Ok(_) => {
                info!("✅ API embedding service connection test successful");
                Ok(true)
            }
            Err(e) => {
                warn!("❌ API embedding service connection test failed: {}", e);
                Ok(false)
            }
        }
    }

    #[allow(dead_code)]
    fn generate_deterministic_embedding(&self, text: &str) -> Vec<f32> {
        // Generate a deterministic embedding based on text features
        let mut embedding = vec![0.0f32; self.config.dimension];

        // Use various text features to create embedding values
        let bytes = text.as_bytes();
        let len = bytes.len();

        for (i, val) in embedding.iter_mut().enumerate() {
            let feature_index = i % 10; // Cycle through different features

            *val = match feature_index {
                0 => (len as f32).sin(), // Length-based feature
                1 => bytes.iter().map(|&b| b as f32).sum::<f32>() / len as f32 / 255.0, // Average byte value
                2 => text.chars().count() as f32 / len as f32, // Character density
                3 => {
                    text.chars().filter(|c| c.is_uppercase()).count() as f32
                        / text.chars().count() as f32
                }
                4 => {
                    text.chars().filter(|c| c.is_alphabetic()).count() as f32
                        / text.chars().count() as f32
                }
                5 => {
                    text.chars().filter(|c| c.is_numeric()).count() as f32
                        / text.chars().count() as f32
                }
                6 => {
                    text.chars().filter(|c| c.is_whitespace()).count() as f32
                        / text.chars().count() as f32
                }
                7 => {
                    text.chars().filter(|c| c.is_ascii_punctuation()).count() as f32
                        / text.chars().count() as f32
                }
                8 => (self.simple_hash(text) as f32).sin(),
                9 => (self.simple_hash(&text.to_lowercase()) as f32).cos(),
                _ => 0.0,
            };
        }

        // Normalize the embedding vector
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in embedding.iter_mut() {
                *val /= magnitude;
            }
        }

        embedding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_service_creation() -> Result<()> {
        let config = EmbeddingServiceConfig::default();
        let service = EmbeddingService::new(config).await?;
        assert_eq!(service.get_config().provider, EmbeddingProvider::OpenAI);
        Ok(())
    }

    #[tokio::test]
    async fn test_fallback_embeddings() -> Result<()> {
        // Skip test if no OpenAI API key is configured
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                println!("Skipping test: OPENAI_API_KEY not configured");
                return Ok(());
            }
        };

        let config = EmbeddingServiceConfig {
            provider: EmbeddingProvider::OpenAI,
            api_key: Some(api_key),
            dimension: 10,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).await?;

        let texts = vec!["Hello world".to_string(), "Test document".to_string()];
        let embeddings = service.get_embeddings(texts).await?;

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 10);
        assert_eq!(embeddings[1].len(), 10);

        // Different texts should produce different embeddings
        assert_ne!(embeddings[0], embeddings[1]);

        Ok(())
    }

    #[tokio::test]
    async fn test_embedding_caching() -> Result<()> {
        // Skip test if no OpenAI API key is configured
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                println!("Skipping test: OPENAI_API_KEY not configured");
                return Ok(());
            }
        };

        let config = EmbeddingServiceConfig {
            provider: EmbeddingProvider::OpenAI,
            api_key: Some(api_key),
            dimension: 5,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).await?;

        let text = "Test caching".to_string();

        // First call - should be cached
        let embedding1 = service.get_embedding(text.clone()).await?;
        let stats1 = service.get_usage_stats().await;

        // Second call - should use cache
        let embedding2 = service.get_embedding(text.clone()).await?;
        let stats2 = service.get_usage_stats().await;

        assert_eq!(embedding1, embedding2);
        assert_eq!(stats2.cache_hits, stats1.cache_hits + 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_deterministic_embeddings() -> Result<()> {
        // Skip test if no OpenAI API key is configured
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                println!("Skipping test: OPENAI_API_KEY not configured");
                return Ok(());
            }
        };

        let config = EmbeddingServiceConfig {
            provider: EmbeddingProvider::OpenAI,
            api_key: Some(api_key),
            dimension: 5,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).await?;

        let text = "Deterministic test";
        let embedding1 = service.get_embedding(text.to_string()).await?;
        let embedding2 = service.get_embedding(text.to_string()).await?;

        assert_eq!(embedding1, embedding2, "Embeddings should be deterministic");

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_embeddings() -> Result<()> {
        // Skip test if no OpenAI API key is configured
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                println!("Skipping test: OPENAI_API_KEY not configured");
                return Ok(());
            }
        };

        let config = EmbeddingServiceConfig {
            provider: EmbeddingProvider::OpenAI,
            api_key: Some(api_key),
            dimension: 3,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).await?;

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = service.get_embeddings(texts.clone()).await?;

        assert_eq!(embeddings.len(), 3);
        for embedding in &embeddings {
            assert_eq!(embedding.len(), 3);
        }

        // Each embedding should be unique
        assert_ne!(embeddings[0], embeddings[1]);
        assert_ne!(embeddings[1], embeddings[2]);
        assert_ne!(embeddings[0], embeddings[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_text_features() -> Result<()> {
        let config = EmbeddingServiceConfig {
            provider: EmbeddingProvider::OpenAI,
            dimension: 10,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).await?;

        // Different texts should have different feature values
        let text1 = "simple text";
        let text2 = "COMPLEX TEXT WITH PUNCTUATION!!! AND NUMBERS 123";

        let embedding1 = service.generate_deterministic_embedding(text1);
        let embedding2 = service.generate_deterministic_embedding(text2);

        assert_ne!(embedding1, embedding2);

        // Check that embeddings are normalized
        let magnitude1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = embedding2.iter().map(|x| x * x).sum::<f32>().sqrt();

        assert!(
            (magnitude1 - 1.0).abs() < 1e-6,
            "Embedding should be normalized"
        );
        assert!(
            (magnitude2 - 1.0).abs() < 1e-6,
            "Embedding should be normalized"
        );

        Ok(())
    }
}
