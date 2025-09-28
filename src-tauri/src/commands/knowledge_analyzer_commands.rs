// src-tauri/src/commands/knowledge_analyzer_commands.rs
//! Tauri commands for knowledge gap analysis functionality

use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::{RelationshipAnalyzer, RelationshipConfig};
use crate::workspace::{
    ExpertRecommendation, IncompleteProcess, KnowledgeAnalysisConfig, KnowledgeAnalyzer,
    KnowledgeGap, KnowledgeGapAnalysis, MissingDocumentType, OutdatedContent, PriorityArea,
    ReferenceGap, WorkspaceIntelligence,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Request structure for knowledge gap analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeAnalysisRequest {
    pub workspace_path: PathBuf,
    pub config: Option<KnowledgeAnalysisConfig>,
}

/// Response structure for comprehensive knowledge gap analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeAnalysisResponse {
    pub success: bool,
    pub analysis: Option<KnowledgeGapAnalysis>,
    pub error: Option<String>,
}

/// Response structure for knowledge gaps
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeGapsResponse {
    pub success: bool,
    pub gaps: Option<Vec<KnowledgeGap>>,
    pub error: Option<String>,
}

/// Response structure for missing document types
#[derive(Debug, Serialize, Deserialize)]
pub struct MissingDocumentTypesResponse {
    pub success: bool,
    pub missing_types: Option<Vec<MissingDocumentType>>,
    pub error: Option<String>,
}

/// Response structure for incomplete processes
#[derive(Debug, Serialize, Deserialize)]
pub struct IncompleteProcessesResponse {
    pub success: bool,
    pub incomplete_processes: Option<Vec<IncompleteProcess>>,
    pub error: Option<String>,
}

/// Response structure for outdated content
#[derive(Debug, Serialize, Deserialize)]
pub struct OutdatedContentResponse {
    pub success: bool,
    pub outdated_content: Option<Vec<OutdatedContent>>,
    pub error: Option<String>,
}

/// Response structure for reference gaps
#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceGapsResponse {
    pub success: bool,
    pub reference_gaps: Option<Vec<ReferenceGap>>,
    pub error: Option<String>,
}

/// Response structure for expert recommendations
#[derive(Debug, Serialize, Deserialize)]
pub struct ExpertRecommendationsResponse {
    pub success: bool,
    pub recommendations: Option<Vec<ExpertRecommendation>>,
    pub error: Option<String>,
}

/// Response structure for priority areas
#[derive(Debug, Serialize, Deserialize)]
pub struct PriorityAreasResponse {
    pub success: bool,
    pub priority_areas: Option<Vec<PriorityArea>>,
    pub error: Option<String>,
}

/// Response structure for knowledge analysis configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeConfigResponse {
    pub success: bool,
    pub config: Option<KnowledgeAnalysisConfig>,
    pub error: Option<String>,
}

/// Response structure for completeness score
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletenessScoreResponse {
    pub success: bool,
    pub score: Option<f32>,
    pub error: Option<String>,
}

/// Perform comprehensive knowledge gap analysis
#[tauri::command]
pub async fn analyze_knowledge_gaps_comprehensive(
    request: KnowledgeAnalysisRequest,
) -> Result<KnowledgeAnalysisResponse, String> {
    // Create required components
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(request.workspace_path.clone())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    let knowledge_analyzer = match KnowledgeAnalyzer::new(
        workspace_intelligence,
        indexer,
        relationship_analyzer,
        None, // AI orchestrator is optional
    ) {
        Ok(analyzer) => analyzer,
        Err(e) => {
            return Ok(KnowledgeAnalysisResponse {
                success: false,
                analysis: None,
                error: Some(format!("Failed to create knowledge analyzer: {}", e)),
            })
        }
    };

    let config = request.config.unwrap_or_default();

    // Perform comprehensive analysis
    match knowledge_analyzer
        .analyze_knowledge_gaps(&request.workspace_path, config)
        .await
    {
        Ok(analysis) => Ok(KnowledgeAnalysisResponse {
            success: true,
            analysis: Some(analysis),
            error: None,
        }),
        Err(e) => Ok(KnowledgeAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Knowledge gap analysis failed: {}", e)),
        }),
    }
}

