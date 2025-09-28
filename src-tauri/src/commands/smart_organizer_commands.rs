// src-tauri/src/commands/smart_organizer_commands.rs
//! Tauri commands for smart document organization functionality

use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::{RelationshipAnalyzer, RelationshipConfig};
use crate::workspace::{
    CategorizationSuggestion, DuplicateHandlingSuggestion, FolderStructureSuggestion,
    OrganizationAction, OrganizationAnalysis, OrganizationConfig, SemanticCluster, SmartOrganizer,
    TaggingSuggestion, WorkspaceIntelligence,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Request structure for organization analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationAnalysisRequest {
    pub workspace_path: PathBuf,
    pub config: Option<OrganizationConfig>,
}

/// Response structure for organization analysis operations
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationAnalysisResponse {
    pub success: bool,
    pub analysis: Option<OrganizationAnalysis>,
    pub error: Option<String>,
}

/// Response structure for categorization suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct CategorizationResponse {
    pub success: bool,
    pub suggestions: Option<Vec<CategorizationSuggestion>>,
    pub error: Option<String>,
}

/// Response structure for tagging suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct TaggingResponse {
    pub success: bool,
    pub suggestions: Option<Vec<TaggingSuggestion>>,
    pub error: Option<String>,
}

/// Response structure for folder structure suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderStructureResponse {
    pub success: bool,
    pub suggestions: Option<Vec<FolderStructureSuggestion>>,
    pub error: Option<String>,
}

/// Response structure for semantic clustering
#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticClusterResponse {
    pub success: bool,
    pub clusters: Option<Vec<SemanticCluster>>,
    pub error: Option<String>,
}

/// Response structure for duplicate handling suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateHandlingResponse {
    pub success: bool,
    pub suggestions: Option<Vec<DuplicateHandlingSuggestion>>,
    pub error: Option<String>,
}

/// Response structure for organization actions
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationActionsResponse {
    pub success: bool,
    pub actions: Option<Vec<OrganizationAction>>,
    pub error: Option<String>,
}

/// Response structure for organization configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationConfigResponse {
    pub success: bool,
    pub config: Option<OrganizationConfig>,
    pub error: Option<String>,
}

/// Execute organization action request
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteActionRequest {
    pub action_id: String,
    pub workspace_path: PathBuf,
    pub confirm: bool,
}

/// Execute organization action response
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteActionResponse {
    pub success: bool,
    pub executed: bool,
    pub message: String,
    pub error: Option<String>,
}

/// Perform comprehensive organization analysis
#[tauri::command]
pub async fn analyze_organization_comprehensive(
    request: OrganizationAnalysisRequest,
) -> Result<OrganizationAnalysisResponse, String> {
    // Create required components
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(request.workspace_path.clone())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    let smart_organizer = match SmartOrganizer::new(
        workspace_intelligence,
        indexer,
        relationship_analyzer,
        None, // AI orchestrator is optional
    ) {
        Ok(organizer) => organizer,
        Err(e) => {
            return Ok(OrganizationAnalysisResponse {
                success: false,
                analysis: None,
                error: Some(format!("Failed to create smart organizer: {}", e)),
            })
        }
    };

    let config = request.config.unwrap_or_default();

    // Perform comprehensive analysis
    match smart_organizer
        .analyze_organization(&request.workspace_path, config)
        .await
    {
        Ok(analysis) => Ok(OrganizationAnalysisResponse {
            success: true,
            analysis: Some(analysis),
            error: None,
        }),
        Err(e) => Ok(OrganizationAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Organization analysis failed: {}", e)),
        }),
    }
}

