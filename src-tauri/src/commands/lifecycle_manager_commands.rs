// src-tauri/src/commands/lifecycle_manager_commands.rs
//! Tauri commands for content lifecycle management

use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::{RelationshipAnalyzer, RelationshipConfig};
use crate::workspace::{
    ArchivalSuggestion, ConsolidationSuggestion, LifecycleAction, LifecycleAnalysis,
    LifecycleConfig, LifecycleManager, UpdateRecommendation, UsagePatternAnalysis,
    WorkspaceIntelligence,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Request structure for lifecycle analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleAnalysisRequest {
    pub workspace_path: PathBuf,
    pub config: Option<LifecycleConfig>,
}

/// Response structure for comprehensive lifecycle analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleAnalysisResponse {
    pub success: bool,
    pub analysis: Option<LifecycleAnalysis>,
    pub error: Option<String>,
}

/// Response structure for update recommendations
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRecommendationsResponse {
    pub success: bool,
    pub recommendations: Option<Vec<UpdateRecommendation>>,
    pub error: Option<String>,
}

/// Response structure for archival suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchivalSuggestionsResponse {
    pub success: bool,
    pub suggestions: Option<Vec<ArchivalSuggestion>>,
    pub error: Option<String>,
}

/// Response structure for consolidation opportunities
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsolidationOpportunitiesResponse {
    pub success: bool,
    pub opportunities: Option<Vec<ConsolidationSuggestion>>,
    pub error: Option<String>,
}

/// Response structure for usage statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageStatisticsResponse {
    pub success: bool,
    pub statistics: Option<UsagePatternAnalysis>,
    pub error: Option<String>,
}

/// Response structure for priority actions
#[derive(Debug, Serialize, Deserialize)]
pub struct PriorityActionsResponse {
    pub success: bool,
    pub actions: Option<Vec<LifecycleAction>>,
    pub error: Option<String>,
}

/// Response structure for scalar values
#[derive(Debug, Serialize, Deserialize)]
pub struct ScalarResponse {
    pub success: bool,
    pub value: Option<f64>,
    pub error: Option<String>,
}

/// Response structure for configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub config: Option<LifecycleConfig>,
    pub error: Option<String>,
}

/// Helper function to create lifecycle manager instance
async fn create_lifecycle_manager(workspace_path: &Path) -> Result<LifecycleManager, String> {
    // Create required components
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(workspace_path.to_path_buf())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    LifecycleManager::new(
        workspace_intelligence,
        indexer,
        relationship_analyzer,
        None, // AI orchestrator is optional
    )
    .map_err(|e| format!("Failed to create lifecycle manager: {}", e))
}

/// Comprehensive lifecycle analysis for workspace content
#[tauri::command]
pub async fn analyze_content_lifecycle_comprehensive(
    request: LifecycleAnalysisRequest,
) -> Result<LifecycleAnalysisResponse, String> {
    let mut lifecycle_manager = match create_lifecycle_manager(&request.workspace_path).await {
        Ok(manager) => manager,
        Err(e) => {
            return Ok(LifecycleAnalysisResponse {
                success: false,
                analysis: None,
                error: Some(e),
            })
        }
    };

    let config = request.config.unwrap_or_default();

    // Perform comprehensive analysis
    match lifecycle_manager
        .analyze_lifecycle(&request.workspace_path, config)
        .await
    {
        Ok(analysis) => Ok(LifecycleAnalysisResponse {
            success: true,
            analysis: Some(analysis),
            error: None,
        }),
        Err(e) => Ok(LifecycleAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Lifecycle analysis failed: {}", e)),
        }),
    }
}

