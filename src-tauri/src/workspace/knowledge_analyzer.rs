// src-tauri/src/workspace/knowledge_analyzer.rs
//! Knowledge gap identification and analysis system
//!
//! This module provides intelligent analysis of knowledge completeness within workspaces,
//! identifying gaps in documentation coverage, missing information, and areas that need
//! additional content or improvement.

use super::intelligence::{GapSeverity, WorkspaceIntelligence};
use crate::ai::AIOrchestrator;
use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::RelationshipAnalyzer;
use crate::document::{ContentClassifier, StructureAnalyzer};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Knowledge gap analyzer for identifying missing and incomplete information
pub struct KnowledgeAnalyzer {
    #[allow(dead_code)]
    workspace_intelligence: WorkspaceIntelligence,
    document_indexer: Arc<Mutex<DocumentIndexer>>,
    #[allow(dead_code)]
    relationship_analyzer: Arc<Mutex<RelationshipAnalyzer>>,
    content_classifier: ContentClassifier,
    structure_analyzer: StructureAnalyzer,
    #[allow(dead_code)]
    ai_orchestrator: Option<Arc<Mutex<AIOrchestrator>>>,
}

/// Configuration for knowledge gap analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeAnalysisConfig {
    /// Enable deep content analysis for gap detection
    pub enable_deep_analysis: bool,
    /// Analyze cross-references and dependencies
    pub analyze_dependencies: bool,
    /// Check for outdated information
    pub check_freshness: bool,
    /// Analyze completeness of processes and procedures
    pub analyze_completeness: bool,
    /// Minimum confidence threshold for gap identification
    pub confidence_threshold: f32,
    /// Include AI-generated gap suggestions
    pub enable_ai_suggestions: bool,
    /// Analysis depth (0-100)
    pub analysis_depth: u8,
}

/// Comprehensive knowledge gap analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGapAnalysis {
    /// Workspace path analyzed
    pub workspace_path: PathBuf,
    /// Timestamp of analysis
    pub analysis_timestamp: DateTime<Utc>,
    /// Overall knowledge completeness score (0-100)
    pub completeness_score: f32,
    /// Identified knowledge gaps
    pub knowledge_gaps: Vec<KnowledgeGap>,
    /// Missing document types
    pub missing_document_types: Vec<MissingDocumentType>,
    /// Incomplete processes and procedures
    pub incomplete_processes: Vec<IncompleteProcess>,
    /// Outdated content requiring updates
    pub outdated_content: Vec<OutdatedContent>,
    /// Cross-reference gaps and broken links
    pub reference_gaps: Vec<ReferenceGap>,
    /// Subject matter expert recommendations
    pub expert_recommendations: Vec<ExpertRecommendation>,
    /// Priority improvement areas
    pub priority_areas: Vec<PriorityArea>,
    /// Analysis configuration used
    pub config: KnowledgeAnalysisConfig,
    /// Number of documents analyzed
    pub documents_analyzed: usize,
    /// Analysis duration in seconds
    pub analysis_duration: f64,
}

/// A specific knowledge gap identified in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    /// Unique identifier for this gap
    pub gap_id: String,
    /// Type of knowledge gap
    pub gap_type: KnowledgeGapType,
    /// Title/summary of the gap
    pub title: String,
    /// Detailed description of what's missing
    pub description: String,
    /// Confidence that this is actually a gap (0-1)
    pub confidence: f32,
    /// Severity of the gap
    pub severity: GapSeverity,
    /// Affected document areas or topics
    pub affected_areas: Vec<String>,
    /// Related documents that reference this gap
    pub related_documents: Vec<PathBuf>,
    /// Suggested content to fill the gap
    pub suggested_content: Option<String>,
    /// Estimated effort to address
    pub effort_estimate: ImplementationEffort,
    /// Business impact of not addressing
    pub business_impact: BusinessImpact,
}

/// Types of knowledge gaps that can be identified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KnowledgeGapType {
    /// Missing procedural documentation
    MissingProcedure,
    /// Incomplete process description
    IncompleteProcess,
    /// Missing conceptual explanation
    MissingConcept,
    /// Outdated information
    OutdatedInformation,
    /// Missing examples or tutorials
    MissingExamples,
    /// Insufficient technical details
    InsufficientDetail,
    /// Missing troubleshooting information
    MissingTroubleshooting,
    /// Broken or missing references
    BrokenReferences,
    /// Missing prerequisites or dependencies
    MissingPrerequisites,
    /// Inconsistent information across documents
    InconsistentInformation,
}

