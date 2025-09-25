// src-tauri/src/document/content_classifier.rs
//! Content categorization system for intelligent document processing
//!
//! This module provides rule-based and AI-assisted content classification
//! to categorize different types of content within documents.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ai::AIOrchestrator;
use crate::document::DocumentStructureAnalysis;

/// Content classifier for categorizing document content
pub struct ContentClassifier {
    /// Rule-based classification patterns
    classification_rules: HashMap<ContentCategory, Vec<ClassificationRule>>,
    /// AI client for advanced categorization
    ai_client: Option<AIOrchestrator>,
}

/// Content category types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum ContentCategory {
    /// Step-by-step procedures
    Procedures,
    /// Explanatory text
    Explanations,
    /// Examples and illustrations
    Examples,
    /// Definitions and terminology
    Definitions,
    /// Questions and answers
    QAndA,
    /// Warnings and cautions
    Warnings,
    /// Best practices and recommendations
    BestPractices,
    /// Technical specifications
    TechnicalSpecs,
    /// Troubleshooting guides
    Troubleshooting,
    /// Background information
    Background,
    /// Reference material
    Reference,
    /// Unknown or mixed content
    Unknown,
}

/// Classification rule for pattern matching
#[derive(Debug, Clone)]
pub struct ClassificationRule {
    /// Regex pattern to match
    pub pattern: Regex,
    /// Weight/confidence of this rule
    pub weight: f64,
    /// Contextual indicators that boost confidence
    pub context_indicators: Vec<String>,
}

/// Content classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentClassification {
    /// Primary category
    pub primary_category: ContentCategory,
    /// Secondary categories with confidence scores
    pub secondary_categories: Vec<(ContentCategory, f64)>,
    /// Overall confidence score (0.0-1.0)
    pub confidence: f64,
    /// Content excerpt that led to classification
    pub evidence: Vec<String>,
    /// Reasoning behind classification
    pub reasoning: Option<String>,
}

/// Document content analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentAnalysis {
    /// Content classifications by section
    pub section_classifications: HashMap<String, ContentClassification>,
    /// Overall document content distribution
    pub content_distribution: HashMap<ContentCategory, f64>,
    /// Dominant content type in document
    pub dominant_content_type: ContentCategory,
    /// Content complexity score
    pub complexity_score: f64,
}

impl ContentClassifier {
    /// Create a new content classifier
    pub fn new(ai_client: Option<AIOrchestrator>) -> Result<Self> {
        let mut classifier = Self {
            classification_rules: HashMap::new(),
            ai_client,
        };

        classifier.initialize_rules()?;
        Ok(classifier)
    }

    /// Initialize classification rules
    fn initialize_rules(&mut self) -> Result<()> {
        // Procedures classification rules
        let procedure_rules = vec![
            ClassificationRule {
                pattern: Regex::new(r"(?i)step\s+\d+|^\d+\.|first,|then,|next,|finally,")?,
                weight: 1.0,
                context_indicators: vec![
                    "procedure".to_string(),
                    "instruction".to_string(),
                    "how to".to_string(),
                ],
            },
            ClassificationRule {
                pattern: Regex::new(r"(?i)follow these steps|to do this|perform the following")?,
                weight: 0.9,
                context_indicators: vec!["guide".to_string(), "tutorial".to_string()],
            },
        ];

        // Explanations classification rules
        let explanation_rules = vec![
            ClassificationRule {
                pattern: Regex::new(
                    r"(?i)this is|this means|in other words|essentially|basically",
                )?,
                weight: 0.9,
                context_indicators: vec![
                    "explanation".to_string(),
                    "description".to_string(),
                    "overview".to_string(),
                ],
            },
            ClassificationRule {
                pattern: Regex::new(r"(?i)because|due to|as a result|therefore|consequently")?,
                weight: 0.7,
                context_indicators: vec!["reason".to_string(), "cause".to_string()],
            },
        ];

        // Examples classification rules
        let example_rules = vec![
            ClassificationRule {
                pattern: Regex::new(r"(?i)for example|such as|like|including|instance")?,
                weight: 0.8,
                context_indicators: vec![
                    "example".to_string(),
                    "sample".to_string(),
                    "illustration".to_string(),
                ],
            },
            ClassificationRule {
                pattern: Regex::new(r"(?i)consider|imagine|suppose|let's say")?,
                weight: 0.6,
                context_indicators: vec!["scenario".to_string(), "case".to_string()],
            },
        ];

        // Definitions classification rules
        let definition_rules = vec![
            ClassificationRule {
                pattern: Regex::new(r"(?i)is defined as|definition:|^(API|Term|Concept|Word):\s")?,
                weight: 0.9,
                context_indicators: vec![
                    "definition".to_string(),
                    "term".to_string(),
                    "glossary".to_string(),
                ],
            },
            ClassificationRule {
                pattern: Regex::new(r"(?i)^[A-Z][a-z]+\s-\s|means that|refers to")?,
                weight: 0.8,
                context_indicators: vec!["terminology".to_string()],
            },
        ];

        // Q&A classification rules
        let qa_rules = vec![ClassificationRule {
            pattern: Regex::new(r"(?i)^Q:|^A:|question:|answer:|what is|how do|why does")?,
            weight: 0.9,
            context_indicators: vec![
                "faq".to_string(),
                "question".to_string(),
                "answer".to_string(),
            ],
        }];

        // Warning classification rules
        let warning_rules = vec![ClassificationRule {
            pattern: Regex::new(r"(?i)warning|caution|danger|important|note:|tip:")?,
            weight: 0.9,
            context_indicators: vec![
                "alert".to_string(),
                "safety".to_string(),
                "critical".to_string(),
            ],
        }];

        // Best practices classification rules
        let best_practice_rules = vec![ClassificationRule {
            pattern: Regex::new(r"(?i)best practice|recommended|should|avoid|don't|do not")?,
            weight: 0.7,
            context_indicators: vec![
                "guideline".to_string(),
                "recommendation".to_string(),
                "standard".to_string(),
            ],
        }];

        // Technical specs classification rules
        let tech_spec_rules = vec![ClassificationRule {
            pattern: Regex::new(r"(?i)specification|requirement|parameter|config|setting")?,
            weight: 0.8,
            context_indicators: vec![
                "technical".to_string(),
                "system".to_string(),
                "api".to_string(),
            ],
        }];

        // Troubleshooting classification rules
        let troubleshooting_rules = vec![ClassificationRule {
            pattern: Regex::new(r"(?i)problem|issue|error|troubleshoot|fix|solve")?,
            weight: 0.8,
            context_indicators: vec![
                "troubleshooting".to_string(),
                "debug".to_string(),
                "support".to_string(),
            ],
        }];

        // Store all rules
        self.classification_rules
            .insert(ContentCategory::Procedures, procedure_rules);
        self.classification_rules
            .insert(ContentCategory::Explanations, explanation_rules);
        self.classification_rules
            .insert(ContentCategory::Examples, example_rules);
        self.classification_rules
            .insert(ContentCategory::Definitions, definition_rules);
        self.classification_rules
            .insert(ContentCategory::QAndA, qa_rules);
        self.classification_rules
            .insert(ContentCategory::Warnings, warning_rules);
        self.classification_rules
            .insert(ContentCategory::BestPractices, best_practice_rules);
        self.classification_rules
            .insert(ContentCategory::TechnicalSpecs, tech_spec_rules);
        self.classification_rules
            .insert(ContentCategory::Troubleshooting, troubleshooting_rules);

        Ok(())
    }

