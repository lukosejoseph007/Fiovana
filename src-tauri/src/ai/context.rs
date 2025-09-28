// src-tauri/src/ai/context.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ai::prompts::{ConversationTurn, DocumentMetadata};
use crate::workspace::{WorkspaceAnalysis, WorkspaceRecommendation};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRef {
    pub id: String,
    pub title: String,
    pub path: Option<String>,
    pub content_preview: String, // First 500 chars for quick reference
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContext {
    pub active_documents: Vec<DocumentRef>,
    pub conversation_history: Vec<ConversationTurn>,
    pub session_metadata: SessionMetadata,
    pub workspace_intelligence: Option<WorkspaceIntelligenceContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub workspace_path: Option<String>,
    pub current_task: Option<String>,
    pub user_preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_response_style: String, // "detailed", "concise", "technical"
    pub include_citations: bool,
    pub max_context_length: usize,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_response_style: "detailed".to_string(),
            include_citations: true,
            max_context_length: 4000,
        }
    }
}

/// Workspace intelligence context for AI conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIntelligenceContext {
    /// Last workspace analysis timestamp
    pub last_analysis_timestamp: DateTime<Utc>,
    /// Overall workspace health score (0-100)
    pub health_score: f32,
    /// Number of total documents in workspace
    pub total_documents: usize,
    /// Top knowledge gaps that need attention
    pub top_knowledge_gaps: Vec<WorkspaceKnowledgeGap>,
    /// High-priority workspace recommendations
    pub priority_recommendations: Vec<WorkspaceRecommendation>,
    /// Workspace organization insights summary
    pub organization_summary: WorkspaceOrganizationSummary,
    /// Content freshness assessment
    pub content_freshness: ContentFreshnessSummary,
}

/// Simplified knowledge gap for conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceKnowledgeGap {
    pub gap_type: String,
    pub description: String,
    pub severity: String,
    pub priority_score: f64,
}

/// Organization summary for conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceOrganizationSummary {
    pub directory_count: usize,
    pub avg_directory_utilization: f32,
    pub naming_consistency_score: f32,
    pub structure_quality_score: f32,
}

/// Content freshness summary for conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFreshnessSummary {
    pub fresh_documents: usize,
    pub stale_documents: usize,
    pub outdated_documents: usize,
    pub average_age_days: f64,
    pub freshness_score: f32,
}

pub struct DocumentContextManager {
    pub contexts: HashMap<String, DocumentContext>, // session_id -> context
    max_conversation_history: usize,
    #[allow(dead_code)]
    max_active_documents: usize,
}

