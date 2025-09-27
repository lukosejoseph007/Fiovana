// src-tauri/src/document/content_adapter.rs
// AI-powered content adaptation engine for different audiences and purposes

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::style_analyzer::{
    StyleAnalyzer, StyleProfile, StyleSimilarity, ToneType, VocabularyComplexity,
};
use crate::ai::{AIConfig, AIOrchestrator, PromptTemplates};

/// Target audience types for content adaptation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudienceType {
    General,       // General public
    Technical,     // Technical professionals
    Executive,     // Executives and decision makers
    Academic,      // Academic/research audience
    Student,       // Students and learners
    Expert,        // Domain experts
    Beginner,      // Beginners in the field
    Child,         // Children/young audience
    International, // International/non-native speakers
    Legal,         // Legal professionals
    Medical,       // Medical professionals
    Business,      // Business professionals
}

/// Content adaptation purposes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdaptationPurpose {
    Simplify,      // Make content simpler
    Formalize,     // Make content more formal
    Summarize,     // Create summary version
    Elaborate,     // Add more detail
    Translate,     // Language/cultural translation
    Modernize,     // Update for current audience
    Specialize,    // Make more domain-specific
    Generalize,    // Make more general
    Instruction,   // Convert to instructional format
    Reference,     // Convert to reference format
    Presentation,  // Convert for presentation
    Documentation, // Convert to documentation
}

/// Content complexity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Elementary,
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

/// Tone adjustment options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToneAdjustment {
    MoreFormal,
    LessFormal,
    MoreFriendly,
    MoreProfessional,
    MoreDirective,
    MoreSupportive,
    MoreEngaging,
    MoreNeutral,
}

/// Content adaptation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationConfig {
    pub target_audience: AudienceType,
    pub purpose: AdaptationPurpose,
    pub complexity_level: ComplexityLevel,
    pub tone_adjustments: Vec<ToneAdjustment>,
    pub preserve_technical_terms: bool,
    pub preserve_structure: bool,
    pub target_length_ratio: f64, // 1.0 = same length, 0.5 = half length, 2.0 = double length
    pub use_examples: bool,
    pub include_definitions: bool,
    pub style_consistency: bool,
    pub cultural_sensitivity: bool,
    // Style-aware generation options
    pub target_style_profile: Option<StyleProfile>,
    pub enforce_style_consistency: bool,
    pub style_matching_strength: f64, // 0.0 = loose, 1.0 = strict
    pub preserve_author_voice: bool,
    pub apply_style_patterns: bool,
}

/// Result of content adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationResult {
    pub adapted_content: String,
    pub adaptation_summary: String,
    pub changes_made: Vec<String>,
    pub quality_score: f64,
    pub readability_improvement: f64,
    pub audience_appropriateness: f64,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub processing_time_ms: u64,
    pub tokens_used: Option<u32>,
    // Style-aware generation results
    pub style_consistency_score: Option<f64>,
    pub style_patterns_applied: Vec<String>,
    pub style_similarity: Option<StyleSimilarity>,
    pub style_preservation_score: Option<f64>,
}

/// Content adaptation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationMetrics {
    pub original_word_count: usize,
    pub adapted_word_count: usize,
    pub complexity_change: f64,
    pub tone_shift_score: f64,
    pub readability_score: f64,
    pub technical_density_change: f64,
    pub structure_preservation: f64,
}

/// Content adaptation engine
pub struct ContentAdapter {
    ai_config: AIConfig,
    default_config: AdaptationConfig,
    audience_profiles: HashMap<AudienceType, AudienceProfile>,
    style_analyzer: StyleAnalyzer,
}

/// Audience profile for adaptation guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceProfile {
    pub preferred_complexity: ComplexityLevel,
    pub preferred_tone: ToneType,
    pub vocabulary_level: VocabularyComplexity,
    pub technical_tolerance: f64,
    pub attention_span: f64, // relative to general audience
    pub preferred_examples: bool,
    pub needs_definitions: bool,
    pub cultural_considerations: Vec<String>,
    pub typical_concerns: Vec<String>,
    pub communication_style: String,
}

impl ContentAdapter {
    pub fn new(ai_config: AIConfig) -> Self {
        let default_config = AdaptationConfig::default();
        let audience_profiles = Self::create_audience_profiles();
        let style_analyzer = StyleAnalyzer::new();

        Self {
            ai_config,
            default_config,
            audience_profiles,
            style_analyzer,
        }
    }

