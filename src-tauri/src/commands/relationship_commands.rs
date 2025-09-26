// src-tauri/src/commands/relationship_commands.rs
// Tauri commands for cross-document relationship analysis

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

use crate::document::{
    DocumentIndexer, DocumentRelationship, RelationshipAnalysisResult, RelationshipAnalyzer,
    RelationshipConfig,
};

/// Request for analyzing relationships between documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRelationshipsRequest {
    /// List of document IDs to analyze
    pub document_ids: Vec<String>,
    /// Analysis configuration
    pub config: Option<RelationshipConfigDto>,
}

/// Configuration for relationship analysis (DTO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipConfigDto {
    /// Minimum similarity threshold for topic similarity
    pub topic_similarity_threshold: Option<f32>,
    /// Minimum overlap for concept relationships
    pub concept_overlap_threshold: Option<f32>,
    /// Minimum shared terms for term-based relationships
    pub shared_terms_threshold: Option<usize>,
    /// Whether to use semantic embeddings
    pub use_semantic_analysis: Option<bool>,
    /// Maximum number of relationships per document
    pub max_relationships_per_document: Option<usize>,
    /// Minimum confidence threshold
    pub min_confidence_threshold: Option<f32>,
}

impl From<RelationshipConfigDto> for RelationshipConfig {
    fn from(dto: RelationshipConfigDto) -> Self {
        let mut config = RelationshipConfig::default();

        if let Some(threshold) = dto.topic_similarity_threshold {
            config.topic_similarity_threshold = threshold;
        }
        if let Some(threshold) = dto.concept_overlap_threshold {
            config.concept_overlap_threshold = threshold;
        }
        if let Some(threshold) = dto.shared_terms_threshold {
            config.shared_terms_threshold = threshold;
        }
        if let Some(use_semantic) = dto.use_semantic_analysis {
            config.use_semantic_analysis = use_semantic;
        }
        if let Some(max_relationships) = dto.max_relationships_per_document {
            config.max_relationships_per_document = max_relationships;
        }
        if let Some(min_confidence) = dto.min_confidence_threshold {
            config.min_confidence_threshold = min_confidence;
        }

        config
    }
}

/// Request for finding documents related to a specific document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindRelatedDocumentsRequest {
    /// Target document ID
    pub document_id: String,
    /// Maximum number of related documents to return
    pub max_results: Option<usize>,
    /// Analysis configuration
    pub config: Option<RelationshipConfigDto>,
}

/// Response for relationship analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipAnalysisResponse {
    /// Analysis results
    pub result: RelationshipAnalysisResult,
    /// Success status
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
}

/// Response for finding related documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocumentsResponse {
    /// Target document ID
    pub document_id: String,
    /// Related documents
    pub relationships: Vec<DocumentRelationship>,
    /// Total number of relationships found
    pub total_found: usize,
    /// Success status
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
}

/// Response for getting relationship types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTypesResponse {
    /// Available relationship types
    pub types: Vec<RelationshipTypeInfo>,
}

/// Information about a relationship type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTypeInfo {
    /// Type identifier
    pub type_name: String,
    /// Human-readable description
    pub description: String,
    /// Whether this type requires semantic analysis
    pub requires_semantic: bool,
}

/// State for relationship analysis
pub struct RelationshipState {
    pub analyzer: Arc<Mutex<Option<RelationshipAnalyzer>>>,
}

impl Default for RelationshipState {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipState {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(Mutex::new(None)),
        }
    }
}

