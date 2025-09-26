// src-tauri/src/commands/workspace_intelligence_commands.rs
//! Tauri commands for workspace intelligence operations

use crate::ai::AIOrchestrator;
use crate::document::indexer::DocumentIndexer;
use crate::workspace::{WorkspaceAnalysis, WorkspaceIntelligence, WorkspaceManager};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Request for workspace analysis
#[derive(Debug, Deserialize)]
pub struct AnalyzeWorkspaceRequest {
    pub workspace_path: PathBuf,
}

/// Response for workspace analysis
#[derive(Debug, Serialize)]
pub struct AnalyzeWorkspaceResponse {
    pub success: bool,
    pub analysis: Option<WorkspaceAnalysis>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
}

/// Request for workspace recommendations
#[derive(Debug, Deserialize)]
pub struct GetWorkspaceRecommendationsRequest {
    pub workspace_path: PathBuf,
    pub recommendation_types: Option<Vec<String>>,
    pub priority_filter: Option<String>,
}

/// Response for workspace recommendations
#[derive(Debug, Serialize)]
pub struct GetWorkspaceRecommendationsResponse {
    pub success: bool,
    pub recommendations: Vec<crate::workspace::WorkspaceRecommendation>,
    pub error: Option<String>,
}

/// Request for workspace insights
#[derive(Debug, Deserialize)]
pub struct GetWorkspaceInsightsRequest {
    pub workspace_path: PathBuf,
    pub _insight_types: Option<Vec<String>>,
}

/// Response for workspace insights
#[derive(Debug, Serialize)]
pub struct GetWorkspaceInsightsResponse {
    pub success: bool,
    pub insights: Option<WorkspaceInsightsSummary>,
    pub error: Option<String>,
}

/// Summary of workspace insights
#[derive(Debug, Serialize)]
pub struct WorkspaceInsightsSummary {
    pub total_documents: usize,
    pub content_health_score: f64,
    pub organization_score: f64,
    pub top_topics: Vec<String>,
    pub critical_issues: Vec<String>,
    pub productivity_metrics: ProductivitySummary,
}

/// Productivity metrics summary
#[derive(Debug, Serialize)]
pub struct ProductivitySummary {
    pub creation_velocity: f64,
    pub update_frequency: f64,
    pub maturity_score: f64,
}

/// State container for workspace intelligence
#[allow(dead_code)]
pub struct WorkspaceIntelligenceState {
    pub analyzer: Arc<Mutex<WorkspaceIntelligence>>,
}

/// Initialize workspace intelligence analyzer
#[tauri::command]
pub async fn initialize_workspace_intelligence(
    document_indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    ai_orchestrator: State<'_, Option<Arc<Mutex<AIOrchestrator>>>>,
) -> Result<String, String> {
    println!("🧠 Initializing workspace intelligence analyzer");

    let _intelligence = WorkspaceIntelligence::new(
        document_indexer.inner().clone(),
        ai_orchestrator.inner().clone(),
    );

    println!("✅ Workspace intelligence analyzer initialized");
    Ok("Workspace intelligence initialized successfully".to_string())
}

/// Perform comprehensive workspace analysis
#[tauri::command]
pub async fn analyze_workspace(
    request: AnalyzeWorkspaceRequest,
    document_indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    ai_orchestrator: State<'_, Option<Arc<Mutex<AIOrchestrator>>>>,
    workspace_manager: State<'_, Arc<Mutex<WorkspaceManager>>>,
) -> Result<AnalyzeWorkspaceResponse, String> {
    let start_time = std::time::Instant::now();

    println!(
        "🔍 Starting workspace analysis for: {}",
        request.workspace_path.display()
    );

    // Load workspace information
    let workspace_info = {
        let manager = workspace_manager.lock().await;
        match manager.load_workspace(&request.workspace_path).await {
            Ok(info) => info,
            Err(e) => {
                let error_msg = format!("Failed to load workspace: {}", e);
                println!("❌ {}", error_msg);
                return Ok(AnalyzeWorkspaceResponse {
                    success: false,
                    analysis: None,
                    error: Some(error_msg),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                });
            }
        }
    };

    // Create workspace intelligence analyzer
    let intelligence = WorkspaceIntelligence::new(
        document_indexer.inner().clone(),
        ai_orchestrator.inner().clone(),
    );

    // Perform analysis
    match intelligence.analyze_workspace(&workspace_info).await {
        Ok(analysis) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            println!("✅ Workspace analysis completed in {}ms", processing_time);

            Ok(AnalyzeWorkspaceResponse {
                success: true,
                analysis: Some(analysis),
                error: None,
                processing_time_ms: processing_time,
            })
        }
        Err(e) => {
            let error_msg = format!("Workspace analysis failed: {}", e);
            println!("❌ {}", error_msg);

            Ok(AnalyzeWorkspaceResponse {
                success: false,
                analysis: None,
                error: Some(error_msg),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            })
        }
    }
}