    /// Classify content using rule-based approach
    pub fn classify_content(&self, content: &str, context: Option<&str>) -> ContentClassification {
        let mut category_scores: HashMap<ContentCategory, f64> = HashMap::new();
        let mut evidence = Vec::new();

        // Apply all classification rules
        for (category, rules) in &self.classification_rules {
            let mut category_score = 0.0;

            for rule in rules {
                if rule.pattern.is_match(content) {
                    let mut rule_score = rule.weight;

                    // Boost score based on context indicators
                    if let Some(ctx) = context {
                        let ctx_lower = ctx.to_lowercase();
                        for indicator in &rule.context_indicators {
                            if ctx_lower.contains(indicator) {
                                rule_score *= 1.2; // 20% boost for context match
                            }
                        }
                    }

                    category_score += rule_score;

                    // Collect evidence
                    if let Some(matched) = rule.pattern.find(content) {
                        evidence.push(matched.as_str().to_string());
                    }
                }
            }

            if category_score > 0.0 {
                category_scores.insert(category.clone(), category_score);
            }
        }

        // Determine primary category and confidence
        let (primary_category, confidence) = if category_scores.is_empty() {
            (ContentCategory::Unknown, 0.0)
        } else {
            let max_score = category_scores.values().fold(0.0f64, |a, &b| a.max(b));
            let primary = category_scores
                .iter()
                .find(|(_, &score)| score == max_score)
                .map(|(cat, _)| cat.clone())
                .unwrap_or(ContentCategory::Unknown);

            // Normalize confidence to 0.0-1.0 range
            let confidence = (max_score / 2.0).min(1.0);
            (primary, confidence)
        };

        // Build secondary categories
        let mut secondary_categories: Vec<(ContentCategory, f64)> = category_scores
            .into_iter()
            .filter(|(cat, _)| *cat != primary_category)
            .map(|(cat, score)| (cat, (score / 2.0).min(1.0)))
            .collect();

        secondary_categories
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        secondary_categories.truncate(3); // Keep top 3 secondary categories

        ContentClassification {
            primary_category,
            secondary_categories,
            confidence,
            evidence: evidence.into_iter().take(5).collect(), // Keep top 5 evidence pieces
            reasoning: None,                                  // Will be filled by AI if available
        }
    }

