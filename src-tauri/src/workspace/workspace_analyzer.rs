// src-tauri/src/workspace/workspace_analyzer.rs
//! Comprehensive workspace analyzer for content gap analysis, redundancy detection, and organization assessment
//!
//! This module provides advanced workspace analysis capabilities to identify issues,
//! improvement opportunities, and generate AI-powered insights and recommendations.

use super::intelligence::{WorkspaceAnalysis, WorkspaceIntelligence, WorkspaceRecommendation};
use crate::ai::AIOrchestrator;
use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::RelationshipAnalyzer;
use crate::document::{ContentClassifier, StyleAnalyzer};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Comprehensive workspace analyzer for detecting issues and opportunities
#[allow(dead_code)]
pub struct WorkspaceAnalyzer {
    workspace_intelligence: WorkspaceIntelligence,
    document_indexer: Arc<Mutex<DocumentIndexer>>,
    relationship_analyzer: Arc<Mutex<RelationshipAnalyzer>>,
    #[allow(dead_code)]
    style_analyzer: StyleAnalyzer,
    #[allow(dead_code)]
    content_classifier: ContentClassifier,
    #[allow(dead_code)]
    ai_orchestrator: Option<Arc<Mutex<AIOrchestrator>>>,
}

/// Workspace analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAnalysisConfig {
    /// Include content gap analysis
    pub analyze_content_gaps: bool,
    /// Include redundancy detection
    pub detect_redundancy: bool,
    /// Include organization assessment
    pub assess_organization: bool,
    /// Include AI-powered insights
    pub generate_ai_insights: bool,
    /// Minimum confidence threshold for recommendations
    pub min_confidence: f32,
    /// Maximum analysis depth (number of subdirectories to analyze)
    pub max_depth: usize,
}

impl Default for WorkspaceAnalysisConfig {
    fn default() -> Self {
        Self {
            analyze_content_gaps: true,
            detect_redundancy: true,
            assess_organization: true,
            generate_ai_insights: true,
            min_confidence: 0.7,
            max_depth: 10,
        }
    }
}

/// Content gap analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGapAnalysis {
    pub identified_gaps: Vec<ContentGap>,
    pub missing_categories: Vec<String>,
    pub incomplete_workflows: Vec<IncompleteWorkflow>,
    pub suggested_content: Vec<ContentSuggestion>,
    pub gap_severity_score: f32,
}

/// Individual content gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGap {
    pub gap_type: ContentGapType,
    pub description: String,
    pub severity: GapSeverity,
    pub affected_areas: Vec<String>,
    pub suggested_actions: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentGapType {
    MissingDocumentation,
    IncompleteProcess,
    OutdatedInformation,
    MissingTutorial,
    MissingReference,
    MissingFAQ,
    MissingGlossary,
    MissingBestPractices,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GapSeverity {
    Critical, // Blocks workflows
    High,     // Significantly impacts productivity
    Medium,   // Moderate impact
    Low,      // Minor improvement opportunity
}

/// Incomplete workflow identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncompleteWorkflow {
    pub workflow_name: String,
    pub missing_steps: Vec<String>,
    pub partial_coverage: f32,
    pub related_documents: Vec<String>,
}

/// Content suggestion for filling gaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSuggestion {
    pub title: String,
    pub content_type: String,
    pub priority: SuggestionPriority,
    pub estimated_effort: EstimatedEffort,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Immediate,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EstimatedEffort {
    Minimal, // < 1 hour
    Low,     // 1-4 hours
    Medium,  // 4-16 hours
    High,    // 16+ hours
}

/// Redundancy detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedundancyAnalysis {
    pub duplicate_content: Vec<ContentDuplication>,
    pub similar_documents: Vec<SimilarDocumentGroup>,
    pub overlapping_topics: Vec<TopicOverlap>,
    pub consolidation_opportunities: Vec<ConsolidationOpportunity>,
    pub redundancy_score: f32,
}

/// Content duplication detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDuplication {
    pub documents: Vec<String>,
    pub similarity_score: f32,
    pub duplicated_sections: Vec<String>,
    pub recommendation: DuplicationRecommendation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DuplicationRecommendation {
    Merge,
    Consolidate,
    KeepSeparate,
    Archive,
}

/// Group of similar documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarDocumentGroup {
    pub documents: Vec<String>,
    pub common_topics: Vec<String>,
    pub similarity_score: f32,
    pub suggested_action: GroupAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GroupAction {
    CreateIndex,
    MergeContent,
    CreateSeries,
    TagSimilarly,
}

/// Topic overlap detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicOverlap {
    pub topic: String,
    pub documents: Vec<String>,
    pub overlap_percentage: f32,
    pub suggested_resolution: OverlapResolution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OverlapResolution {
    CreateMasterDocument,
    CrossReference,
    Specialize,
    Deduplicate,
}

/// Consolidation opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationOpportunity {
    pub target_documents: Vec<String>,
    pub consolidation_type: ConsolidationType,
    pub benefits: Vec<String>,
    pub effort_required: EstimatedEffort,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsolidationType {
    MergeIntoSingle,
    CreateHierarchy,
    ExtractCommon,
    CreateReference,
}

