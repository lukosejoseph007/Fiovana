// src-tauri/src/commands/workspace_analyzer_commands.rs
//! Tauri commands for workspace analyzer functionality

use crate::workspace::{
    WorkspaceAnalyzer, WorkspaceAnalysisConfig, ComprehensiveWorkspaceAnalysis,
    ContentGapAnalysis, RedundancyAnalysis, OrganizationAssessment,
    WorkspaceIntelligence
};
use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::RelationshipAnalyzer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// Response structure for workspace analysis operations
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceAnalysisResponse {
    pub success: bool,
    pub analysis: Option<ComprehensiveWorkspaceAnalysis>,
    pub error: Option<String>,
}

/// Response structure for analysis configuration operations
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisConfigResponse {
    pub success: bool,
    pub config: Option<WorkspaceAnalysisConfig>,
    pub error: Option<String>,
}

/// Response structure for gap analysis operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ContentGapResponse {
    pub success: bool,
    pub gap_analysis: Option<ContentGapAnalysis>,
    pub error: Option<String>,
}

/// Response structure for redundancy analysis operations
#[derive(Debug, Serialize, Deserialize)]
pub struct RedundancyResponse {
    pub success: bool,
    pub redundancy_analysis: Option<RedundancyAnalysis>,
    pub error: Option<String>,
}

/// Response structure for organization assessment operations
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationResponse {
    pub success: bool,
    pub organization_assessment: Option<OrganizationAssessment>,
    pub error: Option<String>,
}

/// Request structure for workspace analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeWorkspaceRequest {
    pub workspace_path: String,
    pub config: Option<WorkspaceAnalysisConfig>,
}

/// Initialize workspace analyzer
#[tauri::command]
pub async fn initialize_workspace_analyzer(
    workspace_path: String,
) -> Result<WorkspaceAnalysisResponse, String> {
    tracing::info!("Initializing workspace analyzer for path: {}", workspace_path);

    let path = PathBuf::from(&workspace_path);

    // Validate workspace path
    if !path.exists() {
        return Ok(WorkspaceAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Workspace path does not exist: {}", workspace_path)),
        });
    }

    if !path.is_dir() {
        return Ok(WorkspaceAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Workspace path is not a directory: {}", workspace_path)),
        });
    }

    Ok(WorkspaceAnalysisResponse {
        success: true,
        analysis: None,
        error: None,
    })
}