/// Missing document type identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDocumentType {
    /// Type of document that's missing
    pub document_type: String,
    /// Why this type is expected
    pub rationale: String,
    /// Confidence that it should exist
    pub confidence: f32,
    /// Suggested location for the document
    pub suggested_location: Option<PathBuf>,
    /// Priority for creation
    pub priority: DocumentPriority,
}

/// Incomplete process identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncompleteProcess {
    /// Process name or identifier
    pub process_name: String,
    /// What aspects are incomplete
    pub missing_aspects: Vec<String>,
    /// Documents that partially describe this process
    pub partial_documents: Vec<PathBuf>,
    /// Completeness percentage
    pub completeness_percentage: f32,
    /// Critical missing steps
    pub critical_gaps: Vec<String>,
}

/// Outdated content identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedContent {
    /// Path to outdated document
    pub document_path: PathBuf,
    /// Indicators of being outdated
    pub outdated_indicators: Vec<String>,
    /// Last modification date
    pub last_modified: DateTime<Utc>,
    /// Estimated age in days
    pub age_days: i64,
    /// Urgency of update needed
    pub update_urgency: UpdateUrgency,
    /// Suggested updates
    pub suggested_updates: Vec<String>,
}

/// Reference gap identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceGap {
    /// Source document with the reference
    pub source_document: PathBuf,
    /// Referenced but missing content
    pub missing_reference: String,
    /// Type of reference gap
    pub gap_type: ReferenceGapType,
    /// Context where reference appears
    pub context: String,
    /// Suggested resolution
    pub suggested_resolution: String,
}

/// Expert recommendation for knowledge improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertRecommendation {
    /// Area of expertise needed
    pub expertise_area: String,
    /// Specific recommendation
    pub recommendation: String,
    /// Justification for the recommendation
    pub justification: String,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Estimated impact of implementing
    pub impact: BusinessImpact,
}

/// Priority area for knowledge improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityArea {
    /// Name of the priority area
    pub area_name: String,
    /// Score indicating priority (higher = more important)
    pub priority_score: f32,
    /// Number of gaps in this area
    pub gap_count: usize,
    /// Potential impact of addressing
    pub potential_impact: String,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Supporting enums and types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentPriority {
    Essential, // Critical for operations
    Important, // Highly beneficial
    Useful,    // Nice to have
    Optional,  // Low priority
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateUrgency {
    Immediate, // Must update ASAP
    High,      // Update within week
    Medium,    // Update within month
    Low,       // Update when convenient
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReferenceGapType {
    BrokenInternalLink,
    MissingDocument,
    MissingSection,
    OutdatedReference,
    MissingExternalResource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal,   // < 15 minutes
    Low,       // 15-60 minutes
    Medium,    // 1-4 hours
    High,      // 4+ hours
    Extensive, // Multiple days
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BusinessImpact {
    Critical, // Business-critical impact
    High,     // Significant operational impact
    Medium,   // Moderate efficiency impact
    Low,      // Minor improvement
}

impl Default for KnowledgeAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_deep_analysis: true,
            analyze_dependencies: true,
            check_freshness: true,
            analyze_completeness: true,
            confidence_threshold: 0.7,
            enable_ai_suggestions: true,
            analysis_depth: 80,
        }
    }
}