/// Organization assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationAssessment {
    pub structure_analysis: StructureAnalysis,
    pub naming_consistency: NamingConsistency,
    pub categorization_quality: CategorizationQuality,
    pub accessibility_assessment: AccessibilityAssessment,
    pub organization_score: f32,
}

/// File and folder structure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureAnalysis {
    pub depth_distribution: HashMap<usize, usize>,
    pub directory_utilization: Vec<DirectoryUtilization>,
    pub structure_issues: Vec<StructureIssue>,
    pub suggested_improvements: Vec<StructureImprovement>,
}

/// Directory utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryUtilization {
    pub path: String,
    pub file_count: usize,
    pub subdirectory_count: usize,
    pub utilization_score: f32,
    pub recommendations: Vec<String>,
}

/// Structure-related issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureIssue {
    pub issue_type: StructureIssueType,
    pub location: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureIssueType {
    TooDeepNesting,
    TooManyFilesInDirectory,
    EmptyDirectories,
    PoorNaming,
    MissingIndexFiles,
    InconsistentStructure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Structure improvement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureImprovement {
    pub improvement_type: ImprovementType,
    pub description: String,
    pub affected_paths: Vec<String>,
    pub expected_benefit: String,
    pub implementation_effort: EstimatedEffort,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImprovementType {
    Reorganize,
    CreateSubdirectories,
    FlattenStructure,
    AddIndexFiles,
    RenameForConsistency,
    GroupByTopic,
}

/// Naming consistency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConsistency {
    pub consistency_score: f32,
    pub naming_patterns: Vec<NamingPattern>,
    pub inconsistencies: Vec<NamingInconsistency>,
    pub suggested_conventions: Vec<NamingConvention>,
}

/// Identified naming pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingPattern {
    pub pattern: String,
    pub usage_count: usize,
    pub examples: Vec<String>,
    pub confidence: f32,
}

/// Naming inconsistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingInconsistency {
    pub files: Vec<String>,
    pub inconsistency_type: InconsistencyType,
    pub suggested_rename: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InconsistencyType {
    CaseInconsistency,
    SeparatorInconsistency,
    NamingConventionMismatch,
    AbbreviationInconsistency,
}

/// Suggested naming convention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConvention {
    pub convention_name: String,
    pub pattern: String,
    pub description: String,
    pub examples: Vec<String>,
    pub adoption_benefit: String,
}

/// Categorization quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationQuality {
    pub quality_score: f32,
    pub category_distribution: HashMap<String, usize>,
    pub miscategorized_content: Vec<MiscategorizedContent>,
    pub missing_categories: Vec<String>,
    pub suggested_recategorization: Vec<RecategorizationSuggestion>,
}

/// Miscategorized content detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscategorizedContent {
    pub document_path: String,
    pub current_category: String,
    pub suggested_category: String,
    pub confidence: f32,
    pub reasoning: String,
}

/// Recategorization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecategorizationSuggestion {
    pub action_type: RecategorizationAction,
    pub affected_documents: Vec<String>,
    pub new_structure: String,
    pub benefits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecategorizationAction {
    CreateNewCategory,
    MergeCategories,
    SplitCategory,
    RenameCategory,
    MoveDocuments,
}

/// Accessibility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityAssessment {
    pub accessibility_score: f32,
    pub search_effectiveness: SearchEffectiveness,
    pub navigation_ease: NavigationEase,
    pub content_discoverability: ContentDiscoverability,
    pub access_barriers: Vec<AccessBarrier>,
}

/// Search effectiveness analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEffectiveness {
    pub findability_score: f32,
    pub search_gaps: Vec<String>,
    pub indexing_coverage: f32,
    pub suggested_improvements: Vec<String>,
}

/// Navigation ease assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationEase {
    pub navigation_score: f32,
    pub path_complexity: f32,
    pub link_coverage: f32,
    pub suggested_shortcuts: Vec<String>,
}

/// Content discoverability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDiscoverability {
    pub discoverability_score: f32,
    pub orphaned_content: Vec<String>,
    pub poorly_linked_content: Vec<String>,
    pub suggested_connections: Vec<ContentConnection>,
}

/// Content connection suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConnection {
    pub source_document: String,
    pub target_document: String,
    pub connection_type: ConnectionType,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionType {
    Related,
    Prerequisite,
    FollowUp,
    Example,
    Reference,
}

/// Access barrier identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessBarrier {
    pub barrier_type: BarrierType,
    pub affected_content: Vec<String>,
    pub severity: IssueSeverity,
    pub resolution: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BarrierType {
    DeepNesting,
    PoorNaming,
    MissingIndex,
    BrokenLinks,
    UntaggedContent,
    ComplexPath,
}