    /// Adapt content for a specific audience and purpose
    pub async fn adapt_content(
        &self,
        content: &str,
        config: Option<AdaptationConfig>,
    ) -> Result<AdaptationResult> {
        let start_time = std::time::Instant::now();
        let config = config.unwrap_or_else(|| self.default_config.clone());

        // Validate input
        if content.trim().is_empty() {
            return Err(anyhow!("Content cannot be empty"));
        }

        // Analyze original content
        let original_metrics = self.analyze_content_metrics(content)?;

        // Get audience profile
        let audience_profile = self.get_audience_profile(&config.target_audience);

        // Create adaptation prompt
        let adaptation_prompt =
            self.create_adaptation_prompt(content, &config, &audience_profile, &original_metrics)?;

        // Perform AI-powered adaptation
        let ai_orchestrator = AIOrchestrator::new(self.ai_config.clone())
            .await
            .context("Failed to initialize AI orchestrator")?;

        let ai_response = ai_orchestrator
            .process_conversation(&adaptation_prompt, None)
            .await
            .context("Failed to generate adapted content")?;

        // Parse and validate response
        let adapted_content = self.extract_adapted_content(&ai_response.content)?;

        // Analyze adapted content
        let adapted_metrics = self.analyze_content_metrics(&adapted_content)?;

        // Style-aware processing
        let (
            style_consistency_score,
            style_patterns_applied,
            style_similarity,
            style_preservation_score,
        ) = if config.apply_style_patterns {
            self.process_style_analysis(content, &adapted_content, &config)
                .await?
        } else {
            (None, Vec::new(), None, None)
        };

        // Calculate quality metrics
        let quality_score = self.calculate_quality_score(
            content,
            &adapted_content,
            &config,
            &original_metrics,
            &adapted_metrics,
        )?;

        // Generate adaptation summary
        let (changes_made, warnings, suggestions) = self.analyze_adaptation_changes(
            content,
            &adapted_content,
            &config,
            &original_metrics,
            &adapted_metrics,
        )?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(AdaptationResult {
            adapted_content,
            adaptation_summary: self.create_adaptation_summary(&config, &changes_made),
            changes_made,
            quality_score,
            readability_improvement: self
                .calculate_readability_improvement(&original_metrics, &adapted_metrics),
            audience_appropriateness: self
                .calculate_audience_appropriateness(&adapted_metrics, &audience_profile),
            warnings,
            suggestions,
            processing_time_ms: processing_time,
            tokens_used: ai_response.metadata.tokens_used,
            style_consistency_score,
            style_patterns_applied,
            style_similarity,
            style_preservation_score,
        })
    }