/// Perform comprehensive workspace analysis
#[tauri::command]
pub async fn analyze_workspace_comprehensive(
    request: AnalyzeWorkspaceRequest,
) -> Result<WorkspaceAnalysisResponse, String> {
    tracing::info!("Starting comprehensive workspace analysis for: {}", request.workspace_path);

    let workspace_path = PathBuf::from(&request.workspace_path);
    let config = request.config.unwrap_or_default();

    // Validate workspace path
    if !workspace_path.exists() || !workspace_path.is_dir() {
        return Ok(WorkspaceAnalysisResponse {
            success: false,
            analysis: None,
            error: Some("Invalid workspace path".to_string()),
        });
    }

    // Create necessary components
    let document_indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(&workspace_path)
            .map_err(|e| format!("Failed to create document indexer: {}", e))?
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new()));

    let workspace_intelligence = WorkspaceIntelligence::new(
        document_indexer.clone(),
        relationship_analyzer.clone(),
        None, // AI orchestrator is optional
    );

    let workspace_analyzer = match WorkspaceAnalyzer::new(
        workspace_intelligence,
        document_indexer,
        relationship_analyzer,
        None, // AI orchestrator is optional
    ) {
        Ok(analyzer) => analyzer,
        Err(e) => return WorkspaceAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Failed to create workspace analyzer: {}", e)),
        },
    };

    // Perform comprehensive analysis
    match workspace_analyzer.analyze_workspace(&workspace_path, config).await {
        Ok(analysis) => {
            tracing::info!(
                "Workspace analysis completed successfully. Health score: {:.2}",
                analysis.overall_health_score
            );

            Ok(WorkspaceAnalysisResponse {
                success: true,
                analysis: Some(analysis),
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Workspace analysis failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(WorkspaceAnalysisResponse {
                success: false,
                analysis: None,
                error: Some(error_msg),
            })
        }
    }
}

/// Get default analysis configuration
#[tauri::command]
pub async fn get_default_analysis_config() -> Result<AnalysisConfigResponse, String> {
    tracing::debug!("Getting default workspace analysis configuration");

    let config = WorkspaceAnalysisConfig::default();

    Ok(AnalysisConfigResponse {
        success: true,
        config: Some(config),
        error: None,
    })
}

/// Create custom analysis configuration
#[tauri::command]
pub async fn create_analysis_config(
    analyze_content_gaps: bool,
    detect_redundancy: bool,
    assess_organization: bool,
    generate_ai_insights: bool,
    min_confidence: f32,
    max_depth: usize,
) -> Result<AnalysisConfigResponse, String> {
    tracing::debug!("Creating custom workspace analysis configuration");

    // Validate configuration parameters
    if min_confidence < 0.0 || min_confidence > 1.0 {
        return Ok(AnalysisConfigResponse {
            success: false,
            config: None,
            error: Some("min_confidence must be between 0.0 and 1.0".to_string()),
        });
    }

    if max_depth == 0 || max_depth > 50 {
        return Ok(AnalysisConfigResponse {
            success: false,
            config: None,
            error: Some("max_depth must be between 1 and 50".to_string()),
        });
    }

    let config = WorkspaceAnalysisConfig {
        analyze_content_gaps,
        detect_redundancy,
        assess_organization,
        generate_ai_insights,
        min_confidence,
        max_depth,
    };

    Ok(AnalysisConfigResponse {
        success: true,
        config: Some(config),
        error: None,
    })
}

/// Analyze content gaps specifically
#[tauri::command]
pub async fn analyze_content_gaps(
    workspace_path: String,
) -> Result<ContentGapResponse, String> {
    tracing::info!("Analyzing content gaps for workspace: {}", workspace_path);

    let path = PathBuf::from(&workspace_path);

    // Validate workspace path
    if !path.exists() || !path.is_dir() {
        return Ok(ContentGapResponse {
            success: false,
            gap_analysis: None,
            error: Some("Invalid workspace path".to_string()),
        });
    }

    // Create analyzer with content gap focus
    let document_indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(&path)
            .map_err(|e| format!("Failed to create document indexer: {}", e))?
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new()));

    let workspace_intelligence = WorkspaceIntelligence::new(
        document_indexer.clone(),
        relationship_analyzer.clone(),
        None,
    );

    let workspace_analyzer = WorkspaceAnalyzer::new(
        workspace_intelligence,
        document_indexer,
        relationship_analyzer,
        None,
    );

    let config = WorkspaceAnalysisConfig {
        analyze_content_gaps: true,
        detect_redundancy: false,
        assess_organization: false,
        generate_ai_insights: false,
        ..Default::default()
    };

    match workspace_analyzer.analyze_workspace(&path, config).await {
        Ok(analysis) => {
            Ok(ContentGapResponse {
                success: true,
                gap_analysis: analysis.content_gaps,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Content gap analysis failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(ContentGapResponse {
                success: false,
                gap_analysis: None,
                error: Some(error_msg),
            })
        }
    }
}

/// Analyze content redundancy specifically
#[tauri::command]
pub async fn analyze_content_redundancy(
    workspace_path: String,
) -> Result<RedundancyResponse, String> {
    tracing::info!("Analyzing content redundancy for workspace: {}", workspace_path);

    let path = PathBuf::from(&workspace_path);

    // Validate workspace path
    if !path.exists() || !path.is_dir() {
        return Ok(RedundancyResponse {
            success: false,
            redundancy_analysis: None,
            error: Some("Invalid workspace path".to_string()),
        });
    }

    // Create analyzer with redundancy focus
    let document_indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(&path)
            .map_err(|e| format!("Failed to create document indexer: {}", e))?
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new()));

    let workspace_intelligence = WorkspaceIntelligence::new(
        document_indexer.clone(),
        relationship_analyzer.clone(),
        None,
    );

    let workspace_analyzer = WorkspaceAnalyzer::new(
        workspace_intelligence,
        document_indexer,
        relationship_analyzer,
        None,
    );

    let config = WorkspaceAnalysisConfig {
        analyze_content_gaps: false,
        detect_redundancy: true,
        assess_organization: false,
        generate_ai_insights: false,
        ..Default::default()
    };

    match workspace_analyzer.analyze_workspace(&path, config).await {
        Ok(analysis) => {
            Ok(RedundancyResponse {
                success: true,
                redundancy_analysis: analysis.redundancy_analysis,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Redundancy analysis failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(RedundancyResponse {
                success: false,
                redundancy_analysis: None,
                error: Some(error_msg),
            })
        }
    }
}

/// Assess workspace organization specifically
#[tauri::command]
pub async fn assess_workspace_organization(
    workspace_path: String,
) -> Result<OrganizationResponse, String> {
    tracing::info!("Assessing workspace organization for: {}", workspace_path);

    let path = PathBuf::from(&workspace_path);

    // Validate workspace path
    if !path.exists() || !path.is_dir() {
        return Ok(OrganizationResponse {
            success: false,
            organization_assessment: None,
            error: Some("Invalid workspace path".to_string()),
        });
    }

    // Create analyzer with organization focus
    let document_indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(&path)
            .map_err(|e| format!("Failed to create document indexer: {}", e))?
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new()));

    let workspace_intelligence = WorkspaceIntelligence::new(
        document_indexer.clone(),
        relationship_analyzer.clone(),
        None,
    );

    let workspace_analyzer = WorkspaceAnalyzer::new(
        workspace_intelligence,
        document_indexer,
        relationship_analyzer,
        None,
    );

    let config = WorkspaceAnalysisConfig {
        analyze_content_gaps: false,
        detect_redundancy: false,
        assess_organization: true,
        generate_ai_insights: false,
        ..Default::default()
    };

    match workspace_analyzer.analyze_workspace(&path, config).await {
        Ok(analysis) => {
            Ok(OrganizationResponse {
                success: true,
                organization_assessment: analysis.organization_assessment,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Organization assessment failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(OrganizationResponse {
                success: false,
                organization_assessment: None,
                error: Some(error_msg),
            })
        }
    }
}

/// Get workspace health score
#[tauri::command]
pub async fn get_workspace_health_score(
    workspace_path: String,
) -> Result<serde_json::Value, String> {
    tracing::info!("Getting workspace health score for: {}", workspace_path);

    let path = PathBuf::from(&workspace_path);

    // Validate workspace path
    if !path.exists() || !path.is_dir() {
        return Ok(serde_json::json!({
            "success": false,
            "health_score": 0.0,
            "error": "Invalid workspace path"
        }));
    }

    // Create analyzer for quick health check
    let document_indexer = Arc::new(Mutex::new(
        DocumentIndexer::new(&path)
            .map_err(|e| format!("Failed to create document indexer: {}", e))?
    ));

    let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new()));

    let workspace_intelligence = WorkspaceIntelligence::new(
        document_indexer.clone(),
        relationship_analyzer.clone(),
        None,
    );

    let workspace_analyzer = WorkspaceAnalyzer::new(
        workspace_intelligence,
        document_indexer,
        relationship_analyzer,
        None,
    );

    // Quick analysis for health score
    let config = WorkspaceAnalysisConfig {
        analyze_content_gaps: true,
        detect_redundancy: true,
        assess_organization: true,
        generate_ai_insights: false, // Skip AI insights for quick check
        min_confidence: 0.5,
        max_depth: 5, // Shallow analysis for speed
    };

    match workspace_analyzer.analyze_workspace(&path, config).await {
        Ok(analysis) => {
            Ok(serde_json::json!({
                "success": true,
                "health_score": analysis.overall_health_score,
                "analysis_timestamp": analysis.analysis_timestamp,
                "documents_analyzed": analysis.documents_analyzed,
                "directories_analyzed": analysis.directories_analyzed,
                "error": null
            }))
        }
        Err(e) => {
            let error_msg = format!("Health score calculation failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(serde_json::json!({
                "success": false,
                "health_score": 0.0,
                "error": error_msg
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initialize_workspace_analyzer() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_string_lossy().to_string();

        let response = initialize_workspace_analyzer(workspace_path).await.unwrap();

        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_initialize_workspace_analyzer_invalid_path() {
        let invalid_path = "/nonexistent/path".to_string();

        let response = initialize_workspace_analyzer(invalid_path).await.unwrap();

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_get_default_analysis_config() {
        let response = get_default_analysis_config().await.unwrap();

        assert!(response.success);
        assert!(response.config.is_some());

        let config = response.config.unwrap();
        assert!(config.analyze_content_gaps);
        assert!(config.detect_redundancy);
        assert!(config.assess_organization);
        assert!(config.generate_ai_insights);
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.max_depth, 10);
    }

    #[tokio::test]
    async fn test_create_analysis_config_valid() {
        let response = create_analysis_config(
            true,  // analyze_content_gaps
            false, // detect_redundancy
            true,  // assess_organization
            false, // generate_ai_insights
            0.8,   // min_confidence
            5,     // max_depth
        ).await.unwrap();

        assert!(response.success);
        assert!(response.config.is_some());

        let config = response.config.unwrap();
        assert!(config.analyze_content_gaps);
        assert!(!config.detect_redundancy);
        assert!(config.assess_organization);
        assert!(!config.generate_ai_insights);
        assert_eq!(config.min_confidence, 0.8);
        assert_eq!(config.max_depth, 5);
    }

    #[tokio::test]
    async fn test_create_analysis_config_invalid_confidence() {
        let response = create_analysis_config(
            true,  // analyze_content_gaps
            true,  // detect_redundancy
            true,  // assess_organization
            true,  // generate_ai_insights
            1.5,   // invalid min_confidence
            5,     // max_depth
        ).await.unwrap();

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_create_analysis_config_invalid_depth() {
        let response = create_analysis_config(
            true,  // analyze_content_gaps
            true,  // detect_redundancy
            true,  // assess_organization
            true,  // generate_ai_insights
            0.7,   // min_confidence
            0,     // invalid max_depth
        ).await.unwrap();

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_analyze_content_gaps() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_string_lossy().to_string();

        // Create some test files
        std::fs::write(temp_dir.path().join("test.md"), "Test content").unwrap();

        let response = analyze_content_gaps(workspace_path).await.unwrap();

        assert!(response.success);
        // Gap analysis should work even with minimal content
    }

    #[tokio::test]
    async fn test_get_workspace_health_score() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_string_lossy().to_string();

        // Create some test content
        std::fs::write(temp_dir.path().join("readme.md"), "# Test Workspace").unwrap();

        let response = get_workspace_health_score(workspace_path).await.unwrap();

        assert!(response["success"].as_bool().unwrap());
        assert!(response["health_score"].as_f64().is_some());
    }
}