/// Comprehensive workspace analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveWorkspaceAnalysis {
    pub workspace_path: PathBuf,
    pub analysis_timestamp: DateTime<Utc>,
    pub config_used: WorkspaceAnalysisConfig,

    // Core analysis components
    pub base_analysis: WorkspaceAnalysis,
    pub content_gaps: Option<ContentGapAnalysis>,
    pub redundancy_analysis: Option<RedundancyAnalysis>,
    pub organization_assessment: Option<OrganizationAssessment>,

    // Overall metrics
    pub overall_health_score: f32,
    pub priority_recommendations: Vec<WorkspaceRecommendation>,
    pub ai_insights: Vec<AIInsight>,

    // Analysis metadata
    pub analysis_duration_ms: u64,
    pub documents_analyzed: usize,
    pub directories_analyzed: usize,
}

/// AI-generated insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub confidence: f32,
    pub actionable_steps: Vec<String>,
    pub expected_impact: ImpactLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsightType {
    Opportunity,
    Risk,
    Trend,
    Anomaly,
    Recommendation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

#[allow(dead_code)]
impl WorkspaceAnalyzer {
    /// Create a new workspace analyzer
    pub fn new(
        workspace_intelligence: WorkspaceIntelligence,
        document_indexer: Arc<Mutex<DocumentIndexer>>,
        relationship_analyzer: Arc<Mutex<RelationshipAnalyzer>>,
        ai_orchestrator: Option<Arc<Mutex<AIOrchestrator>>>,
    ) -> Result<Self> {
        Ok(Self {
            workspace_intelligence,
            document_indexer,
            relationship_analyzer,
            style_analyzer: StyleAnalyzer::new(),
            content_classifier: ContentClassifier::new(None)
                .map_err(|e| anyhow::anyhow!("Failed to create content classifier: {}", e))?,
            ai_orchestrator,
        })
    }

    /// Perform comprehensive workspace analysis
    pub async fn analyze_workspace(
        &self,
        workspace_path: &Path,
        config: WorkspaceAnalysisConfig,
    ) -> Result<ComprehensiveWorkspaceAnalysis> {
        let start_time = std::time::Instant::now();

        // Create workspace info for analysis
        let workspace_info = super::WorkspaceInfo {
            path: workspace_path.to_path_buf(),
            name: workspace_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("workspace")
                .to_string(),
            version: "1.0.0".to_string(),
            created: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            import_settings: super::ImportSettings::default(),
            ai_settings: super::WorkspaceAISettings::default(),
            is_favorite: false,
            access_count: 0,
        };

        // Get base workspace analysis
        let base_analysis = self
            .workspace_intelligence
            .analyze_workspace(&workspace_info)
            .await
            .context("Failed to perform base workspace analysis")?;

        let mut comprehensive_analysis = ComprehensiveWorkspaceAnalysis {
            workspace_path: workspace_path.to_path_buf(),
            analysis_timestamp: Utc::now(),
            config_used: config.clone(),
            base_analysis,
            content_gaps: None,
            redundancy_analysis: None,
            organization_assessment: None,
            overall_health_score: 0.0,
            priority_recommendations: vec![],
            ai_insights: vec![],
            analysis_duration_ms: 0,
            documents_analyzed: 0,
            directories_analyzed: 0,
        };

        // Perform content gap analysis
        if config.analyze_content_gaps {
            comprehensive_analysis.content_gaps =
                Some(self.analyze_content_gaps(workspace_path, &config).await?);
        }

        // Perform redundancy detection
        if config.detect_redundancy {
            comprehensive_analysis.redundancy_analysis =
                Some(self.detect_redundancy(workspace_path, &config).await?);
        }

        // Perform organization assessment
        if config.assess_organization {
            comprehensive_analysis.organization_assessment =
                Some(self.assess_organization(workspace_path, &config).await?);
        }

        // Generate AI insights
        if config.generate_ai_insights {
            comprehensive_analysis.ai_insights =
                self.generate_ai_insights(&comprehensive_analysis).await?;
        }

        // Calculate overall health score
        comprehensive_analysis.overall_health_score =
            self.calculate_health_score(&comprehensive_analysis);

        // Generate priority recommendations
        comprehensive_analysis.priority_recommendations =
            self.generate_priority_recommendations(&comprehensive_analysis);

        // Set analysis metadata
        comprehensive_analysis.analysis_duration_ms = start_time.elapsed().as_millis() as u64;
        comprehensive_analysis.documents_analyzed = self.count_documents(workspace_path).await?;
        comprehensive_analysis.directories_analyzed =
            self.count_directories(workspace_path).await?;

        Ok(comprehensive_analysis)
    }

    /// Analyze content gaps in the workspace
    async fn analyze_content_gaps(
        &self,
        _workspace_path: &Path,
        _config: &WorkspaceAnalysisConfig,
    ) -> Result<ContentGapAnalysis> {
        let indexer = self.document_indexer.lock().await;
        let entries = indexer.get_all_documents();

        let mut gaps = Vec::new();
        let mut missing_categories = Vec::new();
        let mut incomplete_workflows = Vec::new();
        let mut suggested_content = Vec::new();

        // Analyze document types and identify missing categories
        let mut document_types = HashSet::new();
        for entry in &entries {
            if let Some(doc_type) = &entry.metadata.content.detected_mime_type {
                document_types.insert(doc_type.clone());
            }
        }

        // Check for common missing document types
        let expected_types = [
            "documentation",
            "tutorial",
            "reference",
            "faq",
            "glossary",
            "best_practices",
            "troubleshooting",
            "getting_started",
        ];

        for expected_type in &expected_types {
            if !document_types.contains(*expected_type) {
                missing_categories.push(expected_type.to_string());

                gaps.push(ContentGap {
                    gap_type: match *expected_type {
                        "documentation" => ContentGapType::MissingDocumentation,
                        "tutorial" => ContentGapType::MissingTutorial,
                        "reference" => ContentGapType::MissingReference,
                        "faq" => ContentGapType::MissingFAQ,
                        "glossary" => ContentGapType::MissingGlossary,
                        "best_practices" => ContentGapType::MissingBestPractices,
                        _ => ContentGapType::MissingDocumentation,
                    },
                    description: format!("Missing {} content in workspace", expected_type),
                    severity: GapSeverity::Medium,
                    affected_areas: vec!["General workspace".to_string()],
                    suggested_actions: vec![
                        format!("Create {} documentation", expected_type),
                        "Establish content creation process".to_string(),
                    ],
                    confidence: 0.8,
                });

                suggested_content.push(ContentSuggestion {
                    title: format!("{} Documentation", expected_type),
                    content_type: expected_type.to_string(),
                    priority: SuggestionPriority::Medium,
                    estimated_effort: EstimatedEffort::Medium,
                    rationale: format!("No {} content found in workspace", expected_type),
                });
            }
        }

        // Analyze for incomplete workflows
        // This is a simplified analysis - could be enhanced with AI
        if entries.len() > 5 {
            let workflow_keywords = ["setup", "installation", "configuration", "deployment"];
            let mut workflow_coverage = HashMap::new();

            for keyword in &workflow_keywords {
                let count = entries
                    .iter()
                    .filter(|entry| entry.content.to_lowercase().contains(keyword))
                    .count();
                workflow_coverage.insert(*keyword, count);
            }

            for (workflow, count) in workflow_coverage {
                if count == 0 {
                    incomplete_workflows.push(IncompleteWorkflow {
                        workflow_name: workflow.to_string(),
                        missing_steps: vec![format!("No {} documentation found", workflow)],
                        partial_coverage: 0.0,
                        related_documents: vec![],
                    });
                }
            }
        }

        let gap_severity_score = if gaps.is_empty() {
            0.0
        } else {
            gaps.iter()
                .map(|g| match g.severity {
                    GapSeverity::Critical => 1.0,
                    GapSeverity::High => 0.75,
                    GapSeverity::Medium => 0.5,
                    GapSeverity::Low => 0.25,
                })
                .sum::<f32>()
                / gaps.len() as f32
        };

        Ok(ContentGapAnalysis {
            identified_gaps: gaps,
            missing_categories,
            incomplete_workflows,
            suggested_content,
            gap_severity_score,
        })
    }

    /// Detect redundancy in workspace content
    async fn detect_redundancy(
        &self,
        _workspace_path: &Path,
        _config: &WorkspaceAnalysisConfig,
    ) -> Result<RedundancyAnalysis> {
        let indexer = self.document_indexer.lock().await;
        let entries = indexer.get_all_documents();

        let relationship_analyzer = self.relationship_analyzer.lock().await;

        let mut duplicate_content = Vec::new();
        let mut similar_documents = Vec::new();
        let mut overlapping_topics = Vec::new();
        let mut consolidation_opportunities = Vec::new();

        // Find similar documents using relationship analyzer
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                if let Ok(relationships) = relationship_analyzer
                    .analyze_document_pair(entries[i], entries[j])
                    .await
                {
                    for relationship in relationships {
                        let similarity_score = relationship.score;
                        if similarity_score > 0.8 {
                            // High similarity - potential duplicate
                            duplicate_content.push(ContentDuplication {
                                documents: vec![
                                    entries[i].path.to_string_lossy().to_string(),
                                    entries[j].path.to_string_lossy().to_string(),
                                ],
                                similarity_score,
                                duplicated_sections: vec!["Content analysis needed".to_string()],
                                recommendation: if similarity_score > 0.95 {
                                    DuplicationRecommendation::Merge
                                } else {
                                    DuplicationRecommendation::Consolidate
                                },
                            });
                        } else if similarity_score > 0.6 {
                            // Medium similarity - group similar documents
                            similar_documents.push(SimilarDocumentGroup {
                                documents: vec![
                                    entries[i].path.to_string_lossy().to_string(),
                                    entries[j].path.to_string_lossy().to_string(),
                                ],
                                common_topics: relationship
                                    .evidence
                                    .iter()
                                    .map(|e| e.description.clone())
                                    .collect(),
                                similarity_score,
                                suggested_action: GroupAction::CreateIndex,
                            });
                        }
                    }
                }
            }
        }

        // Analyze topic overlaps
        let mut topic_counts: HashMap<String, Vec<String>> = HashMap::new();
        for entry in &entries {
            // Simple keyword extraction for topics
            let words: Vec<&str> = entry
                .content
                .split_whitespace()
                .filter(|w| w.len() > 5)
                .take(10)
                .collect();

            for word in words {
                topic_counts
                    .entry(word.to_lowercase())
                    .or_default()
                    .push(entry.path.to_string_lossy().to_string());
            }
        }

        for (topic, documents) in topic_counts {
            if documents.len() > 2 {
                let overlap_percentage = (documents.len() as f32 / entries.len() as f32) * 100.0;
                overlapping_topics.push(TopicOverlap {
                    topic,
                    documents: documents.clone(),
                    overlap_percentage,
                    suggested_resolution: if overlap_percentage > 50.0 {
                        OverlapResolution::CreateMasterDocument
                    } else {
                        OverlapResolution::CrossReference
                    },
                });

                if documents.len() > 3 {
                    consolidation_opportunities.push(ConsolidationOpportunity {
                        target_documents: documents,
                        consolidation_type: ConsolidationType::CreateHierarchy,
                        benefits: vec![
                            "Reduced redundancy".to_string(),
                            "Improved maintainability".to_string(),
                        ],
                        effort_required: EstimatedEffort::Medium,
                        confidence: 0.7,
                    });
                }
            }
        }

        drop(relationship_analyzer);

        let redundancy_score = if !entries.is_empty() {
            (duplicate_content.len() + similar_documents.len()) as f32 / entries.len() as f32
        } else {
            0.0
        };

        Ok(RedundancyAnalysis {
            duplicate_content,
            similar_documents,
            overlapping_topics,
            consolidation_opportunities,
            redundancy_score,
        })
    }

    /// Assess workspace organization
    async fn assess_organization(
        &self,
        workspace_path: &Path,
        _config: &WorkspaceAnalysisConfig,
    ) -> Result<OrganizationAssessment> {
        let structure_analysis = self.analyze_structure(workspace_path).await?;
        let naming_consistency = self.analyze_naming_consistency(workspace_path).await?;
        let categorization_quality = self.analyze_categorization_quality().await?;
        let accessibility_assessment = self.assess_accessibility(workspace_path).await?;

        let organization_score = (structure_analysis
            .directory_utilization
            .iter()
            .map(|d| d.utilization_score)
            .sum::<f32>()
            / structure_analysis.directory_utilization.len().max(1) as f32
            + naming_consistency.consistency_score
            + categorization_quality.quality_score
            + accessibility_assessment.accessibility_score)
            / 4.0;

        Ok(OrganizationAssessment {
            structure_analysis,
            naming_consistency,
            categorization_quality,
            accessibility_assessment,
            organization_score,
        })
    }

    /// Analyze directory structure
    async fn analyze_structure(&self, workspace_path: &Path) -> Result<StructureAnalysis> {
        let mut depth_distribution = HashMap::new();
        let mut directory_utilization = Vec::new();
        let mut structure_issues = Vec::new();
        let mut suggested_improvements = Vec::new();

        // Walk directory structure
        if let Ok(entries) = std::fs::read_dir(workspace_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let depth = self.calculate_directory_depth(&path, workspace_path);
                    *depth_distribution.entry(depth).or_insert(0) += 1;

                    // Analyze directory utilization
                    let utilization = self.analyze_directory_utilization(&path).await?;
                    directory_utilization.push(utilization);

                    // Check for structure issues
                    if depth > 5 {
                        structure_issues.push(StructureIssue {
                            issue_type: StructureIssueType::TooDeepNesting,
                            location: path.to_string_lossy().to_string(),
                            description: "Directory nesting is too deep".to_string(),
                            severity: IssueSeverity::Medium,
                            suggested_fix: "Consider flattening directory structure".to_string(),
                        });
                    }
                }
            }
        }

        // Generate improvement suggestions
        if depth_distribution.keys().max().unwrap_or(&0) > &4 {
            suggested_improvements.push(StructureImprovement {
                improvement_type: ImprovementType::FlattenStructure,
                description: "Reduce directory nesting depth".to_string(),
                affected_paths: vec![workspace_path.to_string_lossy().to_string()],
                expected_benefit: "Improved navigation and file discovery".to_string(),
                implementation_effort: EstimatedEffort::Medium,
            });
        }

        Ok(StructureAnalysis {
            depth_distribution,
            directory_utilization,
            structure_issues,
            suggested_improvements,
        })
    }

    /// Calculate directory depth relative to workspace root
    fn calculate_directory_depth(&self, dir_path: &Path, workspace_path: &Path) -> usize {
        dir_path
            .strip_prefix(workspace_path)
            .map(|relative| relative.components().count())
            .unwrap_or(0)
    }

    /// Analyze utilization of a specific directory
    async fn analyze_directory_utilization(&self, dir_path: &Path) -> Result<DirectoryUtilization> {
        let mut file_count = 0;
        let mut subdirectory_count = 0;

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    file_count += 1;
                } else if entry.path().is_dir() {
                    subdirectory_count += 1;
                }
            }
        }

        // Calculate utilization score (balance between too empty and too full)
        let total_items = file_count + subdirectory_count;
        let utilization_score = if total_items == 0 {
            0.0
        } else if total_items < 3 {
            0.3 // Too empty
        } else if total_items > 20 {
            0.4 // Too crowded
        } else {
            1.0 // Good balance
        };

        let mut recommendations = Vec::new();
        if total_items == 0 {
            recommendations.push("Consider removing empty directory".to_string());
        } else if total_items > 20 {
            recommendations.push("Consider creating subdirectories to organize files".to_string());
        }

        Ok(DirectoryUtilization {
            path: dir_path.to_string_lossy().to_string(),
            file_count,
            subdirectory_count,
            utilization_score,
            recommendations,
        })
    }

    /// Analyze naming consistency across workspace
    async fn analyze_naming_consistency(&self, workspace_path: &Path) -> Result<NamingConsistency> {
        let mut file_names = Vec::new();

        // Collect all file names
        self.collect_file_names(workspace_path, &mut file_names)?;

        let mut naming_patterns = Vec::new();
        let inconsistencies = Vec::new();
        let mut pattern_counts: HashMap<&str, usize> = HashMap::new();

        // Analyze naming patterns
        for name in &file_names {
            // Check for different separators
            if name.contains('_') {
                *pattern_counts.entry("underscore_case").or_insert(0) += 1;
            }
            if name.contains('-') {
                *pattern_counts.entry("kebab-case").or_insert(0) += 1;
            }
            if name.chars().any(|c| c.is_uppercase()) {
                *pattern_counts.entry("CamelCase").or_insert(0) += 1;
            }
        }

        // Create naming patterns from analysis
        for (pattern, count) in &pattern_counts {
            naming_patterns.push(NamingPattern {
                pattern: pattern.to_string(),
                usage_count: *count,
                examples: file_names
                    .iter()
                    .filter(|name| self.matches_pattern(name, pattern))
                    .take(3)
                    .cloned()
                    .collect(),
                confidence: *count as f32 / file_names.len() as f32,
            });
        }

        // Calculate consistency score
        let dominant_pattern_count = pattern_counts.values().max().unwrap_or(&0);
        let consistency_score = if file_names.is_empty() {
            1.0
        } else {
            *dominant_pattern_count as f32 / file_names.len() as f32
        };

        // Generate suggested conventions
        let suggested_conventions = vec![NamingConvention {
            convention_name: "Consistent Case Style".to_string(),
            pattern: "lowercase_with_underscores".to_string(),
            description: "Use lowercase letters with underscores as separators".to_string(),
            examples: vec!["user_guide.md".to_string(), "api_reference.pdf".to_string()],
            adoption_benefit: "Improved consistency and readability".to_string(),
        }];

        Ok(NamingConsistency {
            consistency_score,
            naming_patterns,
            inconsistencies,
            suggested_conventions,
        })
    }

    /// Check if filename matches naming pattern
    fn matches_pattern(&self, filename: &str, pattern: &str) -> bool {
        match pattern {
            "underscore_case" => filename.contains('_'),
            "kebab-case" => filename.contains('-'),
            "CamelCase" => filename.chars().any(|c| c.is_uppercase()),
            _ => false,
        }
    }

    /// Recursively collect file names
    #[allow(clippy::only_used_in_recursion)]
    fn collect_file_names(&self, dir_path: &Path, file_names: &mut Vec<String>) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        file_names.push(name.to_string_lossy().to_string());
                    }
                } else if path.is_dir() {
                    self.collect_file_names(&path, file_names)?;
                }
            }
        }
        Ok(())
    }

    /// Analyze categorization quality
    async fn analyze_categorization_quality(&self) -> Result<CategorizationQuality> {
        // This would integrate with the content classifier
        // For now, providing a basic implementation

        Ok(CategorizationQuality {
            quality_score: 0.7,
            category_distribution: HashMap::new(),
            miscategorized_content: vec![],
            missing_categories: vec![],
            suggested_recategorization: vec![],
        })
    }

    /// Assess workspace accessibility
    async fn assess_accessibility(
        &self,
        _workspace_path: &Path,
    ) -> Result<AccessibilityAssessment> {
        // Basic accessibility assessment
        let search_effectiveness = SearchEffectiveness {
            findability_score: 0.8,
            search_gaps: vec![],
            indexing_coverage: 0.9,
            suggested_improvements: vec!["Improve metadata tagging".to_string()],
        };

        let navigation_ease = NavigationEase {
            navigation_score: 0.7,
            path_complexity: 0.3,
            link_coverage: 0.6,
            suggested_shortcuts: vec!["Create index files".to_string()],
        };

        let content_discoverability = ContentDiscoverability {
            discoverability_score: 0.75,
            orphaned_content: vec![],
            poorly_linked_content: vec![],
            suggested_connections: vec![],
        };

        let accessibility_score = (search_effectiveness.findability_score
            + navigation_ease.navigation_score
            + content_discoverability.discoverability_score)
            / 3.0;

        Ok(AccessibilityAssessment {
            accessibility_score,
            search_effectiveness,
            navigation_ease,
            content_discoverability,
            access_barriers: vec![],
        })
    }

    /// Generate AI-powered insights
    async fn generate_ai_insights(
        &self,
        analysis: &ComprehensiveWorkspaceAnalysis,
    ) -> Result<Vec<AIInsight>> {
        let mut insights = Vec::new();

        // Generate insights based on analysis results
        if let Some(content_gaps) = &analysis.content_gaps {
            if content_gaps.gap_severity_score > 0.5 {
                insights.push(AIInsight {
                    insight_type: InsightType::Opportunity,
                    title: "Significant Content Gaps Detected".to_string(),
                    description:
                        "Multiple content gaps were identified that could impact productivity"
                            .to_string(),
                    confidence: content_gaps.gap_severity_score,
                    actionable_steps: vec![
                        "Prioritize missing documentation creation".to_string(),
                        "Establish content creation workflows".to_string(),
                    ],
                    expected_impact: ImpactLevel::High,
                });
            }
        }

        if let Some(redundancy) = &analysis.redundancy_analysis {
            if redundancy.redundancy_score > 0.3 {
                insights.push(AIInsight {
                    insight_type: InsightType::Risk,
                    title: "High Content Redundancy".to_string(),
                    description: "Duplicate and overlapping content may cause confusion"
                        .to_string(),
                    confidence: redundancy.redundancy_score,
                    actionable_steps: vec![
                        "Consolidate duplicate content".to_string(),
                        "Establish content governance".to_string(),
                    ],
                    expected_impact: ImpactLevel::Medium,
                });
            }
        }

        if analysis.overall_health_score < 0.6 {
            insights.push(AIInsight {
                insight_type: InsightType::Recommendation,
                title: "Workspace Organization Needs Improvement".to_string(),
                description: "Overall workspace health score indicates room for improvement"
                    .to_string(),
                confidence: 1.0 - analysis.overall_health_score,
                actionable_steps: vec![
                    "Review and implement priority recommendations".to_string(),
                    "Establish workspace maintenance routine".to_string(),
                ],
                expected_impact: ImpactLevel::High,
            });
        }

        Ok(insights)
    }

    /// Calculate overall workspace health score
    fn calculate_health_score(&self, analysis: &ComprehensiveWorkspaceAnalysis) -> f32 {
        let mut scores = Vec::new();

        // Base score from workspace analysis
        scores.push(analysis.base_analysis.quality_assessment.completeness_score);

        // Content gap score (inverted - fewer gaps = higher score)
        if let Some(content_gaps) = &analysis.content_gaps {
            scores.push(1.0 - content_gaps.gap_severity_score as f64);
        }

        // Redundancy score (inverted - less redundancy = higher score)
        if let Some(redundancy) = &analysis.redundancy_analysis {
            scores.push(1.0 - redundancy.redundancy_score.min(1.0) as f64);
        }

        // Organization score
        if let Some(organization) = &analysis.organization_assessment {
            scores.push(organization.organization_score as f64);
        }

        if scores.is_empty() {
            0.5 // Default score if no analysis available
        } else {
            (scores.iter().sum::<f64>() / scores.len() as f64) as f32
        }
    }

    /// Generate priority recommendations
    fn generate_priority_recommendations(
        &self,
        analysis: &ComprehensiveWorkspaceAnalysis,
    ) -> Vec<WorkspaceRecommendation> {
        let mut recommendations = Vec::new();

        // Add recommendations based on analysis
        if let Some(content_gaps) = &analysis.content_gaps {
            for gap in &content_gaps.identified_gaps {
                if matches!(gap.severity, GapSeverity::Critical | GapSeverity::High) {
                    recommendations.push(WorkspaceRecommendation {
                        recommendation_type:
                            super::intelligence::RecommendationType::ContentCreation,
                        title: format!("Address {}", gap.description),
                        description: gap.suggested_actions.join("; "),
                        priority: match gap.severity {
                            GapSeverity::Critical => {
                                super::intelligence::RecommendationPriority::High
                            }
                            GapSeverity::High => super::intelligence::RecommendationPriority::High,
                            _ => super::intelligence::RecommendationPriority::Medium,
                        },
                        estimated_effort: super::intelligence::EffortLevel::Medium,
                        expected_impact: super::intelligence::ImpactLevel::High,
                        actionable_steps: gap.suggested_actions.clone(),
                        affected_files: None,
                    });
                }
            }
        }

        if let Some(redundancy) = &analysis.redundancy_analysis {
            for opportunity in &redundancy.consolidation_opportunities {
                if opportunity.confidence > 0.7 {
                    recommendations.push(WorkspaceRecommendation {
                        recommendation_type: super::intelligence::RecommendationType::Cleanup,
                        title: "Consolidate Redundant Content".to_string(),
                        description: format!(
                            "Consolidate {} documents",
                            opportunity.target_documents.len()
                        ),
                        priority: super::intelligence::RecommendationPriority::Medium,
                        estimated_effort: match opportunity.effort_required {
                            EstimatedEffort::High => super::intelligence::EffortLevel::High,
                            EstimatedEffort::Medium => super::intelligence::EffortLevel::Medium,
                            _ => super::intelligence::EffortLevel::Low,
                        },
                        expected_impact: super::intelligence::ImpactLevel::Medium,
                        actionable_steps: vec![
                            "Review documents".to_string(),
                            "Merge content".to_string(),
                        ],
                        affected_files: Some(
                            opportunity
                                .target_documents
                                .iter()
                                .map(PathBuf::from)
                                .collect(),
                        ),
                    });
                }
            }
        }

        // Sort by priority and return top recommendations
        recommendations.sort_by(|a, b| {
            use super::intelligence::RecommendationPriority;
            let priority_order = |p: &RecommendationPriority| match p {
                RecommendationPriority::Urgent => 4,
                RecommendationPriority::High => 3,
                RecommendationPriority::Medium => 2,
                RecommendationPriority::Low => 1,
            };
            priority_order(&b.priority).cmp(&priority_order(&a.priority))
        });
        recommendations.into_iter().take(10).collect()
    }

    /// Count documents in workspace
    async fn count_documents(&self, _workspace_path: &Path) -> Result<usize> {
        let indexer = self.document_indexer.lock().await;
        Ok(indexer.get_all_documents().len())
    }

    /// Count directories in workspace
    async fn count_directories(&self, workspace_path: &Path) -> Result<usize> {
        let mut count = 0;
        self.count_directories_recursive(workspace_path, &mut count)?;
        Ok(count)
    }

    /// Recursively count directories
    #[allow(clippy::only_used_in_recursion)]
    fn count_directories_recursive(&self, dir_path: &Path, count: &mut usize) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    *count += 1;
                    self.count_directories_recursive(&path, count)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_workspace_analyzer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = Arc::new(Mutex::new(
            DocumentIndexer::new(temp_dir.path().to_path_buf()).unwrap(),
        ));
        let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
            crate::document::relationship_analyzer::RelationshipConfig::default(),
        )));
        let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

        let _analyzer =
            WorkspaceAnalyzer::new(workspace_intelligence, indexer, relationship_analyzer, None)
                .unwrap();

        // Test that analyzer was created successfully - just check it's not panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_workspace_analysis_config_default() {
        let config = WorkspaceAnalysisConfig::default();

        assert!(config.analyze_content_gaps);
        assert!(config.detect_redundancy);
        assert!(config.assess_organization);
        assert!(config.generate_ai_insights);
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.max_depth, 10);
    }

    #[tokio::test]
    async fn test_directory_depth_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // Create nested directory structure
        let nested_dir = workspace_path.join("level1").join("level2").join("level3");
        std::fs::create_dir_all(&nested_dir).unwrap();

        let indexer = Arc::new(Mutex::new(
            DocumentIndexer::new(workspace_path.to_path_buf()).unwrap(),
        ));
        let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
            crate::document::relationship_analyzer::RelationshipConfig::default(),
        )));
        let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

        let analyzer =
            WorkspaceAnalyzer::new(workspace_intelligence, indexer, relationship_analyzer, None)
                .unwrap();

        let depth = analyzer.calculate_directory_depth(&nested_dir, workspace_path);
        assert_eq!(depth, 3);
    }

    #[tokio::test]
    async fn test_structure_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // Create test structure
        std::fs::create_dir_all(workspace_path.join("docs")).unwrap();
        std::fs::create_dir_all(workspace_path.join("guides")).unwrap();
        std::fs::write(workspace_path.join("docs").join("test.md"), "Test content").unwrap();

        let indexer = Arc::new(Mutex::new(
            DocumentIndexer::new(workspace_path.to_path_buf()).unwrap(),
        ));
        let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
            crate::document::relationship_analyzer::RelationshipConfig::default(),
        )));
        let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

        let analyzer =
            WorkspaceAnalyzer::new(workspace_intelligence, indexer, relationship_analyzer, None)
                .unwrap();

        let structure_analysis = analyzer.analyze_structure(workspace_path).await.unwrap();

        assert!(!structure_analysis.directory_utilization.is_empty());
        assert!(structure_analysis.depth_distribution.contains_key(&1));
    }

    #[test]
    fn test_naming_pattern_matching() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = Arc::new(Mutex::new(
            DocumentIndexer::new(temp_dir.path().to_path_buf()).unwrap(),
        ));
        let relationship_analyzer = Arc::new(Mutex::new(RelationshipAnalyzer::new(
            crate::document::relationship_analyzer::RelationshipConfig::default(),
        )));
        let workspace_intelligence = WorkspaceIntelligence::new(indexer.clone(), None);

        let analyzer =
            WorkspaceAnalyzer::new(workspace_intelligence, indexer, relationship_analyzer, None)
                .unwrap();

        assert!(analyzer.matches_pattern("test_file.md", "underscore_case"));
        assert!(analyzer.matches_pattern("test-file.md", "kebab-case"));
        assert!(analyzer.matches_pattern("TestFile.md", "CamelCase"));
        assert!(!analyzer.matches_pattern("testfile.md", "underscore_case"));
    }
}