    /// Batch adapt multiple content pieces
    pub async fn batch_adapt_content(
        &self,
        content_pieces: Vec<&str>,
        config: AdaptationConfig,
    ) -> Result<Vec<AdaptationResult>> {
        let mut results = Vec::new();

        for content in content_pieces {
            let result = self.adapt_content(content, Some(config.clone())).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get available audience types
    pub fn get_available_audiences() -> Vec<AudienceType> {
        vec![
            AudienceType::General,
            AudienceType::Technical,
            AudienceType::Executive,
            AudienceType::Academic,
            AudienceType::Student,
            AudienceType::Expert,
            AudienceType::Beginner,
            AudienceType::Child,
            AudienceType::International,
            AudienceType::Legal,
            AudienceType::Medical,
            AudienceType::Business,
        ]
    }

    /// Get available adaptation purposes
    pub fn get_available_purposes() -> Vec<AdaptationPurpose> {
        vec![
            AdaptationPurpose::Simplify,
            AdaptationPurpose::Formalize,
            AdaptationPurpose::Summarize,
            AdaptationPurpose::Elaborate,
            AdaptationPurpose::Translate,
            AdaptationPurpose::Modernize,
            AdaptationPurpose::Specialize,
            AdaptationPurpose::Generalize,
            AdaptationPurpose::Instruction,
            AdaptationPurpose::Reference,
            AdaptationPurpose::Presentation,
            AdaptationPurpose::Documentation,
        ]
    }

    /// Create audience-specific adaptation suggestions
    pub fn get_adaptation_suggestions(
        &self,
        audience: &AudienceType,
        content_analysis: &AdaptationMetrics,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();
        let audience_profile = self.get_audience_profile(audience);

        // Complexity suggestions
        if content_analysis.complexity_change > 0.5 {
            suggestions.push(format!(
                "Consider simplifying complex concepts for {:?} audience",
                audience
            ));
        }

        // Technical density suggestions
        if content_analysis.technical_density_change > audience_profile.technical_tolerance {
            suggestions.push(
                "High technical density detected - consider adding definitions or examples"
                    .to_string(),
            );
        }

        // Length suggestions
        let word_ratio = content_analysis.adapted_word_count as f64
            / content_analysis.original_word_count as f64;
        if word_ratio > 1.5 && audience_profile.attention_span < 0.8 {
            suggestions.push(
                "Content may be too long for target audience - consider shortening".to_string(),
            );
        }

        // Tone suggestions
        if content_analysis.tone_shift_score < 0.3 {
            suggestions.push(format!(
                "Consider adjusting tone to better match {:?} communication preferences",
                audience_profile.preferred_tone
            ));
        }

        suggestions
    }

    // Private helper methods

    fn create_audience_profiles() -> HashMap<AudienceType, AudienceProfile> {
        let mut profiles = HashMap::new();

        profiles.insert(
            AudienceType::General,
            AudienceProfile {
                preferred_complexity: ComplexityLevel::Basic,
                preferred_tone: ToneType::Conversational,
                vocabulary_level: VocabularyComplexity::Basic,
                technical_tolerance: 0.3,
                attention_span: 1.0,
                preferred_examples: true,
                needs_definitions: true,
                cultural_considerations: vec!["Avoid jargon".to_string()],
                typical_concerns: vec!["Understanding".to_string(), "Relevance".to_string()],
                communication_style: "Clear and engaging".to_string(),
            },
        );

        profiles.insert(
            AudienceType::Technical,
            AudienceProfile {
                preferred_complexity: ComplexityLevel::Advanced,
                preferred_tone: ToneType::Technical,
                vocabulary_level: VocabularyComplexity::Expert,
                technical_tolerance: 0.9,
                attention_span: 1.2,
                preferred_examples: true,
                needs_definitions: false,
                cultural_considerations: vec!["Precision matters".to_string()],
                typical_concerns: vec!["Accuracy".to_string(), "Implementation".to_string()],
                communication_style: "Precise and detailed".to_string(),
            },
        );

        profiles.insert(
            AudienceType::Executive,
            AudienceProfile {
                preferred_complexity: ComplexityLevel::Intermediate,
                preferred_tone: ToneType::Professional,
                vocabulary_level: VocabularyComplexity::Advanced,
                technical_tolerance: 0.5,
                attention_span: 0.7,
                preferred_examples: false,
                needs_definitions: false,
                cultural_considerations: vec!["Focus on outcomes".to_string()],
                typical_concerns: vec!["ROI".to_string(), "Strategy".to_string()],
                communication_style: "Concise and strategic".to_string(),
            },
        );

        profiles.insert(
            AudienceType::Student,
            AudienceProfile {
                preferred_complexity: ComplexityLevel::Intermediate,
                preferred_tone: ToneType::Instructional,
                vocabulary_level: VocabularyComplexity::Intermediate,
                technical_tolerance: 0.6,
                attention_span: 0.9,
                preferred_examples: true,
                needs_definitions: true,
                cultural_considerations: vec!["Learning-focused".to_string()],
                typical_concerns: vec!["Understanding".to_string(), "Application".to_string()],
                communication_style: "Educational and supportive".to_string(),
            },
        );

        profiles.insert(
            AudienceType::Beginner,
            AudienceProfile {
                preferred_complexity: ComplexityLevel::Elementary,
                preferred_tone: ToneType::Friendly,
                vocabulary_level: VocabularyComplexity::Basic,
                technical_tolerance: 0.2,
                attention_span: 0.8,
                preferred_examples: true,
                needs_definitions: true,
                cultural_considerations: vec!["Avoid intimidation".to_string()],
                typical_concerns: vec!["Basics".to_string(), "Getting started".to_string()],
                communication_style: "Simple and encouraging".to_string(),
            },
        );

        // Add more audience profiles...
        profiles
    }

    pub fn get_audience_profile(&self, audience: &AudienceType) -> AudienceProfile {
        self.audience_profiles
            .get(audience)
            .cloned()
            .unwrap_or_else(|| {
                // Default profile for unknown audiences
                AudienceProfile {
                    preferred_complexity: ComplexityLevel::Intermediate,
                    preferred_tone: ToneType::Professional,
                    vocabulary_level: VocabularyComplexity::Intermediate,
                    technical_tolerance: 0.5,
                    attention_span: 1.0,
                    preferred_examples: true,
                    needs_definitions: true,
                    cultural_considerations: vec![],
                    typical_concerns: vec![],
                    communication_style: "Clear and professional".to_string(),
                }
            })
    }

    /// Process style analysis for content adaptation
    async fn process_style_analysis(
        &self,
        original_content: &str,
        adapted_content: &str,
        config: &AdaptationConfig,
    ) -> Result<(
        Option<f64>,
        Vec<String>,
        Option<StyleSimilarity>,
        Option<f64>,
    )> {
        // Analyze original content style
        let original_style = self
            .style_analyzer
            .analyze_content_style(original_content)?;

        // Analyze adapted content style
        let adapted_style = self.style_analyzer.analyze_content_style(adapted_content)?;

        let mut style_patterns_applied = Vec::new();

        // If target style profile is specified, compare against it
        let (style_consistency_score, style_similarity) = if let Some(ref target_style) =
            config.target_style_profile
        {
            let similarity = self
                .style_analyzer
                .compare_styles(target_style, &adapted_style);
            let consistency_score =
                self.calculate_style_consistency_score(&similarity, config.style_matching_strength);

            // Record applied patterns
            if similarity.overall_similarity > 0.7 {
                style_patterns_applied
                    .push("Target style profile successfully applied".to_string());
            }
            if similarity.tone_similarity > 0.8 {
                style_patterns_applied.push("Tone successfully matched to target".to_string());
            }
            if similarity.structure_similarity > 0.8 {
                style_patterns_applied.push("Structure successfully matched to target".to_string());
            }
            if similarity.vocabulary_similarity > 0.8 {
                style_patterns_applied.push("Vocabulary style successfully matched".to_string());
            }

            (Some(consistency_score), Some(similarity))
        } else {
            (None, None)
        };

        // Calculate style preservation score (how well original author voice is preserved)
        let style_preservation_score = if config.preserve_author_voice {
            let original_adapted_similarity = self
                .style_analyzer
                .compare_styles(&original_style, &adapted_style);
            Some(original_adapted_similarity.overall_similarity)
        } else {
            None
        };

        // Add general style patterns applied
        if config.enforce_style_consistency {
            style_patterns_applied
                .push("Style consistency enforced throughout document".to_string());
        }

        // Analyze tone adjustments applied
        for tone_adj in &config.tone_adjustments {
            match tone_adj {
                ToneAdjustment::MoreFormal => {
                    if adapted_style.tone.primary_tone == ToneType::Formal
                        || adapted_style.tone.primary_tone == ToneType::Professional
                    {
                        style_patterns_applied.push("Formal tone pattern applied".to_string());
                    }
                }
                ToneAdjustment::MoreFriendly => {
                    if adapted_style.tone.primary_tone == ToneType::Friendly
                        || adapted_style.tone.primary_tone == ToneType::Conversational
                    {
                        style_patterns_applied.push("Friendly tone pattern applied".to_string());
                    }
                }
                ToneAdjustment::MoreProfessional => {
                    if adapted_style.tone.primary_tone == ToneType::Professional {
                        style_patterns_applied
                            .push("Professional tone pattern applied".to_string());
                    }
                }
                _ => {
                    style_patterns_applied
                        .push(format!("Tone adjustment pattern applied: {:?}", tone_adj));
                }
            }
        }

        Ok((
            style_consistency_score,
            style_patterns_applied,
            style_similarity,
            style_preservation_score,
        ))
    }

    /// Calculate style consistency score based on similarity and matching strength
    fn calculate_style_consistency_score(
        &self,
        similarity: &StyleSimilarity,
        matching_strength: f64,
    ) -> f64 {
        let base_score = (similarity.overall_similarity
            + similarity.tone_similarity
            + similarity.structure_similarity
            + similarity.vocabulary_similarity)
            / 4.0;

        // Apply matching strength - stricter matching requires higher similarity scores
        if matching_strength > 0.8 {
            // Strict matching - penalize if not very similar
            if base_score < 0.8 {
                base_score * 0.7
            } else {
                base_score
            }
        } else if matching_strength > 0.5 {
            // Moderate matching - mild penalties for low similarity
            if base_score < 0.6 {
                base_score * 0.9
            } else {
                base_score
            }
        } else {
            // Loose matching - accept lower similarity scores
            base_score.max(0.5)
        }
    }

    fn create_adaptation_prompt(
        &self,
        content: &str,
        config: &AdaptationConfig,
        audience_profile: &AudienceProfile,
        metrics: &AdaptationMetrics,
    ) -> Result<String> {
        let mut prompt = String::new();

        prompt.push_str(PromptTemplates::CONTENT_ADAPTATION_SYSTEM);

        prompt.push_str(&format!(
            "\n\nTASK: Adapt the following content for a {:?} audience with the purpose of {:?}.\n",
            config.target_audience, config.purpose
        ));

        prompt.push_str(&format!(
            "TARGET COMPLEXITY: {:?}\n",
            config.complexity_level
        ));

        prompt.push_str(&format!(
            "AUDIENCE PROFILE:\n- Communication style: {}\n- Technical tolerance: {:.1}\n- Attention span: {:.1}\n- Needs definitions: {}\n- Prefers examples: {}\n",
            audience_profile.communication_style,
            audience_profile.technical_tolerance,
            audience_profile.attention_span,
            audience_profile.needs_definitions,
            audience_profile.preferred_examples
        ));

        if !config.tone_adjustments.is_empty() {
            prompt.push_str(&format!(
                "TONE ADJUSTMENTS: {:?}\n",
                config.tone_adjustments
            ));
        }

        prompt.push_str(&format!(
            "\nCONFIGURATION:\n- Preserve technical terms: {}\n- Preserve structure: {}\n- Target length ratio: {:.1}\n- Use examples: {}\n- Include definitions: {}\n",
            config.preserve_technical_terms,
            config.preserve_structure,
            config.target_length_ratio,
            config.use_examples,
            config.include_definitions
        ));

        prompt.push_str(&format!(
            "\nORIGINAL CONTENT METRICS:\n- Word count: {}\n- Complexity: {:.2}\n- Technical density: {:.2}\n",
            metrics.original_word_count,
            metrics.complexity_change,
            metrics.technical_density_change
        ));

        prompt.push_str("\nORIGINAL CONTENT:\n");
        prompt.push_str(content);

        prompt.push_str("\n\nINSTRUCTIONS:\n");
        prompt.push_str("1. Adapt the content according to the specifications above\n");
        prompt.push_str("2. Maintain the core message while adjusting complexity and tone\n");
        prompt.push_str("3. Add examples or definitions if specified\n");
        prompt.push_str("4. Ensure cultural sensitivity and appropriateness\n");
        prompt.push_str("5. Return only the adapted content, no explanations\n");

        Ok(prompt)
    }

    fn extract_adapted_content(&self, ai_response: &str) -> Result<String> {
        // Clean up AI response and extract the main content
        let content = ai_response
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if content.trim().is_empty() {
            return Err(anyhow!("AI response contained no usable content"));
        }

        Ok(content)
    }

    pub fn analyze_content_metrics(&self, content: &str) -> Result<AdaptationMetrics> {
        let word_count = content.split_whitespace().count();

        // Simple complexity analysis (in production, use more sophisticated NLP)
        let sentence_count = content.matches('.').count().max(1);
        let avg_sentence_length = word_count as f64 / sentence_count as f64;
        let complexity_score = (avg_sentence_length / 15.0).min(1.0); // Normalize to 0-1

        // Technical density (simple heuristic)
        let technical_terms = [
            "API",
            "algorithm",
            "implementation",
            "configuration",
            "infrastructure",
            "optimization",
            "architecture",
            "deployment",
            "integration",
            "methodology",
        ];
        let technical_count = technical_terms
            .iter()
            .map(|term| content.to_lowercase().matches(&term.to_lowercase()).count())
            .sum::<usize>();
        let technical_density = technical_count as f64 / word_count as f64;

        Ok(AdaptationMetrics {
            original_word_count: word_count,
            adapted_word_count: word_count, // Will be updated for adapted content
            complexity_change: complexity_score,
            tone_shift_score: 0.5, // Placeholder
            readability_score: 1.0 - complexity_score,
            technical_density_change: technical_density,
            structure_preservation: 1.0,
        })
    }

    fn calculate_quality_score(
        &self,
        _original: &str,
        _adapted: &str,
        config: &AdaptationConfig,
        original_metrics: &AdaptationMetrics,
        adapted_metrics: &AdaptationMetrics,
    ) -> Result<f64> {
        let mut score = 0.0;

        // Length appropriateness (25% of score)
        let length_ratio =
            adapted_metrics.adapted_word_count as f64 / original_metrics.original_word_count as f64;
        let length_target = config.target_length_ratio;
        let length_score = 1.0 - (length_ratio - length_target).abs().min(1.0);
        score += length_score * 0.25;

        // Complexity appropriateness (25% of score)
        let complexity_diff =
            (adapted_metrics.complexity_change - original_metrics.complexity_change).abs();
        let complexity_score = 1.0 - complexity_diff.min(1.0);
        score += complexity_score * 0.25;

        // Readability improvement (25% of score)
        let readability_score = adapted_metrics.readability_score;
        score += readability_score * 0.25;

        // Structure preservation (25% of score)
        let structure_score = if config.preserve_structure {
            adapted_metrics.structure_preservation
        } else {
            1.0 // Not required, so full score
        };
        score += structure_score * 0.25;

        Ok(score)
    }

    fn calculate_readability_improvement(
        &self,
        original: &AdaptationMetrics,
        adapted: &AdaptationMetrics,
    ) -> f64 {
        adapted.readability_score - original.readability_score
    }

    pub fn calculate_audience_appropriateness(
        &self,
        metrics: &AdaptationMetrics,
        audience_profile: &AudienceProfile,
    ) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Technical tolerance
        if metrics.technical_density_change <= audience_profile.technical_tolerance {
            score += 1.0;
        } else {
            score += audience_profile.technical_tolerance / metrics.technical_density_change;
        }
        factors += 1;

        // Complexity appropriateness
        let complexity_match = match audience_profile.preferred_complexity {
            ComplexityLevel::Elementary => 1.0 - metrics.complexity_change.max(0.0),
            ComplexityLevel::Basic => 1.0 - (metrics.complexity_change - 0.2).abs().max(0.0),
            ComplexityLevel::Intermediate => 1.0 - (metrics.complexity_change - 0.5).abs().max(0.0),
            ComplexityLevel::Advanced => 1.0 - (metrics.complexity_change - 0.8).abs().max(0.0),
            ComplexityLevel::Expert => metrics.complexity_change.min(1.0),
        };
        score += complexity_match;
        factors += 1;

        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }

    fn analyze_adaptation_changes(
        &self,
        original: &str,
        adapted: &str,
        config: &AdaptationConfig,
        original_metrics: &AdaptationMetrics,
        adapted_metrics: &AdaptationMetrics,
    ) -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
        let mut changes_made = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Analyze length changes
        let length_change =
            adapted_metrics.adapted_word_count as f64 / original_metrics.original_word_count as f64;

        if length_change > 1.2 {
            changes_made.push("Expanded content with additional explanations".to_string());
        } else if length_change < 0.8 {
            changes_made.push("Condensed content for brevity".to_string());
        }

        // Analyze complexity changes
        let complexity_change =
            adapted_metrics.complexity_change - original_metrics.complexity_change;
        if complexity_change < -0.2 {
            changes_made.push("Simplified language and structure".to_string());
        } else if complexity_change > 0.2 {
            changes_made.push("Enhanced technical depth".to_string());
        }

        // Check for potential issues
        if adapted.len() < original.len() / 3 {
            warnings.push(
                "Significant content reduction - verify important information is preserved"
                    .to_string(),
            );
        }

        if adapted_metrics.technical_density_change > 0.5 {
            warnings
                .push("High technical density may be challenging for target audience".to_string());
        }

        // Generate suggestions based on config
        if config.use_examples && !adapted.contains("example") && !adapted.contains("Example") {
            suggestions
                .push("Consider adding specific examples to illustrate concepts".to_string());
        }

        if config.include_definitions && adapted_metrics.technical_density_change > 0.3 {
            suggestions.push("Consider adding definitions for technical terms".to_string());
        }

        Ok((changes_made, warnings, suggestions))
    }

