use serde::{Deserialize, Serialize};
use tauri::State;

use crate::document::indexer::DocumentIndexer;
use crate::document::{DocumentIndexEntry, StyleAnalyzer, StyleProfile, StyleSimilarity};

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleAnalysisRequest {
    pub document_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CorpusStyleAnalysisRequest {
    pub document_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleComparisonRequest {
    pub document_id1: String,
    pub document_id2: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleGroupAnalysisRequest {
    pub document_ids: Vec<String>,
    pub group_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleProfileResponse {
    pub success: bool,
    pub profile: Option<StyleProfile>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleSimilarityResponse {
    pub success: bool,
    pub similarity: Option<StyleSimilarity>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleGroupAnalysisResponse {
    pub success: bool,
    pub group_profile: Option<StyleProfile>,
    pub individual_profiles: Option<Vec<(String, StyleProfile)>>,
    pub similarities: Option<Vec<(String, String, f64)>>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn analyze_document_style(
    request: StyleAnalysisRequest,
    indexer: State<'_, tokio::sync::Mutex<DocumentIndexer>>,
) -> Result<StyleProfileResponse, String> {
    let indexer = indexer.lock().await;
    let analyzer = StyleAnalyzer::new();

    match indexer.get_document(&request.document_id) {
        Some(document) => match analyzer.analyze_document_style(document) {
            Ok(profile) => Ok(StyleProfileResponse {
                success: true,
                profile: Some(profile),
                error: None,
            }),
            Err(e) => Ok(StyleProfileResponse {
                success: false,
                profile: None,
                error: Some(e.to_string()),
            }),
        },
        None => Ok(StyleProfileResponse {
            success: false,
            profile: None,
            error: Some(format!("Document not found: {}", request.document_id)),
        }),
    }
}

#[tauri::command]
pub async fn analyze_corpus_style(
    request: CorpusStyleAnalysisRequest,
    indexer: State<'_, tokio::sync::Mutex<DocumentIndexer>>,
) -> Result<StyleProfileResponse, String> {
    let indexer = indexer.lock().await;
    let analyzer = StyleAnalyzer::new();

    let documents: Vec<DocumentIndexEntry> = request
        .document_ids
        .iter()
        .filter_map(|id| indexer.get_document(id).cloned())
        .collect();

    if documents.is_empty() {
        return Ok(StyleProfileResponse {
            success: false,
            profile: None,
            error: Some("No valid documents found".to_string()),
        });
    }

    match analyzer.analyze_corpus_style(&documents) {
        Ok(profile) => Ok(StyleProfileResponse {
            success: true,
            profile: Some(profile),
            error: None,
        }),
        Err(e) => Ok(StyleProfileResponse {
            success: false,
            profile: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn compare_document_styles(
    request: StyleComparisonRequest,
    indexer: State<'_, tokio::sync::Mutex<DocumentIndexer>>,
) -> Result<StyleSimilarityResponse, String> {
    let indexer = indexer.lock().await;
    let analyzer = StyleAnalyzer::new();

    let doc1 = match indexer.get_document(&request.document_id1) {
        Some(doc) => doc,
        None => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Document not found: {}", request.document_id1)),
            })
        }
    };

    let doc2 = match indexer.get_document(&request.document_id2) {
        Some(doc) => doc,
        None => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Document not found: {}", request.document_id2)),
            })
        }
    };

    // Analyze both documents
    let style1 = match analyzer.analyze_document_style(doc1) {
        Ok(style) => style,
        Err(e) => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Failed to analyze first document: {}", e)),
            })
        }
    };

    let style2 = match analyzer.analyze_document_style(doc2) {
        Ok(style) => style,
        Err(e) => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Failed to analyze second document: {}", e)),
            })
        }
    };

    let similarity = analyzer.compare_styles(&style1, &style2);

    Ok(StyleSimilarityResponse {
        success: true,
        similarity: Some(similarity),
        error: None,
    })
}

