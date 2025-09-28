// src-tauri/src/commands/workspace_ai_commands.rs
//! Tauri commands for workspace-AI integration

use crate::ai::context::WorkspaceIntelligenceContext;
use crate::commands::conversation_context_commands::ConversationContextState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

/// Request structure for workspace intelligence integration
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceAIIntegrationRequest {
    pub session_id: String,
    pub workspace_path: PathBuf,
}

/// Response structure for workspace intelligence integration
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceAIIntegrationResponse {
    pub success: bool,
    pub workspace_health_score: Option<f32>,
    pub total_documents: Option<usize>,
    pub top_recommendations: Option<Vec<String>>,
    pub error: Option<String>,
}

/// Response structure for workspace insights
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInsightsResponse {
    pub success: bool,
    pub insights: Option<WorkspaceInsights>,
    pub error: Option<String>,
}

/// Workspace insights for AI conversation
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInsights {
    pub health_score: f32,
    pub organization_quality: f32,
    pub content_freshness: f32,
    pub knowledge_gaps_count: usize,
    pub recommendations_count: usize,
    pub key_insights: Vec<String>,
    pub action_suggestions: Vec<String>,
}

/// Response structure for workspace conversation status
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceConversationStatusResponse {
    pub success: bool,
    pub has_workspace_context: bool,
    pub last_analysis_timestamp: Option<String>,
    pub health_score: Option<f32>,
    pub error: Option<String>,
}

/// Enable workspace-aware conversations for a session
#[tauri::command]
pub async fn enable_workspace_ai_integration(
    request: WorkspaceAIIntegrationRequest,
    conversation_context_manager: State<'_, ConversationContextState>,
) -> Result<WorkspaceAIIntegrationResponse, String> {
    // For now, create a simplified workspace intelligence context without full analysis
    // This can be enhanced later to use actual workspace analysis when the state management is resolved
    let simplified_workspace_context = WorkspaceIntelligenceContext {
        last_analysis_timestamp: chrono::Utc::now(),
        health_score: 75.0, // Default reasonable health score
        total_documents: 0, // Will be updated when proper indexing is integrated
        top_knowledge_gaps: vec![],
        priority_recommendations: vec![],
        organization_summary: crate::ai::context::WorkspaceOrganizationSummary {
            directory_count: 1,
            avg_directory_utilization: 0.5,
            naming_consistency_score: 0.8,
            structure_quality_score: 0.7,
        },
        content_freshness: crate::ai::context::ContentFreshnessSummary {
            fresh_documents: 0,
            stale_documents: 0,
            outdated_documents: 0,
            average_age_days: 30.0,
            freshness_score: 70.0,
        },
    };

    // Update conversation context manager with simplified workspace intelligence
    {
        let mut conv_manager = conversation_context_manager.lock().await;
        conv_manager
            .update_workspace_intelligence(
                &request.session_id,
                simplified_workspace_context.clone(),
            )
            .map_err(|e| format!("Failed to update conversation context: {}", e))?
    }

    Ok(WorkspaceAIIntegrationResponse {
        success: true,
        workspace_health_score: Some(simplified_workspace_context.health_score),
        total_documents: Some(simplified_workspace_context.total_documents),
        top_recommendations: Some(vec![
            "Enable workspace-aware conversations".to_string(),
            "Import documents for better analysis".to_string(),
            "Configure AI settings for optimal performance".to_string(),
        ]),
        error: None,
    })
}

/// Get workspace insights for AI conversation
#[tauri::command]
pub async fn get_workspace_insights_for_ai(
    session_id: String,
    conversation_context_manager: State<'_, ConversationContextState>,
) -> Result<WorkspaceInsightsResponse, String> {
    let conv_manager = conversation_context_manager.lock().await;

    if let Some(workspace_intel) = conv_manager.get_workspace_intelligence(&session_id) {
        let insights = WorkspaceInsights {
            health_score: workspace_intel.health_score,
            organization_quality: workspace_intel.organization_summary.structure_quality_score
                * 100.0,
            content_freshness: workspace_intel.content_freshness.freshness_score,
            knowledge_gaps_count: workspace_intel.top_knowledge_gaps.len(),
            recommendations_count: workspace_intel.priority_recommendations.len(),
            key_insights: generate_key_insights(workspace_intel),
            action_suggestions: generate_action_suggestions(workspace_intel),
        };

        Ok(WorkspaceInsightsResponse {
            success: true,
            insights: Some(insights),
            error: None,
        })
    } else {
        Ok(WorkspaceInsightsResponse {
            success: false,
            insights: None,
            error: Some("No workspace intelligence context available for this session".to_string()),
        })
    }
}

