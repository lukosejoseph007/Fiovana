use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::document::indexer::DocumentIndexEntry;
use crate::document::style_analyzer::{
    StyleAnalyzer, StyleProfile, ToneType, VocabularyComplexity, VoiceType,
};

#[derive(Error, Debug)]
pub enum StyleLearnerError {
    #[error("Insufficient data for learning: {0}")]
    InsufficientData(String),
    #[error("Style learning failed: {0}")]
    #[allow(dead_code)]
    LearningFailed(String),
    #[error("Invalid corpus: {0}")]
    #[allow(dead_code)]
    InvalidCorpus(String),
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermFrequency {
    pub term: String,
    pub frequency: i32,
    pub relative_frequency: f64,
    pub contexts: Vec<String>,
    pub is_preferred: bool,
    pub is_avoided: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePattern {
    pub pattern_type: String,
    pub pattern: String,
    pub frequency: i32,
    pub confidence: f64,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalStyle {
    pub organization_id: String,
    pub preferred_terminology: HashMap<String, TermFrequency>,
    pub avoided_terminology: Vec<String>,
    pub common_patterns: Vec<StylePattern>,
    pub preferred_tone: ToneType,
    pub preferred_voice: VoiceType,
    pub vocabulary_complexity: VocabularyComplexity,
    pub formality_preference: f64,
    pub structural_patterns: Vec<String>,
    pub sample_documents: Vec<String>,
    pub learning_confidence: f64,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleLearningResult {
    pub organizational_style: OrganizationalStyle,
    pub learned_patterns: Vec<StylePattern>,
    pub terminology_recommendations: Vec<String>,
    pub style_guidelines: Vec<String>,
    pub confidence_score: f64,
    pub sample_size: usize,
}

pub struct StyleLearner {
    style_analyzer: StyleAnalyzer,
    min_frequency_threshold: i32,
    #[allow(dead_code)]
    min_confidence_threshold: f64,
    // Pattern matching for organizational style learning
    business_patterns: Vec<&'static str>,
    academic_patterns: Vec<&'static str>,
    technical_patterns: Vec<&'static str>,
    #[allow(dead_code)]
    informal_patterns: Vec<&'static str>,
}

impl StyleLearner {
    pub fn new() -> Self {
        Self {
            style_analyzer: StyleAnalyzer::new(),
            min_frequency_threshold: 3,
            min_confidence_threshold: 0.6,
            business_patterns: vec![
                "leverage",
                "synergy",
                "stakeholder",
                "deliverable",
                "actionable",
                "strategic",
                "optimize",
                "streamline",
                "scalable",
                "enterprise",
                "solution",
                "framework",
                "methodology",
                "best practice",
                "value proposition",
                "ROI",
                "KPI",
                "milestone",
                "roadmap",
                "touchpoint",
            ],
            academic_patterns: vec![
                "furthermore",
                "however",
                "moreover",
                "nevertheless",
                "consequently",
                "therefore",
                "in conclusion",
                "it is evident that",
                "research indicates",
                "studies show",
                "empirical evidence",
                "theoretical framework",
                "methodology",
                "hypothesis",
                "analysis reveals",
                "significant findings",
                "peer-reviewed",
                "systematic review",
                "meta-analysis",
                "scholarly",
            ],
            technical_patterns: vec![
                "implementation",
                "configuration",
                "deployment",
                "integration",
                "architecture",
                "infrastructure",
                "protocol",
                "algorithm",
                "optimization",
                "debugging",
                "troubleshooting",
                "specification",
                "documentation",
                "validation",
                "verification",
                "scalability",
                "performance",
                "security",
                "compliance",
                "backward compatibility",
            ],
            informal_patterns: vec![
                "you'll",
                "we'll",
                "let's",
                "here's",
                "that's",
                "okay",
                "sure",
                "cool",
                "awesome",
                "stuff",
                "things",
                "folks",
                "guys",
                "basically",
                "pretty much",
                "kind of",
                "sort of",
                "a lot",
                "tons of",
                "super",
            ],
        }
    }

    /// Learn organizational style patterns from a corpus of documents
    pub fn learn_from_corpus(
        &self,
        documents: &[DocumentIndexEntry],
        organization_id: String,
    ) -> Result<StyleLearningResult, StyleLearnerError> {
        if documents.is_empty() {
            return Err(StyleLearnerError::InsufficientData(
                "No documents provided for learning".to_string(),
            ));
        }

        if documents.len() < 3 {
            return Err(StyleLearnerError::InsufficientData(format!(
                "Need at least 3 documents, got {}",
                documents.len()
            )));
        }

        // Analyze each document for style patterns
        let mut document_styles = Vec::new();
        for doc in documents {
            match self.style_analyzer.analyze_document_style(doc) {
                Ok(style) => document_styles.push(style),
                Err(e) => {
                    return Err(StyleLearnerError::AnalysisError(format!(
                        "Failed to analyze document {}: {}",
                        doc.path.display(),
                        e
                    )));
                }
            }
        }

        // Extract organizational patterns from document corpus
        let terminology = self.extract_preferred_terminology(&document_styles)?;
        let common_patterns = self.identify_common_patterns(&document_styles)?;
        let preferred_styles = self.determine_preferred_styles(&document_styles)?;

        // Build organizational style profile
        let organizational_style = OrganizationalStyle {
            organization_id: organization_id.clone(),
            preferred_terminology: terminology.clone(),
            avoided_terminology: self.identify_avoided_terms(&document_styles),
            common_patterns: common_patterns.clone(),
            preferred_tone: preferred_styles.tone,
            preferred_voice: preferred_styles.voice,
            vocabulary_complexity: preferred_styles.vocabulary_complexity,
            formality_preference: preferred_styles.formality,
            structural_patterns: self.extract_structural_patterns(&document_styles),
            sample_documents: documents
                .iter()
                .map(|d| d.path.to_string_lossy().to_string())
                .collect(),
            learning_confidence: self.calculate_learning_confidence(&document_styles),
            last_updated: chrono::Utc::now().to_rfc3339(),
        };

        // Generate recommendations
        let terminology_recommendations = self.generate_terminology_recommendations(&terminology);
        let style_guidelines = self.generate_style_guidelines(&organizational_style);

        Ok(StyleLearningResult {
            organizational_style,
            learned_patterns: common_patterns,
            terminology_recommendations,
            style_guidelines,
            confidence_score: self.calculate_learning_confidence(&document_styles),
            sample_size: documents.len(),
        })
    }

    /// Extract preferred terminology from document corpus
    fn extract_preferred_terminology(
        &self,
        document_styles: &[StyleProfile],
    ) -> Result<HashMap<String, TermFrequency>, StyleLearnerError> {
        let mut term_frequencies: HashMap<String, (i32, Vec<String>)> = HashMap::new();
        let total_docs = document_styles.len() as f64;

        // Collect term frequencies across all documents
        for style in document_styles {
            for (term, freq) in &style.vocabulary.preferred_terms {
                let entry = term_frequencies
                    .entry(term.clone())
                    .or_insert((0, Vec::new()));
                entry.0 += freq;
                entry.1.push(format!("Document context for '{}'", term));
            }
        }

        // Convert to TermFrequency objects with analysis
        let mut preferred_terminology = HashMap::new();
        for (term, (total_freq, contexts)) in term_frequencies {
            if total_freq >= self.min_frequency_threshold {
                let relative_freq = total_freq as f64 / total_docs;
                let is_preferred = relative_freq > 0.5; // Appears in >50% of documents
                let is_avoided = self.is_term_avoided(&term, document_styles);

                preferred_terminology.insert(
                    term.clone(),
                    TermFrequency {
                        term,
                        frequency: total_freq,
                        relative_frequency: relative_freq,
                        contexts: contexts.into_iter().take(5).collect(), // Limit to 5 examples
                        is_preferred,
                        is_avoided,
                    },
                );
            }
        }

        Ok(preferred_terminology)
    }

    /// Identify common style patterns across documents
    fn identify_common_patterns(
        &self,
        document_styles: &[StyleProfile],
    ) -> Result<Vec<StylePattern>, StyleLearnerError> {
        let mut patterns = Vec::new();

        // Analyze tone patterns
        let tone_patterns = self.analyze_tone_patterns(document_styles);
        patterns.extend(tone_patterns);

        // Analyze vocabulary patterns
        let vocab_patterns = self.analyze_vocabulary_patterns(document_styles);
        patterns.extend(vocab_patterns);

        // Analyze structural patterns
        let structural_patterns = self.analyze_structural_patterns(document_styles);
        patterns.extend(structural_patterns);

        // Sort by confidence and frequency
        patterns.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.frequency.cmp(&a.frequency))
        });

        Ok(patterns)
    }

    /// Analyze tone patterns across document corpus
    fn analyze_tone_patterns(&self, document_styles: &[StyleProfile]) -> Vec<StylePattern> {
        let mut patterns = Vec::new();
        let total_docs = document_styles.len() as f64;

        // Count tone types
        let mut tone_counts: HashMap<String, i32> = HashMap::new();
        for style in document_styles {
            let tone_key = format!("{:?}", style.tone.primary_tone);
            *tone_counts.entry(tone_key).or_insert(0) += 1;
        }

        // Create patterns for common tones
        for (tone, count) in tone_counts {
            let frequency_ratio = count as f64 / total_docs;
            if frequency_ratio > 0.3 {
                // Appears in >30% of documents
                patterns.push(StylePattern {
                    pattern_type: "tone".to_string(),
                    pattern: tone.clone(),
                    frequency: count,
                    confidence: frequency_ratio,
                    examples: vec![format!("Consistent {} tone usage", tone)],
                });
            }
        }

        patterns
    }

    /// Analyze vocabulary patterns across document corpus
    fn analyze_vocabulary_patterns(&self, document_styles: &[StyleProfile]) -> Vec<StylePattern> {
        let mut patterns = Vec::new();

        // Analyze domain-specific vocabulary usage
        let domain_usage = self.analyze_domain_vocabulary_usage(document_styles);
        for (domain, usage_ratio) in domain_usage {
            if usage_ratio > 0.4 {
                // Domain terms used in >40% of documents
                patterns.push(StylePattern {
                    pattern_type: "vocabulary_domain".to_string(),
                    pattern: domain.clone(),
                    frequency: (usage_ratio * document_styles.len() as f64) as i32,
                    confidence: usage_ratio,
                    examples: vec![format!("Consistent {} terminology usage", domain)],
                });
            }
        }

        // Analyze formality patterns
        let avg_formality: f64 = document_styles
            .iter()
            .map(|s| s.tone.formality_score)
            .sum::<f64>()
            / document_styles.len() as f64;

        let formality_category = if avg_formality > 0.7 {
            "formal"
        } else if avg_formality < 0.3 {
            "informal"
        } else {
            "balanced"
        };

        patterns.push(StylePattern {
            pattern_type: "formality".to_string(),
            pattern: formality_category.to_string(),
            frequency: document_styles.len() as i32,
            confidence: 1.0
                - (document_styles
                    .iter()
                    .map(|s| (s.tone.formality_score - avg_formality).abs())
                    .sum::<f64>()
                    / document_styles.len() as f64),
            examples: vec![format!("Consistent {} formality level", formality_category)],
        });

        patterns
    }

    /// Analyze structural patterns across document corpus
    fn analyze_structural_patterns(&self, document_styles: &[StyleProfile]) -> Vec<StylePattern> {
        let mut patterns = Vec::new();

        // Analyze heading style consistency
        let mut heading_styles: HashMap<String, i32> = HashMap::new();
        for style in document_styles {
            *heading_styles
                .entry(style.structure.heading_style.clone())
                .or_insert(0) += 1;
        }

        for (style, count) in heading_styles {
            let frequency_ratio = count as f64 / document_styles.len() as f64;
            if frequency_ratio > 0.5 {
                patterns.push(StylePattern {
                    pattern_type: "heading_style".to_string(),
                    pattern: style.clone(),
                    frequency: count,
                    confidence: frequency_ratio,
                    examples: vec![format!("Consistent {} heading style", style)],
                });
            }
        }

        // Analyze list formatting consistency
        let mut list_formats: HashMap<String, i32> = HashMap::new();
        for style in document_styles {
            *list_formats
                .entry(style.structure.list_preferences.clone())
                .or_insert(0) += 1;
        }

        for (format, count) in list_formats {
            let frequency_ratio = count as f64 / document_styles.len() as f64;
            if frequency_ratio > 0.5 {
                patterns.push(StylePattern {
                    pattern_type: "list_format".to_string(),
                    pattern: format.clone(),
                    frequency: count,
                    confidence: frequency_ratio,
                    examples: vec![format!("Consistent {} list formatting", format)],
                });
            }
        }

        patterns
    }

    /// Determine preferred organizational styles
    fn determine_preferred_styles(
        &self,
        document_styles: &[StyleProfile],
    ) -> Result<PreferredStyles, StyleLearnerError> {
        // Determine most common tone
        let mut tone_counts: HashMap<String, i32> = HashMap::new();
        for style in document_styles {
            let tone_key = format!("{:?}", style.tone.primary_tone);
            *tone_counts.entry(tone_key).or_insert(0) += 1;
        }
        let preferred_tone = tone_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(tone, _)| tone)
            .unwrap_or_else(|| "Professional".to_string());

        // Determine most common voice
        let mut voice_counts: HashMap<String, i32> = HashMap::new();
        for style in document_styles {
            let voice_key = format!("{:?}", style.tone.voice_type);
            *voice_counts.entry(voice_key).or_insert(0) += 1;
        }
        let preferred_voice = voice_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(voice, _)| voice)
            .unwrap_or_else(|| "Neutral".to_string());