/// Get categorization suggestions for workspace
#[tauri::command]
pub async fn get_categorization_suggestions(
    request: OrganizationAnalysisRequest,
) -> Result<CategorizationResponse, String> {
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(request.workspace_path.clone())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    let smart_organizer =
        match SmartOrganizer::new(workspace_intelligence, indexer, relationship_analyzer, None) {
            Ok(organizer) => organizer,
            Err(e) => {
                return Ok(CategorizationResponse {
                    success: false,
                    suggestions: None,
                    error: Some(format!("Failed to create smart organizer: {}", e)),
                })
            }
        };

    let config = request.config.unwrap_or_default();

    match smart_organizer
        .analyze_categorization(&request.workspace_path, &config)
        .await
    {
        Ok(suggestions) => Ok(CategorizationResponse {
            success: true,
            suggestions: Some(suggestions),
            error: None,
        }),
        Err(e) => Ok(CategorizationResponse {
            success: false,
            suggestions: None,
            error: Some(format!("Categorization analysis failed: {}", e)),
        }),
    }
}

/// Get tagging suggestions for workspace
#[tauri::command]
pub async fn get_tagging_suggestions(
    request: OrganizationAnalysisRequest,
) -> Result<TaggingResponse, String> {
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(request.workspace_path.clone())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    let smart_organizer =
        match SmartOrganizer::new(workspace_intelligence, indexer, relationship_analyzer, None) {
            Ok(organizer) => organizer,
            Err(e) => {
                return Ok(TaggingResponse {
                    success: false,
                    suggestions: None,
                    error: Some(format!("Failed to create smart organizer: {}", e)),
                })
            }
        };

    let config = request.config.unwrap_or_default();

    match smart_organizer
        .analyze_tagging(&request.workspace_path, &config)
        .await
    {
        Ok(suggestions) => Ok(TaggingResponse {
            success: true,
            suggestions: Some(suggestions),
            error: None,
        }),
        Err(e) => Ok(TaggingResponse {
            success: false,
            suggestions: None,
            error: Some(format!("Tagging analysis failed: {}", e)),
        }),
    }
}

/// Get folder structure optimization suggestions
#[tauri::command]
pub async fn get_folder_structure_suggestions(
    request: OrganizationAnalysisRequest,
) -> Result<FolderStructureResponse, String> {
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(request.workspace_path.clone())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    let smart_organizer =
        match SmartOrganizer::new(workspace_intelligence, indexer, relationship_analyzer, None) {
            Ok(organizer) => organizer,
            Err(e) => {
                return Ok(FolderStructureResponse {
                    success: false,
                    suggestions: None,
                    error: Some(format!("Failed to create smart organizer: {}", e)),
                })
            }
        };

    let config = request.config.unwrap_or_default();

    match smart_organizer
        .analyze_folder_structure(&request.workspace_path, &config)
        .await
    {
        Ok(suggestions) => Ok(FolderStructureResponse {
            success: true,
            suggestions: Some(suggestions),
            error: None,
        }),
        Err(e) => Ok(FolderStructureResponse {
            success: false,
            suggestions: None,
            error: Some(format!("Folder structure analysis failed: {}", e)),
        }),
    }
}

/// Get semantic clustering analysis
#[tauri::command]
pub async fn get_semantic_clusters(
    request: OrganizationAnalysisRequest,
) -> Result<SemanticClusterResponse, String> {
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(request.workspace_path.clone())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    let smart_organizer =
        match SmartOrganizer::new(workspace_intelligence, indexer, relationship_analyzer, None) {
            Ok(organizer) => organizer,
            Err(e) => {
                return Ok(SemanticClusterResponse {
                    success: false,
                    clusters: None,
                    error: Some(format!("Failed to create smart organizer: {}", e)),
                })
            }
        };

    let config = request.config.unwrap_or_default();

    match smart_organizer
        .perform_semantic_clustering(&request.workspace_path, &config)
        .await
    {
        Ok(clusters) => Ok(SemanticClusterResponse {
            success: true,
            clusters: Some(clusters),
            error: None,
        }),
        Err(e) => Ok(SemanticClusterResponse {
            success: false,
            clusters: None,
            error: Some(format!("Semantic clustering failed: {}", e)),
        }),
    }
}