/// Initialize relationship analyzer
#[tauri::command]
pub async fn initialize_relationship_analyzer(
    config_dto: Option<RelationshipConfigDto>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    tracing::info!("🔗 Initializing relationship analyzer");

    let config: RelationshipConfig = config_dto.map(Into::into).unwrap_or_default();

    // Try to get embedding engine and vector store for enhanced analysis
    let embedding_engine = match crate::vector::EmbeddingEngine::new(Default::default()).await {
        Ok(engine) => Some(engine),
        Err(e) => {
            tracing::warn!(
                "⚠️ Could not initialize embedding engine for relationship analysis: {}",
                e
            );
            None
        }
    };

    let vector_store = embedding_engine
        .as_ref()
        .map(|engine| crate::vector::VectorStore::new(engine.get_config().dimension));

    // Create analyzer
    let analyzer = if let (Some(engine), Some(store)) = (embedding_engine, vector_store) {
        match RelationshipAnalyzer::new_with_embeddings(config.clone(), engine, store).await {
            Ok(analyzer) => analyzer,
            Err(e) => {
                tracing::warn!(
                    "⚠️ Could not create enhanced relationship analyzer: {}, using basic version",
                    e
                );
                RelationshipAnalyzer::new(config)
            }
        }
    } else {
        RelationshipAnalyzer::new(config)
    };

    // Store in app state
    if let Some(state) = app_handle.try_state::<RelationshipState>() {
        let mut analyzer_lock = state.analyzer.lock().await;
        *analyzer_lock = Some(analyzer);

        tracing::info!("✅ Relationship analyzer initialized successfully");
        Ok(true)
    } else {
        let error = "Failed to access relationship analyzer state";
        tracing::error!("❌ {}", error);
        Err(error.to_string())
    }
}

