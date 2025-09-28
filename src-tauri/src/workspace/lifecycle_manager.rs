// src-tauri/src/workspace/lifecycle_manager.rs
//! Content lifecycle management system for tracking document age, usage, and update needs
//!
//! This module provides intelligent lifecycle management capabilities that track document
//! usage patterns, age, relevance, and automatically suggest updates, archival, or consolidation
//! to maintain current and relevant documentation.

use super::intelligence::WorkspaceIntelligence;
use crate::ai::AIOrchestrator;
use crate::document::indexer::DocumentIndexer;
use crate::document::relationship_analyzer::RelationshipAnalyzer;
use crate::document::{ContentClassifier, StructureAnalyzer};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Content lifecycle manager for tracking and managing document lifecycles
pub struct LifecycleManager {
    #[allow(dead_code)]
    workspace_intelligence: WorkspaceIntelligence,
    document_indexer: Arc<Mutex<DocumentIndexer>>,
    #[allow(dead_code)]
    relationship_analyzer: Arc<Mutex<RelationshipAnalyzer>>,
    content_classifier: ContentClassifier,
    structure_analyzer: StructureAnalyzer,
    #[allow(dead_code)]
    ai_orchestrator: Option<Arc<Mutex<AIOrchestrator>>>,
    usage_tracker: UsageTracker,
}

/// Configuration for lifecycle management operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConfig {
    /// Enable automated lifecycle analysis
    pub enable_automated_analysis: bool,
    /// Track document usage patterns
    pub track_usage: bool,
    /// Enable update recommendations
    pub enable_update_recommendations: bool,
    /// Enable archival suggestions
    pub enable_archival_suggestions: bool,
    /// Enable consolidation recommendations
    pub enable_consolidation_suggestions: bool,
    /// Age threshold for considering documents stale (days)
    pub stale_threshold_days: i64,
    /// Minimum usage threshold for archival consideration
    pub min_usage_threshold: u32,
    /// Confidence threshold for recommendations
    pub confidence_threshold: f32,
}

/// Comprehensive lifecycle analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleAnalysis {
    /// Workspace path analyzed
    pub workspace_path: PathBuf,
    /// Timestamp of analysis
    pub analysis_timestamp: DateTime<Utc>,
    /// Overall workspace health score (0-100)
    pub health_score: f32,
    /// Documents requiring updates
    pub update_recommendations: Vec<UpdateRecommendation>,
    /// Documents suggested for archival
    pub archival_suggestions: Vec<ArchivalSuggestion>,
    /// Documents suggested for consolidation
    pub consolidation_suggestions: Vec<ConsolidationSuggestion>,
    /// Lifecycle stage distribution
    pub lifecycle_distribution: LifecycleDistribution,
    /// Usage pattern analysis
    pub usage_patterns: UsagePatternAnalysis,
    /// Content freshness analysis
    pub freshness_analysis: FreshnessAnalysis,
    /// Priority actions for lifecycle management
    pub priority_actions: Vec<LifecycleAction>,
    /// Analysis configuration used
    pub config: LifecycleConfig,
    /// Number of documents analyzed
    pub documents_analyzed: usize,
    /// Analysis duration in seconds
    pub analysis_duration: f64,
}

/// Document update recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecommendation {
    /// Document path
    pub document_path: PathBuf,
    /// Recommendation priority
    pub priority: UpdatePriority,
    /// Reasons for update recommendation
    pub update_reasons: Vec<String>,
    /// Confidence in recommendation (0-1)
    pub confidence: f32,
    /// Estimated effort required
    pub effort_estimate: EffortLevel,
    /// Suggested update actions
    pub suggested_actions: Vec<String>,
    /// Last modification date
    pub last_modified: DateTime<Utc>,
    /// Days since last update
    pub days_since_update: i64,
    /// Usage frequency
    pub usage_frequency: UsageFrequency,
    /// Content staleness indicators
    pub staleness_indicators: Vec<String>,
}

/// Document archival suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalSuggestion {
    /// Document path
    pub document_path: PathBuf,
    /// Archival rationale
    pub rationale: String,
    /// Confidence in suggestion (0-1)
    pub confidence: f32,
    /// Usage statistics
    pub usage_stats: DocumentUsageStats,
    /// Last access date
    pub last_accessed: Option<DateTime<Utc>>,
    /// Archival priority
    pub priority: ArchivalPriority,
    /// Suggested archival location
    pub suggested_location: Option<PathBuf>,
    /// Dependencies that might be affected
    pub affected_dependencies: Vec<PathBuf>,
}