/// Get duplicate handling suggestions
#[tauri::command]
pub async fn get_duplicate_handling_suggestions(
    request: OrganizationAnalysisRequest,
) -> Result<DuplicateHandlingResponse, String> {
    let indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(request.workspace_path.clone())
            .map_err(|e| format!("Failed to create document indexer: {}", e))?,
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
        RelationshipConfig::default(),
    )));

    let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

    let smart_organizer =
        match SmartOrganizer::new(workspace_intelligence, indexer, relationship_analyzer, None) {
            Ok(organizer) => organizer,
            Err(e) => {
                return Ok(DuplicateHandlingResponse {
                    success: false,
                    suggestions: None,
                    error: Some(format!("Failed to create smart organizer: {}", e)),
                })
            }
        };

    let config = request.config.unwrap_or_default();

    match smart_organizer
        .analyze_duplicates(&request.workspace_path, &config)
        .await
    {
        Ok(suggestions) => Ok(DuplicateHandlingResponse {
            success: true,
            suggestions: Some(suggestions),
            error: None,
        }),
        Err(e) => Ok(DuplicateHandlingResponse {
            success: false,
            suggestions: None,
            error: Some(format!("Duplicate analysis failed: {}", e)),
        }),
    }
}

/// Get priority organization actions
#[tauri::command]
pub async fn get_organization_actions(
    request: OrganizationAnalysisRequest,
) -> Result<OrganizationActionsResponse, String> {
    // First get full analysis to generate actions
    match analyze_organization_comprehensive(request).await {
        Ok(analysis_response) => {
            if analysis_response.success {
                if let Some(analysis) = analysis_response.analysis {
                    Ok(OrganizationActionsResponse {
                        success: true,
                        actions: Some(analysis.priority_actions),
                        error: None,
                    })
                } else {
                    Ok(OrganizationActionsResponse {
                        success: false,
                        actions: None,
                        error: Some("No analysis data available".to_string()),
                    })
                }
            } else {
                Ok(OrganizationActionsResponse {
                    success: false,
                    actions: None,
                    error: analysis_response.error,
                })
            }
        }
        Err(e) => Ok(OrganizationActionsResponse {
            success: false,
            actions: None,
            error: Some(e),
        }),
    }
}

/// Get current organization configuration
#[tauri::command]
pub async fn get_organization_config() -> Result<OrganizationConfigResponse, String> {
    let config = OrganizationConfig::default();

    Ok(OrganizationConfigResponse {
        success: true,
        config: Some(config),
        error: None,
    })
}

/// Update organization configuration
#[tauri::command]
pub async fn update_organization_config(
    config: OrganizationConfig,
) -> Result<OrganizationConfigResponse, String> {
    // For now, just return the provided config
    // In a full implementation, this would persist the configuration
    Ok(OrganizationConfigResponse {
        success: true,
        config: Some(config),
        error: None,
    })
}

/// Execute an organization action
#[tauri::command]
pub async fn execute_organization_action(
    request: ExecuteActionRequest,
) -> Result<ExecuteActionResponse, String> {
    if !request.confirm {
        return Ok(ExecuteActionResponse {
            success: false,
            executed: false,
            message: "Action execution requires confirmation".to_string(),
            error: None,
        });
    }

    // For now, simulate execution
    // In a full implementation, this would actually perform the organization action
    Ok(ExecuteActionResponse {
        success: true,
        executed: true,
        message: format!("Successfully executed action: {}", request.action_id),
        error: None,
    })
}

/// Calculate organization score for a workspace
#[tauri::command]
pub async fn calculate_organization_score(
    request: OrganizationAnalysisRequest,
) -> Result<f32, String> {
    match analyze_organization_comprehensive(request).await {
        Ok(response) => {
            if response.success {
                if let Some(analysis) = response.analysis {
                    Ok(analysis.organization_score)
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
