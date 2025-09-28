// src-tauri/src/workspace/smart_organizer.rs
//! Smart document organization system for AI-suggested categorization, tagging, and folder structure
//!
//! This module provides intelligent document organization capabilities that analyze document
//! relationships, content, and metadata to suggest optimal organizational structures.

use super::intelligence::WorkspaceIntelligence;
use crate::ai::AIOrchestrator;
use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::RelationshipAnalyzer;
use crate::document::{ContentClassifier, StructureAnalyzer, StyleAnalyzer};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Smart document organizer for AI-suggested organization improvements
#[allow(dead_code)]
pub struct SmartOrganizer {
    workspace_intelligence: WorkspaceIntelligence,
    document_indexer: Arc<Mutex<DocumentIndexer>>,
    relationship_analyzer: Arc<Mutex<RelationshipAnalyzer>>,
    content_classifier: ContentClassifier,
    style_analyzer: StyleAnalyzer,
    structure_analyzer: StructureAnalyzer,
    ai_orchestrator: Option<Arc<Mutex<AIOrchestrator>>>,
}

/// Configuration for smart organization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationConfig {
    /// Enable AI-suggested categorization
    pub enable_categorization: bool,
    /// Enable automatic tagging
    pub enable_tagging: bool,
    /// Enable folder structure optimization
    pub enable_folder_optimization: bool,
    /// Minimum confidence threshold for suggestions
    pub min_confidence: f32,
    /// Maximum folder depth for organization
    pub max_folder_depth: usize,
    /// Enable semantic clustering
    pub enable_semantic_clustering: bool,
    /// Enable temporal organization (by creation/modification date)
    pub enable_temporal_organization: bool,
}

impl Default for OrganizationConfig {
    fn default() -> Self {
        Self {
            enable_categorization: true,
            enable_tagging: true,
            enable_folder_optimization: true,
            min_confidence: 0.7,
            max_folder_depth: 5,
            enable_semantic_clustering: true,
            enable_temporal_organization: false,
        }
    }
}

/// Comprehensive organization analysis and suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationAnalysis {
    pub workspace_path: PathBuf,
    pub analysis_timestamp: DateTime<Utc>,
    pub config_used: OrganizationConfig,

    // Core analysis components
    pub categorization_suggestions: Vec<CategorizationSuggestion>,
    pub tagging_suggestions: Vec<TaggingSuggestion>,
    pub folder_structure_suggestions: Vec<FolderStructureSuggestion>,
    pub semantic_clusters: Vec<SemanticCluster>,
    pub duplicate_handling: Vec<DuplicateHandlingSuggestion>,

    // Overall metrics
    pub organization_score: f32,
    pub improvement_potential: f32,
    pub priority_actions: Vec<OrganizationAction>,

    // Analysis metadata
    pub documents_analyzed: usize,
    pub analysis_duration_ms: u64,
}

/// Categorization suggestion for documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationSuggestion {
    pub document_path: PathBuf,
    pub current_category: Option<String>,
    pub suggested_category: String,
    pub confidence: f32,
    pub reasoning: String,
    pub alternative_categories: Vec<AlternativeCategory>,
    pub category_hierarchy: Vec<String>,
}

/// Alternative category suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeCategory {
    pub category: String,
    pub confidence: f32,
    pub reasoning: String,
}

/// Tagging suggestion for enhanced searchability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggingSuggestion {
    pub document_path: PathBuf,
    pub current_tags: Vec<String>,
    pub suggested_tags: Vec<SuggestedTag>,
    pub tag_categories: HashMap<String, Vec<String>>,
    pub semantic_tags: Vec<String>,
    pub auto_generated_tags: Vec<String>,
}

/// Individual tag suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTag {
    pub tag: String,
    pub confidence: f32,
    pub tag_type: TagType,
    pub reasoning: String,
    pub frequency_in_workspace: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TagType {
    Topic,
    Format,
    Source,
    Priority,
    Status,
    Audience,
    Custom,
}

/// Folder structure optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderStructureSuggestion {
    pub suggestion_type: StructureSuggestionType,
    pub target_documents: Vec<PathBuf>,
    pub proposed_structure: FolderStructure,
    pub current_structure: Option<FolderStructure>,
    pub benefits: Vec<String>,
    pub implementation_effort: ImplementationEffort,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureSuggestionType {
    CreateNewFolder,
    MergeExistingFolders,
    SplitOvercrowdedFolder,
    ReorganizeByTopic,
    ReorganizeByType,
    ReorganizeByDate,
    FlattenHierarchy,
    CreateTopicHierarchy,
}