/// Analyze relationships between specified documents
#[tauri::command]
pub async fn analyze_document_relationships(
    request: AnalyzeRelationshipsRequest,
    indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    relationship_state: State<'_, RelationshipState>,
) -> Result<RelationshipAnalysisResponse, String> {
    tracing::info!(
        "🔍 Starting relationship analysis for {} documents",
        request.document_ids.len()
    );

    let analyzer_guard = relationship_state.analyzer.lock().await;
    let analyzer = match analyzer_guard.as_ref() {
        Some(analyzer) => analyzer,
        None => {
            return Ok(RelationshipAnalysisResponse {
                result: RelationshipAnalysisResult {
                    relationships: vec![],
                    stats: crate::document::RelationshipStats {
                        documents_analyzed: 0,
                        total_relationships: 0,
                        relationships_by_type: std::collections::HashMap::new(),
                        relationships_by_strength: std::collections::HashMap::new(),
                        average_strength: 0.0,
                        most_connected_document: None,
                        max_connections: 0,
                    },
                    metadata: crate::document::RelationshipAnalysisMetadata {
                        analyzed_at: chrono::Utc::now(),
                        config_summary: "uninitialized".to_string(),
                        duration_ms: 0,
                        used_semantic_analysis: false,
                        isolated_documents: request.document_ids.len(),
                    },
                },
                success: false,
                error: Some("Relationship analyzer not initialized".to_string()),
            });
        }
    };

    // Get documents from indexer
    let indexer_lock = indexer.lock().await;
    let mut documents = Vec::new();

    for document_id in &request.document_ids {
        if let Some(doc) = indexer_lock.get_document(document_id) {
            documents.push(doc.clone());
        } else {
            tracing::warn!("⚠️ Document not found in index: {}", document_id);
        }
    }
    drop(indexer_lock);

    if documents.is_empty() {
        return Ok(RelationshipAnalysisResponse {
            result: RelationshipAnalysisResult {
                relationships: vec![],
                stats: crate::document::RelationshipStats {
                    documents_analyzed: 0,
                    total_relationships: 0,
                    relationships_by_type: std::collections::HashMap::new(),
                    relationships_by_strength: std::collections::HashMap::new(),
                    average_strength: 0.0,
                    most_connected_document: None,
                    max_connections: 0,
                },
                metadata: crate::document::RelationshipAnalysisMetadata {
                    analyzed_at: chrono::Utc::now(),
                    config_summary: "no documents".to_string(),
                    duration_ms: 0,
                    used_semantic_analysis: false,
                    isolated_documents: 0,
                },
            },
            success: false,
            error: Some("No valid documents found for analysis".to_string()),
        });
    }

    // Perform relationship analysis
    match analyzer.analyze_relationships(&documents).await {
        Ok(result) => {
            tracing::info!(
                "✅ Relationship analysis completed: {} relationships found",
                result.relationships.len()
            );

            Ok(RelationshipAnalysisResponse {
                result,
                success: true,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Failed to analyze relationships: {}", e);
            tracing::error!("❌ {}", error_msg);

            Ok(RelationshipAnalysisResponse {
                result: RelationshipAnalysisResult {
                    relationships: vec![],
                    stats: crate::document::RelationshipStats {
                        documents_analyzed: documents.len(),
                        total_relationships: 0,
                        relationships_by_type: std::collections::HashMap::new(),
                        relationships_by_strength: std::collections::HashMap::new(),
                        average_strength: 0.0,
                        most_connected_document: None,
                        max_connections: 0,
                    },
                    metadata: crate::document::RelationshipAnalysisMetadata {
                        analyzed_at: chrono::Utc::now(),
                        config_summary: "error".to_string(),
                        duration_ms: 0,
                        used_semantic_analysis: false,
                        isolated_documents: documents.len(),
                    },
                },
                success: false,
                error: Some(error_msg),
            })
        }
    }
}

/// Find documents related to a specific document
#[tauri::command]
pub async fn find_related_documents(
    request: FindRelatedDocumentsRequest,
    indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    relationship_state: State<'_, RelationshipState>,
) -> Result<RelatedDocumentsResponse, String> {
    tracing::info!("🔍 Finding documents related to: {}", request.document_id);

    let analyzer_guard = relationship_state.analyzer.lock().await;
    let analyzer = match analyzer_guard.as_ref() {
        Some(analyzer) => analyzer,
        None => {
            return Ok(RelatedDocumentsResponse {
                document_id: request.document_id,
                relationships: vec![],
                total_found: 0,
                success: false,
                error: Some("Relationship analyzer not initialized".to_string()),
            });
        }
    };

    // Get target document and all documents from indexer
    let indexer_lock = indexer.lock().await;

    let target_document = match indexer_lock.get_document(&request.document_id) {
        Some(doc) => doc.clone(),
        None => {
            return Ok(RelatedDocumentsResponse {
                document_id: request.document_id,
                relationships: vec![],
                total_found: 0,
                success: false,
                error: Some("Target document not found".to_string()),
            });
        }
    };

    let all_documents: Vec<_> = indexer_lock
        .get_all_documents()
        .into_iter()
        .cloned()
        .collect();
    drop(indexer_lock);

    let max_results = request.max_results.unwrap_or(10);

    // Find related documents
    match analyzer
        .find_related_documents(&target_document, &all_documents, max_results)
        .await
    {
        Ok(relationships) => {
            tracing::info!(
                "✅ Found {} related documents for: {}",
                relationships.len(),
                request.document_id
            );

            let total_found = relationships.len();
            Ok(RelatedDocumentsResponse {
                document_id: request.document_id,
                relationships,
                total_found,
                success: true,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Failed to find related documents: {}", e);
            tracing::error!("❌ {}", error_msg);

            Ok(RelatedDocumentsResponse {
                document_id: request.document_id,
                relationships: vec![],
                total_found: 0,
                success: false,
                error: Some(error_msg),
            })
        }
    }
}

/// Get available relationship types
#[tauri::command]
pub async fn get_relationship_types() -> Result<RelationshipTypesResponse, String> {
    let types = vec![
        RelationshipTypeInfo {
            type_name: "TopicSimilarity".to_string(),
            description: "Documents covering similar topics based on shared keywords and concepts"
                .to_string(),
            requires_semantic: false,
        },
        RelationshipTypeInfo {
            type_name: "ConceptOverlap".to_string(),
            description: "Documents with overlapping concepts and terminology".to_string(),
            requires_semantic: false,
        },
        RelationshipTypeInfo {
            type_name: "CrossReference".to_string(),
            description: "Documents that reference each other or related content".to_string(),
            requires_semantic: false,
        },
        RelationshipTypeInfo {
            type_name: "Prerequisite".to_string(),
            description: "Documents that are prerequisites for understanding others".to_string(),
            requires_semantic: true,
        },
        RelationshipTypeInfo {
            type_name: "Complementary".to_string(),
            description: "Documents that complement each other with additional information"
                .to_string(),
            requires_semantic: true,
        },
        RelationshipTypeInfo {
            type_name: "DetailLevel".to_string(),
            description: "Documents at different detail levels of the same topic".to_string(),
            requires_semantic: true,
        },
        RelationshipTypeInfo {
            type_name: "ProcessFlow".to_string(),
            description: "Documents that are part of the same process or workflow".to_string(),
            requires_semantic: false,
        },
        RelationshipTypeInfo {
            type_name: "StructuralSimilarity".to_string(),
            description: "Documents with similar structure and organization".to_string(),
            requires_semantic: false,
        },
    ];

    Ok(RelationshipTypesResponse { types })
}

/// Analyze relationships between two specific documents
#[tauri::command]
pub async fn analyze_document_pair(
    document_a_id: String,
    document_b_id: String,
    indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    relationship_state: State<'_, RelationshipState>,
) -> Result<Vec<DocumentRelationship>, String> {
    tracing::info!(
        "🔍 Analyzing relationship between {} and {}",
        document_a_id,
        document_b_id
    );

    let analyzer_guard = relationship_state.analyzer.lock().await;
    let analyzer = match analyzer_guard.as_ref() {
        Some(analyzer) => analyzer,
        None => {
            return Err("Relationship analyzer not initialized".to_string());
        }
    };

    // Get both documents from indexer
    let indexer_lock = indexer.lock().await;

    let doc_a = match indexer_lock.get_document(&document_a_id) {
        Some(doc) => doc.clone(),
        None => return Err("First document not found".to_string()),
    };

    let doc_b = match indexer_lock.get_document(&document_b_id) {
        Some(doc) => doc.clone(),
        None => return Err("Second document not found".to_string()),
    };
    drop(indexer_lock);

    // Analyze the pair
    match analyzer.analyze_document_pair(&doc_a, &doc_b).await {
        Ok(relationships) => {
            tracing::info!(
                "✅ Found {} relationships between {} and {}",
                relationships.len(),
                document_a_id,
                document_b_id
            );
            Ok(relationships)
        }
        Err(e) => {
            let error_msg = format!("Failed to analyze document pair: {}", e);
            tracing::error!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_config_conversion() {
        let dto = RelationshipConfigDto {
            topic_similarity_threshold: Some(0.5),
            concept_overlap_threshold: Some(0.3),
            shared_terms_threshold: Some(5),
            use_semantic_analysis: Some(false),
            max_relationships_per_document: Some(15),
            min_confidence_threshold: Some(0.4),
        };

        let config: RelationshipConfig = dto.into();
        assert_eq!(config.topic_similarity_threshold, 0.5);
        assert_eq!(config.concept_overlap_threshold, 0.3);
        assert_eq!(config.shared_terms_threshold, 5);
        assert_eq!(config.use_semantic_analysis, false);
        assert_eq!(config.max_relationships_per_document, 15);
        assert_eq!(config.min_confidence_threshold, 0.4);
    }

    #[test]
    fn test_relationship_config_partial_conversion() {
        let dto = RelationshipConfigDto {
            topic_similarity_threshold: Some(0.8),
            concept_overlap_threshold: None,
            shared_terms_threshold: None,
            use_semantic_analysis: Some(true),
            max_relationships_per_document: None,
            min_confidence_threshold: None,
        };

        let config: RelationshipConfig = dto.into();
        assert_eq!(config.topic_similarity_threshold, 0.8);
        assert_eq!(config.concept_overlap_threshold, 0.25); // default
        assert_eq!(config.use_semantic_analysis, true);
        assert_eq!(config.max_relationships_per_document, 10); // default
    }

    #[test]
    fn test_relationship_types_info() {
        let types = vec![RelationshipTypeInfo {
            type_name: "TopicSimilarity".to_string(),
            description: "Test description".to_string(),
            requires_semantic: false,
        }];

        assert_eq!(types[0].type_name, "TopicSimilarity");
        assert!(!types[0].requires_semantic);
    }
}