/// Get update recommendations for workspace content
#[tauri::command]
pub async fn get_update_recommendations(
    request: LifecycleAnalysisRequest,
) -> Result<UpdateRecommendationsResponse, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(UpdateRecommendationsResponse {
                        success: true,
                        recommendations: Some(analysis.update_recommendations),
                        error: None,
                    })
                } else {
                    Ok(UpdateRecommendationsResponse {
                        success: false,
                        recommendations: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(UpdateRecommendationsResponse {
                    success: false,
                    recommendations: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(UpdateRecommendationsResponse {
            success: false,
            recommendations: None,
            error: Some(e),
        }),
    }
}

/// Get archival suggestions for old or unused content
#[tauri::command]
pub async fn get_archival_suggestions(
    request: LifecycleAnalysisRequest,
) -> Result<ArchivalSuggestionsResponse, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(ArchivalSuggestionsResponse {
                        success: true,
                        suggestions: Some(analysis.archival_suggestions),
                        error: None,
                    })
                } else {
                    Ok(ArchivalSuggestionsResponse {
                        success: false,
                        suggestions: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(ArchivalSuggestionsResponse {
                    success: false,
                    suggestions: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(ArchivalSuggestionsResponse {
            success: false,
            suggestions: None,
            error: Some(e),
        }),
    }
}

/// Get consolidation opportunities for related content
#[tauri::command]
pub async fn get_consolidation_opportunities(
    request: LifecycleAnalysisRequest,
) -> Result<ConsolidationOpportunitiesResponse, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(ConsolidationOpportunitiesResponse {
                        success: true,
                        opportunities: Some(analysis.consolidation_suggestions),
                        error: None,
                    })
                } else {
                    Ok(ConsolidationOpportunitiesResponse {
                        success: false,
                        opportunities: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(ConsolidationOpportunitiesResponse {
                    success: false,
                    opportunities: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(ConsolidationOpportunitiesResponse {
            success: false,
            opportunities: None,
            error: Some(e),
        }),
    }
}

/// Get usage statistics for workspace documents
#[tauri::command]
pub async fn get_usage_statistics(
    request: LifecycleAnalysisRequest,
) -> Result<UsageStatisticsResponse, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(UsageStatisticsResponse {
                        success: true,
                        statistics: Some(analysis.usage_patterns),
                        error: None,
                    })
                } else {
                    Ok(UsageStatisticsResponse {
                        success: false,
                        statistics: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(UsageStatisticsResponse {
                    success: false,
                    statistics: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(UsageStatisticsResponse {
            success: false,
            statistics: None,
            error: Some(e),
        }),
    }
}

/// Get priority actions for content lifecycle management
#[tauri::command]
pub async fn get_priority_actions(
    request: LifecycleAnalysisRequest,
) -> Result<PriorityActionsResponse, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(PriorityActionsResponse {
                        success: true,
                        actions: Some(analysis.priority_actions),
                        error: None,
                    })
                } else {
                    Ok(PriorityActionsResponse {
                        success: false,
                        actions: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(PriorityActionsResponse {
                    success: false,
                    actions: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(PriorityActionsResponse {
            success: false,
            actions: None,
            error: Some(e),
        }),
    }
}

/// Calculate content freshness score for workspace
#[tauri::command]
pub async fn calculate_content_freshness_score(
    request: LifecycleAnalysisRequest,
) -> Result<ScalarResponse, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(ScalarResponse {
                        success: true,
                        value: Some(analysis.freshness_analysis.freshness_score as f64),
                        error: None,
                    })
                } else {
                    Ok(ScalarResponse {
                        success: false,
                        value: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(ScalarResponse {
                    success: false,
                    value: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(ScalarResponse {
            success: false,
            value: None,
            error: Some(e),
        }),
    }
}

/// Get lifecycle management configuration
#[tauri::command]
pub async fn get_lifecycle_config() -> Result<ConfigResponse, String> {
    Ok(ConfigResponse {
        success: true,
        config: Some(LifecycleConfig::default()),
        error: None,
    })
}

/// Update lifecycle management configuration
#[tauri::command]
pub async fn update_lifecycle_config(_config: LifecycleConfig) -> Result<ConfigResponse, String> {
    // For now, just return success since we don't persist config yet
    Ok(ConfigResponse {
        success: true,
        config: None,
        error: None,
    })
}

/// Get lifecycle analysis summary statistics
#[tauri::command]
pub async fn get_lifecycle_analysis_stats(
    request: LifecycleAnalysisRequest,
) -> Result<JsonValue, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    let stats = serde_json::json!({
                        "documents_analyzed": analysis.documents_analyzed,
                        "update_recommendations_count": analysis.update_recommendations.len(),
                        "archival_suggestions_count": analysis.archival_suggestions.len(),
                        "consolidation_opportunities_count": analysis.consolidation_suggestions.len(),
                        "priority_actions_count": analysis.priority_actions.len(),
                        "freshness_score": analysis.freshness_analysis.freshness_score,
                        "health_score": analysis.health_score,
                        "average_age_days": analysis.freshness_analysis.average_age_days,
                        "analysis_duration": analysis.analysis_duration,
                        "lifecycle_distribution": {
                            "active": analysis.lifecycle_distribution.active,
                            "maintenance": analysis.lifecycle_distribution.maintenance,
                            "stale": analysis.lifecycle_distribution.stale,
                            "deprecated": analysis.lifecycle_distribution.deprecated,
                            "legacy": analysis.lifecycle_distribution.legacy
                        },
                        "usage_patterns": {
                            "high_usage_count": analysis.usage_patterns.high_usage_count,
                            "medium_usage_count": analysis.usage_patterns.medium_usage_count,
                            "low_usage_count": analysis.usage_patterns.low_usage_count,
                            "unused_count": analysis.usage_patterns.unused_count,
                            "average_usage": analysis.usage_patterns.average_usage,
                            "usage_trend": analysis.usage_patterns.usage_trend
                        }
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

/// Mark document as accessed for usage tracking
#[tauri::command]
pub async fn track_document_access(
    workspace_path: String,
    document_path: String,
) -> Result<JsonValue, String> {
    let workspace_pb = PathBuf::from(workspace_path);
    let mut lifecycle_manager = match create_lifecycle_manager(&workspace_pb).await {
        Ok(manager) => manager,
        Err(e) => return Err(e),
    };

    match lifecycle_manager
        .track_document_access(&document_path)
        .await
    {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Document access tracked successfully"
        })),
        Err(e) => Err(format!("Failed to track document access: {}", e)),
    }
}

/// Get health score for workspace content management
#[tauri::command]
pub async fn calculate_workspace_health_score(
    request: LifecycleAnalysisRequest,
) -> Result<ScalarResponse, String> {
    match analyze_content_lifecycle_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(ScalarResponse {
                        success: true,
                        value: Some(analysis.health_score as f64),
                        error: None,
                    })
                } else {
                    Ok(ScalarResponse {
                        success: false,
                        value: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(ScalarResponse {
                    success: false,
                    value: None,
                    error: response.error,
                })
            }
        }
        Err(e) => Ok(ScalarResponse {
            success: false,
            value: None,
            error: Some(e),
        }),
    }
}