impl Default for DocumentContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentContextManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            max_conversation_history: 20, // Keep last 20 turns
            max_active_documents: 10,     // Track up to 10 documents
        }
    }

    /// Get or create a document context for a session
    pub fn get_or_create_context(&mut self, session_id: &str) -> &mut DocumentContext {
        self.contexts
            .entry(session_id.to_string())
            .or_insert_with(|| DocumentContext {
                active_documents: Vec::new(),
                conversation_history: Vec::new(),
                session_metadata: SessionMetadata {
                    session_id: session_id.to_string(),
                    workspace_path: None,
                    current_task: None,
                    user_preferences: UserPreferences::default(),
                },
                workspace_intelligence: None,
            })
    }

    /// Add relevant documents to the context
    #[allow(dead_code)]
    pub fn add_relevant_documents(
        &mut self,
        session_id: &str,
        documents: Vec<DocumentRef>,
    ) -> Result<()> {
        let max_docs = self.max_active_documents;
        let context = self.get_or_create_context(session_id);

        // Add new documents, removing oldest if we exceed the limit
        for doc in documents {
            // Check if document already exists (update relevance if so)
            if let Some(existing_doc) = context.active_documents.iter_mut().find(|d| d.id == doc.id)
            {
                existing_doc.relevance_score = doc.relevance_score;
                existing_doc.content_preview = doc.content_preview;
            } else {
                context.active_documents.push(doc);
            }
        }

        // Sort by relevance and keep only the most relevant documents
        context
            .active_documents
            .sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        context.active_documents.truncate(max_docs);

        Ok(())
    }

    /// Add a conversation turn to the history
    pub fn add_conversation_turn(
        &mut self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<()> {
        let max_history = self.max_conversation_history;
        let context = self.get_or_create_context(session_id);

        let turn = ConversationTurn {
            role: role.to_string(),
            content: content.to_string(),
        };

        context.conversation_history.push(turn);

        // Keep only the most recent conversation turns
        if context.conversation_history.len() > max_history {
            context
                .conversation_history
                .drain(0..context.conversation_history.len() - max_history);
        }

        Ok(())
    }

    /// Update workspace intelligence context for a session
    #[allow(dead_code)]
    pub fn update_workspace_intelligence(
        &mut self,
        session_id: &str,
        workspace_analysis: &WorkspaceAnalysis,
    ) -> Result<()> {
        let context = self.get_or_create_context(session_id);

        // Extract top 3 knowledge gaps for conversation context
        let top_knowledge_gaps = workspace_analysis
            .knowledge_gaps
            .iter()
            .take(3)
            .map(|gap| WorkspaceKnowledgeGap {
                gap_type: format!("{:?}", gap.gap_type),
                description: gap.description.clone(),
                severity: format!("{:?}", gap.severity),
                priority_score: gap.priority_score,
            })
            .collect();

        // Extract top 5 priority recommendations
        let priority_recommendations = workspace_analysis
            .recommendations
            .iter()
            .take(5)
            .cloned()
            .collect();

        // Create organization summary
        let organization_summary = WorkspaceOrganizationSummary {
            directory_count: workspace_analysis
                .organization_insights
                .directory_utilization
                .len(),
            avg_directory_utilization: workspace_analysis
                .organization_insights
                .directory_utilization
                .values()
                .map(|metrics| metrics.organization_score as f32)
                .sum::<f32>()
                / workspace_analysis
                    .organization_insights
                    .directory_utilization
                    .len()
                    .max(1) as f32,
            naming_consistency_score: workspace_analysis
                .organization_insights
                .naming_consistency
                .naming_convention_score as f32,
            structure_quality_score: workspace_analysis.quality_assessment.overall_quality_score
                as f32,
        };

        // Create content freshness summary
        let content_freshness = ContentFreshnessSummary {
            fresh_documents: (workspace_analysis
                .productivity_metrics
                .content_creation_velocity
                * 7.0) as usize,
            stale_documents: (workspace_analysis.productivity_metrics.update_frequency * 30.0)
                as usize,
            outdated_documents: workspace_analysis
                .document_overview
                .total_documents
                .saturating_sub(
                    (workspace_analysis.productivity_metrics.update_frequency * 30.0) as usize,
                ),
            average_age_days: 30.0, // Placeholder - would need real calculation
            freshness_score: workspace_analysis.quality_assessment.currency_score as f32 * 100.0,
        };

        context.workspace_intelligence = Some(WorkspaceIntelligenceContext {
            last_analysis_timestamp: workspace_analysis.analysis_timestamp,
            health_score: workspace_analysis.quality_assessment.overall_quality_score as f32
                * 100.0,
            total_documents: workspace_analysis.document_overview.total_documents,
            top_knowledge_gaps,
            priority_recommendations,
            organization_summary,
            content_freshness,
        });

        Ok(())
    }

    /// Get relevant context for AI processing
    pub fn get_relevant_context(&self, session_id: &str, query: &str) -> String {
        if let Some(context) = self.contexts.get(session_id) {
            self.build_context_string(context, query)
        } else {
            "No document context available. Providing general response.".to_string()
        }
    }

    /// Build a comprehensive context string for AI processing
    fn build_context_string(&self, context: &DocumentContext, query: &str) -> String {
        let mut context_parts = Vec::new();

        // Add active documents context
        if !context.active_documents.is_empty() {
            context_parts.push("=== RELEVANT DOCUMENTS ===".to_string());

            for (i, doc) in context.active_documents.iter().enumerate() {
                let doc_section = format!(
                    "Document {}: {} (relevance: {:.2})\n{}\n",
                    i + 1,
                    doc.title,
                    doc.relevance_score,
                    doc.content_preview
                );
                context_parts.push(doc_section);
            }
        }

        // Add recent conversation history for context
        if !context.conversation_history.is_empty() {
            context_parts.push("=== RECENT CONVERSATION ===".to_string());

            // Only include the last few turns to avoid overwhelming the prompt
            let recent_turns = context
                .conversation_history
                .iter()
                .rev()
                .take(5)
                .rev()
                .collect::<Vec<_>>();

            for turn in recent_turns {
                context_parts.push(format!("{}: {}", turn.role.to_uppercase(), turn.content));
            }
        }

        // Add workspace intelligence context
        if let Some(workspace_intel) = &context.workspace_intelligence {
            context_parts.push("=== WORKSPACE INTELLIGENCE ===".to_string());

            context_parts.push(format!(
                "Workspace Health Score: {:.1}/100 ({} documents total)",
                workspace_intel.health_score, workspace_intel.total_documents
            ));

            // Add content freshness information
            context_parts.push(format!(
                "Content Status: {} fresh, {} stale, {} outdated documents (freshness score: {:.1}/100)",
                workspace_intel.content_freshness.fresh_documents,
                workspace_intel.content_freshness.stale_documents,
                workspace_intel.content_freshness.outdated_documents,
                workspace_intel.content_freshness.freshness_score
            ));

            // Add organization summary
            context_parts.push(format!(
                "Organization Quality: {:.1}/100 (naming consistency: {:.1}/100, {} directories)",
                workspace_intel.organization_summary.structure_quality_score * 100.0,
                workspace_intel
                    .organization_summary
                    .naming_consistency_score
                    * 100.0,
                workspace_intel.organization_summary.directory_count
            ));

            // Add top knowledge gaps
            if !workspace_intel.top_knowledge_gaps.is_empty() {
                context_parts.push("Top Knowledge Gaps:".to_string());
                for (i, gap) in workspace_intel.top_knowledge_gaps.iter().enumerate() {
                    context_parts.push(format!(
                        "  {}. {} ({}): {}",
                        i + 1,
                        gap.gap_type,
                        gap.severity,
                        gap.description
                    ));
                }
            }

            // Add priority recommendations
            if !workspace_intel.priority_recommendations.is_empty() {
                context_parts.push("Priority Recommendations:".to_string());
                for (i, rec) in workspace_intel
                    .priority_recommendations
                    .iter()
                    .take(3)
                    .enumerate()
                {
                    context_parts.push(format!(
                        "  {}. {} (Priority: {:.1}): {}",
                        i + 1,
                        rec.title,
                        match rec.priority {
                            crate::workspace::intelligence::RecommendationPriority::Urgent => 10.0,
                            crate::workspace::intelligence::RecommendationPriority::High => 8.0,
                            crate::workspace::intelligence::RecommendationPriority::Medium => 6.0,
                            crate::workspace::intelligence::RecommendationPriority::Low => 4.0,
                        },
                        rec.description
                    ));
                }
            }

            context_parts.push(format!(
                "Last Analysis: {}",
                workspace_intel
                    .last_analysis_timestamp
                    .format("%Y-%m-%d %H:%M UTC")
            ));
        }

        // Add current query context
        context_parts.push("=== CURRENT QUERY ===".to_string());
        context_parts.push(format!("User Question: {}", query));

        // Add user preferences
        if context.session_metadata.user_preferences.include_citations {
            context_parts.push("\nNote: Please include citations to specific documents when referencing information.".to_string());
        }

        // Add workspace consultation guidance
        if context.workspace_intelligence.is_some() {
            context_parts.push("\nNote: You have access to workspace intelligence data. You can proactively mention workspace health, suggest improvements, identify issues, and act as a workspace consultant. Offer insights about content organization, freshness, and recommend actions to improve the workspace.".to_string());
        }

        let response_style = &context
            .session_metadata
            .user_preferences
            .preferred_response_style;
        match response_style.as_str() {
            "concise" => {
                context_parts.push("Please provide a concise, focused response.".to_string())
            }
            "technical" => context_parts
                .push("Please provide a detailed technical response with specifics.".to_string()),
            _ => {
                context_parts.push("Please provide a comprehensive, helpful response.".to_string())
            }
        }

        let full_context = context_parts.join("\n\n");

        // Truncate if too long
        let max_length = context.session_metadata.user_preferences.max_context_length;
        if full_context.len() > max_length {
            format!(
                "{}...\n\n[Context truncated for length]",
                &full_context[..max_length]
            )
        } else {
            full_context
        }
    }

    /// Update user preferences
    #[allow(dead_code)]
    pub fn update_preferences(
        &mut self,
        session_id: &str,
        preferences: UserPreferences,
    ) -> Result<()> {
        let context = self.get_or_create_context(session_id);
        context.session_metadata.user_preferences = preferences;
        Ok(())
    }

    /// Set current task context
    #[allow(dead_code)]
    pub fn set_current_task(&mut self, session_id: &str, task: Option<String>) -> Result<()> {
        let context = self.get_or_create_context(session_id);
        context.session_metadata.current_task = task;
        Ok(())
    }

    /// Clear context for a session
    #[allow(dead_code)]
    pub fn clear_session(&mut self, session_id: &str) {
        self.contexts.remove(session_id);
    }

    /// Get session statistics
    #[allow(dead_code)]
    pub fn get_session_stats(&self, session_id: &str) -> Option<SessionStats> {
        self.contexts.get(session_id).map(|context| SessionStats {
            active_documents_count: context.active_documents.len(),
            conversation_turns: context.conversation_history.len(),
            average_document_relevance: context
                .active_documents
                .iter()
                .map(|doc| doc.relevance_score)
                .fold(0.0, |acc, score| acc + score)
                / context.active_documents.len().max(1) as f32,
        })
    }

    /// Extract document metadata from context
    pub fn extract_document_metadata(&self, session_id: &str) -> Option<DocumentMetadata> {
        if let Some(context) = self.contexts.get(session_id) {
            context
                .active_documents
                .first()
                .map(|primary_doc| DocumentMetadata {
                    title: Some(primary_doc.title.clone()),
                    document_type: None,  // Could be enhanced to detect type
                    sections: vec![],     // Could be enhanced to extract sections
                    key_concepts: vec![], // Could be enhanced with NLP
                })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub active_documents_count: usize,
    pub conversation_turns: usize,
    pub average_document_relevance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_creation() {
        let mut manager = DocumentContextManager::new();
        let context = manager.get_or_create_context("test_session");

        assert_eq!(context.session_metadata.session_id, "test_session");
        assert!(context.active_documents.is_empty());
        assert!(context.conversation_history.is_empty());
    }

    #[test]
    fn test_adding_documents() {
        let mut manager = DocumentContextManager::new();

        let docs = vec![DocumentRef {
            id: "doc1".to_string(),
            title: "Test Document".to_string(),
            path: None,
            content_preview: "This is a test document".to_string(),
            relevance_score: 0.8,
        }];

        manager
            .add_relevant_documents("test_session", docs)
            .unwrap();

        let context = manager.get_or_create_context("test_session");
        assert_eq!(context.active_documents.len(), 1);
        assert_eq!(context.active_documents[0].title, "Test Document");
    }

    #[test]
    fn test_conversation_history() {
        let mut manager = DocumentContextManager::new();

        manager
            .add_conversation_turn("test_session", "user", "Hello")
            .unwrap();
        manager
            .add_conversation_turn("test_session", "assistant", "Hi there!")
            .unwrap();

        let context = manager.get_or_create_context("test_session");
        assert_eq!(context.conversation_history.len(), 2);
        assert_eq!(context.conversation_history[0].role, "user");
        assert_eq!(context.conversation_history[1].role, "assistant");
    }

    #[test]
    fn test_context_building() {
        let mut manager = DocumentContextManager::new();

        let docs = vec![DocumentRef {
            id: "doc1".to_string(),
            title: "API Guide".to_string(),
            path: None,
            content_preview: "This guide covers API usage and authentication".to_string(),
            relevance_score: 0.9,
        }];

        manager
            .add_relevant_documents("test_session", docs)
            .unwrap();
        manager
            .add_conversation_turn("test_session", "user", "How do I authenticate?")
            .unwrap();

        let context_string =
            manager.get_relevant_context("test_session", "What's the API key format?");

        assert!(context_string.contains("API Guide"));
        assert!(context_string.contains("authenticate"));
        assert!(context_string.contains("API key format"));
    }
}