/// Get workspace recommendations
#[tauri::command]
pub async fn get_workspace_recommendations(
    request: GetWorkspaceRecommendationsRequest,
    document_indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    ai_orchestrator: State<'_, Option<Arc<Mutex<AIOrchestrator>>>>,
    workspace_manager: State<'_, Arc<Mutex<WorkspaceManager>>>,
) -> Result<GetWorkspaceRecommendationsResponse, String> {
    println!(
        "💡 Getting workspace recommendations for: {}",
        request.workspace_path.display()
    );

    // Load workspace information
    let workspace_info = {
        let manager = workspace_manager.lock().await;
        match manager.load_workspace(&request.workspace_path).await {
            Ok(info) => info,
            Err(e) => {
                let error_msg = format!("Failed to load workspace: {}", e);
                return Ok(GetWorkspaceRecommendationsResponse {
                    success: false,
                    recommendations: Vec::new(),
                    error: Some(error_msg),
                });
            }
        }
    };

    // Create intelligence analyzer and perform analysis
    let intelligence = WorkspaceIntelligence::new(
        document_indexer.inner().clone(),
        ai_orchestrator.inner().clone(),
    );

    match intelligence.analyze_workspace(&workspace_info).await {
        Ok(analysis) => {
            let mut recommendations = analysis.recommendations;

            // Filter by recommendation types if specified
            if let Some(types) = &request.recommendation_types {
                recommendations.retain(|rec| {
                    let rec_type = match rec.recommendation_type {
                        crate::workspace::RecommendationType::Organization => "organization",
                        crate::workspace::RecommendationType::ContentCreation => "content_creation",
                        crate::workspace::RecommendationType::ContentUpdate => "content_update",
                        crate::workspace::RecommendationType::Cleanup => "cleanup",
                        crate::workspace::RecommendationType::ProcessImprovement => {
                            "process_improvement"
                        }
                        crate::workspace::RecommendationType::QualityImprovement => {
                            "quality_improvement"
                        }
                    };
                    types.contains(&rec_type.to_string())
                });
            }

            // Filter by priority if specified
            if let Some(priority_filter) = &request.priority_filter {
                let filter_priority = match priority_filter.as_str() {
                    "urgent" => crate::workspace::RecommendationPriority::Urgent,
                    "high" => crate::workspace::RecommendationPriority::High,
                    "medium" => crate::workspace::RecommendationPriority::Medium,
                    "low" => crate::workspace::RecommendationPriority::Low,
                    _ => crate::workspace::RecommendationPriority::Medium,
                };

                recommendations.retain(|rec| {
                    matches!(
                        (&rec.priority, &filter_priority),
                        (
                            crate::workspace::RecommendationPriority::Urgent,
                            crate::workspace::RecommendationPriority::Urgent
                        ) | (
                            crate::workspace::RecommendationPriority::High,
                            crate::workspace::RecommendationPriority::High
                        ) | (
                            crate::workspace::RecommendationPriority::Medium,
                            crate::workspace::RecommendationPriority::Medium
                        ) | (
                            crate::workspace::RecommendationPriority::Low,
                            crate::workspace::RecommendationPriority::Low
                        )
                    )
                });
            }

            println!("✅ Found {} recommendations", recommendations.len());

            Ok(GetWorkspaceRecommendationsResponse {
                success: true,
                recommendations,
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Failed to analyze workspace for recommendations: {}", e);
            Ok(GetWorkspaceRecommendationsResponse {
                success: false,
                recommendations: Vec::new(),
                error: Some(error_msg),
            })
        }
    }
}

/// Get workspace insights summary
#[tauri::command]
pub async fn get_workspace_insights(
    request: GetWorkspaceInsightsRequest,
    document_indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    ai_orchestrator: State<'_, Option<Arc<Mutex<AIOrchestrator>>>>,
    workspace_manager: State<'_, Arc<Mutex<WorkspaceManager>>>,
) -> Result<GetWorkspaceInsightsResponse, String> {
    println!(
        "🔍 Getting workspace insights for: {}",
        request.workspace_path.display()
    );

    // Load workspace information
    let workspace_info = {
        let manager = workspace_manager.lock().await;
        match manager.load_workspace(&request.workspace_path).await {
            Ok(info) => info,
            Err(e) => {
                let error_msg = format!("Failed to load workspace: {}", e);
                return Ok(GetWorkspaceInsightsResponse {
                    success: false,
                    insights: None,
                    error: Some(error_msg),
                });
            }
        }
    };

    // Create intelligence analyzer and perform analysis
    let intelligence = WorkspaceIntelligence::new(
        document_indexer.inner().clone(),
        ai_orchestrator.inner().clone(),
    );

    match intelligence.analyze_workspace(&workspace_info).await {
        Ok(analysis) => {
            let top_topics = analysis
                .content_patterns
                .dominant_topics
                .iter()
                .take(5)
                .map(|topic| topic.topic_name.clone())
                .collect();

            let critical_issues = analysis
                .knowledge_gaps
                .iter()
                .filter(|gap| {
                    matches!(
                        gap.severity,
                        crate::workspace::GapSeverity::Critical
                            | crate::workspace::GapSeverity::High
                    )
                })
                .map(|gap| gap.description.clone())
                .collect();

            let insights = WorkspaceInsightsSummary {
                total_documents: analysis.document_overview.total_documents,
                content_health_score: analysis.quality_assessment.overall_quality_score,
                organization_score: analysis.organization_insights.file_organization_score,
                top_topics,
                critical_issues,
                productivity_metrics: ProductivitySummary {
                    creation_velocity: analysis.productivity_metrics.content_creation_velocity,
                    update_frequency: analysis.productivity_metrics.update_frequency,
                    maturity_score: analysis.productivity_metrics.workspace_maturity_score,
                },
            };

            println!("✅ Generated workspace insights");

            Ok(GetWorkspaceInsightsResponse {
                success: true,
                insights: Some(insights),
                error: None,
            })
        }
        Err(e) => {
            let error_msg = format!("Failed to generate workspace insights: {}", e);
            Ok(GetWorkspaceInsightsResponse {
                success: false,
                insights: None,
                error: Some(error_msg),
            })
        }
    }
}

/// Get workspace health score
#[tauri::command]
pub async fn get_workspace_health_score(
    workspace_path: PathBuf,
    document_indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    ai_orchestrator: State<'_, Option<Arc<Mutex<AIOrchestrator>>>>,
    workspace_manager: State<'_, Arc<Mutex<WorkspaceManager>>>,
) -> Result<f64, String> {
    println!(
        "❤️ Calculating workspace health score for: {}",
        workspace_path.display()
    );

    // Load workspace information
    let workspace_info = {
        let manager = workspace_manager.lock().await;
        match manager.load_workspace(&workspace_path).await {
            Ok(info) => info,
            Err(e) => {
                return Err(format!("Failed to load workspace: {}", e));
            }
        }
    };

    // Create intelligence analyzer and perform analysis
    let intelligence = WorkspaceIntelligence::new(
        document_indexer.inner().clone(),
        ai_orchestrator.inner().clone(),
    );

    match intelligence.analyze_workspace(&workspace_info).await {
        Ok(analysis) => {
            // Calculate composite health score
            let quality_weight = 0.4;
            let organization_weight = 0.3;
            let productivity_weight = 0.3;

            let health_score = (analysis.quality_assessment.overall_quality_score * quality_weight)
                + (analysis.organization_insights.file_organization_score * organization_weight)
                + (analysis.productivity_metrics.efficiency_score * productivity_weight);

            println!("✅ Workspace health score: {:.2}", health_score);
            Ok(health_score)
        }
        Err(e) => Err(format!("Failed to calculate health score: {}", e)),
    }
}

/// Compare two workspaces
#[tauri::command]
pub async fn compare_workspaces(
    workspace_a_path: PathBuf,
    workspace_b_path: PathBuf,
    document_indexer: State<'_, Arc<Mutex<DocumentIndexer>>>,
    ai_orchestrator: State<'_, Option<Arc<Mutex<AIOrchestrator>>>>,
    workspace_manager: State<'_, Arc<Mutex<WorkspaceManager>>>,
) -> Result<WorkspaceComparisonResult, String> {
    println!(
        "🔄 Comparing workspaces: {} vs {}",
        workspace_a_path.display(),
        workspace_b_path.display()
    );

    let manager = workspace_manager.lock().await;

    // Load both workspaces
    let (workspace_a, workspace_b) = tokio::try_join!(
        manager.load_workspace(&workspace_a_path),
        manager.load_workspace(&workspace_b_path)
    )
    .map_err(|e| format!("Failed to load workspaces: {}", e))?;

    drop(manager); // Release the lock

    // Create intelligence analyzer
    let intelligence = WorkspaceIntelligence::new(
        document_indexer.inner().clone(),
        ai_orchestrator.inner().clone(),
    );

    // Analyze both workspaces
    let (analysis_a, analysis_b) = tokio::try_join!(
        intelligence.analyze_workspace(&workspace_a),
        intelligence.analyze_workspace(&workspace_b)
    )
    .map_err(|e| format!("Failed to analyze workspaces: {}", e))?;

    // Create comparison result
    let comparison = WorkspaceComparisonResult {
        workspace_a_name: workspace_a.name.clone(),
        workspace_b_name: workspace_b.name.clone(),
        document_count_a: analysis_a.document_overview.total_documents,
        document_count_b: analysis_b.document_overview.total_documents,
        quality_score_a: analysis_a.quality_assessment.overall_quality_score,
        quality_score_b: analysis_b.quality_assessment.overall_quality_score,
        organization_score_a: analysis_a.organization_insights.file_organization_score,
        organization_score_b: analysis_b.organization_insights.file_organization_score,
        maturity_score_a: analysis_a.productivity_metrics.workspace_maturity_score,
        maturity_score_b: analysis_b.productivity_metrics.workspace_maturity_score,
        recommendations_a: analysis_a.recommendations.len(),
        recommendations_b: analysis_b.recommendations.len(),
        better_workspace: determine_better_workspace(&analysis_a, &analysis_b),
    };

    println!("✅ Workspace comparison completed");
    Ok(comparison)
}

/// Workspace comparison result
#[derive(Debug, Serialize)]
pub struct WorkspaceComparisonResult {
    pub workspace_a_name: String,
    pub workspace_b_name: String,
    pub document_count_a: usize,
    pub document_count_b: usize,
    pub quality_score_a: f64,
    pub quality_score_b: f64,
    pub organization_score_a: f64,
    pub organization_score_b: f64,
    pub maturity_score_a: f64,
    pub maturity_score_b: f64,
    pub recommendations_a: usize,
    pub recommendations_b: usize,
    pub better_workspace: String,
}

/// Determine which workspace is better overall
fn determine_better_workspace(
    analysis_a: &WorkspaceAnalysis,
    analysis_b: &WorkspaceAnalysis,
) -> String {
    let score_a = (analysis_a.quality_assessment.overall_quality_score
        + analysis_a.organization_insights.file_organization_score
        + analysis_a.productivity_metrics.workspace_maturity_score)
        / 3.0;

    let score_b = (analysis_b.quality_assessment.overall_quality_score
        + analysis_b.organization_insights.file_organization_score
        + analysis_b.productivity_metrics.workspace_maturity_score)
        / 3.0;

    if score_a > score_b + 0.1 {
        format!(
            "{} (score: {:.2})",
            analysis_a
                .workspace_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            score_a
        )
    } else if score_b > score_a + 0.1 {
        format!(
            "{} (score: {:.2})",
            analysis_b
                .workspace_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            score_b
        )
    } else {
        "Similar quality".to_string()
    }
}
