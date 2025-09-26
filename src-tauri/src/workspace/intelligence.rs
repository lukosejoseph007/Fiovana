// src-tauri/src/workspace/intelligence.rs
//! Workspace-level intelligence analysis
//!
//! This module provides AI-powered workspace analysis capabilities to identify
//! patterns, gaps, and generate intelligent recommendations for workspace organization
//! and content management.

use super::{WorkspaceInfo, WorkspaceResult};
use crate::ai::AIOrchestrator;
use crate::document::indexer::{DocumentIndexEntry, DocumentIndexer};
use crate::document::relationship_analyzer::{
    DocumentRelationship, RelationshipAnalyzer, RelationshipConfig,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Workspace intelligence analyzer for generating insights and recommendations
pub struct WorkspaceIntelligence {
    document_indexer: Arc<Mutex<DocumentIndexer>>,
    relationship_analyzer: Arc<Mutex<RelationshipAnalyzer>>,
    _ai_orchestrator: Option<Arc<Mutex<AIOrchestrator>>>,
}

/// Comprehensive workspace analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAnalysis {
    pub workspace_path: PathBuf,
    pub analysis_timestamp: DateTime<Utc>,
    pub document_overview: DocumentOverview,
    pub content_patterns: ContentPatterns,
    pub knowledge_gaps: Vec<KnowledgeGap>,
    pub organization_insights: OrganizationInsights,
    pub recommendations: Vec<WorkspaceRecommendation>,
    pub productivity_metrics: ProductivityMetrics,
    pub quality_assessment: QualityAssessment,
}

/// Overview of documents in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOverview {
    pub total_documents: usize,
    pub documents_by_type: HashMap<String, usize>,
    pub documents_by_directory: HashMap<String, usize>,
    pub average_document_size: f64,
    pub total_content_size: usize,
    pub creation_timeline: Vec<DocumentCreationPoint>,
    pub most_recent_activity: Option<DateTime<Utc>>,
}

/// Content patterns identified across the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPatterns {
    pub dominant_topics: Vec<TopicCluster>,
    pub document_relationships: Vec<DocumentRelationship>,
    pub duplicate_content: Vec<DuplicateContentGroup>,
    pub content_distribution: ContentDistribution,
    pub language_patterns: LanguagePatterns,
}

/// Identified knowledge gaps in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub gap_type: KnowledgeGapType,
    pub description: String,
    pub affected_topics: Vec<String>,
    pub severity: GapSeverity,
    pub suggested_actions: Vec<String>,
    pub priority_score: f64,
}

/// Types of knowledge gaps that can be identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeGapType {
    MissingDocumentation,
    OutdatedContent,
    IncompleteTopicCoverage,
    LackOfExamples,
    MissingReferences,
    OrganizationalGap,
    ProcessGap,
}

/// Severity level of knowledge gaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Organizational insights about the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInsights {
    pub directory_utilization: HashMap<String, DirectoryMetrics>,
    pub naming_consistency: ConsistencyAnalysis,
    pub file_organization_score: f64,
    pub redundancy_analysis: RedundancyAnalysis,
    pub accessibility_score: f64,
}

/// Workspace improvement recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_effort: EffortLevel,
    pub expected_impact: ImpactLevel,
    pub actionable_steps: Vec<String>,
    pub affected_files: Option<Vec<PathBuf>>,
}

/// Types of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    Organization,
    ContentCreation,
    ContentUpdate,
    Cleanup,
    ProcessImprovement,
    QualityImprovement,
}

/// Priority levels for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Effort estimation for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,   // < 1 hour
    Low,       // 1-4 hours
    Medium,    // 1-2 days
    High,      // 1 week
    Extensive, // > 1 week
}

/// Expected impact of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Transformative,
}

/// Productivity metrics for the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub content_creation_velocity: f64,
    pub update_frequency: f64,
    pub collaboration_indicators: CollaborationMetrics,
    pub workspace_maturity_score: f64,
    pub efficiency_score: f64,
}

/// Quality assessment of workspace content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub overall_quality_score: f64,
    pub completeness_score: f64,
    pub consistency_score: f64,
    pub currency_score: f64,
    pub quality_by_category: HashMap<String, f64>,
    pub quality_issues: Vec<QualityIssue>,
}