impl KnowledgeAnalyzer {
    /// Create a new knowledge analyzer
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
            structure_analyzer: StructureAnalyzer::new()?,
            ai_orchestrator,
        })
    }

    /// Perform comprehensive knowledge gap analysis
    pub async fn analyze_knowledge_gaps(
        &self,
        workspace_path: &Path,
        config: KnowledgeAnalysisConfig,
    ) -> Result<KnowledgeGapAnalysis> {
        let start_time = std::time::Instant::now();

        let document_count;
        let documents = {
            let indexer = self.document_indexer.lock().await;
            let docs = indexer.get_all_documents();
            document_count = docs.len();
            docs.into_iter().cloned().collect::<Vec<_>>()
        };

        let mut analysis = KnowledgeGapAnalysis {
            workspace_path: workspace_path.to_path_buf(),
            analysis_timestamp: Utc::now(),
            completeness_score: 0.0,
            knowledge_gaps: Vec::new(),
            missing_document_types: Vec::new(),
            incomplete_processes: Vec::new(),
            outdated_content: Vec::new(),
            reference_gaps: Vec::new(),
            expert_recommendations: Vec::new(),
            priority_areas: Vec::new(),
            config: config.clone(),
            documents_analyzed: document_count,
            analysis_duration: 0.0,
        };

        // Analyze knowledge gaps
        if config.enable_deep_analysis {
            analysis.knowledge_gaps = self.identify_knowledge_gaps(&documents, &config).await?;
        }

        // Identify missing document types
        analysis.missing_document_types = self
            .identify_missing_document_types(&documents, &config)
            .await?;

        // Analyze process completeness
        if config.analyze_completeness {
            analysis.incomplete_processes = self
                .analyze_process_completeness(&documents, &config)
                .await?;
        }

        // Check for outdated content
        if config.check_freshness {
            analysis.outdated_content = self.identify_outdated_content(&documents, &config).await?;
        }

        // Analyze reference gaps
        if config.analyze_dependencies {
            analysis.reference_gaps = self.analyze_reference_gaps(&documents, &config).await?;
        }

        // Generate expert recommendations
        if config.enable_ai_suggestions {
            analysis.expert_recommendations = self
                .generate_expert_recommendations(&analysis, &config)
                .await?;
        }

        // Identify priority areas
        analysis.priority_areas = self.identify_priority_areas(&analysis);

        // Calculate overall completeness score
        analysis.completeness_score = self.calculate_completeness_score(&analysis);

        analysis.analysis_duration = start_time.elapsed().as_secs_f64();
        Ok(analysis)
    }

    /// Identify specific knowledge gaps in the workspace
    async fn identify_knowledge_gaps(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<KnowledgeGap>> {
        let mut gaps = Vec::new();

        // Analyze each document for gaps
        for (index, document) in documents.iter().enumerate() {
            // Analyze document structure for completeness
            let structure_analysis = self
                .structure_analyzer
                .analyze_structure(&document.content)?;

            // Use content classifier to understand document type and completeness
            let content_analysis = self
                .content_classifier
                .analyze_document_content(&structure_analysis, &document.content)
                .await?;

            // Check for missing sections based on document type
            gaps.extend(
                self.check_missing_sections(document, &structure_analysis, config)
                    .await?,
            );

            // Check for incomplete explanations
            gaps.extend(
                self.check_incomplete_explanations(document, &content_analysis, config)
                    .await?,
            );

            // Check for missing examples or procedures
            gaps.extend(
                self.check_missing_examples(document, &content_analysis, config)
                    .await?,
            );

            // Limit analysis based on depth setting
            if config.analysis_depth < 100
                && index >= ((documents.len() * config.analysis_depth as usize) / 100)
            {
                break;
            }
        }

        // Filter by confidence threshold
        gaps.retain(|gap| gap.confidence >= config.confidence_threshold);

        Ok(gaps)
    }

    /// Check for missing sections in documents
    async fn check_missing_sections(
        &self,
        document: &crate::document::indexer::DocumentIndexEntry,
        structure_analysis: &crate::document::structure_analyzer::DocumentStructureAnalysis,
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<KnowledgeGap>> {
        let mut gaps = Vec::new();

        // Check for common missing sections based on document type
        let expected_sections = self.get_expected_sections(&document.title, &document.content);
        let existing_sections: HashSet<String> = structure_analysis
            .sections
            .iter()
            .map(|s| s.content.to_lowercase())
            .collect();

        for expected_section in expected_sections {
            let section_key = expected_section.to_lowercase();
            if !existing_sections.iter().any(|s| s.contains(&section_key)) {
                gaps.push(KnowledgeGap {
                    gap_id: format!(
                        "missing_section_{}_{}",
                        document.id,
                        expected_section.replace(' ', "_")
                    ),
                    gap_type: KnowledgeGapType::IncompleteProcess,
                    title: format!("Missing {} section", expected_section),
                    description: format!(
                        "Document '{}' appears to be missing a {} section",
                        document.title, expected_section
                    ),
                    confidence: 0.75,
                    severity: GapSeverity::Medium,
                    affected_areas: vec![expected_section.clone()],
                    related_documents: vec![document.path.clone()],
                    suggested_content: Some(format!(
                        "Add a comprehensive {} section to complete the documentation",
                        expected_section
                    )),
                    effort_estimate: ImplementationEffort::Medium,
                    business_impact: BusinessImpact::Medium,
                });
            }
        }

        Ok(gaps)
    }

    /// Check for incomplete explanations
    async fn check_incomplete_explanations(
        &self,
        document: &crate::document::indexer::DocumentIndexEntry,
        content_analysis: &crate::document::content_classifier::DocumentContentAnalysis,
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<KnowledgeGap>> {
        let mut gaps = Vec::new();

        // Check if explanations are too brief or lack detail
        if content_analysis.complexity_score < 0.3 && document.content.len() < 1000 {
            gaps.push(KnowledgeGap {
                gap_id: format!("insufficient_detail_{}", document.id),
                gap_type: KnowledgeGapType::InsufficientDetail,
                title: "Insufficient technical detail".to_string(),
                description:
                    "Document appears to lack sufficient detail for comprehensive understanding"
                        .to_string(),
                confidence: 0.8,
                severity: GapSeverity::Medium,
                affected_areas: vec!["Content depth".to_string()],
                related_documents: vec![document.path.clone()],
                suggested_content: Some(
                    "Expand with additional details, examples, and explanations".to_string(),
                ),
                effort_estimate: ImplementationEffort::High,
                business_impact: BusinessImpact::Medium,
            });
        }

        Ok(gaps)
    }

    /// Check for missing examples or procedures
    async fn check_missing_examples(
        &self,
        document: &crate::document::indexer::DocumentIndexEntry,
        content_analysis: &crate::document::content_classifier::DocumentContentAnalysis,
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<KnowledgeGap>> {
        let mut gaps = Vec::new();

        // Check if document lacks examples when they would be helpful
        let has_examples = content_analysis
            .content_distribution
            .get(&crate::document::content_classifier::ContentCategory::Examples)
            .unwrap_or(&0.0)
            > &0.1;

        let needs_examples = content_analysis
            .content_distribution
            .get(&crate::document::content_classifier::ContentCategory::Procedures)
            .unwrap_or(&0.0)
            > &0.3;

        if needs_examples && !has_examples {
            gaps.push(KnowledgeGap {
                gap_id: format!("missing_examples_{}", document.id),
                gap_type: KnowledgeGapType::MissingExamples,
                title: "Missing practical examples".to_string(),
                description: "Document contains procedures but lacks practical examples"
                    .to_string(),
                confidence: 0.85,
                severity: GapSeverity::High,
                affected_areas: vec!["Examples and tutorials".to_string()],
                related_documents: vec![document.path.clone()],
                suggested_content: Some(
                    "Add practical examples demonstrating the procedures".to_string(),
                ),
                effort_estimate: ImplementationEffort::Medium,
                business_impact: BusinessImpact::High,
            });
        }

        Ok(gaps)
    }

    /// Identify missing document types that should exist
    async fn identify_missing_document_types(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<MissingDocumentType>> {
        let mut missing_types = Vec::new();

        // Analyze existing document types
        let mut existing_types = HashSet::new();
        for document in documents {
            existing_types.insert(self.classify_document_type(&document.title, &document.content));
        }

        // Define expected document types for a complete workspace
        let expected_types = vec![
            (
                "Getting Started Guide",
                "Helps new users understand the system",
            ),
            ("API Documentation", "Technical reference for developers"),
            ("Troubleshooting Guide", "Problem resolution procedures"),
            ("FAQ", "Common questions and answers"),
            ("Best Practices", "Recommended approaches and guidelines"),
            ("Security Guidelines", "Security policies and procedures"),
            ("Installation Guide", "Setup and configuration instructions"),
        ];

        for (doc_type, rationale) in expected_types {
            if !existing_types
                .iter()
                .any(|t| t.contains(&doc_type.to_lowercase()))
            {
                missing_types.push(MissingDocumentType {
                    document_type: doc_type.to_string(),
                    rationale: rationale.to_string(),
                    confidence: 0.8,
                    suggested_location: Some(PathBuf::from(format!(
                        "docs/{}.md",
                        doc_type.replace(' ', "_").to_lowercase()
                    ))),
                    priority: DocumentPriority::Important,
                });
            }
        }

        Ok(missing_types)
    }

    /// Analyze completeness of processes and procedures
    async fn analyze_process_completeness(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<IncompleteProcess>> {
        let mut incomplete_processes = Vec::new();

        // Group documents by process or topic
        let mut process_documents: HashMap<
            String,
            Vec<&crate::document::indexer::DocumentIndexEntry>,
        > = HashMap::new();

        for document in documents {
            let process_name = self.extract_process_name(&document.title, &document.content);
            process_documents
                .entry(process_name)
                .or_default()
                .push(document);
        }

        // Analyze each process for completeness
        for (process_name, docs) in process_documents {
            if docs.len() == 1 && docs[0].content.len() < 2000 {
                // Single, short document might indicate incomplete process
                let missing_aspects = self.identify_missing_process_aspects(&docs[0].content);

                if !missing_aspects.is_empty() {
                    incomplete_processes.push(IncompleteProcess {
                        process_name: process_name.clone(),
                        missing_aspects,
                        partial_documents: docs.iter().map(|d| d.path.clone()).collect(),
                        completeness_percentage: 0.4, // Estimated based on single short document
                        critical_gaps: vec![
                            "Process steps".to_string(),
                            "Prerequisites".to_string(),
                        ],
                    });
                }
            }
        }

        Ok(incomplete_processes)
    }

    /// Identify outdated content that needs updates
    async fn identify_outdated_content(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<OutdatedContent>> {
        let mut outdated_content = Vec::new();
        let now = Utc::now();

        for document in documents {
            let modified_time = document
                .metadata
                .basic
                .modified
                .and_then(|sys_time| {
                    DateTime::<Utc>::from_timestamp(
                        sys_time
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        0,
                    )
                })
                .unwrap_or_else(|| Utc::now() - chrono::Duration::days(365)); // Default to 1 year ago if no modified date

            let age_days = now.signed_duration_since(modified_time).num_days();

            // Consider content outdated based on various indicators
            let mut outdated_indicators = Vec::new();
            let mut update_urgency = UpdateUrgency::Low;

            if age_days > 365 {
                outdated_indicators.push("Document is over 1 year old".to_string());
                update_urgency = UpdateUrgency::Medium;
            }

            // Check for outdated technology references
            if self.contains_outdated_references(&document.content) {
                outdated_indicators
                    .push("Contains potentially outdated technology references".to_string());
                update_urgency = UpdateUrgency::High;
            }

            // Check for broken links or references
            if self.contains_broken_references(&document.content) {
                outdated_indicators.push("Contains broken or invalid references".to_string());
                update_urgency = UpdateUrgency::Medium;
            }

            if !outdated_indicators.is_empty() {
                outdated_content.push(OutdatedContent {
                    document_path: document.path.clone(),
                    outdated_indicators,
                    last_modified: modified_time,
                    age_days,
                    update_urgency,
                    suggested_updates: vec![
                        "Review and update technical references".to_string(),
                        "Verify all links and references".to_string(),
                        "Check for new best practices".to_string(),
                    ],
                });
            }
        }

        Ok(outdated_content)
    }

    /// Analyze reference gaps and broken links
    async fn analyze_reference_gaps(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<ReferenceGap>> {
        let mut reference_gaps = Vec::new();

        for document in documents {
            // Check for common reference patterns that might be broken
            let potential_refs = self.extract_potential_references(&document.content);

            for reference in potential_refs {
                if self.is_likely_broken_reference(&reference) {
                    reference_gaps.push(ReferenceGap {
                        source_document: document.path.clone(),
                        missing_reference: reference.clone(),
                        gap_type: ReferenceGapType::BrokenInternalLink,
                        context: "Referenced in document but target not found".to_string(),
                        suggested_resolution: format!("Create or fix reference to {}", reference),
                    });
                }
            }
        }

        Ok(reference_gaps)
    }

    /// Generate expert recommendations based on analysis
    async fn generate_expert_recommendations(
        &self,
        analysis: &KnowledgeGapAnalysis,
        _config: &KnowledgeAnalysisConfig,
    ) -> Result<Vec<ExpertRecommendation>> {
        let mut recommendations = Vec::new();

        // Generate recommendations based on gaps found
        if !analysis.knowledge_gaps.is_empty() {
            recommendations.push(ExpertRecommendation {
                expertise_area: "Content Architecture".to_string(),
                recommendation: "Implement a content audit process to regularly identify and address knowledge gaps".to_string(),
                justification: format!("Found {} knowledge gaps that could impact user experience", analysis.knowledge_gaps.len()),
                priority: RecommendationPriority::High,
                impact: BusinessImpact::High,
            });
        }

        if !analysis.missing_document_types.is_empty() {
            recommendations.push(ExpertRecommendation {
                expertise_area: "Documentation Strategy".to_string(),
                recommendation: "Create a documentation template library for missing document types".to_string(),
                justification: format!("Identified {} missing document types that would improve completeness", analysis.missing_document_types.len()),
                priority: RecommendationPriority::Medium,
                impact: BusinessImpact::Medium,
            });
        }

        if !analysis.outdated_content.is_empty() {
            recommendations.push(ExpertRecommendation {
                expertise_area: "Content Maintenance".to_string(),
                recommendation: "Establish a content freshness monitoring and update schedule"
                    .to_string(),
                justification: format!(
                    "Found {} documents with outdated content",
                    analysis.outdated_content.len()
                ),
                priority: RecommendationPriority::High,
                impact: BusinessImpact::High,
            });
        }

        Ok(recommendations)
    }

    /// Identify priority areas for improvement
    fn identify_priority_areas(&self, analysis: &KnowledgeGapAnalysis) -> Vec<PriorityArea> {
        let mut priority_areas = Vec::new();

        // Group gaps by type and calculate priority scores
        let mut gap_types: HashMap<String, usize> = HashMap::new();
        for gap in &analysis.knowledge_gaps {
            let type_name = format!("{:?}", gap.gap_type);
            *gap_types.entry(type_name).or_insert(0) += 1;
        }

        for (gap_type, count) in gap_types {
            let priority_score = count as f32 * 10.0; // Simple scoring based on frequency

            priority_areas.push(PriorityArea {
                area_name: gap_type.clone(),
                priority_score,
                gap_count: count,
                potential_impact: format!("Addressing {} gaps in {} area", count, gap_type),
                recommended_actions: vec![
                    format!("Focus on {} improvements", gap_type),
                    "Create content templates for this area".to_string(),
                    "Assign subject matter expert".to_string(),
                ],
            });
        }

        // Sort by priority score (highest first)
        priority_areas.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());

        priority_areas
    }

    /// Calculate overall completeness score
    fn calculate_completeness_score(&self, analysis: &KnowledgeGapAnalysis) -> f32 {
        let mut score_factors = Vec::new();

        // Base score from number of gaps relative to documents
        let gap_ratio = if analysis.documents_analyzed > 0 {
            analysis.knowledge_gaps.len() as f32 / analysis.documents_analyzed as f32
        } else {
            1.0
        };
        score_factors.push(1.0 - gap_ratio.min(1.0));

        // Missing document types factor
        let missing_types_penalty = analysis.missing_document_types.len() as f32 * 0.1;
        score_factors.push(1.0 - missing_types_penalty.min(1.0));

        // Outdated content factor
        let outdated_penalty = analysis.outdated_content.len() as f32 * 0.05;
        score_factors.push(1.0 - outdated_penalty.min(1.0));

        // Process completeness factor
        let process_penalty = analysis.incomplete_processes.len() as f32 * 0.15;
        score_factors.push(1.0 - process_penalty.min(1.0));

        // Average the factors and convert to 0-100 scale
        let avg_score = score_factors.iter().sum::<f32>() / score_factors.len() as f32;
        (avg_score * 100.0).clamp(0.0, 100.0)
    }

    // Helper methods

    fn get_expected_sections(&self, title: &str, content: &str) -> Vec<String> {
        let mut sections = Vec::new();

        let title_lower = title.to_lowercase();
        let content_lower = content.to_lowercase();

        if title_lower.contains("guide") || title_lower.contains("tutorial") {
            sections.extend(vec![
                "Prerequisites".to_string(),
                "Step-by-step instructions".to_string(),
                "Examples".to_string(),
                "Troubleshooting".to_string(),
            ]);
        }

        if title_lower.contains("api") || content_lower.contains("endpoint") {
            sections.extend(vec![
                "Authentication".to_string(),
                "Request format".to_string(),
                "Response format".to_string(),
                "Error codes".to_string(),
            ]);
        }

        if title_lower.contains("install") || title_lower.contains("setup") {
            sections.extend(vec![
                "System requirements".to_string(),
                "Installation steps".to_string(),
                "Configuration".to_string(),
                "Verification".to_string(),
            ]);
        }

        sections
    }

    fn classify_document_type(&self, title: &str, content: &str) -> String {
        let title_lower = title.to_lowercase();
        let content_lower = content.to_lowercase();

        if title_lower.contains("api") || content_lower.contains("endpoint") {
            "API Documentation".to_string()
        } else if title_lower.contains("guide") || title_lower.contains("tutorial") {
            "Guide".to_string()
        } else if title_lower.contains("faq") || content_lower.contains("frequently asked") {
            "FAQ".to_string()
        } else if title_lower.contains("troubleshoot") || content_lower.contains("problem") {
            "Troubleshooting".to_string()
        } else if title_lower.contains("install") || title_lower.contains("setup") {
            "Installation Guide".to_string()
        } else {
            "General Documentation".to_string()
        }
    }

    fn extract_process_name(&self, title: &str, _content: &str) -> String {
        // Extract process name from title
        let words: Vec<&str> = title.split_whitespace().collect();
        if words.len() > 2 {
            words[0..2].join(" ")
        } else {
            title.to_string()
        }
    }

    fn identify_missing_process_aspects(&self, content: &str) -> Vec<String> {
        let mut missing = Vec::new();
        let content_lower = content.to_lowercase();

        if !content_lower.contains("prerequisite") && !content_lower.contains("requirement") {
            missing.push("Prerequisites".to_string());
        }

        if !content_lower.contains("step") && !content_lower.contains("procedure") {
            missing.push("Step-by-step instructions".to_string());
        }

        if !content_lower.contains("example") && !content_lower.contains("sample") {
            missing.push("Examples".to_string());
        }

        if !content_lower.contains("troubleshoot") && !content_lower.contains("problem") {
            missing.push("Troubleshooting information".to_string());
        }

        missing
    }

    fn contains_outdated_references(&self, content: &str) -> bool {
        let outdated_patterns = [
            "internet explorer",
            "windows xp",
            "python 2.",
            "flash player",
            "java 6",
            "java 7",
            "php 5.",
        ];

        let content_lower = content.to_lowercase();
        outdated_patterns
            .iter()
            .any(|pattern| content_lower.contains(pattern))
    }

    fn contains_broken_references(&self, content: &str) -> bool {
        // Simple check for common broken reference patterns
        content.contains("TODO")
            || content.contains("FIXME")
            || content.contains("[broken]")
            || content.contains("404")
    }

    fn extract_potential_references(&self, content: &str) -> Vec<String> {
        let mut references = Vec::new();

        // Extract markdown-style links
        for line in content.lines() {
            if line.contains("[") && line.contains("]") && line.contains("(") {
                // This is a simplified extraction - could be enhanced
                references.push(line.trim().to_string());
            }
        }

        references
    }

    fn is_likely_broken_reference(&self, reference: &str) -> bool {
        // Simple heuristic for broken references
        reference.contains("TODO") || reference.contains("PLACEHOLDER") || reference.contains("###")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_knowledge_analyzer_creation() {
        let indexer = Arc::new(Mutex::new(
            crate::document::indexer::DocumentIndexer::new("/tmp/test".into()).unwrap(),
        ));
        let relationship_analyzer = Arc::new(Mutex::new(
            crate::document::relationship_analyzer::RelationshipAnalyzer::new(
                crate::document::relationship_analyzer::RelationshipConfig::default(),
            ),
        ));
        let workspace_intelligence =
            crate::workspace::intelligence::WorkspaceIntelligence::new(indexer.clone(), None);

        let analyzer =
            KnowledgeAnalyzer::new(workspace_intelligence, indexer, relationship_analyzer, None);

        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_knowledge_analysis_config_default() {
        let config = KnowledgeAnalysisConfig::default();
        assert!(config.enable_deep_analysis);
        assert!(config.analyze_dependencies);
        assert!(config.check_freshness);
        assert_eq!(config.confidence_threshold, 0.7);
        assert_eq!(config.analysis_depth, 80);
    }

    #[test]
    fn test_knowledge_gap_types() {
        let gap_type = KnowledgeGapType::MissingProcedure;
        assert_eq!(gap_type, KnowledgeGapType::MissingProcedure);

        let severity = GapSeverity::Critical;
        assert_eq!(severity, GapSeverity::Critical);
    }

    #[test]
    fn test_business_impact_levels() {
        let impact = BusinessImpact::Critical;
        assert_eq!(impact, BusinessImpact::Critical);

        let effort = ImplementationEffort::Medium;
        assert_eq!(effort, ImplementationEffort::Medium);
    }
}