/// Get specific knowledge gaps identified in workspace
#[tauri::command]
pub async fn get_knowledge_gaps(
    request: KnowledgeAnalysisRequest,
) -> Result<KnowledgeGapsResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(KnowledgeGapsResponse {
                        success: true,
                        gaps: Some(analysis.knowledge_gaps),
                        error: None,
                    })
                } else {
                    Ok(KnowledgeGapsResponse {
                        success: false,
                        gaps: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(KnowledgeGapsResponse {
                    success: false,
                    gaps: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(KnowledgeGapsResponse {
            success: false,
            gaps: None,
            error: Some(e),
        }),
    }
}

/// Get missing document types that should exist
#[tauri::command]
pub async fn get_missing_document_types(
    request: KnowledgeAnalysisRequest,
) -> Result<MissingDocumentTypesResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(MissingDocumentTypesResponse {
                        success: true,
                        missing_types: Some(analysis.missing_document_types),
                        error: None,
                    })
                } else {
                    Ok(MissingDocumentTypesResponse {
                        success: false,
                        missing_types: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(MissingDocumentTypesResponse {
                    success: false,
                    missing_types: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(MissingDocumentTypesResponse {
            success: false,
            missing_types: None,
            error: Some(e),
        }),
    }
}

/// Get incomplete processes and procedures
#[tauri::command]
pub async fn get_incomplete_processes(
    request: KnowledgeAnalysisRequest,
) -> Result<IncompleteProcessesResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(IncompleteProcessesResponse {
                        success: true,
                        incomplete_processes: Some(analysis.incomplete_processes),
                        error: None,
                    })
                } else {
                    Ok(IncompleteProcessesResponse {
                        success: false,
                        incomplete_processes: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(IncompleteProcessesResponse {
                    success: false,
                    incomplete_processes: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(IncompleteProcessesResponse {
            success: false,
            incomplete_processes: None,
            error: Some(e),
        }),
    }
}

/// Get outdated content that needs updates
#[tauri::command]
pub async fn get_outdated_content(
    request: KnowledgeAnalysisRequest,
) -> Result<OutdatedContentResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(OutdatedContentResponse {
                        success: true,
                        outdated_content: Some(analysis.outdated_content),
                        error: None,
                    })
                } else {
                    Ok(OutdatedContentResponse {
                        success: false,
                        outdated_content: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(OutdatedContentResponse {
                    success: false,
                    outdated_content: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(OutdatedContentResponse {
            success: false,
            outdated_content: None,
            error: Some(e),
        }),
    }
}

/// Get reference gaps and broken links
#[tauri::command]
pub async fn get_reference_gaps(
    request: KnowledgeAnalysisRequest,
) -> Result<ReferenceGapsResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(ReferenceGapsResponse {
                        success: true,
                        reference_gaps: Some(analysis.reference_gaps),
                        error: None,
                    })
                } else {
                    Ok(ReferenceGapsResponse {
                        success: false,
                        reference_gaps: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(ReferenceGapsResponse {
                    success: false,
                    reference_gaps: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(ReferenceGapsResponse {
            success: false,
            reference_gaps: None,
            error: Some(e),
        }),
    }
}

/// Get expert recommendations for knowledge improvement
#[tauri::command]
pub async fn get_expert_recommendations(
    request: KnowledgeAnalysisRequest,
) -> Result<ExpertRecommendationsResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(ExpertRecommendationsResponse {
                        success: true,
                        recommendations: Some(analysis.expert_recommendations),
                        error: None,
                    })
                } else {
                    Ok(ExpertRecommendationsResponse {
                        success: false,
                        recommendations: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(ExpertRecommendationsResponse {
                    success: false,
                    recommendations: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(ExpertRecommendationsResponse {
            success: false,
            recommendations: None,
            error: Some(e),
        }),
    }
}

/// Get priority areas for knowledge improvement
#[tauri::command]
pub async fn get_priority_areas(
    request: KnowledgeAnalysisRequest,
) -> Result<PriorityAreasResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(PriorityAreasResponse {
                        success: true,
                        priority_areas: Some(analysis.priority_areas),
                        error: None,
                    })
                } else {
                    Ok(PriorityAreasResponse {
                        success: false,
                        priority_areas: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(PriorityAreasResponse {
                    success: false,
                    priority_areas: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(PriorityAreasResponse {
            success: false,
            priority_areas: None,
            error: Some(e),
        }),
    }
}

/// Get current knowledge analysis configuration
#[tauri::command]
pub async fn get_knowledge_analysis_config() -> Result<KnowledgeConfigResponse, String> {
    let config = KnowledgeAnalysisConfig::default();

    Ok(KnowledgeConfigResponse {
        success: true,
        config: Some(config),
        error: None,
    })
}

/// Update knowledge analysis configuration
#[tauri::command]
pub async fn update_knowledge_analysis_config(
    config: KnowledgeAnalysisConfig,
) -> Result<KnowledgeConfigResponse, String> {
    // For now, just return the provided config
    // In a full implementation, this would persist the configuration
    Ok(KnowledgeConfigResponse {
        success: true,
        config: Some(config),
        error: None,
    })
}

/// Calculate workspace knowledge completeness score
#[tauri::command]
pub async fn calculate_knowledge_completeness_score(
    request: KnowledgeAnalysisRequest,
) -> Result<CompletenessScoreResponse, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(CompletenessScoreResponse {
                        success: true,
                        score: Some(analysis.completeness_score),
                        error: None,
                    })
                } else {
                    Ok(CompletenessScoreResponse {
                        success: false,
                        score: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(CompletenessScoreResponse {
                    success: false,
                    score: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(CompletenessScoreResponse {
            success: false,
            score: None,
            error: Some(e),
        }),
    }
}

/// Get knowledge analysis statistics
#[tauri::command]
pub async fn get_knowledge_analysis_stats(
    request: KnowledgeAnalysisRequest,
) -> Result<serde_json::Value, String> {
    match analyze_knowledge_gaps_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    let stats = serde_json::json!({
                        "completeness_score": analysis.completeness_score,
                        "documents_analyzed": analysis.documents_analyzed,
                        "total_gaps": analysis.knowledge_gaps.len(),
                        "missing_document_types": analysis.missing_document_types.len(),
                        "incomplete_processes": analysis.incomplete_processes.len(),
                        "outdated_content": analysis.outdated_content.len(),
                        "reference_gaps": analysis.reference_gaps.len(),
                        "expert_recommendations": analysis.expert_recommendations.len(),
                        "priority_areas": analysis.priority_areas.len(),
                        "analysis_duration": analysis.analysis_duration,
                        "analysis_timestamp": analysis.analysis_timestamp
                    });

                    Ok(stats)
                } else {
                    Err("No analysis data available".to_string())
                }
            } else {
                Err(response
                    .error
                    .unwrap_or_else(|| "Analysis failed".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}