    fn create_adaptation_summary(&self, config: &AdaptationConfig, changes: &[String]) -> String {
        format!(
            "Adapted content for {:?} audience with purpose: {:?}. Changes made: {}",
            config.target_audience,
            config.purpose,
            changes.join(", ")
        )
    }
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            target_audience: AudienceType::General,
            purpose: AdaptationPurpose::Simplify,
            complexity_level: ComplexityLevel::Intermediate,
            tone_adjustments: vec![],
            preserve_technical_terms: false,
            preserve_structure: true,
            target_length_ratio: 1.0,
            use_examples: true,
            include_definitions: true,
            style_consistency: true,
            cultural_sensitivity: true,
            target_style_profile: None,
            enforce_style_consistency: true,
            style_matching_strength: 0.7,
            preserve_author_voice: false,
            apply_style_patterns: false,
        }
    }
}

// Add the content adaptation prompt template to the AI system
impl PromptTemplates {
    pub const CONTENT_ADAPTATION_SYSTEM: &'static str = r#"
You are an expert content adaptation specialist. Your role is to adapt written content for different audiences and purposes while maintaining the core message and ensuring appropriateness.

Key principles:
1. Understand the target audience's needs, knowledge level, and communication preferences
2. Adjust vocabulary, complexity, and tone appropriately
3. Preserve essential information while adapting presentation
4. Add context, examples, or definitions when helpful for the target audience
5. Ensure cultural sensitivity and appropriateness
6. Maintain logical flow and structure unless otherwise specified
7. Be consistent in style and terminology throughout the adaptation

When adapting content:
- For technical audiences: Use precise terminology, assume domain knowledge, focus on implementation details
- For general audiences: Use plain language, provide context, include analogies and examples
- For executive audiences: Focus on outcomes, be concise, highlight strategic implications
- For students: Be instructional, include learning aids, break down complex concepts
- For beginners: Start with basics, avoid jargon, provide step-by-step guidance
- For international audiences: Use clear, simple language and avoid cultural references

Always consider the purpose of adaptation and adjust accordingly.
"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audience_types() {
        let audiences = ContentAdapter::get_available_audiences();
        assert!(audiences.contains(&AudienceType::General));
        assert!(audiences.contains(&AudienceType::Technical));
        assert!(audiences.contains(&AudienceType::Executive));
    }

    #[test]
    fn test_adaptation_purposes() {
        let purposes = ContentAdapter::get_available_purposes();
        assert!(purposes.contains(&AdaptationPurpose::Simplify));
        assert!(purposes.contains(&AdaptationPurpose::Formalize));
        assert!(purposes.contains(&AdaptationPurpose::Summarize));
    }

    #[test]
    fn test_adaptation_config_default() {
        let config = AdaptationConfig::default();
        assert_eq!(config.target_audience, AudienceType::General);
        assert_eq!(config.purpose, AdaptationPurpose::Simplify);
        assert_eq!(config.complexity_level, ComplexityLevel::Intermediate);
        assert!(config.use_examples);
        assert!(config.include_definitions);
    }

    #[test]
    fn test_content_metrics_analysis() {
        let adapter = ContentAdapter::new(AIConfig::default());
        let content = "This is a simple test sentence. It contains multiple sentences for testing.";

        let metrics = adapter.analyze_content_metrics(content).unwrap();
        assert!(metrics.original_word_count > 0);
        assert!(metrics.complexity_change >= 0.0);
        assert!(metrics.readability_score >= 0.0);
    }

    #[test]
    fn test_audience_profile_retrieval() {
        let adapter = ContentAdapter::new(AIConfig::default());
        let profile = adapter.get_audience_profile(&AudienceType::Technical);

        assert_eq!(profile.preferred_complexity, ComplexityLevel::Advanced);
        assert!(profile.technical_tolerance > 0.8);
    }

    #[test]
    fn test_adaptation_suggestions() {
        let adapter = ContentAdapter::new(AIConfig::default());
        let metrics = AdaptationMetrics {
            original_word_count: 100,
            adapted_word_count: 200,
            complexity_change: 0.8,
            tone_shift_score: 0.2,
            readability_score: 0.6,
            technical_density_change: 0.9,
            structure_preservation: 0.9,
        };

        let suggestions = adapter.get_adaptation_suggestions(&AudienceType::Beginner, &metrics);
        assert!(!suggestions.is_empty());
    }
}