/// Folder structure representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderStructure {
    pub root_path: PathBuf,
    pub folders: Vec<FolderNode>,
    pub organization_principle: OrganizationPrinciple,
    pub estimated_efficiency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderNode {
    pub name: String,
    pub path: PathBuf,
    pub subfolders: Vec<FolderNode>,
    pub estimated_file_count: usize,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrganizationPrinciple {
    ByTopic,
    ByDocumentType,
    ByProject,
    ByDate,
    ByImportance,
    BySource,
    Hybrid(Vec<OrganizationPrinciple>),
}

/// Semantic clustering of related documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCluster {
    pub cluster_id: String,
    pub cluster_name: String,
    pub documents: Vec<PathBuf>,
    pub central_topics: Vec<String>,
    pub cohesion_score: f32,
    pub suggested_organization: ClusterOrganization,
    pub related_clusters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterOrganization {
    pub suggested_folder: String,
    pub organization_rationale: String,
    pub sub_organization: Option<SubOrganization>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubOrganization {
    pub principle: OrganizationPrinciple,
    pub subcategories: Vec<String>,
}

/// Duplicate handling suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateHandlingSuggestion {
    pub duplicate_group: Vec<PathBuf>,
    pub similarity_score: f32,
    pub recommended_action: DuplicateAction,
    pub canonical_document: Option<PathBuf>,
    pub consolidation_plan: Option<ConsolidationPlan>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DuplicateAction {
    KeepPrimary,
    MergeContent,
    CreateReference,
    ArchiveRedundant,
    TagAsVersions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationPlan {
    pub primary_document: PathBuf,
    pub documents_to_merge: Vec<PathBuf>,
    pub merge_strategy: MergeStrategy,
    pub backup_plan: BackupPlan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    AppendContent,
    MergeSections,
    CreateIndex,
    ExtractCommon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPlan {
    pub backup_location: PathBuf,
    pub backup_strategy: BackupStrategy,
    pub retention_period: Option<u32>, // days
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackupStrategy {
    ArchiveFolder,
    VersionControl,
    CloudBackup,
    LocalCopy,
}

/// Organization action with priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationAction {
    pub action_id: String,
    pub action_type: ActionType,
    pub priority: ActionPriority,
    pub description: String,
    pub affected_documents: Vec<PathBuf>,
    pub implementation_steps: Vec<String>,
    pub estimated_time: ImplementationEffort,
    pub expected_benefit: BenefitLevel,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    Categorize,
    Tag,
    Restructure,
    Merge,
    Archive,
    CreateFolder,
    MoveDocuments,
    AddMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionPriority {
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal, // < 15 minutes
    Low,     // 15-60 minutes
    Medium,  // 1-4 hours
    High,    // 4+ hours
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BenefitLevel {
    High,   // Significantly improves organization and findability
    Medium, // Moderate improvement
    Low,    // Minor improvement
}

#[allow(dead_code)]
impl SmartOrganizer {
    /// Create a new smart organizer
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
            content_classifier: ContentClassifier::new(None)
                .map_err(|e| anyhow::anyhow!("Failed to create content classifier: {}", e))?,
            style_analyzer: StyleAnalyzer::new(),
            structure_analyzer: StructureAnalyzer::new()?,
            ai_orchestrator,
        })
    }

    /// Perform comprehensive organization analysis
    pub async fn analyze_organization(
        &self,
        workspace_path: &Path,
        config: OrganizationConfig,
    ) -> Result<OrganizationAnalysis> {
        let start_time = std::time::Instant::now();

        let indexer = self.document_indexer.lock().await;
        let documents = indexer.get_all_documents();
        let document_count = documents.len();
        drop(indexer);

        let mut analysis = OrganizationAnalysis {
            workspace_path: workspace_path.to_path_buf(),
            analysis_timestamp: Utc::now(),
            config_used: config.clone(),
            categorization_suggestions: vec![],
            tagging_suggestions: vec![],
            folder_structure_suggestions: vec![],
            semantic_clusters: vec![],
            duplicate_handling: vec![],
            organization_score: 0.0,
            improvement_potential: 0.0,
            priority_actions: vec![],
            documents_analyzed: document_count,
            analysis_duration_ms: 0,
        };

        // Perform categorization analysis
        if config.enable_categorization {
            analysis.categorization_suggestions =
                self.analyze_categorization(workspace_path, &config).await?;
        }

        // Perform tagging analysis
        if config.enable_tagging {
            analysis.tagging_suggestions = self.analyze_tagging(workspace_path, &config).await?;
        }

        // Perform folder structure analysis
        if config.enable_folder_optimization {
            analysis.folder_structure_suggestions = self
                .analyze_folder_structure(workspace_path, &config)
                .await?;
        }

        // Perform semantic clustering
        if config.enable_semantic_clustering {
            analysis.semantic_clusters = self
                .perform_semantic_clustering(workspace_path, &config)
                .await?;
        }

        // Analyze duplicates
        analysis.duplicate_handling = self.analyze_duplicates(workspace_path, &config).await?;

        // Calculate organization metrics
        analysis.organization_score = self.calculate_organization_score(&analysis);
        analysis.improvement_potential = self.calculate_improvement_potential(&analysis);

        // Generate priority actions
        analysis.priority_actions = self.generate_priority_actions(&analysis);

        analysis.analysis_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(analysis)
    }

    /// Analyze categorization opportunities
    pub async fn analyze_categorization(
        &self,
        _workspace_path: &Path,
        config: &OrganizationConfig,
    ) -> Result<Vec<CategorizationSuggestion>> {
        let indexer = self.document_indexer.lock().await;
        let documents = indexer.get_all_documents();

        let mut suggestions = Vec::new();

        for document in &documents {
            // Analyze document content to suggest category
            let suggested_category = self.classify_document_category(document, config).await?;

            if suggested_category.confidence >= config.min_confidence {
                suggestions.push(CategorizationSuggestion {
                    document_path: document.path.clone(),
                    current_category: self.extract_current_category(&document.path),
                    suggested_category: suggested_category.category.clone(),
                    confidence: suggested_category.confidence,
                    reasoning: suggested_category.reasoning.clone(),
                    alternative_categories: suggested_category.alternatives,
                    category_hierarchy: self.build_category_hierarchy(&suggested_category.category),
                });
            }
        }

        // Sort by confidence
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(suggestions)
    }

    /// Analyze tagging opportunities
    pub async fn analyze_tagging(
        &self,
        _workspace_path: &Path,
        config: &OrganizationConfig,
    ) -> Result<Vec<TaggingSuggestion>> {
        let indexer = self.document_indexer.lock().await;
        let documents = indexer.get_all_documents();

        let mut suggestions = Vec::new();

        for document in &documents {
            let current_tags = self.extract_current_tags(&document.path);
            let suggested_tags = self.generate_tags_for_document(document, config).await?;

            if !suggested_tags.is_empty() {
                let semantic_tags = self.generate_semantic_tags(&document.content).await?;
                let auto_tags = self.generate_auto_tags(document).await?;

                suggestions.push(TaggingSuggestion {
                    document_path: document.path.clone(),
                    current_tags,
                    suggested_tags,
                    tag_categories: self.organize_tags_by_category(&document.keywords),
                    semantic_tags,
                    auto_generated_tags: auto_tags,
                });
            }
        }

        Ok(suggestions)
    }

    /// Analyze folder structure optimization opportunities
    pub async fn analyze_folder_structure(
        &self,
        workspace_path: &Path,
        config: &OrganizationConfig,
    ) -> Result<Vec<FolderStructureSuggestion>> {
        let mut suggestions = Vec::new();

        // Analyze current structure
        let current_structure = self.analyze_current_structure(workspace_path).await?;

        // Check for overcrowded folders
        suggestions.extend(
            self.detect_overcrowded_folders(&current_structure, config)
                .await?,
        );

        // Check for under-utilized folders
        suggestions.extend(
            self.detect_underutilized_folders(&current_structure, config)
                .await?,
        );

        // Suggest topic-based organization
        suggestions.extend(
            self.suggest_topic_organization(workspace_path, config)
                .await?,
        );

        // Suggest type-based organization
        suggestions.extend(
            self.suggest_type_organization(workspace_path, config)
                .await?,
        );

        // Sort by confidence and benefit
        suggestions.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap().then(
                match (&a.implementation_effort, &b.implementation_effort) {
                    (ImplementationEffort::Minimal, _) => std::cmp::Ordering::Less,
                    (_, ImplementationEffort::Minimal) => std::cmp::Ordering::Greater,
                    (ImplementationEffort::Low, ImplementationEffort::High) => {
                        std::cmp::Ordering::Less
                    }
                    (ImplementationEffort::High, ImplementationEffort::Low) => {
                        std::cmp::Ordering::Greater
                    }
                    _ => std::cmp::Ordering::Equal,
                },
            )
        });

        Ok(suggestions)
    }

    /// Perform semantic clustering of documents
    pub async fn perform_semantic_clustering(
        &self,
        _workspace_path: &Path,
        _config: &OrganizationConfig,
    ) -> Result<Vec<SemanticCluster>> {
        let indexer = self.document_indexer.lock().await;
        let documents = indexer.get_all_documents();

        let relationship_analyzer = self.relationship_analyzer.lock().await;
        let mut clusters = Vec::new();
        let mut processed_docs = HashSet::new();

        for (i, document) in documents.iter().enumerate() {
            if processed_docs.contains(&document.id) {
                continue;
            }

            let mut cluster_docs = vec![document.path.clone()];
            let mut cluster_topics = HashSet::new();
            processed_docs.insert(document.id.clone());

            // Add document keywords to topics
            cluster_topics.extend(document.keywords.iter().cloned());

            // Find related documents
            for other_doc in documents.iter().skip(i + 1) {
                if processed_docs.contains(&other_doc.id) {
                    continue;
                }

                if let Ok(relationships) = relationship_analyzer
                    .analyze_document_pair(document, other_doc)
                    .await
                {
                    for relationship in relationships {
                        if relationship.score > 0.6 {
                            cluster_docs.push(other_doc.path.clone());
                            cluster_topics.extend(other_doc.keywords.iter().cloned());
                            processed_docs.insert(other_doc.id.clone());
                            break;
                        }
                    }
                }
            }

            if cluster_docs.len() > 1 {
                let cluster_name = self.generate_cluster_name(&cluster_topics);
                let cohesion_score = self.calculate_cluster_cohesion(&cluster_docs).await;

                clusters.push(SemanticCluster {
                    cluster_id: format!("cluster_{}", clusters.len()),
                    cluster_name: cluster_name.clone(),
                    documents: cluster_docs,
                    central_topics: cluster_topics.into_iter().take(5).collect(),
                    cohesion_score,
                    suggested_organization: ClusterOrganization {
                        suggested_folder: cluster_name,
                        organization_rationale: "Documents share common themes and topics"
                            .to_string(),
                        sub_organization: None,
                    },
                    related_clusters: vec![],
                });
            }
        }

        drop(relationship_analyzer);
        Ok(clusters)
    }

    /// Analyze duplicate handling opportunities
    pub async fn analyze_duplicates(
        &self,
        _workspace_path: &Path,
        _config: &OrganizationConfig,
    ) -> Result<Vec<DuplicateHandlingSuggestion>> {
        let indexer = self.document_indexer.lock().await;
        let documents = indexer.get_all_documents();

        let relationship_analyzer = self.relationship_analyzer.lock().await;
        let mut suggestions = Vec::new();
        let mut processed_pairs = HashSet::new();

        for i in 0..documents.len() {
            for j in (i + 1)..documents.len() {
                let pair_key = format!("{}:{}", documents[i].id, documents[j].id);
                if processed_pairs.contains(&pair_key) {
                    continue;
                }

                if let Ok(relationships) = relationship_analyzer
                    .analyze_document_pair(documents[i], documents[j])
                    .await
                {
                    for relationship in relationships {
                        if relationship.score > 0.8 {
                            let action = if relationship.score > 0.95 {
                                DuplicateAction::MergeContent
                            } else if relationship.score > 0.9 {
                                DuplicateAction::CreateReference
                            } else {
                                DuplicateAction::TagAsVersions
                            };

                            suggestions.push(DuplicateHandlingSuggestion {
                                duplicate_group: vec![
                                    documents[i].path.clone(),
                                    documents[j].path.clone(),
                                ],
                                similarity_score: relationship.score,
                                recommended_action: action,
                                canonical_document: Some(documents[i].path.clone()),
                                consolidation_plan: None,
                            });

                            processed_pairs.insert(pair_key);
                            break;
                        }
                    }
                }
            }
        }

        drop(relationship_analyzer);
        Ok(suggestions)
    }

    /// Generate priority actions based on analysis
    fn generate_priority_actions(
        &self,
        analysis: &OrganizationAnalysis,
    ) -> Vec<OrganizationAction> {
        let mut actions = Vec::new();

        // High-impact categorization actions
        for suggestion in &analysis.categorization_suggestions {
            if suggestion.confidence > 0.8 {
                actions.push(OrganizationAction {
                    action_id: format!("categorize_{}", actions.len()),
                    action_type: ActionType::Categorize,
                    priority: ActionPriority::High,
                    description: format!(
                        "Categorize {} as {}",
                        suggestion.document_path.display(),
                        suggestion.suggested_category
                    ),
                    affected_documents: vec![suggestion.document_path.clone()],
                    implementation_steps: vec![
                        "Review document content".to_string(),
                        "Confirm categorization".to_string(),
                        "Move to appropriate folder".to_string(),
                    ],
                    estimated_time: ImplementationEffort::Minimal,
                    expected_benefit: BenefitLevel::Medium,
                    dependencies: vec![],
                });
            }
        }

        // Folder restructuring actions
        for suggestion in &analysis.folder_structure_suggestions {
            if suggestion.confidence > 0.7 {
                let priority = match suggestion.implementation_effort {
                    ImplementationEffort::Minimal => ActionPriority::High,
                    ImplementationEffort::Low => ActionPriority::Medium,
                    _ => ActionPriority::Low,
                };

                actions.push(OrganizationAction {
                    action_id: format!("restructure_{}", actions.len()),
                    action_type: ActionType::Restructure,
                    priority,
                    description: format!(
                        "Restructure folder organization: {:?}",
                        suggestion.suggestion_type
                    ),
                    affected_documents: suggestion.target_documents.clone(),
                    implementation_steps: vec![
                        "Create new folder structure".to_string(),
                        "Move documents to new locations".to_string(),
                        "Update references".to_string(),
                    ],
                    estimated_time: suggestion.implementation_effort.clone(),
                    expected_benefit: BenefitLevel::High,
                    dependencies: vec![],
                });
            }
        }

        // Duplicate handling actions
        for suggestion in &analysis.duplicate_handling {
            if suggestion.similarity_score > 0.9 {
                actions.push(OrganizationAction {
                    action_id: format!("dedupe_{}", actions.len()),
                    action_type: ActionType::Merge,
                    priority: ActionPriority::Medium,
                    description: format!(
                        "Handle duplicate documents: {:?}",
                        suggestion.recommended_action
                    ),
                    affected_documents: suggestion.duplicate_group.clone(),
                    implementation_steps: vec![
                        "Review duplicate documents".to_string(),
                        "Select canonical version".to_string(),
                        "Merge or archive duplicates".to_string(),
                    ],
                    estimated_time: ImplementationEffort::Low,
                    expected_benefit: BenefitLevel::Medium,
                    dependencies: vec![],
                });
            }
        }

        // Sort by priority and benefit
        actions.sort_by(|a, b| {
            let priority_order = |p: &ActionPriority| match p {
                ActionPriority::Urgent => 4,
                ActionPriority::High => 3,
                ActionPriority::Medium => 2,
                ActionPriority::Low => 1,
            };
            priority_order(&b.priority).cmp(&priority_order(&a.priority))
        });

        actions.into_iter().take(20).collect() // Return top 20 actions
    }

    /// Calculate overall organization score
    fn calculate_organization_score(&self, analysis: &OrganizationAnalysis) -> f32 {
        let mut score_factors = Vec::new();

        // Categorization completeness
        let categorization_score = if analysis.documents_analyzed > 0 {
            analysis.categorization_suggestions.len() as f32 / analysis.documents_analyzed as f32
        } else {
            1.0
        };
        score_factors.push(1.0 - categorization_score.min(1.0));

        // Folder structure efficiency
        let structure_score = analysis
            .folder_structure_suggestions
            .iter()
            .map(|s| s.confidence)
            .fold(0.0, |acc, conf| acc + conf)
            / analysis.folder_structure_suggestions.len().max(1) as f32;
        score_factors.push(1.0 - structure_score);

        // Duplicate management
        let duplicate_score = if analysis.documents_analyzed > 0 {
            analysis.duplicate_handling.len() as f32 / analysis.documents_analyzed as f32
        } else {
            1.0
        };
        score_factors.push(1.0 - duplicate_score.min(1.0));

        // Semantic clustering cohesion
        let clustering_score = analysis
            .semantic_clusters
            .iter()
            .map(|c| c.cohesion_score)
            .fold(0.0, |acc, score| acc + score)
            / analysis.semantic_clusters.len().max(1) as f32;
        score_factors.push(clustering_score);

        score_factors.iter().sum::<f32>() / score_factors.len() as f32
    }

    /// Calculate improvement potential
    fn calculate_improvement_potential(&self, analysis: &OrganizationAnalysis) -> f32 {
        let high_impact_actions = analysis
            .priority_actions
            .iter()
            .filter(|a| matches!(a.expected_benefit, BenefitLevel::High))
            .count();

        let total_actions = analysis.priority_actions.len().max(1);
        high_impact_actions as f32 / total_actions as f32
    }

    // Helper methods
    async fn classify_document_category(
        &self,
        document: &crate::document::indexer::DocumentIndexEntry,
        _config: &OrganizationConfig,
    ) -> Result<CategoryClassification> {
        // Analyze document structure first
        let structure_analysis = self
            .structure_analyzer
            .analyze_structure(&document.content)?;

        // Use content classifier to determine category
        let content_analysis = self
            .content_classifier
            .analyze_document_content(&structure_analysis, &document.content)
            .await?;

        let category = match content_analysis.dominant_content_type {
            crate::document::content_classifier::ContentCategory::Procedures => {
                "Policies and Procedures"
            }
            crate::document::content_classifier::ContentCategory::Explanations => {
                "Technical Documentation"
            }
            crate::document::content_classifier::ContentCategory::Examples => {
                "Guides and Tutorials"
            }
            crate::document::content_classifier::ContentCategory::Definitions => {
                "Reference Materials"
            }
            crate::document::content_classifier::ContentCategory::QAndA => "Q&A and FAQs",
            crate::document::content_classifier::ContentCategory::Warnings => "Safety and Warnings",
            crate::document::content_classifier::ContentCategory::BestPractices => "Best Practices",
            _ => "General Documents",
        };

        Ok(CategoryClassification {
            category: category.to_string(),
            confidence: content_analysis.complexity_score as f32,
            reasoning: format!(
                "Classified based on dominant content type: {:?}",
                content_analysis.dominant_content_type
            ),
            alternatives: vec![],
        })
    }

    fn extract_current_category(&self, path: &Path) -> Option<String> {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    fn build_category_hierarchy(&self, category: &str) -> Vec<String> {
        // Simple hierarchy building - could be enhanced with AI
        match category {
            "Technical Documentation" => vec!["Documentation".to_string(), "Technical".to_string()],
            "Guides and Tutorials" => vec!["Documentation".to_string(), "Educational".to_string()],
            "Reference Materials" => vec!["Documentation".to_string(), "Reference".to_string()],
            "Policies and Procedures" => vec!["Administrative".to_string(), "Policies".to_string()],
            "Reports and Analysis" => vec!["Business".to_string(), "Reports".to_string()],
            "Meeting Notes" => vec!["Communications".to_string(), "Meetings".to_string()],
            "Communications" => vec!["Communications".to_string()],
            _ => vec!["General".to_string()],
        }
    }

    fn extract_current_tags(&self, _path: &Path) -> Vec<String> {
        // Extract tags from metadata or filename
        // For now, return empty - could be enhanced
        vec![]
    }

    async fn generate_tags_for_document(
        &self,
        document: &crate::document::indexer::DocumentIndexEntry,
        _config: &OrganizationConfig,
    ) -> Result<Vec<SuggestedTag>> {
        let mut tags = Vec::new();

        // Generate tags based on keywords
        for keyword in &document.keywords {
            if keyword.len() > 3 {
                tags.push(SuggestedTag {
                    tag: keyword.clone(),
                    confidence: 0.7,
                    tag_type: TagType::Topic,
                    reasoning: "Extracted from document keywords".to_string(),
                    frequency_in_workspace: 1, // Would be calculated from workspace analysis
                });
            }
        }

        // Add format-based tags
        if let Some(extension) = document.path.extension() {
            if let Some(ext_str) = extension.to_str() {
                tags.push(SuggestedTag {
                    tag: format!("format:{}", ext_str.to_uppercase()),
                    confidence: 1.0,
                    tag_type: TagType::Format,
                    reasoning: "File format classification".to_string(),
                    frequency_in_workspace: 1,
                });
            }
        }

        Ok(tags)
    }

    async fn generate_semantic_tags(&self, content: &str) -> Result<Vec<String>> {
        // Extract semantic tags using simple keyword analysis
        let words: Vec<&str> = content.split_whitespace().filter(|w| w.len() > 5).collect();

        let mut word_counts = HashMap::new();
        for word in words {
            *word_counts.entry(word.to_lowercase()).or_insert(0) += 1;
        }

        let semantic_tags: Vec<String> = word_counts
            .into_iter()
            .filter(|(_, count)| *count > 2)
            .map(|(word, _)| word)
            .take(10)
            .collect();

        Ok(semantic_tags)
    }

    async fn generate_auto_tags(
        &self,
        document: &crate::document::indexer::DocumentIndexEntry,
    ) -> Result<Vec<String>> {
        let mut auto_tags = Vec::new();

        // Size-based tags
        let size_mb = document.metadata.basic.file_size as f64 / (1024.0 * 1024.0);
        if size_mb > 10.0 {
            auto_tags.push("large-file".to_string());
        } else if size_mb < 0.1 {
            auto_tags.push("small-file".to_string());
        }

        // Date-based tags
        if let Some(modified) = document.metadata.basic.modified {
            let now = std::time::SystemTime::now();
            if let Ok(duration) = now.duration_since(modified) {
                let days = duration.as_secs() / (24 * 3600);
                if days < 7 {
                    auto_tags.push("recent".to_string());
                } else if days > 365 {
                    auto_tags.push("old".to_string());
                }
            }
        }

        Ok(auto_tags)
    }

    fn organize_tags_by_category(&self, keywords: &[String]) -> HashMap<String, Vec<String>> {
        let mut categories = HashMap::new();

        for keyword in keywords {
            let category = if keyword.contains("tech") || keyword.contains("api") {
                "Technical"
            } else if keyword.contains("business") || keyword.contains("strategy") {
                "Business"
            } else if keyword.contains("process") || keyword.contains("procedure") {
                "Process"
            } else {
                "General"
            };

            categories
                .entry(category.to_string())
                .or_insert_with(Vec::new)
                .push(keyword.clone());
        }

        categories
    }

    async fn analyze_current_structure(&self, workspace_path: &Path) -> Result<FolderStructure> {
        let mut folders = Vec::new();

        if let Ok(entries) = std::fs::read_dir(workspace_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let file_count = self.count_files_in_directory(&entry.path()).await?;
                    folders.push(FolderNode {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry.path(),
                        subfolders: vec![], // Could be recursive
                        estimated_file_count: file_count,
                        category: None,
                    });
                }
            }
        }

        Ok(FolderStructure {
            root_path: workspace_path.to_path_buf(),
            folders,
            organization_principle: OrganizationPrinciple::ByTopic,
            estimated_efficiency: 0.7,
        })
    }

    async fn count_files_in_directory(&self, path: &Path) -> Result<usize> {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    async fn detect_overcrowded_folders(
        &self,
        structure: &FolderStructure,
        _config: &OrganizationConfig,
    ) -> Result<Vec<FolderStructureSuggestion>> {
        let mut suggestions = Vec::new();

        for folder in &structure.folders {
            if folder.estimated_file_count > 20 {
                suggestions.push(FolderStructureSuggestion {
                    suggestion_type: StructureSuggestionType::SplitOvercrowdedFolder,
                    target_documents: vec![], // Would list actual documents
                    proposed_structure: FolderStructure {
                        root_path: folder.path.clone(),
                        folders: vec![], // Would contain proposed subfolders
                        organization_principle: OrganizationPrinciple::ByTopic,
                        estimated_efficiency: 0.8,
                    },
                    current_structure: None,
                    benefits: vec![
                        "Improved organization".to_string(),
                        "Easier navigation".to_string(),
                        "Better findability".to_string(),
                    ],
                    implementation_effort: ImplementationEffort::Medium,
                    confidence: 0.8,
                });
            }
        }

        Ok(suggestions)
    }

    async fn detect_underutilized_folders(
        &self,
        structure: &FolderStructure,
        _config: &OrganizationConfig,
    ) -> Result<Vec<FolderStructureSuggestion>> {
        let mut suggestions = Vec::new();

        for folder in &structure.folders {
            if folder.estimated_file_count < 3 && folder.estimated_file_count > 0 {
                suggestions.push(FolderStructureSuggestion {
                    suggestion_type: StructureSuggestionType::MergeExistingFolders,
                    target_documents: vec![], // Would list actual documents
                    proposed_structure: FolderStructure {
                        root_path: structure.root_path.clone(),
                        folders: vec![], // Would contain merged structure
                        organization_principle: OrganizationPrinciple::ByTopic,
                        estimated_efficiency: 0.8,
                    },
                    current_structure: None,
                    benefits: vec![
                        "Reduced folder clutter".to_string(),
                        "Simplified navigation".to_string(),
                    ],
                    implementation_effort: ImplementationEffort::Low,
                    confidence: 0.7,
                });
            }
        }

        Ok(suggestions)
    }

    async fn suggest_topic_organization(
        &self,
        _workspace_path: &Path,
        _config: &OrganizationConfig,
    ) -> Result<Vec<FolderStructureSuggestion>> {
        // Analyze document topics and suggest topic-based organization
        Ok(vec![])
    }

    async fn suggest_type_organization(
        &self,
        _workspace_path: &Path,
        _config: &OrganizationConfig,
    ) -> Result<Vec<FolderStructureSuggestion>> {
        // Analyze document types and suggest type-based organization
        Ok(vec![])
    }

    fn generate_cluster_name(&self, topics: &HashSet<String>) -> String {
        if let Some(topic) = topics.iter().next() {
            format!("{} Documents", topic)
        } else {
            "Related Documents".to_string()
        }
    }

    async fn calculate_cluster_cohesion(&self, _documents: &[PathBuf]) -> f32 {
        // Calculate how well documents in cluster relate to each other
        0.75 // Placeholder
    }
}

/// Category classification result
struct CategoryClassification {
    category: String,
    confidence: f32,
    reasoning: String,
    alternatives: Vec<AlternativeCategory>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_smart_organizer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = Arc::new(Mutex::new(
            crate::document::indexer::DocumentIndexer::new(temp_dir.path().to_path_buf()).unwrap(),
        ));
        let relationship_analyzer = Arc::new(Mutex::new(
            crate::document::relationship_analyzer::RelationshipAnalyzer::new(
                crate::document::relationship_analyzer::RelationshipConfig::default(),
            ),
        ));
        let workspace_intelligence =
            crate::workspace::intelligence::WorkspaceIntelligence::new(indexer.clone(), None);

        let _organizer =
            SmartOrganizer::new(workspace_intelligence, indexer, relationship_analyzer, None)
                .unwrap();

        // Test that organizer was created successfully
        assert!(true);
    }

    #[test]
    fn test_organization_config_default() {
        let config = OrganizationConfig::default();

        assert!(config.enable_categorization);
        assert!(config.enable_tagging);
        assert!(config.enable_folder_optimization);
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.max_folder_depth, 5);
        assert!(config.enable_semantic_clustering);
        assert!(!config.enable_temporal_organization);
    }

    #[test]
    fn test_action_priority_ordering() {
        let urgent = ActionPriority::Urgent;
        let high = ActionPriority::High;
        let medium = ActionPriority::Medium;
        let low = ActionPriority::Low;

        assert_ne!(urgent, high);
        assert_ne!(high, medium);
        assert_ne!(medium, low);
    }

    #[test]
    fn test_organization_principle_types() {
        let by_topic = OrganizationPrinciple::ByTopic;
        let by_type = OrganizationPrinciple::ByDocumentType;
        let hybrid = OrganizationPrinciple::Hybrid(vec![by_topic.clone(), by_type.clone()]);

        assert_ne!(by_topic, by_type);
        assert!(matches!(hybrid, OrganizationPrinciple::Hybrid(_)));
    }
}