/// Document consolidation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationSuggestion {
    /// Documents to consolidate
    pub documents: Vec<PathBuf>,
    /// Consolidation type
    pub consolidation_type: ConsolidationType,
    /// Rationale for consolidation
    pub rationale: String,
    /// Confidence in suggestion (0-1)
    pub confidence: f32,
    /// Suggested consolidated document path
    pub suggested_path: PathBuf,
    /// Estimated consolidation effort
    pub effort_estimate: EffortLevel,
    /// Expected benefits
    pub expected_benefits: Vec<String>,
}

/// Lifecycle stage distribution across workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleDistribution {
    /// Active documents (regularly used and current)
    pub active: usize,
    /// Maintenance documents (need updates)
    pub maintenance: usize,
    /// Stale documents (outdated but potentially useful)
    pub stale: usize,
    /// Deprecated documents (should be archived)
    pub deprecated: usize,
    /// Legacy documents (historical value only)
    pub legacy: usize,
}

/// Usage pattern analysis across workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatternAnalysis {
    /// High-usage documents
    pub high_usage_count: usize,
    /// Medium-usage documents
    pub medium_usage_count: usize,
    /// Low-usage documents
    pub low_usage_count: usize,
    /// Unused documents
    pub unused_count: usize,
    /// Average usage frequency
    pub average_usage: f32,
    /// Usage trend over time
    pub usage_trend: UsageTrend,
}

/// Content freshness analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessAnalysis {
    /// Fresh documents (recently updated)
    pub fresh_count: usize,
    /// Aging documents (moderately outdated)
    pub aging_count: usize,
    /// Stale documents (significantly outdated)
    pub stale_count: usize,
    /// Critical documents (urgently need updates)
    pub critical_count: usize,
    /// Average age in days
    pub average_age_days: f64,
    /// Freshness score (0-100)
    pub freshness_score: f32,
}

/// Lifecycle management action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleAction {
    /// Action identifier
    pub action_id: String,
    /// Type of lifecycle action
    pub action_type: LifecycleActionType,
    /// Action priority
    pub priority: ActionPriority,
    /// Action description
    pub description: String,
    /// Affected documents
    pub affected_documents: Vec<PathBuf>,
    /// Implementation steps
    pub implementation_steps: Vec<String>,
    /// Estimated effort
    pub estimated_effort: EffortLevel,
    /// Expected impact
    pub expected_impact: ImpactLevel,
    /// Due date recommendation
    pub recommended_due_date: Option<DateTime<Utc>>,
}

/// Document usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUsageStats {
    /// Total access count
    pub access_count: u32,
    /// Access count in last 30 days
    pub recent_access_count: u32,
    /// Last access date
    pub last_accessed: Option<DateTime<Utc>>,
    /// Average accesses per month
    pub monthly_average: f32,
    /// Usage frequency classification
    pub frequency: UsageFrequency,
    /// Usage trend
    pub trend: UsageTrend,
}

/// Usage tracking system
#[derive(Debug, Clone, Default)]
pub struct UsageTracker {
    /// Document access history
    access_history: HashMap<PathBuf, Vec<DateTime<Utc>>>,
    /// Usage statistics cache
    stats_cache: HashMap<PathBuf, DocumentUsageStats>,
    /// Last cache update
    last_cache_update: Option<DateTime<Utc>>,
}