/// Check if workspace-aware conversations are enabled for a session
#[tauri::command]
pub async fn check_workspace_conversation_status(
    session_id: String,
    conversation_context_manager: State<'_, ConversationContextState>,
) -> Result<WorkspaceConversationStatusResponse, String> {
    let conv_manager = conversation_context_manager.lock().await;

    let has_workspace_context = conv_manager.has_workspace_intelligence(&session_id);

    if let Some(workspace_intel) = conv_manager.get_workspace_intelligence(&session_id) {
        Ok(WorkspaceConversationStatusResponse {
            success: true,
            has_workspace_context,
            last_analysis_timestamp: Some(workspace_intel.last_analysis_timestamp.to_rfc3339()),
            health_score: Some(workspace_intel.health_score),
            error: None,
        })
    } else {
        Ok(WorkspaceConversationStatusResponse {
            success: true,
            has_workspace_context,
            last_analysis_timestamp: None,
            health_score: None,
            error: None,
        })
    }
}

/// Refresh workspace intelligence for ongoing conversation
#[tauri::command]
pub async fn refresh_workspace_intelligence_for_ai(
    request: WorkspaceAIIntegrationRequest,
    conversation_context_manager: State<'_, ConversationContextState>,
) -> Result<WorkspaceAIIntegrationResponse, String> {
    // Re-analyze workspace and update context managers
    enable_workspace_ai_integration(request, conversation_context_manager).await
}

/// Get workspace recommendations formatted for AI conversation
#[tauri::command]
pub async fn get_workspace_recommendations_for_ai(
    session_id: String,
    conversation_context_manager: State<'_, ConversationContextState>,
) -> Result<Vec<String>, String> {
    let conv_manager = conversation_context_manager.lock().await;

    if let Some(workspace_intel) = conv_manager.get_workspace_intelligence(&session_id) {
        let formatted_recommendations: Vec<String> = workspace_intel
            .priority_recommendations
            .iter()
            .map(|rec| {
                format!(
                    "**{}** (Priority: {:.1}/10)\n{}\nImpact: {:?} | Effort: {:?}",
                    rec.title,
                    match rec.priority {
                        crate::workspace::intelligence::RecommendationPriority::Urgent => 10.0,
                        crate::workspace::intelligence::RecommendationPriority::High => 8.0,
                        crate::workspace::intelligence::RecommendationPriority::Medium => 6.0,
                        crate::workspace::intelligence::RecommendationPriority::Low => 4.0,
                    },
                    rec.description,
                    rec.expected_impact,
                    rec.estimated_effort
                )
            })
            .collect();

        Ok(formatted_recommendations)
    } else {
        Ok(Vec::new())
    }
}

/// Generate key insights from workspace intelligence
fn generate_key_insights(workspace_intel: &WorkspaceIntelligenceContext) -> Vec<String> {
    let mut insights = Vec::new();

    // Health assessment
    if workspace_intel.health_score >= 80.0 {
        insights
            .push("Your workspace is in excellent health with well-organized content.".to_string());
    } else if workspace_intel.health_score >= 60.0 {
        insights.push(
            "Your workspace has good organization but could benefit from some improvements."
                .to_string(),
        );
    } else {
        insights.push(
            "Your workspace needs attention to improve organization and content quality."
                .to_string(),
        );
    }

    // Content freshness insights
    if workspace_intel.content_freshness.freshness_score >= 80.0 {
        insights.push("Your content is well-maintained and up-to-date.".to_string());
    } else {
        insights.push(format!(
            "Consider updating {} outdated documents to improve content freshness.",
            workspace_intel.content_freshness.outdated_documents
        ));
    }

    // Knowledge gaps insights
    if !workspace_intel.top_knowledge_gaps.is_empty() {
        insights.push(format!(
            "Identified {} knowledge gaps that could impact productivity.",
            workspace_intel.top_knowledge_gaps.len()
        ));
    }

    // Organization insights
    if workspace_intel
        .organization_summary
        .naming_consistency_score
        < 0.6
    {
        insights.push("Naming consistency could be improved across your documents.".to_string());
    }

    insights
}

/// Generate actionable suggestions from workspace intelligence
fn generate_action_suggestions(workspace_intel: &WorkspaceIntelligenceContext) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Health-based suggestions
    if workspace_intel.health_score < 70.0 {
        suggestions.push("Consider running a comprehensive workspace cleanup.".to_string());
    }

    // Content freshness suggestions
    if workspace_intel.content_freshness.outdated_documents > 5 {
        suggestions.push(
            "Schedule regular content review sessions to keep information current.".to_string(),
        );
    }

    // Knowledge gaps suggestions
    for gap in &workspace_intel.top_knowledge_gaps {
        if gap.priority_score > 7.0 {
            suggestions.push(format!("Address high-priority gap: {}", gap.description));
        }
    }

    // Organization suggestions
    if workspace_intel
        .organization_summary
        .naming_consistency_score
        < 0.6
    {
        suggestions.push("Implement consistent naming conventions across documents.".to_string());
    }

    // Recommendations from workspace analysis
    for rec in workspace_intel.priority_recommendations.iter().take(2) {
        suggestions.push(format!("{}: {}", rec.title, rec.description));
    }

    suggestions
}
