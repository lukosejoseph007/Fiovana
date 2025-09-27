use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::document::indexer::DocumentIndexer;
use crate::document::style_learner::{OrganizationalStyle, StyleLearner, StyleLearningResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleLearningRequest {
    pub document_paths: Vec<String>,
    pub organization_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleLearningResponse {
    pub success: bool,
    pub result: Option<StyleLearningResult>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationalStyleRequest {
    pub organization_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationalStyleResponse {
    pub success: bool,
    pub style: Option<OrganizationalStyle>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleGuidelinesRequest {
    pub organization_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleGuidelinesResponse {
    pub success: bool,
    pub guidelines: Vec<String>,
    pub terminology_recommendations: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminologyAnalysisRequest {
    pub document_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminologyAnalysisResponse {
    pub success: bool,
    pub preferred_terms:
        std::collections::HashMap<String, crate::document::style_learner::TermFrequency>,
    pub avoided_terms: Vec<String>,
    pub domain_analysis: std::collections::HashMap<String, f64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StylePatternsRequest {
    pub document_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StylePatternsResponse {
    pub success: bool,
    pub patterns: Vec<crate::document::style_learner::StylePattern>,
    pub confidence_score: f64,
    pub error: Option<String>,
}

/// Learn organizational style from a corpus of documents
#[tauri::command]
pub async fn learn_organizational_style(
    request: StyleLearningRequest,
    indexer: State<'_, Mutex<DocumentIndexer>>,
) -> Result<StyleLearningResponse, String> {
    println!(
        "Learning organizational style for organization: {}",
        request.organization_id
    );

    let indexer = indexer
        .lock()
        .map_err(|e| format!("Failed to lock indexer: {}", e))?;
    let style_learner = StyleLearner::new();

    // Get document entries from paths
    let mut documents = Vec::new();
    let all_docs = indexer.get_all_documents();
    for path in &request.document_paths {
        if let Some(doc) = all_docs.iter().find(|d| d.path.to_string_lossy() == *path) {
            documents.push((*doc).clone());
        } else {
            return Ok(StyleLearningResponse {
                success: false,
                result: None,
                error: Some(format!("Document not found: {}", path)),
            });
        }
    }

    if documents.is_empty() {
        return Ok(StyleLearningResponse {
            success: false,
            result: None,
            error: Some("No valid documents found for learning".to_string()),
        });
    }

    match style_learner.learn_from_corpus(&documents, request.organization_id) {
        Ok(result) => {
            println!(
                "Successfully learned organizational style with {} patterns",
                result.learned_patterns.len()
            );
            Ok(StyleLearningResponse {
                success: true,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => {
            println!("Failed to learn organizational style: {}", e);
            Ok(StyleLearningResponse {
                success: false,
                result: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Get organizational style for a specific organization
#[tauri::command]
pub async fn get_organizational_style(
    request: OrganizationalStyleRequest,
) -> Result<OrganizationalStyleResponse, String> {
    println!(
        "Getting organizational style for: {}",
        request.organization_id
    );

    // For now, return a placeholder response
    // In a full implementation, this would load from storage
    Ok(OrganizationalStyleResponse {
        success: false,
        style: None,
        error: Some("Organizational style storage not yet implemented".to_string()),
    })
}

/// Generate style guidelines for an organization
#[tauri::command]
pub async fn generate_style_guidelines(
    request: StyleGuidelinesRequest,
    indexer: State<'_, Mutex<DocumentIndexer>>,
) -> Result<StyleGuidelinesResponse, String> {
    println!(
        "Generating style guidelines for organization: {}",
        request.organization_id
    );

    let indexer = indexer
        .lock()
        .map_err(|e| format!("Failed to lock indexer: {}", e))?;
    let style_learner = StyleLearner::new();

    // Get all documents for the organization (simplified approach)
    let all_documents: Vec<_> = indexer.get_all_documents().into_iter().cloned().collect();

    if all_documents.len() < 3 {
        return Ok(StyleGuidelinesResponse {
            success: false,
            guidelines: vec![],
            terminology_recommendations: vec![],
            error: Some("Need at least 3 documents to generate style guidelines".to_string()),
        });
    }

    match style_learner.learn_from_corpus(&all_documents, request.organization_id) {
        Ok(result) => {
            println!(
                "Generated {} style guidelines",
                result.style_guidelines.len()
            );
            Ok(StyleGuidelinesResponse {
                success: true,
                guidelines: result.style_guidelines,
                terminology_recommendations: result.terminology_recommendations,
                error: None,
            })
        }
        Err(e) => {
            println!("Failed to generate style guidelines: {}", e);
            Ok(StyleGuidelinesResponse {
                success: false,
                guidelines: vec![],
                terminology_recommendations: vec![],
                error: Some(e.to_string()),
            })
        }
    }
}

/// Analyze terminology usage across documents
#[tauri::command]
pub async fn analyze_terminology_usage(
    request: TerminologyAnalysisRequest,
    indexer: State<'_, Mutex<DocumentIndexer>>,
) -> Result<TerminologyAnalysisResponse, String> {
    println!(
        "Analyzing terminology usage across {} documents",
        request.document_paths.len()
    );

    let indexer = indexer
        .lock()
        .map_err(|e| format!("Failed to lock indexer: {}", e))?;
    let style_learner = StyleLearner::new();

    // Get document entries from paths
    let mut documents = Vec::new();
    let all_docs = indexer.get_all_documents();
    for path in &request.document_paths {
        if let Some(doc) = all_docs.iter().find(|d| d.path.to_string_lossy() == *path) {
            documents.push((*doc).clone());
        }
    }

    if documents.is_empty() {
        return Ok(TerminologyAnalysisResponse {
            success: false,
            preferred_terms: std::collections::HashMap::new(),
            avoided_terms: vec![],
            domain_analysis: std::collections::HashMap::new(),
            error: Some("No valid documents found for analysis".to_string()),
        });
    }

    match style_learner.learn_from_corpus(&documents, "temp_analysis".to_string()) {
        Ok(result) => {
            // Extract domain analysis from organizational style
            let mut domain_analysis = std::collections::HashMap::new();
            for pattern in &result.learned_patterns {
                if pattern.pattern_type == "vocabulary_domain" {
                    domain_analysis.insert(pattern.pattern.clone(), pattern.confidence);
                }
            }

            println!(
                "Analyzed {} preferred terms and {} avoided terms",
                result.organizational_style.preferred_terminology.len(),
                result.organizational_style.avoided_terminology.len()
            );

            Ok(TerminologyAnalysisResponse {
                success: true,
                preferred_terms: result.organizational_style.preferred_terminology,
                avoided_terms: result.organizational_style.avoided_terminology,
                domain_analysis,
                error: None,
            })
        }
        Err(e) => {
            println!("Failed to analyze terminology: {}", e);
            Ok(TerminologyAnalysisResponse {
                success: false,
                preferred_terms: std::collections::HashMap::new(),
                avoided_terms: vec![],
                domain_analysis: std::collections::HashMap::new(),
                error: Some(e.to_string()),
            })
        }
    }
}

/// Identify style patterns across documents
#[tauri::command]
pub async fn identify_style_patterns(
    request: StylePatternsRequest,
    indexer: State<'_, Mutex<DocumentIndexer>>,
) -> Result<StylePatternsResponse, String> {
    println!(
        "Identifying style patterns across {} documents",
        request.document_paths.len()
    );

    let indexer = indexer
        .lock()
        .map_err(|e| format!("Failed to lock indexer: {}", e))?;
    let style_learner = StyleLearner::new();

    // Get document entries from paths
    let mut documents = Vec::new();
    let all_docs = indexer.get_all_documents();
    for path in &request.document_paths {
        if let Some(doc) = all_docs.iter().find(|d| d.path.to_string_lossy() == *path) {
            documents.push((*doc).clone());
        }
    }

    if documents.is_empty() {
        return Ok(StylePatternsResponse {
            success: false,
            patterns: vec![],
            confidence_score: 0.0,
            error: Some("No valid documents found for pattern analysis".to_string()),
        });
    }

    match style_learner.learn_from_corpus(&documents, "temp_patterns".to_string()) {
        Ok(result) => {
            println!(
                "Identified {} style patterns with confidence {:.2}",
                result.learned_patterns.len(),
                result.confidence_score
            );

            Ok(StylePatternsResponse {
                success: true,
                patterns: result.learned_patterns,
                confidence_score: result.confidence_score,
                error: None,
            })
        }
        Err(e) => {
            println!("Failed to identify style patterns: {}", e);
            Ok(StylePatternsResponse {
                success: false,
                patterns: vec![],
                confidence_score: 0.0,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Get available style learning capabilities
#[tauri::command]
pub async fn get_style_learning_capabilities() -> Result<serde_json::Value, String> {
    println!("Getting style learning capabilities");

    Ok(serde_json::json!({
        "supported_features": [
            "organizational_style_learning",
            "terminology_analysis",
            "style_pattern_identification",
            "style_guidelines_generation",
            "domain_vocabulary_detection"
        ],
        "supported_domains": [
            "business",
            "academic",
            "technical",
            "informal"
        ],
        "minimum_documents": 3,
        "confidence_threshold": 0.6,
        "version": "1.0.0"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_style_learning_capabilities() {
        let result = get_style_learning_capabilities().await;
        assert!(result.is_ok());

        let capabilities = result.unwrap();
        assert!(capabilities.get("supported_features").is_some());
        assert!(capabilities.get("minimum_documents").is_some());
    }

    // Note: Tests that require State<'_, Mutex<DocumentIndexer>> are complex to set up
    // in unit tests due to Tauri's state management system. These functions are tested
    // through integration tests when the full Tauri app is running.
}