#[tauri::command]
pub async fn analyze_style_group(
    request: StyleGroupAnalysisRequest,
    indexer: State<'_, tokio::sync::Mutex<DocumentIndexer>>,
) -> Result<StyleGroupAnalysisResponse, String> {
    let indexer = indexer.lock().await;
    let analyzer = StyleAnalyzer::new();

    let documents: Vec<DocumentIndexEntry> = request
        .document_ids
        .iter()
        .filter_map(|id| indexer.get_document(id).cloned())
        .collect();

    if documents.is_empty() {
        return Ok(StyleGroupAnalysisResponse {
            success: false,
            group_profile: None,
            individual_profiles: None,
            similarities: None,
            error: Some("No valid documents found".to_string()),
        });
    }

    // Analyze group style
    let group_profile = match analyzer.analyze_corpus_style(&documents) {
        Ok(profile) => profile,
        Err(e) => {
            return Ok(StyleGroupAnalysisResponse {
                success: false,
                group_profile: None,
                individual_profiles: None,
                similarities: None,
                error: Some(format!("Failed to analyze group style: {}", e)),
            })
        }
    };

    // Analyze individual documents
    let mut individual_profiles = Vec::new();
    for (i, doc) in documents.iter().enumerate() {
        match analyzer.analyze_document_style(doc) {
            Ok(profile) => {
                individual_profiles.push((request.document_ids[i].clone(), profile));
            }
            Err(e) => {
                eprintln!(
                    "Failed to analyze document {}: {}",
                    request.document_ids[i], e
                );
            }
        }
    }

    // Calculate pairwise similarities
    let mut similarities = Vec::new();
    for i in 0..individual_profiles.len() {
        for j in (i + 1)..individual_profiles.len() {
            let similarity =
                analyzer.compare_styles(&individual_profiles[i].1, &individual_profiles[j].1);
            similarities.push((
                individual_profiles[i].0.clone(),
                individual_profiles[j].0.clone(),
                similarity.overall_similarity,
            ));
        }
    }

    Ok(StyleGroupAnalysisResponse {
        success: true,
        group_profile: Some(group_profile),
        individual_profiles: Some(individual_profiles),
        similarities: Some(similarities),
        error: None,
    })
}

#[tauri::command]
pub async fn find_style_outliers(
    request: CorpusStyleAnalysisRequest,
    threshold: Option<f64>,
    indexer: State<'_, tokio::sync::Mutex<DocumentIndexer>>,
) -> Result<StyleGroupAnalysisResponse, String> {
    let indexer = indexer.lock().await;
    let analyzer = StyleAnalyzer::new();
    let similarity_threshold = threshold.unwrap_or(0.7);

    let documents: Vec<DocumentIndexEntry> = request
        .document_ids
        .iter()
        .filter_map(|id| indexer.get_document(id).cloned())
        .collect();

    if documents.is_empty() {
        return Ok(StyleGroupAnalysisResponse {
            success: false,
            group_profile: None,
            individual_profiles: None,
            similarities: None,
            error: Some("No valid documents found".to_string()),
        });
    }

    // Analyze group style (reference style)
    let group_profile = match analyzer.analyze_corpus_style(&documents) {
        Ok(profile) => profile,
        Err(e) => {
            return Ok(StyleGroupAnalysisResponse {
                success: false,
                group_profile: None,
                individual_profiles: None,
                similarities: None,
                error: Some(format!("Failed to analyze group style: {}", e)),
            })
        }
    };

    // Find outliers by comparing each document to the group style
    let mut outlier_profiles = Vec::new();
    let mut outlier_similarities = Vec::new();

    for (i, doc) in documents.iter().enumerate() {
        match analyzer.analyze_document_style(doc) {
            Ok(profile) => {
                let similarity = analyzer.compare_styles(&group_profile, &profile);
                if similarity.overall_similarity < similarity_threshold {
                    outlier_profiles.push((request.document_ids[i].clone(), profile));
                    outlier_similarities.push((
                        "group".to_string(),
                        request.document_ids[i].clone(),
                        similarity.overall_similarity,
                    ));
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to analyze document {}: {}",
                    request.document_ids[i], e
                );
            }
        }
    }

    Ok(StyleGroupAnalysisResponse {
        success: true,
        group_profile: Some(group_profile),
        individual_profiles: Some(outlier_profiles),
        similarities: Some(outlier_similarities),
        error: None,
    })
}

#[tauri::command]
pub async fn get_style_recommendations(
    source_document_id: String,
    target_document_id: String,
    indexer: State<'_, tokio::sync::Mutex<DocumentIndexer>>,
) -> Result<StyleSimilarityResponse, String> {
    let indexer = indexer.lock().await;
    let analyzer = StyleAnalyzer::new();

    let source_doc = match indexer.get_document(&source_document_id) {
        Some(doc) => doc,
        None => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Source document not found: {}", source_document_id)),
            })
        }
    };

    let target_doc = match indexer.get_document(&target_document_id) {
        Some(doc) => doc,
        None => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Target document not found: {}", target_document_id)),
            })
        }
    };

    // Analyze both documents
    let source_style = match analyzer.analyze_document_style(source_doc) {
        Ok(style) => style,
        Err(e) => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Failed to analyze source document: {}", e)),
            })
        }
    };

    let target_style = match analyzer.analyze_document_style(target_doc) {
        Ok(style) => style,
        Err(e) => {
            return Ok(StyleSimilarityResponse {
                success: false,
                similarity: None,
                error: Some(format!("Failed to analyze target document: {}", e)),
            })
        }
    };

    let similarity = analyzer.compare_styles(&source_style, &target_style);

    Ok(StyleSimilarityResponse {
        success: true,
        similarity: Some(similarity),
        error: None,
    })
}