    /// Classify content with AI assistance
    pub async fn classify_content_with_ai(
        &self,
        content: &str,
        context: Option<&str>,
    ) -> Result<ContentClassification> {
        // Start with rule-based classification
        let mut classification = self.classify_content(content, context);

        // Enhance with AI if available
        if let Some(ai) = &self.ai_client {
            let ai_prompt = format!(
                "Analyze this content and classify its type. Consider categories like: procedures, explanations, examples, definitions, Q&A, warnings, best practices, technical specs, troubleshooting, background, reference.\n\nContent:\n{}\n\nContext: {}\n\nProvide a brief reasoning for your classification.",
                content,
                context.unwrap_or("No additional context")
            );

            if let Ok(ai_response) = ai.process_conversation(&ai_prompt, None).await {
                classification.reasoning = Some(ai_response.content);
                // Boost confidence if AI agrees with rule-based classification
                if classification.confidence < 0.8 {
                    classification.confidence = (classification.confidence + 0.2).min(1.0);
                }
            }
        }

        Ok(classification)
    }

    /// Analyze entire document structure for content types
    pub async fn analyze_document_content(
        &self,
        structure_analysis: &DocumentStructureAnalysis,
        _full_content: &str,
    ) -> Result<DocumentContentAnalysis> {
        let mut section_classifications = HashMap::new();
        let mut category_totals: HashMap<ContentCategory, f64> = HashMap::new();
        let mut total_content_length = 0.0;

        // Classify each section
        for section in &structure_analysis.sections {
            let section_context = format!("{:?}", section.section_type);

            let classification = if self.ai_client.is_some() {
                self.classify_content_with_ai(&section.content, Some(&section_context))
                    .await?
            } else {
                self.classify_content(&section.content, Some(&section_context))
            };

            let content_length = section.content.len() as f64;
            total_content_length += content_length;

            // Update category totals weighted by content length
            let weight = content_length * classification.confidence;
            *category_totals
                .entry(classification.primary_category.clone())
                .or_insert(0.0) += weight;

            section_classifications.insert(section.id.clone(), classification);
        }

        // Calculate content distribution
        let content_distribution: HashMap<ContentCategory, f64> = category_totals
            .iter()
            .map(|(cat, &total)| (cat.clone(), total / total_content_length))
            .collect();

        // Determine dominant content type
        let dominant_content_type = content_distribution
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(cat, _)| cat.clone())
            .unwrap_or(ContentCategory::Unknown);

        // Calculate complexity score based on content diversity
        let complexity_score = self.calculate_complexity_score(&content_distribution);

        Ok(DocumentContentAnalysis {
            section_classifications,
            content_distribution,
            dominant_content_type,
            complexity_score,
        })
    }

    /// Calculate document complexity based on content type diversity
    fn calculate_complexity_score(&self, distribution: &HashMap<ContentCategory, f64>) -> f64 {
        let num_categories = distribution.len() as f64;
        let max_categories = 12.0; // Total number of content categories

        // Diversity score: more categories = more complex
        let diversity_score = (num_categories / max_categories).min(1.0);

        // Balance score: even distribution = more complex
        let total_weight: f64 = distribution.values().sum();
        let balance_score = if total_weight > 0.0 {
            let entropy = distribution
                .values()
                .map(|&weight| {
                    let p = weight / total_weight;
                    if p > 0.0 {
                        -p * p.log2()
                    } else {
                        0.0
                    }
                })
                .sum::<f64>();

            // Normalize entropy (max entropy for uniform distribution)
            entropy / (num_categories.log2()).max(1.0)
        } else {
            0.0
        };

        // Combine diversity and balance for final complexity score
        (diversity_score * 0.6 + balance_score * 0.4).min(1.0)
    }
}

impl Default for ContentClassifier {
    fn default() -> Self {
        Self::new(None).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedure_classification() {
        let classifier = ContentClassifier::default();
        let content = "Step 1: First, open the application. Step 2: Then, navigate to settings.";

        let result = classifier.classify_content(content, None);
        assert_eq!(result.primary_category, ContentCategory::Procedures);
        assert!(result.confidence > 0.4);
    }

    #[test]
    fn test_explanation_classification() {
        let classifier = ContentClassifier::default();
        let content = "This is a comprehensive explanation of how the system works. Essentially, it processes data.";

        let result = classifier.classify_content(content, None);
        assert_eq!(result.primary_category, ContentCategory::Explanations);
    }

    #[test]
    fn test_example_classification() {
        let classifier = ContentClassifier::default();
        let content = "For example, consider a scenario where you need to process multiple files.";

        let result = classifier.classify_content(content, None);
        assert_eq!(result.primary_category, ContentCategory::Examples);
    }

    #[test]
    fn test_definition_classification() {
        let classifier = ContentClassifier::default();
        let content = "API: An Application Programming Interface is defined as a set of protocols.";

        let result = classifier.classify_content(content, None);
        assert_eq!(result.primary_category, ContentCategory::Definitions);
    }

    #[test]
    fn test_warning_classification() {
        let classifier = ContentClassifier::default();
        let content =
            "Warning: This operation will permanently delete all data. Proceed with caution.";

        let result = classifier.classify_content(content, None);
        assert_eq!(result.primary_category, ContentCategory::Warnings);
    }

    #[test]
    fn test_unknown_classification() {
        let classifier = ContentClassifier::default();
        let content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

        let result = classifier.classify_content(content, None);
        assert_eq!(result.primary_category, ContentCategory::Unknown);
        assert_eq!(result.confidence, 0.0);
    }
}