        // Determine vocabulary complexity
        let mut complexity_counts: HashMap<String, i32> = HashMap::new();
        for style in document_styles {
            let complexity_key = format!("{:?}", style.vocabulary.complexity);
            *complexity_counts.entry(complexity_key).or_insert(0) += 1;
        }
        let preferred_complexity = complexity_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(complexity, _)| complexity)
            .unwrap_or_else(|| "Intermediate".to_string());

        // Calculate average formality
        let avg_formality: f64 = document_styles
            .iter()
            .map(|s| s.tone.formality_score)
            .sum::<f64>()
            / document_styles.len() as f64;

        Ok(PreferredStyles {
            tone: self.parse_tone_type(&preferred_tone),
            voice: self.parse_voice_type(&preferred_voice),
            vocabulary_complexity: self.parse_vocabulary_complexity(&preferred_complexity),
            formality: avg_formality,
        })
    }

    /// Parse tone type from string
    fn parse_tone_type(&self, tone_str: &str) -> ToneType {
        match tone_str {
            "Formal" => ToneType::Formal,
            "Informal" => ToneType::Informal,
            "Friendly" => ToneType::Friendly,
            "Professional" => ToneType::Professional,
            "Academic" => ToneType::Academic,
            "Conversational" => ToneType::Conversational,
            "Instructional" => ToneType::Instructional,
            "Technical" => ToneType::Technical,
            "Authoritative" => ToneType::Authoritative,
            "Empathetic" => ToneType::Empathetic,
            "Persuasive" => ToneType::Persuasive,
            "Objective" => ToneType::Objective,
            "Enthusiastic" => ToneType::Enthusiastic,
            "Cautious" => ToneType::Cautious,
            _ => ToneType::Professional,
        }
    }

    /// Parse voice type from string
    fn parse_voice_type(&self, voice_str: &str) -> VoiceType {
        match voice_str {
            "Authoritative" => VoiceType::Authoritative,
            "Friendly" => VoiceType::Friendly,
            "Neutral" => VoiceType::Neutral,
            "Encouraging" => VoiceType::Encouraging,
            "Explanatory" => VoiceType::Explanatory,
            "Conversational" => VoiceType::Conversational,
            "Expert" => VoiceType::Expert,
            "Empathetic" => VoiceType::Empathetic,
            _ => VoiceType::Neutral,
        }
    }

    /// Parse vocabulary complexity from string
    fn parse_vocabulary_complexity(&self, complexity_str: &str) -> VocabularyComplexity {
        match complexity_str {
            "Basic" => VocabularyComplexity::Basic,
            "Intermediate" => VocabularyComplexity::Intermediate,
            "Advanced" => VocabularyComplexity::Advanced,
            "Expert" => VocabularyComplexity::Expert,
            _ => VocabularyComplexity::Intermediate,
        }
    }

    /// Identify terms that should be avoided
    fn identify_avoided_terms(&self, document_styles: &[StyleProfile]) -> Vec<String> {
        let mut avoided_terms = Vec::new();

        // Terms that appear infrequently or inconsistently
        let mut term_frequencies: HashMap<String, i32> = HashMap::new();
        for style in document_styles {
            for term in &style.vocabulary.avoided_terms {
                *term_frequencies.entry(term.clone()).or_insert(0) += 1;
            }
        }

        // Terms that appear in avoided_terms frequently across documents
        let threshold = document_styles.len() as i32 / 3; // Appears in at least 1/3 of documents
        for (term, freq) in term_frequencies {
            if freq >= threshold {
                avoided_terms.push(term);
            }
        }

        avoided_terms
    }

    /// Check if a term is consistently avoided
    fn is_term_avoided(&self, term: &str, document_styles: &[StyleProfile]) -> bool {
        let avoided_count = document_styles
            .iter()
            .filter(|style| style.vocabulary.avoided_terms.contains(&term.to_string()))
            .count();

        avoided_count as f64 / document_styles.len() as f64 > 0.3 // Avoided in >30% of documents
    }

    /// Extract structural patterns from document styles
    fn extract_structural_patterns(&self, document_styles: &[StyleProfile]) -> Vec<String> {
        let mut patterns = Vec::new();

        // Extract common flow patterns
        for style in document_styles {
            for flow in &style.structure.typical_flow {
                if !patterns.contains(flow) {
                    // Check if this pattern appears in multiple documents
                    let count = document_styles
                        .iter()
                        .filter(|s| s.structure.typical_flow.contains(flow))
                        .count();
                    if count >= 2 {
                        patterns.push(flow.clone());
                    }
                }
            }
        }

        patterns
    }

    /// Analyze domain vocabulary usage across documents
    fn analyze_domain_vocabulary_usage(
        &self,
        document_styles: &[StyleProfile],
    ) -> HashMap<String, f64> {
        let mut domain_usage = HashMap::new();
        let total_docs = document_styles.len() as f64;

        // Business domain analysis
        let business_usage = document_styles
            .iter()
            .filter(|style| self.contains_business_terms(&style.vocabulary.common_terms))
            .count() as f64
            / total_docs;
        domain_usage.insert("business".to_string(), business_usage);

        // Academic domain analysis
        let academic_usage = document_styles
            .iter()
            .filter(|style| self.contains_academic_terms(&style.vocabulary.common_terms))
            .count() as f64
            / total_docs;
        domain_usage.insert("academic".to_string(), academic_usage);

        // Technical domain analysis
        let technical_usage = document_styles
            .iter()
            .filter(|style| self.contains_technical_terms(&style.vocabulary.common_terms))
            .count() as f64
            / total_docs;
        domain_usage.insert("technical".to_string(), technical_usage);

        domain_usage
    }

    /// Check if vocabulary contains business terms
    fn contains_business_terms(&self, terms: &[String]) -> bool {
        terms.iter().any(|term| {
            self.business_patterns
                .iter()
                .any(|pattern| term.to_lowercase().contains(&pattern.to_lowercase()))
        })
    }

    /// Check if vocabulary contains academic terms
    fn contains_academic_terms(&self, terms: &[String]) -> bool {
        terms.iter().any(|term| {
            self.academic_patterns
                .iter()
                .any(|pattern| term.to_lowercase().contains(&pattern.to_lowercase()))
        })
    }

    /// Check if vocabulary contains technical terms
    fn contains_technical_terms(&self, terms: &[String]) -> bool {
        terms.iter().any(|term| {
            self.technical_patterns
                .iter()
                .any(|pattern| term.to_lowercase().contains(&pattern.to_lowercase()))
        })
    }

    /// Calculate confidence score for learning results
    fn calculate_learning_confidence(&self, document_styles: &[StyleProfile]) -> f64 {
        if document_styles.is_empty() {
            return 0.0;
        }

        // Base confidence on sample size
        let sample_confidence = if document_styles.len() >= 10 {
            1.0
        } else if document_styles.len() >= 5 {
            0.8
        } else {
            0.6
        };

        // Adjust for consistency across documents
        let avg_individual_confidence: f64 = document_styles
            .iter()
            .map(|s| s.confidence_score)
            .sum::<f64>()
            / document_styles.len() as f64;

        // Calculate tone consistency
        let tone_consistency = self.calculate_tone_consistency(document_styles);

        // Combine factors
        (sample_confidence + avg_individual_confidence + tone_consistency) / 3.0
    }

    /// Calculate tone consistency across documents
    fn calculate_tone_consistency(&self, document_styles: &[StyleProfile]) -> f64 {
        if document_styles.len() <= 1 {
            return 1.0;
        }

        let avg_formality: f64 = document_styles
            .iter()
            .map(|s| s.tone.formality_score)
            .sum::<f64>()
            / document_styles.len() as f64;

        let formality_variance: f64 = document_styles
            .iter()
            .map(|s| (s.tone.formality_score - avg_formality).powi(2))
            .sum::<f64>()
            / document_styles.len() as f64;

        // Lower variance = higher consistency
        1.0 - formality_variance.sqrt().min(1.0)
    }

    /// Generate terminology recommendations
    fn generate_terminology_recommendations(
        &self,
        terminology: &HashMap<String, TermFrequency>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Recommend preferred terms
        let preferred_terms: Vec<_> = terminology
            .values()
            .filter(|term| term.is_preferred && term.relative_frequency > 0.6)
            .collect();

        if !preferred_terms.is_empty() {
            recommendations.push(format!(
                "Use these consistently preferred terms: {}",
                preferred_terms
                    .iter()
                    .map(|t| t.term.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Recommend avoiding inconsistent terms
        let inconsistent_terms: Vec<_> = terminology
            .values()
            .filter(|term| !term.is_preferred && term.relative_frequency < 0.3)
            .collect();

        if !inconsistent_terms.is_empty() {
            recommendations.push(format!(
                "Consider avoiding these inconsistently used terms: {}",
                inconsistent_terms
                    .iter()
                    .map(|t| t.term.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Add domain-specific recommendations
        if terminology
            .keys()
            .any(|term| self.business_patterns.iter().any(|p| term.contains(p)))
        {
            recommendations.push("Maintain consistent business terminology usage".to_string());
        }

        if terminology
            .keys()
            .any(|term| self.technical_patterns.iter().any(|p| term.contains(p)))
        {
            recommendations.push("Use technical terms consistently across documents".to_string());
        }

        recommendations
    }

    /// Generate style guidelines
    fn generate_style_guidelines(&self, organizational_style: &OrganizationalStyle) -> Vec<String> {
        let mut guidelines = Vec::new();

        // Tone guidelines
        guidelines.push(format!(
            "Maintain {:?} tone with {:?} voice",
            organizational_style.preferred_tone, organizational_style.preferred_voice
        ));

        // Formality guidelines
        let formality_guidance = if organizational_style.formality_preference > 0.7 {
            "Use formal language and professional terminology"
        } else if organizational_style.formality_preference < 0.3 {
            "Use conversational and accessible language"
        } else {
            "Maintain balanced formality appropriate to context"
        };
        guidelines.push(formality_guidance.to_string());

        // Vocabulary guidelines
        guidelines.push(format!(
            "Target {:?} vocabulary complexity level",
            organizational_style.vocabulary_complexity
        ));

        // Pattern-specific guidelines
        for pattern in &organizational_style.common_patterns {
            if pattern.confidence > 0.7 {
                guidelines.push(format!(
                    "Follow {} pattern: {}",
                    pattern.pattern_type, pattern.pattern
                ));
            }
        }

        // Structural guidelines
        if !organizational_style.structural_patterns.is_empty() {
            guidelines.push(format!(
                "Use consistent structural patterns: {}",
                organizational_style.structural_patterns.join(", ")
            ));
        }

        guidelines
    }
}

#[derive(Debug)]
struct PreferredStyles {
    tone: ToneType,
    voice: VoiceType,
    vocabulary_complexity: VocabularyComplexity,
    formality: f64,
}

impl Default for StyleLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_learner_creation() {
        let learner = StyleLearner::new();
        assert_eq!(learner.min_frequency_threshold, 3);
        assert_eq!(learner.min_confidence_threshold, 0.6);
    }

    #[test]
    fn test_insufficient_data_error() {
        let learner = StyleLearner::new();
        let documents = vec![];
        let result = learner.learn_from_corpus(&documents, "test_org".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StyleLearnerError::InsufficientData(_)
        ));
    }

    #[test]
    fn test_domain_vocabulary_detection() {
        let learner = StyleLearner::new();

        let business_terms = vec![
            "leverage".to_string(),
            "synergy".to_string(),
            "stakeholder".to_string(),
        ];
        assert!(learner.contains_business_terms(&business_terms));

        let technical_terms = vec![
            "implementation".to_string(),
            "configuration".to_string(),
            "deployment".to_string(),
        ];
        assert!(learner.contains_technical_terms(&technical_terms));

        let academic_terms = vec![
            "hypothesis".to_string(),
            "methodology".to_string(),
            "empirical".to_string(),
        ];
        assert!(learner.contains_academic_terms(&academic_terms));
    }

    #[test]
    fn test_tone_parsing() {
        let learner = StyleLearner::new();

        assert!(matches!(
            learner.parse_tone_type("Formal"),
            ToneType::Formal
        ));
        assert!(matches!(
            learner.parse_tone_type("Professional"),
            ToneType::Professional
        ));
        assert!(matches!(
            learner.parse_tone_type("Invalid"),
            ToneType::Professional
        )); // Default
    }

    #[test]
    fn test_voice_parsing() {
        let learner = StyleLearner::new();

        assert!(matches!(
            learner.parse_voice_type("Authoritative"),
            VoiceType::Authoritative
        ));
        assert!(matches!(
            learner.parse_voice_type("Friendly"),
            VoiceType::Friendly
        ));
        assert!(matches!(
            learner.parse_voice_type("Invalid"),
            VoiceType::Neutral
        )); // Default
    }

    #[test]
    fn test_vocabulary_complexity_parsing() {
        let learner = StyleLearner::new();

        assert!(matches!(
            learner.parse_vocabulary_complexity("Basic"),
            VocabularyComplexity::Basic
        ));
        assert!(matches!(
            learner.parse_vocabulary_complexity("Expert"),
            VocabularyComplexity::Expert
        ));
        assert!(matches!(
            learner.parse_vocabulary_complexity("Invalid"),
            VocabularyComplexity::Intermediate
        )); // Default
    }

    #[test]
    fn test_confidence_calculation_empty() {
        let learner = StyleLearner::new();
        let empty_styles = vec![];
        let confidence = learner.calculate_learning_confidence(&empty_styles);
        assert_eq!(confidence, 0.0);
    }
}