/// Supporting enums and types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdatePriority {
    Critical, // Must update immediately
    High,     // Update within days
    Medium,   // Update within weeks
    Low,      // Update when convenient
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArchivalPriority {
    Immediate, // Archive ASAP
    High,      // Archive within month
    Medium,    // Archive within quarter
    Low,       // Archive when convenient
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsolidationType {
    Merge,     // Combine similar documents
    Refactor,  // Restructure related content
    Supersede, // Replace with newer version
    Aggregate, // Collect scattered information
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UsageFrequency {
    VeryHigh, // Multiple times per day
    High,     // Daily
    Medium,   // Weekly
    Low,      // Monthly
    VeryLow,  // Rarely
    Unused,   // Never accessed
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UsageTrend {
    Increasing, // Usage going up
    Stable,     // Consistent usage
    Decreasing, // Usage declining
    Sporadic,   // Irregular usage
    Inactive,   // No recent usage
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecycleActionType {
    Update,      // Update document content
    Archive,     // Move to archive
    Consolidate, // Merge/combine documents
    Review,      // Manual review needed
    Deprecate,   // Mark as deprecated
    Migrate,     // Move to new location
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionPriority {
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,   // < 1 hour
    Low,       // 1-4 hours
    Medium,    // 4-16 hours
    High,      // 1-3 days
    Extensive, // > 3 days
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactLevel {
    Critical, // Business-critical impact
    High,     // Significant improvement
    Medium,   // Moderate benefit
    Low,      // Minor improvement
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            enable_automated_analysis: true,
            track_usage: true,
            enable_update_recommendations: true,
            enable_archival_suggestions: true,
            enable_consolidation_suggestions: true,
            stale_threshold_days: 180, // 6 months
            min_usage_threshold: 1,    // At least 1 access
            confidence_threshold: 0.7,
        }
    }
}

impl LifecycleManager {
    /// Create a new lifecycle manager
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
            usage_tracker: UsageTracker::default(),
        })
    }

    /// Perform comprehensive lifecycle analysis
    pub async fn analyze_lifecycle(
        &mut self,
        workspace_path: &Path,
        config: LifecycleConfig,
    ) -> Result<LifecycleAnalysis> {
        let start_time = std::time::Instant::now();

        let document_count;
        let documents = {
            let indexer = self.document_indexer.lock().await;
            let docs = indexer.get_all_documents();
            document_count = docs.len();
            docs.into_iter().cloned().collect::<Vec<_>>()
        };

        let mut analysis = LifecycleAnalysis {
            workspace_path: workspace_path.to_path_buf(),
            analysis_timestamp: Utc::now(),
            health_score: 0.0,
            update_recommendations: Vec::new(),
            archival_suggestions: Vec::new(),
            consolidation_suggestions: Vec::new(),
            lifecycle_distribution: LifecycleDistribution {
                active: 0,
                maintenance: 0,
                stale: 0,
                deprecated: 0,
                legacy: 0,
            },
            usage_patterns: UsagePatternAnalysis {
                high_usage_count: 0,
                medium_usage_count: 0,
                low_usage_count: 0,
                unused_count: 0,
                average_usage: 0.0,
                usage_trend: UsageTrend::Stable,
            },
            freshness_analysis: FreshnessAnalysis {
                fresh_count: 0,
                aging_count: 0,
                stale_count: 0,
                critical_count: 0,
                average_age_days: 0.0,
                freshness_score: 0.0,
            },
            priority_actions: Vec::new(),
            config: config.clone(),
            documents_analyzed: document_count,
            analysis_duration: 0.0,
        };

        // Update usage tracker with current documents
        self.update_usage_tracking(&documents).await?;

        // Analyze update needs
        if config.enable_update_recommendations {
            analysis.update_recommendations =
                self.analyze_update_needs(&documents, &config).await?;
        }

        // Analyze archival opportunities
        if config.enable_archival_suggestions {
            analysis.archival_suggestions = self
                .analyze_archival_opportunities(&documents, &config)
                .await?;
        }

        // Analyze consolidation opportunities
        if config.enable_consolidation_suggestions {
            analysis.consolidation_suggestions = self
                .analyze_consolidation_opportunities(&documents, &config)
                .await?;
        }

        // Analyze lifecycle distribution
        analysis.lifecycle_distribution = self
            .analyze_lifecycle_distribution(&documents, &config)
            .await?;

        // Analyze usage patterns
        analysis.usage_patterns = self.analyze_usage_patterns(&documents, &config).await?;

        // Analyze content freshness
        analysis.freshness_analysis = self.analyze_content_freshness(&documents, &config).await?;

        // Generate priority actions
        analysis.priority_actions = self.generate_priority_actions(&analysis);

        // Calculate overall health score
        analysis.health_score = self.calculate_health_score(&analysis);

        analysis.analysis_duration = start_time.elapsed().as_secs_f64();
        Ok(analysis)
    }

    /// Update usage tracking for documents
    async fn update_usage_tracking(
        &mut self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
    ) -> Result<()> {
        let now = Utc::now();

        // Update access history for tracked documents
        for document in documents {
            // Simulate usage tracking - in a real implementation, this would be based on actual access logs
            let usage_stats = self.calculate_document_usage_stats(&document.path).await?;
            self.usage_tracker
                .stats_cache
                .insert(document.path.clone(), usage_stats);
        }

        self.usage_tracker.last_cache_update = Some(now);
        Ok(())
    }

    /// Calculate usage statistics for a document
    async fn calculate_document_usage_stats(&self, _path: &Path) -> Result<DocumentUsageStats> {
        // In a real implementation, this would read from access logs, analytics, etc.
        // For now, simulate realistic usage patterns
        let now = Utc::now();
        let access_count = fastrand::u32(0..=100);
        let recent_access_count = fastrand::u32(0..=20);

        let frequency = match access_count {
            0 => UsageFrequency::Unused,
            1..=5 => UsageFrequency::VeryLow,
            6..=20 => UsageFrequency::Low,
            21..=50 => UsageFrequency::Medium,
            51..=80 => UsageFrequency::High,
            _ => UsageFrequency::VeryHigh,
        };

        let trend = if recent_access_count > access_count / 3 {
            UsageTrend::Increasing
        } else if recent_access_count < access_count / 6 {
            UsageTrend::Decreasing
        } else {
            UsageTrend::Stable
        };

        Ok(DocumentUsageStats {
            access_count,
            recent_access_count,
            last_accessed: if access_count > 0 {
                Some(now - Duration::days(fastrand::i64(1..=90)))
            } else {
                None
            },
            monthly_average: access_count as f32 / 12.0,
            frequency,
            trend,
        })
    }

    /// Analyze documents that need updates
    async fn analyze_update_needs(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        config: &LifecycleConfig,
    ) -> Result<Vec<UpdateRecommendation>> {
        let mut recommendations = Vec::new();
        let now = Utc::now();

        for document in documents {
            let modified_time = self.get_document_modified_time(document);
            let days_since_update = now.signed_duration_since(modified_time).num_days();

            let mut update_reasons = Vec::new();
            let mut staleness_indicators = Vec::new();
            let mut priority = UpdatePriority::Low;
            let mut confidence = 0.5;

            // Age-based analysis
            if days_since_update > config.stale_threshold_days {
                update_reasons.push("Document is older than staleness threshold".to_string());
                staleness_indicators.push(format!("Last updated {} days ago", days_since_update));
                priority = UpdatePriority::Medium;
                confidence += 0.2;
            }

            // Content analysis
            if let Ok(structure_analysis) =
                self.structure_analyzer.analyze_structure(&document.content)
            {
                if let Ok(content_analysis) = self
                    .content_classifier
                    .analyze_document_content(&structure_analysis, &document.content)
                    .await
                {
                    // Check content complexity and completeness
                    if content_analysis.complexity_score < 0.3 && document.content.len() < 1000 {
                        update_reasons
                            .push("Content appears incomplete or lacks detail".to_string());
                        confidence += 0.1;
                    }
                }
            }

            // Technology and reference checks
            if self.contains_outdated_technology_references(&document.content) {
                update_reasons.push("Contains outdated technology references".to_string());
                staleness_indicators.push("Outdated technology references detected".to_string());
                priority = UpdatePriority::High;
                confidence += 0.3;
            }

            if self.contains_broken_links(&document.content) {
                update_reasons.push("Contains broken links or references".to_string());
                staleness_indicators.push("Broken links detected".to_string());
                confidence += 0.2;
            }

            // Usage pattern analysis
            let usage_stats = self.calculate_document_usage_stats(&document.path).await?;
            let usage_frequency = usage_stats.frequency;

            // High-usage documents get higher priority for updates
            if matches!(
                usage_frequency,
                UsageFrequency::High | UsageFrequency::VeryHigh
            ) {
                if priority == UpdatePriority::Low {
                    priority = UpdatePriority::Medium;
                }
                confidence += 0.1;
            }

            // Generate recommendation if confidence threshold is met
            if confidence >= config.confidence_threshold && !update_reasons.is_empty() {
                recommendations.push(UpdateRecommendation {
                    document_path: document.path.clone(),
                    priority,
                    update_reasons,
                    confidence,
                    effort_estimate: self
                        .estimate_update_effort(&document.content, &staleness_indicators),
                    suggested_actions: self.generate_update_actions(&staleness_indicators),
                    last_modified: modified_time,
                    days_since_update,
                    usage_frequency,
                    staleness_indicators,
                });
            }
        }

        // Sort by priority and confidence
        recommendations.sort_by(|a, b| {
            let priority_order = |p: &UpdatePriority| match p {
                UpdatePriority::Critical => 0,
                UpdatePriority::High => 1,
                UpdatePriority::Medium => 2,
                UpdatePriority::Low => 3,
            };

            priority_order(&a.priority)
                .cmp(&priority_order(&b.priority))
                .then(b.confidence.partial_cmp(&a.confidence).unwrap())
        });

        Ok(recommendations)
    }

    /// Analyze documents for archival opportunities
    async fn analyze_archival_opportunities(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        config: &LifecycleConfig,
    ) -> Result<Vec<ArchivalSuggestion>> {
        let mut suggestions = Vec::new();

        for document in documents {
            let usage_stats = self.calculate_document_usage_stats(&document.path).await?;

            // Consider for archival if usage is very low and document is old
            if usage_stats.access_count < config.min_usage_threshold {
                let modified_time = self.get_document_modified_time(document);
                let days_old = Utc::now().signed_duration_since(modified_time).num_days();

                if days_old > config.stale_threshold_days * 2 {
                    // 2x staleness threshold
                    let rationale = format!(
                        "Document has {} accesses and is {} days old with {} recent activity",
                        usage_stats.access_count,
                        days_old,
                        if usage_stats.recent_access_count == 0 {
                            "no"
                        } else {
                            "minimal"
                        }
                    );

                    let priority = match usage_stats.frequency {
                        UsageFrequency::Unused => ArchivalPriority::High,
                        UsageFrequency::VeryLow => ArchivalPriority::Medium,
                        _ => ArchivalPriority::Low,
                    };

                    suggestions.push(ArchivalSuggestion {
                        document_path: document.path.clone(),
                        rationale,
                        confidence: 0.8,
                        usage_stats,
                        last_accessed: None, // Would be populated from real usage data
                        priority,
                        suggested_location: Some(PathBuf::from("archive/").join(&document.path)),
                        affected_dependencies: Vec::new(), // Would be calculated from relationship analysis
                    });
                }
            }
        }

        Ok(suggestions)
    }

    /// Analyze documents for consolidation opportunities
    async fn analyze_consolidation_opportunities(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        _config: &LifecycleConfig,
    ) -> Result<Vec<ConsolidationSuggestion>> {
        let mut suggestions = Vec::new();

        // Group documents by topic/similarity for consolidation analysis
        let mut topic_groups: HashMap<String, Vec<&crate::document::indexer::DocumentIndexEntry>> =
            HashMap::new();

        for document in documents {
            let topic = self.extract_document_topic(&document.title, &document.content);
            topic_groups.entry(topic).or_default().push(document);
        }

        // Suggest consolidation for groups with multiple small documents
        for (topic, docs) in topic_groups {
            if docs.len() > 1 {
                let total_content_length: usize = docs.iter().map(|d| d.content.len()).sum();
                let avg_content_length = total_content_length / docs.len();

                // Suggest consolidation if documents are small and related
                if avg_content_length < 2000 && docs.len() >= 2 {
                    suggestions.push(ConsolidationSuggestion {
                        documents: docs.iter().map(|d| d.path.clone()).collect(),
                        consolidation_type: ConsolidationType::Merge,
                        rationale: format!(
                            "Multiple small documents ({}) on topic '{}' could be consolidated",
                            docs.len(),
                            topic
                        ),
                        confidence: 0.7,
                        suggested_path: PathBuf::from(format!(
                            "{}_consolidated.md",
                            topic.replace(' ', "_")
                        )),
                        effort_estimate: EffortLevel::Medium,
                        expected_benefits: vec![
                            "Improved document discovery".to_string(),
                            "Reduced maintenance overhead".to_string(),
                            "Better information coherence".to_string(),
                        ],
                    });
                }
            }
        }

        Ok(suggestions)
    }

    /// Analyze lifecycle stage distribution
    async fn analyze_lifecycle_distribution(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        config: &LifecycleConfig,
    ) -> Result<LifecycleDistribution> {
        let mut distribution = LifecycleDistribution {
            active: 0,
            maintenance: 0,
            stale: 0,
            deprecated: 0,
            legacy: 0,
        };

        let now = Utc::now();

        for document in documents {
            let modified_time = self.get_document_modified_time(document);
            let days_old = now.signed_duration_since(modified_time).num_days();
            let usage_stats = self.calculate_document_usage_stats(&document.path).await?;

            // Classify documents based on age and usage
            match (days_old, usage_stats.frequency) {
                (_, UsageFrequency::High | UsageFrequency::VeryHigh)
                    if days_old < config.stale_threshold_days =>
                {
                    distribution.active += 1;
                }
                (_, UsageFrequency::Medium) if days_old < config.stale_threshold_days * 2 => {
                    distribution.active += 1;
                }
                (days, _) if days < config.stale_threshold_days && usage_stats.access_count > 0 => {
                    distribution.maintenance += 1;
                }
                (days, _) if days < config.stale_threshold_days * 2 => {
                    distribution.stale += 1;
                }
                (_, UsageFrequency::Unused) => {
                    distribution.deprecated += 1;
                }
                _ => {
                    distribution.legacy += 1;
                }
            }
        }

        Ok(distribution)
    }

    /// Analyze usage patterns across workspace
    async fn analyze_usage_patterns(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        _config: &LifecycleConfig,
    ) -> Result<UsagePatternAnalysis> {
        let mut patterns = UsagePatternAnalysis {
            high_usage_count: 0,
            medium_usage_count: 0,
            low_usage_count: 0,
            unused_count: 0,
            average_usage: 0.0,
            usage_trend: UsageTrend::Stable,
        };

        let mut total_usage = 0u32;
        let mut increasing_trend_count = 0;
        let mut decreasing_trend_count = 0;

        for document in documents {
            let usage_stats = self.calculate_document_usage_stats(&document.path).await?;
            total_usage += usage_stats.access_count;

            match usage_stats.frequency {
                UsageFrequency::VeryHigh | UsageFrequency::High => patterns.high_usage_count += 1,
                UsageFrequency::Medium => patterns.medium_usage_count += 1,
                UsageFrequency::Low | UsageFrequency::VeryLow => patterns.low_usage_count += 1,
                UsageFrequency::Unused => patterns.unused_count += 1,
            }

            match usage_stats.trend {
                UsageTrend::Increasing => increasing_trend_count += 1,
                UsageTrend::Decreasing => decreasing_trend_count += 1,
                _ => {}
            }
        }

        patterns.average_usage = if documents.is_empty() {
            0.0
        } else {
            total_usage as f32 / documents.len() as f32
        };

        patterns.usage_trend = if increasing_trend_count > decreasing_trend_count * 2 {
            UsageTrend::Increasing
        } else if decreasing_trend_count > increasing_trend_count * 2 {
            UsageTrend::Decreasing
        } else {
            UsageTrend::Stable
        };

        Ok(patterns)
    }

    /// Analyze content freshness across workspace
    async fn analyze_content_freshness(
        &self,
        documents: &[crate::document::indexer::DocumentIndexEntry],
        config: &LifecycleConfig,
    ) -> Result<FreshnessAnalysis> {
        let mut freshness = FreshnessAnalysis {
            fresh_count: 0,
            aging_count: 0,
            stale_count: 0,
            critical_count: 0,
            average_age_days: 0.0,
            freshness_score: 0.0,
        };

        let now = Utc::now();
        let mut total_age_days = 0i64;

        for document in documents {
            let modified_time = self.get_document_modified_time(document);
            let age_days = now.signed_duration_since(modified_time).num_days();
            total_age_days += age_days;

            // Classify freshness based on age thresholds
            if age_days < 30 {
                freshness.fresh_count += 1;
            } else if age_days < config.stale_threshold_days / 2 {
                freshness.aging_count += 1;
            } else if age_days < config.stale_threshold_days {
                freshness.stale_count += 1;
            } else {
                freshness.critical_count += 1;
            }
        }

        freshness.average_age_days = if documents.is_empty() {
            0.0
        } else {
            total_age_days as f64 / documents.len() as f64
        };

        // Calculate freshness score (0-100)
        let total_docs = documents.len() as f32;
        if total_docs > 0.0 {
            freshness.freshness_score = ((freshness.fresh_count as f32 * 1.0
                + freshness.aging_count as f32 * 0.7
                + freshness.stale_count as f32 * 0.3
                + freshness.critical_count as f32 * 0.0)
                / total_docs)
                * 100.0;
        }

        Ok(freshness)
    }

    /// Generate priority actions for lifecycle management
    fn generate_priority_actions(&self, analysis: &LifecycleAnalysis) -> Vec<LifecycleAction> {
        let mut actions = Vec::new();

        // High-priority update actions
        for (index, recommendation) in analysis.update_recommendations.iter().enumerate() {
            if matches!(
                recommendation.priority,
                UpdatePriority::Critical | UpdatePriority::High
            ) {
                actions.push(LifecycleAction {
                    action_id: format!("update_{}", index),
                    action_type: LifecycleActionType::Update,
                    priority: match recommendation.priority {
                        UpdatePriority::Critical => ActionPriority::Urgent,
                        UpdatePriority::High => ActionPriority::High,
                        _ => ActionPriority::Medium,
                    },
                    description: format!(
                        "Update document: {}",
                        recommendation.document_path.display()
                    ),
                    affected_documents: vec![recommendation.document_path.clone()],
                    implementation_steps: recommendation.suggested_actions.clone(),
                    estimated_effort: recommendation.effort_estimate.clone(),
                    expected_impact: ImpactLevel::High,
                    recommended_due_date: Some(Utc::now() + Duration::days(7)),
                });
            }
        }

        // Archival actions
        for (index, suggestion) in analysis.archival_suggestions.iter().enumerate() {
            if matches!(
                suggestion.priority,
                ArchivalPriority::High | ArchivalPriority::Immediate
            ) {
                actions.push(LifecycleAction {
                    action_id: format!("archive_{}", index),
                    action_type: LifecycleActionType::Archive,
                    priority: ActionPriority::Medium,
                    description: format!(
                        "Archive unused document: {}",
                        suggestion.document_path.display()
                    ),
                    affected_documents: vec![suggestion.document_path.clone()],
                    implementation_steps: vec![
                        "Review document for dependencies".to_string(),
                        "Move to archive location".to_string(),
                        "Update references".to_string(),
                    ],
                    estimated_effort: EffortLevel::Low,
                    expected_impact: ImpactLevel::Medium,
                    recommended_due_date: Some(Utc::now() + Duration::days(30)),
                });
            }
        }

        // Consolidation actions
        for (index, suggestion) in analysis.consolidation_suggestions.iter().enumerate() {
            actions.push(LifecycleAction {
                action_id: format!("consolidate_{}", index),
                action_type: LifecycleActionType::Consolidate,
                priority: ActionPriority::Medium,
                description: format!(
                    "Consolidate {} related documents",
                    suggestion.documents.len()
                ),
                affected_documents: suggestion.documents.clone(),
                implementation_steps: vec![
                    "Review documents for consolidation".to_string(),
                    "Merge content logically".to_string(),
                    "Update cross-references".to_string(),
                    "Archive original documents".to_string(),
                ],
                estimated_effort: suggestion.effort_estimate.clone(),
                expected_impact: ImpactLevel::Medium,
                recommended_due_date: Some(Utc::now() + Duration::days(14)),
            });
        }

        // Sort by priority
        actions.sort_by(|a, b| {
            let priority_order = |p: &ActionPriority| match p {
                ActionPriority::Urgent => 0,
                ActionPriority::High => 1,
                ActionPriority::Medium => 2,
                ActionPriority::Low => 3,
            };

            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        actions
    }

    /// Calculate overall workspace health score
    fn calculate_health_score(&self, analysis: &LifecycleAnalysis) -> f32 {
        let mut score_factors = Vec::new();

        // Freshness factor (40% weight)
        score_factors.push(analysis.freshness_analysis.freshness_score * 0.4);

        // Usage factor (30% weight)
        let total_docs = analysis.documents_analyzed as f32;
        if total_docs > 0.0 {
            let active_ratio = analysis.lifecycle_distribution.active as f32 / total_docs;
            score_factors.push(active_ratio * 100.0 * 0.3);
        }

        // Maintenance factor (20% weight)
        let maintenance_penalty = if total_docs > 0.0 {
            (analysis.update_recommendations.len() as f32 / total_docs) * 100.0
        } else {
            0.0
        };
        score_factors.push((100.0 - maintenance_penalty).max(0.0) * 0.2);

        // Organization factor (10% weight)
        let consolidation_penalty = if total_docs > 0.0 {
            (analysis.consolidation_suggestions.len() as f32 / total_docs) * 50.0
        } else {
            0.0
        };
        score_factors.push((100.0 - consolidation_penalty).max(0.0) * 0.1);

        score_factors.iter().sum::<f32>().clamp(0.0, 100.0)
    }

    // Helper methods

    fn get_document_modified_time(
        &self,
        document: &crate::document::indexer::DocumentIndexEntry,
    ) -> DateTime<Utc> {
        document
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
            .unwrap_or_else(|| Utc::now() - Duration::days(365))
    }

    fn contains_outdated_technology_references(&self, content: &str) -> bool {
        let outdated_patterns = [
            "internet explorer",
            "windows xp",
            "python 2.",
            "flash player",
            "java 6",
            "java 7",
            "php 5.",
            "node 10",
            "angular 1.",
        ];

        let content_lower = content.to_lowercase();
        outdated_patterns
            .iter()
            .any(|pattern| content_lower.contains(pattern))
    }

    fn contains_broken_links(&self, content: &str) -> bool {
        content.contains("404")
            || content.contains("[broken]")
            || content.contains("TODO: add link")
            || content.contains("FIXME")
    }

    fn estimate_update_effort(&self, content: &str, indicators: &[String]) -> EffortLevel {
        let content_length = content.len();
        let indicator_count = indicators.len();

        match (content_length, indicator_count) {
            (_, count) if count > 5 => EffortLevel::Extensive,
            (len, count) if len > 10000 && count > 2 => EffortLevel::High,
            (len, count) if len > 5000 || count > 1 => EffortLevel::Medium,
            (len, _) if len > 1000 => EffortLevel::Low,
            _ => EffortLevel::Minimal,
        }
    }

    fn generate_update_actions(&self, indicators: &[String]) -> Vec<String> {
        let mut actions = vec!["Review document content".to_string()];

        for indicator in indicators {
            if indicator.contains("outdated") {
                actions.push("Update technology references".to_string());
            }
            if indicator.contains("broken") {
                actions.push("Fix broken links and references".to_string());
            }
            if indicator.contains("incomplete") {
                actions.push("Add missing content and details".to_string());
            }
        }

        if actions.len() == 1 {
            actions.push("Update content as needed".to_string());
        }

        actions.push("Verify accuracy and completeness".to_string());
        actions
    }

    fn extract_document_topic(&self, title: &str, content: &str) -> String {
        // Simple topic extraction based on title and content keywords
        let title_words: Vec<&str> = title.split_whitespace().take(2).collect();
        let topic = if title_words.len() >= 2 {
            title_words.join(" ")
        } else if !title_words.is_empty() {
            title_words[0].to_string()
        } else {
            // Extract from content
            let content_words: Vec<&str> = content.split_whitespace().take(10).collect();
            if content_words.len() >= 2 {
                content_words[0..2].join(" ")
            } else {
                "General".to_string()
            }
        };

        topic.to_lowercase()
    }

    /// Track document access for usage statistics
    pub async fn track_document_access(&mut self, document_path: &str) -> Result<()> {
        let path = PathBuf::from(document_path);
        let now = Utc::now();

        // Add access record to usage tracker
        self.usage_tracker
            .access_history
            .entry(path.clone())
            .or_default()
            .push(now);

        // Update statistics cache
        let usage_stats = self.calculate_document_usage_stats(&path).await?;
        self.usage_tracker.stats_cache.insert(path, usage_stats);
        self.usage_tracker.last_cache_update = Some(now);

        Ok(())
    }

    /// Get configuration for lifecycle management
    #[allow(dead_code)]
    pub fn get_config(&self) -> &LifecycleConfig {
        // For now, return default config since we don't store it yet
        static DEFAULT_CONFIG: std::sync::OnceLock<LifecycleConfig> = std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(LifecycleConfig::default)
    }

    /// Update configuration for lifecycle management
    #[allow(dead_code)]
    pub fn update_config(&mut self, _config: LifecycleConfig) {
        // For now, this is a no-op since we don't persist config yet
        // In the future, this would update the stored configuration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
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

        let manager =
            LifecycleManager::new(workspace_intelligence, indexer, relationship_analyzer, None);

        assert!(manager.is_ok());
    }

    #[test]
    fn test_lifecycle_config_default() {
        let config = LifecycleConfig::default();
        assert!(config.enable_automated_analysis);
        assert!(config.track_usage);
        assert_eq!(config.stale_threshold_days, 180);
        assert_eq!(config.confidence_threshold, 0.7);
    }

    #[test]
    fn test_usage_frequency_classification() {
        let freq = UsageFrequency::High;
        assert_eq!(freq, UsageFrequency::High);

        let trend = UsageTrend::Increasing;
        assert_eq!(trend, UsageTrend::Increasing);
    }

    #[test]
    fn test_lifecycle_action_priority() {
        let priority = ActionPriority::Urgent;
        assert_eq!(priority, ActionPriority::Urgent);

        let effort = EffortLevel::Medium;
        assert_eq!(effort, EffortLevel::Medium);
    }
}