// Supporting data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreationPoint {
    pub date: DateTime<Utc>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCluster {
    pub topic_name: String,
    pub document_count: usize,
    pub confidence: f64,
    pub keywords: Vec<String>,
    pub representative_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateContentGroup {
    pub similarity_score: f64,
    pub document_paths: Vec<PathBuf>,
    pub content_type: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDistribution {
    pub by_content_type: HashMap<String, f64>,
    pub by_complexity: HashMap<String, f64>,
    pub by_purpose: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePatterns {
    pub dominant_language: String,
    pub technical_complexity: f64,
    pub readability_scores: HashMap<String, f64>,
    pub tone_analysis: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryMetrics {
    pub file_count: usize,
    pub total_size: usize,
    pub last_activity: Option<DateTime<Utc>>,
    pub organization_score: f64,
    pub usage_frequency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyAnalysis {
    pub naming_convention_score: f64,
    pub structural_consistency_score: f64,
    pub inconsistencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedundancyAnalysis {
    pub duplicate_files: Vec<PathBuf>,
    pub similar_content_groups: Vec<DuplicateContentGroup>,
    pub redundancy_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMetrics {
    pub multiple_contributors_detected: bool,
    pub update_patterns: HashMap<String, usize>,
    pub collaborative_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub issue_type: String,
    pub severity: String,
    pub affected_documents: Vec<PathBuf>,
    pub description: String,
    pub suggested_fix: String,
}

impl WorkspaceIntelligence {
    /// Create a new workspace intelligence analyzer
    pub fn new(
        document_indexer: Arc<Mutex<DocumentIndexer>>,
        ai_orchestrator: Option<Arc<Mutex<AIOrchestrator>>>,
    ) -> Self {
        let relationship_config = RelationshipConfig::default();
        let relationship_analyzer =
            Arc::new(Mutex::new(RelationshipAnalyzer::new(relationship_config)));

        Self {
            document_indexer,
            relationship_analyzer,
            _ai_orchestrator: ai_orchestrator,
        }
    }

    /// Perform comprehensive workspace analysis
    pub async fn analyze_workspace(
        &self,
        workspace_info: &WorkspaceInfo,
    ) -> WorkspaceResult<WorkspaceAnalysis> {
        let analysis_start = Utc::now();
        println!(
            "🧠 Starting comprehensive workspace analysis for: {}",
            workspace_info.name
        );

        // Get all documents in the workspace
        let documents = self.get_workspace_documents(&workspace_info.path).await?;

        if documents.is_empty() {
            return Ok(self.create_empty_workspace_analysis(&workspace_info.path, analysis_start));
        }

        // Perform parallel analysis of different aspects
        let (document_overview, content_patterns) = tokio::try_join!(
            self.analyze_document_overview(&documents),
            self.analyze_content_patterns(&documents)
        )?;

        // Sequential analysis that depends on previous results
        let knowledge_gaps = self
            .identify_knowledge_gaps(&documents, &content_patterns)
            .await?;
        let organization_insights = self
            .analyze_organization(&workspace_info.path, &documents)
            .await?;
        let productivity_metrics = self
            .calculate_productivity_metrics(&documents, &workspace_info.path)
            .await?;
        let quality_assessment = self.assess_content_quality(&documents).await?;

        // Generate AI-powered recommendations
        let recommendations = self
            .generate_recommendations(
                &document_overview,
                &content_patterns,
                &knowledge_gaps,
                &organization_insights,
                &quality_assessment,
            )
            .await?;

        let analysis = WorkspaceAnalysis {
            workspace_path: workspace_info.path.clone(),
            analysis_timestamp: analysis_start,
            document_overview,
            content_patterns,
            knowledge_gaps,
            organization_insights,
            recommendations,
            productivity_metrics,
            quality_assessment,
        };

        println!(
            "✅ Workspace analysis completed in {:.2}s",
            (Utc::now() - analysis_start).num_milliseconds() as f64 / 1000.0
        );

        Ok(analysis)
    }

    /// Get all documents in a workspace
    async fn get_workspace_documents(
        &self,
        workspace_path: &Path,
    ) -> WorkspaceResult<Vec<DocumentIndexEntry>> {
        let indexer = self.document_indexer.lock().await;

        let all_docs = indexer.get_all_documents();

        // Filter documents that belong to this workspace
        let workspace_docs: Vec<DocumentIndexEntry> = all_docs
            .iter()
            .filter(|doc| {
                Path::new(&doc.path)
                    .canonicalize()
                    .map(|p| p.starts_with(workspace_path))
                    .unwrap_or(false)
            })
            .map(|doc| (*doc).clone())
            .collect();

        Ok(workspace_docs)
    }

    /// Create analysis for empty workspace
    fn create_empty_workspace_analysis(
        &self,
        workspace_path: &Path,
        timestamp: DateTime<Utc>,
    ) -> WorkspaceAnalysis {
        WorkspaceAnalysis {
            workspace_path: workspace_path.to_path_buf(),
            analysis_timestamp: timestamp,
            document_overview: DocumentOverview {
                total_documents: 0,
                documents_by_type: HashMap::new(),
                documents_by_directory: HashMap::new(),
                average_document_size: 0.0,
                total_content_size: 0,
                creation_timeline: Vec::new(),
                most_recent_activity: None,
            },
            content_patterns: ContentPatterns {
                dominant_topics: Vec::new(),
                document_relationships: Vec::new(),
                duplicate_content: Vec::new(),
                content_distribution: ContentDistribution {
                    by_content_type: HashMap::new(),
                    by_complexity: HashMap::new(),
                    by_purpose: HashMap::new(),
                },
                language_patterns: LanguagePatterns {
                    dominant_language: "unknown".to_string(),
                    technical_complexity: 0.0,
                    readability_scores: HashMap::new(),
                    tone_analysis: HashMap::new(),
                },
            },
            knowledge_gaps: vec![KnowledgeGap {
                gap_type: KnowledgeGapType::MissingDocumentation,
                description: "Workspace is empty - no documents found".to_string(),
                affected_topics: Vec::new(),
                severity: GapSeverity::High,
                suggested_actions: vec![
                    "Import initial documents into sources/imports/".to_string(),
                    "Create README.md to document workspace purpose".to_string(),
                    "Add reference materials to sources/references/".to_string(),
                ],
                priority_score: 0.9,
            }],
            organization_insights: OrganizationInsights {
                directory_utilization: HashMap::new(),
                naming_consistency: ConsistencyAnalysis {
                    naming_convention_score: 1.0,
                    structural_consistency_score: 1.0,
                    inconsistencies: Vec::new(),
                },
                file_organization_score: 0.0,
                redundancy_analysis: RedundancyAnalysis {
                    duplicate_files: Vec::new(),
                    similar_content_groups: Vec::new(),
                    redundancy_score: 0.0,
                },
                accessibility_score: 1.0,
            },
            recommendations: vec![WorkspaceRecommendation {
                recommendation_type: RecommendationType::ContentCreation,
                title: "Initialize Workspace Content".to_string(),
                description:
                    "This workspace is currently empty. Start by importing some documents."
                        .to_string(),
                priority: RecommendationPriority::High,
                estimated_effort: EffortLevel::Minimal,
                expected_impact: ImpactLevel::High,
                actionable_steps: vec![
                    "Use File Management to import documents".to_string(),
                    "Organize documents into appropriate directories".to_string(),
                    "Create a workspace README".to_string(),
                ],
                affected_files: None,
            }],
            productivity_metrics: ProductivityMetrics {
                content_creation_velocity: 0.0,
                update_frequency: 0.0,
                collaboration_indicators: CollaborationMetrics {
                    multiple_contributors_detected: false,
                    update_patterns: HashMap::new(),
                    collaborative_documents: Vec::new(),
                },
                workspace_maturity_score: 0.0,
                efficiency_score: 0.0,
            },
            quality_assessment: QualityAssessment {
                overall_quality_score: 0.0,
                completeness_score: 0.0,
                consistency_score: 1.0,
                currency_score: 0.0,
                quality_by_category: HashMap::new(),
                quality_issues: Vec::new(),
            },
        }
    }

    /// Analyze document overview
    async fn analyze_document_overview(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<DocumentOverview> {
        let mut documents_by_type = HashMap::new();
        let mut documents_by_directory = HashMap::new();
        let mut total_size = 0;
        let mut creation_timeline = HashMap::new();
        let mut most_recent_activity = None;

        for doc in documents {
            // Count by document type
            let type_name = self.format_document_type(&doc.structure.document_type);
            *documents_by_type.entry(type_name).or_insert(0) += 1;

            // Count by directory
            if let Some(parent) = Path::new(&doc.path).parent() {
                let dir_name = parent
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                *documents_by_directory.entry(dir_name).or_insert(0) += 1;
            }

            // Calculate total size
            total_size += doc.content.len();

            // Track creation timeline (convert SystemTime to DateTime)
            let datetime = DateTime::<Utc>::from(doc.indexed_at);
            let indexed_date = datetime.date_naive();
            *creation_timeline.entry(indexed_date).or_insert(0) += 1;

            // Track most recent activity
            match most_recent_activity {
                Some(current) if datetime > current => {
                    most_recent_activity = Some(datetime);
                }
                None => {
                    most_recent_activity = Some(datetime);
                }
                _ => {}
            }
        }

        let average_size = if documents.is_empty() {
            0.0
        } else {
            total_size as f64 / documents.len() as f64
        };

        // Convert timeline to sorted vec
        let mut timeline_points: Vec<DocumentCreationPoint> = creation_timeline
            .into_iter()
            .map(|(date, count)| DocumentCreationPoint {
                date: date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                count,
            })
            .collect();
        timeline_points.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(DocumentOverview {
            total_documents: documents.len(),
            documents_by_type,
            documents_by_directory,
            average_document_size: average_size,
            total_content_size: total_size,
            creation_timeline: timeline_points,
            most_recent_activity,
        })
    }

    /// Analyze content patterns across documents
    async fn analyze_content_patterns(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<ContentPatterns> {
        // Analyze topics using keyword clustering
        let dominant_topics = self.identify_topic_clusters(documents).await?;

        // Analyze document relationships
        let document_relationships = self.analyze_document_relationships(documents).await?;

        // Find duplicate content
        let duplicate_content = self.find_duplicate_content(documents).await?;

        // Analyze content distribution
        let content_distribution = self.analyze_content_distribution(documents).await?;

        // Analyze language patterns
        let language_patterns = self.analyze_language_patterns(documents).await?;

        Ok(ContentPatterns {
            dominant_topics,
            document_relationships,
            duplicate_content,
            content_distribution,
            language_patterns,
        })
    }

    /// Identify topic clusters from document keywords
    async fn identify_topic_clusters(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<Vec<TopicCluster>> {
        let mut keyword_counts = HashMap::new();
        let mut doc_keywords = HashMap::new();

        // Collect all keywords and track which documents contain them
        for (doc_idx, doc) in documents.iter().enumerate() {
            doc_keywords.insert(doc_idx, doc.keywords.clone());

            for keyword in &doc.keywords {
                let keyword_lower = keyword.to_lowercase();
                keyword_counts
                    .entry(keyword_lower.clone())
                    .or_insert_with(Vec::new)
                    .push(doc_idx);
            }
        }

        // Find clusters of related keywords (simple co-occurrence analysis)
        let mut clusters = Vec::new();
        let mut processed_keywords = HashSet::new();

        for (keyword, doc_indices) in &keyword_counts {
            if processed_keywords.contains(keyword) || doc_indices.len() < 2 {
                continue;
            }

            // Find related keywords that co-occur frequently
            let mut cluster_keywords = vec![keyword.clone()];
            let mut cluster_docs: HashSet<usize> = HashSet::from_iter(doc_indices.clone());

            // Simple clustering: find keywords that appear in at least 50% of the same documents
            for (other_keyword, other_docs) in &keyword_counts {
                if processed_keywords.contains(other_keyword) || other_keyword == keyword {
                    continue;
                }

                let intersection: HashSet<_> = doc_indices
                    .iter()
                    .collect::<HashSet<_>>()
                    .intersection(&other_docs.iter().collect::<HashSet<_>>())
                    .cloned()
                    .collect();

                let similarity =
                    intersection.len() as f64 / doc_indices.len().max(other_docs.len()) as f64;

                if similarity >= 0.5 {
                    cluster_keywords.push(other_keyword.clone());
                    cluster_docs.extend(other_docs);
                    processed_keywords.insert(other_keyword.clone());
                }
            }

            processed_keywords.insert(keyword.clone());

            // Create representative documents list
            let mut representative_docs = Vec::new();
            for &doc_idx in cluster_docs.iter().take(3) {
                if let Some(doc) = documents.get(doc_idx) {
                    let title = if !doc.title.is_empty() {
                        doc.title.clone()
                    } else {
                        Path::new(&doc.path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    };
                    representative_docs.push(title);
                }
            }

            clusters.push(TopicCluster {
                topic_name: cluster_keywords[0].clone(),
                document_count: cluster_docs.len(),
                confidence: doc_indices.len() as f64 / documents.len() as f64,
                keywords: cluster_keywords,
                representative_documents: representative_docs,
            });
        }

        // Sort by document count (most prevalent topics first)
        clusters.sort_by(|a, b| b.document_count.cmp(&a.document_count));
        clusters.truncate(10); // Keep top 10 topics

        Ok(clusters)
    }

    /// Analyze document relationships using the relationship analyzer
    async fn analyze_document_relationships(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<Vec<DocumentRelationship>> {
        if documents.len() < 2 {
            return Ok(Vec::new());
        }

        let analyzer = self.relationship_analyzer.lock().await;

        // Analyze relationships between all documents (limit to avoid combinatorial explosion)
        let max_docs = 20; // Analyze relationships for up to 20 documents
        let docs_to_analyze = if documents.len() > max_docs {
            &documents[..max_docs]
        } else {
            documents
        };

        let mut all_relationships = Vec::new();

        for (i, doc_a) in docs_to_analyze.iter().enumerate() {
            for doc_b in docs_to_analyze.iter().skip(i + 1) {
                if let Ok(relationships) = analyzer.analyze_document_pair(doc_a, doc_b).await {
                    all_relationships.extend(relationships);
                }
            }
        }

        // Sort by strength (convert enum to numeric for comparison)
        all_relationships.sort_by(|a, b| {
            let strength_a = match a.strength {
                crate::document::relationship_analyzer::RelationshipStrength::VeryStrong => 4,
                crate::document::relationship_analyzer::RelationshipStrength::Strong => 3,
                crate::document::relationship_analyzer::RelationshipStrength::Moderate => 2,
                crate::document::relationship_analyzer::RelationshipStrength::Weak => 1,
            };
            let strength_b = match b.strength {
                crate::document::relationship_analyzer::RelationshipStrength::VeryStrong => 4,
                crate::document::relationship_analyzer::RelationshipStrength::Strong => 3,
                crate::document::relationship_analyzer::RelationshipStrength::Moderate => 2,
                crate::document::relationship_analyzer::RelationshipStrength::Weak => 1,
            };
            strength_b.cmp(&strength_a)
        });
        all_relationships.truncate(50); // Keep top 50 relationships

        Ok(all_relationships)
    }

    /// Find duplicate or very similar content
    async fn find_duplicate_content(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<Vec<DuplicateContentGroup>> {
        let mut duplicates = Vec::new();

        // Simple content similarity check based on size and keywords
        for (i, doc_a) in documents.iter().enumerate() {
            let mut similar_docs = Vec::new();

            for doc_b in documents.iter().skip(i + 1) {
                // Check size similarity
                let size_diff = (doc_a.content.len() as i64 - doc_b.content.len() as i64).abs();
                let size_similarity =
                    1.0 - (size_diff as f64 / doc_a.content.len().max(doc_b.content.len()) as f64);

                // Check keyword overlap
                let keywords_a: HashSet<&String> = doc_a.keywords.iter().collect();
                let keywords_b: HashSet<&String> = doc_b.keywords.iter().collect();
                let intersection: HashSet<_> = keywords_a.intersection(&keywords_b).collect();
                let union: HashSet<_> = keywords_a.union(&keywords_b).collect();

                let keyword_similarity = if union.is_empty() {
                    0.0
                } else {
                    intersection.len() as f64 / union.len() as f64
                };

                let overall_similarity = (size_similarity + keyword_similarity) / 2.0;

                if overall_similarity > 0.7 {
                    similar_docs.push((doc_b.path.clone(), overall_similarity));
                }
            }

            if !similar_docs.is_empty() {
                let mut paths = vec![PathBuf::from(&doc_a.path)];
                let max_similarity = similar_docs.iter().map(|(_, sim)| *sim).fold(0.0, f64::max);

                paths.extend(similar_docs.into_iter().map(|(path, _)| path));

                duplicates.push(DuplicateContentGroup {
                    similarity_score: max_similarity,
                    document_paths: paths,
                    content_type: "text".to_string(),
                    recommendation: if max_similarity > 0.9 {
                        "Consider consolidating these nearly identical documents".to_string()
                    } else {
                        "Review these similar documents for potential consolidation".to_string()
                    },
                });
            }
        }

        Ok(duplicates)
    }

    /// Analyze content distribution patterns
    async fn analyze_content_distribution(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<ContentDistribution> {
        let mut by_content_type = HashMap::new();
        let mut by_complexity = HashMap::new();
        let mut by_purpose = HashMap::new();

        let total_docs = documents.len() as f64;

        for doc in documents {
            // Analyze by document type
            let doc_type = self.format_document_type(&doc.structure.document_type);
            *by_content_type.entry(doc_type).or_insert(0.0) += 1.0 / total_docs;

            // Analyze complexity based on structure
            let complexity = if doc.structure.sections.len() > 10 {
                "high"
            } else if doc.structure.sections.len() > 3 {
                "medium"
            } else {
                "low"
            };
            *by_complexity.entry(complexity.to_string()).or_insert(0.0) += 1.0 / total_docs;

            // Analyze purpose based on directory location
            let path_str = doc.path.to_string_lossy();
            let purpose = if path_str.contains("imports") {
                "source"
            } else if path_str.contains("references") {
                "reference"
            } else if path_str.contains("drafts") {
                "draft"
            } else if path_str.contains("approved") {
                "final"
            } else {
                "other"
            };
            *by_purpose.entry(purpose.to_string()).or_insert(0.0) += 1.0 / total_docs;
        }

        Ok(ContentDistribution {
            by_content_type,
            by_complexity,
            by_purpose,
        })
    }

    /// Analyze language patterns in the workspace
    async fn analyze_language_patterns(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<LanguagePatterns> {
        if documents.is_empty() {
            return Ok(LanguagePatterns {
                dominant_language: "unknown".to_string(),
                technical_complexity: 0.0,
                readability_scores: HashMap::new(),
                tone_analysis: HashMap::new(),
            });
        }

        // Simple language detection based on content
        let total_words: usize = documents
            .iter()
            .map(|doc| doc.content.split_whitespace().count())
            .sum();

        let avg_word_length: f64 = documents
            .iter()
            .flat_map(|doc| doc.content.split_whitespace())
            .map(|word| word.len())
            .sum::<usize>() as f64
            / total_words.max(1) as f64;

        // Calculate technical complexity based on technical keywords
        let technical_keywords = [
            "algorithm",
            "implementation",
            "configuration",
            "deployment",
            "architecture",
            "infrastructure",
            "optimization",
            "integration",
            "specification",
            "protocol",
        ];

        let technical_word_count = documents
            .iter()
            .flat_map(|doc| {
                let lowercase_content = doc.content.to_lowercase();
                lowercase_content
                    .split_whitespace()
                    .map(|word| word.to_string())
                    .collect::<Vec<String>>()
            })
            .filter(|word| technical_keywords.contains(&word.as_str()))
            .count();

        let technical_complexity = technical_word_count as f64 / total_words.max(1) as f64;

        let mut readability_scores = HashMap::new();
        readability_scores.insert("avg_word_length".to_string(), avg_word_length);
        readability_scores.insert("avg_sentence_length".to_string(), 15.0); // Placeholder

        let mut tone_analysis = HashMap::new();
        tone_analysis.insert("formal".to_string(), 0.7); // Placeholder
        tone_analysis.insert("technical".to_string(), technical_complexity);

        Ok(LanguagePatterns {
            dominant_language: "english".to_string(), // Simplified assumption
            technical_complexity,
            readability_scores,
            tone_analysis,
        })
    }

    /// Identify knowledge gaps in the workspace
    async fn identify_knowledge_gaps(
        &self,
        documents: &[DocumentIndexEntry],
        content_patterns: &ContentPatterns,
    ) -> WorkspaceResult<Vec<KnowledgeGap>> {
        let mut gaps = Vec::new();

        // Check for topic coverage gaps
        if content_patterns.dominant_topics.len() < 3 {
            gaps.push(KnowledgeGap {
                gap_type: KnowledgeGapType::IncompleteTopicCoverage,
                description: "Limited topic diversity detected. Workspace may benefit from broader content coverage.".to_string(),
                affected_topics: content_patterns.dominant_topics.iter().map(|t| t.topic_name.clone()).collect(),
                severity: GapSeverity::Medium,
                suggested_actions: vec![
                    "Identify additional topic areas relevant to your workspace purpose".to_string(),
                    "Import documents covering complementary subjects".to_string(),
                    "Consider creating overview documents to bridge topic gaps".to_string(),
                ],
                priority_score: 0.6,
            });
        }

        // Check for outdated content (simplified - based on document age)
        let now = Utc::now();
        let old_documents: Vec<_> = documents
            .iter()
            .filter(|doc| {
                let doc_datetime = DateTime::<Utc>::from(doc.indexed_at);
                (now - doc_datetime).num_days() > 180 // 6 months old
            })
            .collect();

        if !old_documents.is_empty() {
            gaps.push(KnowledgeGap {
                gap_type: KnowledgeGapType::OutdatedContent,
                description: format!(
                    "Found {} documents that may be outdated (older than 6 months)",
                    old_documents.len()
                ),
                affected_topics: Vec::new(),
                severity: if old_documents.len() > documents.len() / 2 {
                    GapSeverity::High
                } else {
                    GapSeverity::Medium
                },
                suggested_actions: vec![
                    "Review older documents for currency and relevance".to_string(),
                    "Update or archive outdated content".to_string(),
                    "Establish a regular content review schedule".to_string(),
                ],
                priority_score: 0.7,
            });
        }

        // Check for missing examples/procedures
        let has_procedures = documents.iter().any(|doc| {
            doc.content.to_lowercase().contains("procedure")
                || doc.content.to_lowercase().contains("step")
                || doc.content.to_lowercase().contains("how to")
        });

        if !has_procedures {
            gaps.push(KnowledgeGap {
                gap_type: KnowledgeGapType::LackOfExamples,
                description:
                    "No procedural or how-to content detected. Consider adding practical examples."
                        .to_string(),
                affected_topics: Vec::new(),
                severity: GapSeverity::Medium,
                suggested_actions: vec![
                    "Create step-by-step procedural documents".to_string(),
                    "Add practical examples to existing documentation".to_string(),
                    "Document common workflows and processes".to_string(),
                ],
                priority_score: 0.5,
            });
        }

        Ok(gaps)
    }

    /// Analyze workspace organization
    async fn analyze_organization(
        &self,
        workspace_path: &Path,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<OrganizationInsights> {
        let mut directory_utilization = HashMap::new();
        let mut all_files = HashSet::new();

        // Analyze directory usage
        for doc in documents {
            if let Some(parent) = Path::new(&doc.path).parent() {
                let dir_name = parent
                    .strip_prefix(workspace_path)
                    .unwrap_or(parent)
                    .to_string_lossy()
                    .to_string();

                let entry = directory_utilization
                    .entry(dir_name)
                    .or_insert(DirectoryMetrics {
                        file_count: 0,
                        total_size: 0,
                        last_activity: None,
                        organization_score: 0.0,
                        usage_frequency: 0.0,
                    });

                entry.file_count += 1;
                entry.total_size += doc.content.len();

                // Convert SystemTime to DateTime for consistency
                let datetime = DateTime::<Utc>::from(doc.indexed_at);
                match entry.last_activity {
                    Some(current) if datetime > current => {
                        entry.last_activity = Some(datetime);
                    }
                    None => {
                        entry.last_activity = Some(datetime);
                    }
                    _ => {}
                }

                all_files.insert(doc.path.clone());
            }
        }

        // Calculate organization scores
        for metrics in directory_utilization.values_mut() {
            metrics.organization_score = if metrics.file_count > 0 { 0.8 } else { 0.0 };
            metrics.usage_frequency = metrics.file_count as f64 / documents.len().max(1) as f64;
        }

        // Analyze naming consistency (simplified)
        let mut consistent_names = 0;
        let mut total_names = 0;

        for doc in documents {
            total_names += 1;
            let filename = Path::new(&doc.path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();

            // Check for consistent patterns (simplified heuristics)
            if filename.contains('_') || filename.contains('-') {
                consistent_names += 1;
            }
        }

        let naming_score = if total_names > 0 {
            consistent_names as f64 / total_names as f64
        } else {
            1.0
        };

        // Calculate overall file organization score
        let file_organization_score = directory_utilization
            .values()
            .map(|m| m.organization_score)
            .sum::<f64>()
            / directory_utilization.len().max(1) as f64;

        // Redundancy analysis (simplified)
        let redundancy_score = 1.0
            - (documents.len().saturating_sub(all_files.len()) as f64
                / documents.len().max(1) as f64);

        Ok(OrganizationInsights {
            directory_utilization,
            naming_consistency: ConsistencyAnalysis {
                naming_convention_score: naming_score,
                structural_consistency_score: 0.8, // Placeholder
                inconsistencies: Vec::new(),
            },
            file_organization_score,
            redundancy_analysis: RedundancyAnalysis {
                duplicate_files: Vec::new(),
                similar_content_groups: Vec::new(),
                redundancy_score,
            },
            accessibility_score: 0.8, // Placeholder
        })
    }

    /// Calculate productivity metrics
    async fn calculate_productivity_metrics(
        &self,
        documents: &[DocumentIndexEntry],
        _workspace_path: &Path,
    ) -> WorkspaceResult<ProductivityMetrics> {
        if documents.is_empty() {
            return Ok(ProductivityMetrics {
                content_creation_velocity: 0.0,
                update_frequency: 0.0,
                collaboration_indicators: CollaborationMetrics {
                    multiple_contributors_detected: false,
                    update_patterns: HashMap::new(),
                    collaborative_documents: Vec::new(),
                },
                workspace_maturity_score: 0.0,
                efficiency_score: 0.0,
            });
        }

        // Calculate content creation velocity (docs per week)
        let oldest_doc = documents.iter().min_by_key(|doc| doc.indexed_at).unwrap();

        let time_span = {
            let oldest_datetime = DateTime::<Utc>::from(oldest_doc.indexed_at);
            Utc::now() - oldest_datetime
        };

        let weeks = time_span.num_weeks().max(1) as f64;
        let velocity = documents.len() as f64 / weeks;

        // Simple collaboration detection
        let multiple_contributors = documents.len() > 5; // Simplified heuristic

        let mut update_patterns = HashMap::new();
        for doc in documents {
            let datetime = DateTime::<Utc>::from(doc.indexed_at);
            let month = datetime.format("%Y-%m").to_string();
            *update_patterns.entry(month).or_insert(0) += 1;
        }

        // Maturity score based on document count, diversity, and organization
        let maturity_score = ((documents.len() as f64).ln() / 5.0).min(1.0);

        // Efficiency score based on organization and redundancy
        let efficiency_score = 0.7; // Placeholder

        Ok(ProductivityMetrics {
            content_creation_velocity: velocity,
            update_frequency: 1.0 / weeks, // Updates per week
            collaboration_indicators: CollaborationMetrics {
                multiple_contributors_detected: multiple_contributors,
                update_patterns,
                collaborative_documents: Vec::new(),
            },
            workspace_maturity_score: maturity_score,
            efficiency_score,
        })
    }

    /// Assess content quality
    async fn assess_content_quality(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> WorkspaceResult<QualityAssessment> {
        if documents.is_empty() {
            return Ok(QualityAssessment {
                overall_quality_score: 0.0,
                completeness_score: 0.0,
                consistency_score: 1.0,
                currency_score: 0.0,
                quality_by_category: HashMap::new(),
                quality_issues: Vec::new(),
            });
        }

        // Calculate completeness score based on document structure
        let completeness_score = documents
            .iter()
            .map(|doc| {
                let has_title = !doc.title.is_empty();
                let has_sections = !doc.structure.sections.is_empty();
                let has_keywords = !doc.keywords.is_empty();
                let has_content = doc.content.len() > 100;

                let score = [has_title, has_sections, has_keywords, has_content]
                    .iter()
                    .map(|&b| if b { 1.0 } else { 0.0 })
                    .sum::<f64>()
                    / 4.0;

                score
            })
            .sum::<f64>()
            / documents.len() as f64;

        // Calculate consistency score
        let avg_section_count = documents
            .iter()
            .map(|doc| doc.structure.sections.len())
            .sum::<usize>() as f64
            / documents.len() as f64;

        let section_variance = documents
            .iter()
            .map(|doc| (doc.structure.sections.len() as f64 - avg_section_count).powi(2))
            .sum::<f64>()
            / documents.len() as f64;

        let consistency_score = (1.0 / (1.0 + section_variance / 10.0)).max(0.0);

        // Calculate currency score based on document age
        let now = Utc::now();
        let currency_score = documents
            .iter()
            .map(|doc| {
                let doc_datetime = DateTime::<Utc>::from(doc.indexed_at);
                let days_old = (now - doc_datetime).num_days();
                (1.0 - (days_old as f64 / 365.0)).max(0.0f64) // Decay over 1 year
            })
            .sum::<f64>()
            / documents.len() as f64;

        let overall_quality_score = (completeness_score + consistency_score + currency_score) / 3.0;

        // Identify quality issues
        let mut quality_issues = Vec::new();

        // Check for documents without titles
        let untitled_docs: Vec<_> = documents
            .iter()
            .filter(|doc| doc.title.is_empty())
            .map(|doc| PathBuf::from(&doc.path))
            .collect();

        if !untitled_docs.is_empty() {
            quality_issues.push(QualityIssue {
                issue_type: "Missing Titles".to_string(),
                severity: "Medium".to_string(),
                affected_documents: untitled_docs,
                description: "Some documents lack proper titles".to_string(),
                suggested_fix: "Add descriptive titles to these documents".to_string(),
            });
        }

        Ok(QualityAssessment {
            overall_quality_score,
            completeness_score,
            consistency_score,
            currency_score,
            quality_by_category: HashMap::new(),
            quality_issues,
        })
    }

    /// Generate AI-powered recommendations
    async fn generate_recommendations(
        &self,
        document_overview: &DocumentOverview,
        content_patterns: &ContentPatterns,
        knowledge_gaps: &[KnowledgeGap],
        organization_insights: &OrganizationInsights,
        quality_assessment: &QualityAssessment,
    ) -> WorkspaceResult<Vec<WorkspaceRecommendation>> {
        let mut recommendations = Vec::new();

        // Organization recommendations
        if organization_insights.file_organization_score < 0.7 {
            recommendations.push(WorkspaceRecommendation {
                recommendation_type: RecommendationType::Organization,
                title: "Improve File Organization".to_string(),
                description: "Your workspace could benefit from better file organization"
                    .to_string(),
                priority: RecommendationPriority::Medium,
                estimated_effort: EffortLevel::Low,
                expected_impact: ImpactLevel::Medium,
                actionable_steps: vec![
                    "Review file naming conventions".to_string(),
                    "Organize files into appropriate directories".to_string(),
                    "Create consistent folder structure".to_string(),
                ],
                affected_files: None,
            });
        }

        // Content creation recommendations
        if document_overview.total_documents < 5 {
            recommendations.push(WorkspaceRecommendation {
                recommendation_type: RecommendationType::ContentCreation,
                title: "Expand Content Collection".to_string(),
                description:
                    "Consider adding more documents to build a comprehensive knowledge base"
                        .to_string(),
                priority: RecommendationPriority::Medium,
                estimated_effort: EffortLevel::Medium,
                expected_impact: ImpactLevel::High,
                actionable_steps: vec![
                    "Import additional relevant documents".to_string(),
                    "Create documentation for key topics".to_string(),
                    "Add reference materials".to_string(),
                ],
                affected_files: None,
            });
        }

        // Quality improvement recommendations
        if quality_assessment.overall_quality_score < 0.7 {
            recommendations.push(WorkspaceRecommendation {
                recommendation_type: RecommendationType::QualityImprovement,
                title: "Improve Content Quality".to_string(),
                description: "Several quality issues were identified in your workspace content"
                    .to_string(),
                priority: RecommendationPriority::High,
                estimated_effort: EffortLevel::Medium,
                expected_impact: ImpactLevel::High,
                actionable_steps: vec![
                    "Review and update document titles".to_string(),
                    "Ensure all documents have proper structure".to_string(),
                    "Update outdated content".to_string(),
                ],
                affected_files: None,
            });
        }

        // Duplicate content recommendations
        if !content_patterns.duplicate_content.is_empty() {
            let duplicate_files: Vec<PathBuf> = content_patterns
                .duplicate_content
                .iter()
                .flat_map(|group| group.document_paths.clone())
                .collect();

            recommendations.push(WorkspaceRecommendation {
                recommendation_type: RecommendationType::Cleanup,
                title: "Address Duplicate Content".to_string(),
                description: format!(
                    "Found {} groups of potentially duplicate content",
                    content_patterns.duplicate_content.len()
                ),
                priority: RecommendationPriority::Medium,
                estimated_effort: EffortLevel::Low,
                expected_impact: ImpactLevel::Medium,
                actionable_steps: vec![
                    "Review flagged duplicate content".to_string(),
                    "Consolidate or remove redundant documents".to_string(),
                    "Update cross-references as needed".to_string(),
                ],
                affected_files: Some(duplicate_files),
            });
        }

        // Convert knowledge gaps to recommendations
        for gap in knowledge_gaps {
            let priority = match gap.severity {
                GapSeverity::Critical => RecommendationPriority::Urgent,
                GapSeverity::High => RecommendationPriority::High,
                GapSeverity::Medium => RecommendationPriority::Medium,
                GapSeverity::Low => RecommendationPriority::Low,
            };

            recommendations.push(WorkspaceRecommendation {
                recommendation_type: RecommendationType::ContentCreation,
                title: format!(
                    "Address {}",
                    gap.description.split('.').next().unwrap_or("Knowledge Gap")
                ),
                description: gap.description.clone(),
                priority,
                estimated_effort: EffortLevel::Medium,
                expected_impact: ImpactLevel::Medium,
                actionable_steps: gap.suggested_actions.clone(),
                affected_files: None,
            });
        }

        // Sort recommendations by priority and expected impact
        recommendations.sort_by(|a, b| {
            let priority_order = |p: &RecommendationPriority| match p {
                RecommendationPriority::Urgent => 0,
                RecommendationPriority::High => 1,
                RecommendationPriority::Medium => 2,
                RecommendationPriority::Low => 3,
            };

            priority_order(&a.priority)
                .cmp(&priority_order(&b.priority))
                .then_with(|| {
                    let impact_order = |i: &ImpactLevel| match i {
                        ImpactLevel::Transformative => 0,
                        ImpactLevel::High => 1,
                        ImpactLevel::Medium => 2,
                        ImpactLevel::Low => 3,
                    };
                    impact_order(&a.expected_impact).cmp(&impact_order(&b.expected_impact))
                })
        });

        Ok(recommendations)
    }

    /// Format document type for display
    fn format_document_type(&self, doc_type: &crate::document::indexer::DocumentType) -> String {
        use crate::document::indexer::DocumentType;
        match doc_type {
            DocumentType::Manual => "Manual".to_string(),
            DocumentType::Guide => "Guide".to_string(),
            DocumentType::Procedure => "Procedure".to_string(),
            DocumentType::Reference => "Reference".to_string(),
            DocumentType::Training => "Training".to_string(),
            DocumentType::Policy => "Policy".to_string(),
            DocumentType::Template => "Template".to_string(),
            DocumentType::Other(s) => format!("Other ({})", s),
        }
    }
}
