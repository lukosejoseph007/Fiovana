use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::document::indexer::DocumentIndexEntry;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum StyleAnalysisError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    #[error("Style analysis failed: {0}")]
    AnalysisFailed(String),
    #[error("Invalid content: {0}")]
    InvalidContent(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VocabularyComplexity {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyProfile {
    pub complexity: VocabularyComplexity,
    pub average_word_length: f64,
    pub technical_terms_ratio: f64,
    pub unique_words_ratio: f64,
    pub common_terms: Vec<String>,
    pub technical_terms: Vec<String>,
    pub preferred_terms: HashMap<String, i32>,
    pub avoided_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentencePatterns {
    pub average_length: f64,
    pub complexity_score: f64,
    pub compound_sentence_ratio: f64,
    pub question_ratio: f64,
    pub exclamation_ratio: f64,
    pub passive_voice_ratio: f64,
    pub typical_structures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToneType {
    Formal,
    Informal,
    Friendly,
    Professional,
    Academic,
    Conversational,
    Instructional,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneAnalysis {
    pub primary_tone: ToneType,
    pub formality_score: f64,  // 0.0 = very informal, 1.0 = very formal
    pub directness_score: f64, // 0.0 = indirect, 1.0 = direct
    pub supportiveness: f64,   // 0.0 = unsupportive, 1.0 = very supportive
    pub confidence_indicators: Vec<String>,
    pub characteristic_phrases: Vec<String>,
    pub tone_consistency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPatterns {
    pub typical_flow: Vec<String>,
    pub heading_style: String,
    pub list_preferences: String,
    pub paragraph_length_avg: f64,
    pub section_organization: String,
    pub explanation_patterns: Vec<String>,
    pub transition_style: Vec<String>,
    pub visual_elements_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingProfile {
    pub emphasis_patterns: HashMap<String, i32>,
    pub heading_conventions: Vec<String>,
    pub list_formatting: String,
    pub spacing_preferences: String,
    pub visual_density: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    pub vocabulary: VocabularyProfile,
    pub sentence_structure: SentencePatterns,
    pub tone: ToneAnalysis,
    pub structure: StructuralPatterns,
    pub formatting: FormattingProfile,
    pub confidence_score: f64,
    pub sample_size: usize,
    pub document_sources: Vec<String>,
    pub analysis_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSimilarity {
    pub overall_similarity: f64,
    pub vocabulary_similarity: f64,
    pub tone_similarity: f64,
    pub structure_similarity: f64,
    pub formatting_similarity: f64,
    pub differences: Vec<String>,
    pub recommendations: Vec<String>,
}

pub struct StyleAnalyzer {
    // Technical term patterns for identifying complexity
    technical_patterns: Vec<&'static str>,
    // Common transition words
    transition_words: Vec<&'static str>,
    // Formality indicators
    formal_indicators: Vec<&'static str>,
    informal_indicators: Vec<&'static str>,
}

impl StyleAnalyzer {
    pub fn new() -> Self {
        Self {
            technical_patterns: vec![
                "implementation",
                "configuration",
                "authentication",
                "authorization",
                "optimization",
                "algorithm",
                "protocol",
                "interface",
                "methodology",
                "specification",
                "architecture",
                "framework",
                "infrastructure",
                "deployment",
                "integration",
                "validation",
                "verification",
            ],
            transition_words: vec![
                "however",
                "therefore",
                "furthermore",
                "moreover",
                "consequently",
                "nevertheless",
                "additionally",
                "specifically",
                "particularly",
                "subsequently",
                "accordingly",
                "meanwhile",
                "likewise",
                "otherwise",
            ],
            formal_indicators: vec![
                "shall",
                "must",
                "require",
                "ensure",
                "establish",
                "maintain",
                "implement",
                "demonstrate",
                "facilitate",
                "utilize",
                "obtain",
                "pursuant",
                "whereas",
                "herein",
                "thereof",
                "aforementioned",
            ],
            informal_indicators: vec![
                "you'll",
                "we'll",
                "it's",
                "don't",
                "can't",
                "won't",
                "let's",
                "here's",
                "that's",
                "what's",
                "okay",
                "sure",
                "cool",
                "awesome",
                "stuff",
                "things",
                "guys",
                "folks",
                "basically",
                "pretty much",
            ],
        }
    }

    pub fn analyze_document_style(
        &self,
        document: &DocumentIndexEntry,
    ) -> Result<StyleProfile, StyleAnalysisError> {
        let content = &document.content;

        if content.trim().is_empty() {
            return Err(StyleAnalysisError::InvalidContent(
                "Document content is empty".to_string(),
            ));
        }

        let vocabulary = self.analyze_vocabulary(content);
        let sentence_structure = self.analyze_sentence_structure(content);
        let tone = self.analyze_tone(content);
        let structure = self.analyze_structural_patterns(content, &document.title);
        let formatting = self.analyze_formatting(content);

        let confidence_score = self.calculate_confidence_score(content);

        Ok(StyleProfile {
            vocabulary,
            sentence_structure,
            tone,
            structure,
            formatting,
            confidence_score,
            sample_size: content.len(),
            document_sources: vec![document.title.clone()],
            analysis_date: format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
        })
    }

    pub fn analyze_corpus_style(
        &self,
        documents: &[DocumentIndexEntry],
    ) -> Result<StyleProfile, StyleAnalysisError> {
        if documents.is_empty() {
            return Err(StyleAnalysisError::InvalidContent(
                "No documents provided".to_string(),
            ));
        }

        // Combine all content for analysis
        let combined_content = documents
            .iter()
            .map(|doc| doc.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let vocabulary = self.analyze_vocabulary(&combined_content);
        let sentence_structure = self.analyze_sentence_structure(&combined_content);
        let tone = self.analyze_tone(&combined_content);
        let structure = self.analyze_structural_patterns(&combined_content, "Corpus Analysis");
        let formatting = self.analyze_formatting(&combined_content);

        let confidence_score = self.calculate_confidence_score(&combined_content);

        Ok(StyleProfile {
            vocabulary,
            sentence_structure,
            tone,
            structure,
            formatting,
            confidence_score,
            sample_size: combined_content.len(),
            document_sources: documents.iter().map(|doc| doc.title.clone()).collect(),
            analysis_date: format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
        })
    }

    pub fn compare_styles(&self, style1: &StyleProfile, style2: &StyleProfile) -> StyleSimilarity {
        let vocab_sim =
            self.calculate_vocabulary_similarity(&style1.vocabulary, &style2.vocabulary);
        let tone_sim = self.calculate_tone_similarity(&style1.tone, &style2.tone);
        let structure_sim =
            self.calculate_structure_similarity(&style1.structure, &style2.structure);
        let formatting_sim =
            self.calculate_formatting_similarity(&style1.formatting, &style2.formatting);

        let overall_sim = (vocab_sim + tone_sim + structure_sim + formatting_sim) / 4.0;

        let differences = self.identify_differences(style1, style2);
        let recommendations = self.generate_recommendations(&differences);

        StyleSimilarity {
            overall_similarity: overall_sim,
            vocabulary_similarity: vocab_sim,
            tone_similarity: tone_sim,
            structure_similarity: structure_sim,
            formatting_similarity: formatting_sim,
            differences,
            recommendations,
        }
    }

    fn analyze_vocabulary(&self, content: &str) -> VocabularyProfile {
        let lower_content = content.to_lowercase();
        let words: Vec<&str> = lower_content
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        if words.is_empty() {
            return VocabularyProfile {
                complexity: VocabularyComplexity::Basic,
                average_word_length: 0.0,
                technical_terms_ratio: 0.0,
                unique_words_ratio: 0.0,
                common_terms: vec![],
                technical_terms: vec![],
                preferred_terms: HashMap::new(),
                avoided_terms: vec![],
            };
        }

        let average_word_length =
            words.iter().map(|w| w.len()).sum::<usize>() as f64 / words.len() as f64;

        let mut word_counts = HashMap::new();
        for word in &words {
            *word_counts.entry(word.to_string()).or_insert(0) += 1;
        }

        let unique_words_ratio = word_counts.len() as f64 / words.len() as f64;

        let technical_terms: Vec<String> = words
            .iter()
            .filter(|word| {
                self.technical_patterns
                    .iter()
                    .any(|pattern| word.contains(pattern))
            })
            .map(|w| w.to_string())
            .collect();

        let technical_terms_ratio = technical_terms.len() as f64 / words.len() as f64;

        // Determine complexity based on multiple factors
        let complexity = if technical_terms_ratio > 0.15 && average_word_length > 6.0 {
            VocabularyComplexity::Expert
        } else if technical_terms_ratio > 0.1 || average_word_length > 5.5 {
            VocabularyComplexity::Advanced
        } else if average_word_length > 4.5 {
            VocabularyComplexity::Intermediate
        } else {
            VocabularyComplexity::Basic
        };

        // Get most common terms (excluding stop words)
        let mut common_terms: Vec<(String, i32)> = word_counts.into_iter().collect();
        common_terms.sort_by(|a, b| b.1.cmp(&a.1));
        common_terms.truncate(20);

        let common_terms_list: Vec<String> = common_terms
            .iter()
            .filter(|(word, count)| *count > 2 && word.len() > 3)
            .take(10)
            .map(|(word, _)| word.clone())
            .collect();

        let preferred_terms: HashMap<String, i32> = common_terms.into_iter().collect();

        VocabularyProfile {
            complexity,
            average_word_length,
            technical_terms_ratio,
            unique_words_ratio,
            common_terms: common_terms_list,
            technical_terms: technical_terms.into_iter().take(10).collect(),
            preferred_terms,
            avoided_terms: vec![], // Could be enhanced with more analysis
        }
    }

    fn analyze_sentence_structure(&self, content: &str) -> SentencePatterns {
        let sentences: Vec<&str> = content
            .split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() > 5)
            .collect();

        if sentences.is_empty() {
            return SentencePatterns {
                average_length: 0.0,
                complexity_score: 0.0,
                compound_sentence_ratio: 0.0,
                question_ratio: 0.0,
                exclamation_ratio: 0.0,
                passive_voice_ratio: 0.0,
                typical_structures: vec![],
            };
        }

        let total_words: usize = sentences.iter().map(|s| s.split_whitespace().count()).sum();
        let average_length = total_words as f64 / sentences.len() as f64;

        // Count compound sentences (containing conjunctions)
        let compound_count = sentences
            .iter()
            .filter(|s| {
                s.contains(" and ")
                    || s.contains(" but ")
                    || s.contains(" or ")
                    || s.contains(", which")
            })
            .count();
        let compound_sentence_ratio = compound_count as f64 / sentences.len() as f64;

        // Count questions and exclamations from original content
        let question_count = content.matches('?').count();
        let exclamation_count = content.matches('!').count();
        let total_sentences = sentences.len() + question_count + exclamation_count;

        let question_ratio = question_count as f64 / total_sentences as f64;
        let exclamation_ratio = exclamation_count as f64 / total_sentences as f64;

        // Simple passive voice detection
        let passive_count = sentences
            .iter()
            .filter(|s| {
                let lower = s.to_lowercase();
                lower.contains("was ") && (lower.contains("ed ") || lower.contains("en "))
                    || lower.contains("were ") && (lower.contains("ed ") || lower.contains("en "))
                    || lower.contains("is ") && (lower.contains("ed ") || lower.contains("en "))
                    || lower.contains("are ") && (lower.contains("ed ") || lower.contains("en "))
            })
            .count();
        let passive_voice_ratio = passive_count as f64 / sentences.len() as f64;

        // Calculate complexity based on multiple factors
        let complexity_score = (average_length / 15.0).min(1.0) * 0.4
            + compound_sentence_ratio * 0.3
            + passive_voice_ratio * 0.2
            + (if average_length > 20.0 { 0.1 } else { 0.0 });

        let typical_structures = if average_length > 15.0 {
            vec![
                "Complex sentences".to_string(),
                "Detailed explanations".to_string(),
            ]
        } else {
            vec![
                "Simple sentences".to_string(),
                "Direct statements".to_string(),
            ]
        };

        SentencePatterns {
            average_length,
            complexity_score: complexity_score.min(1.0),
            compound_sentence_ratio,
            question_ratio,
            exclamation_ratio,
            passive_voice_ratio,
            typical_structures,
        }
    }

    fn analyze_tone(&self, content: &str) -> ToneAnalysis {
        let lower_content = content.to_lowercase();
        let word_count = content.split_whitespace().count() as f64;

        if word_count == 0.0 {
            return ToneAnalysis {
                primary_tone: ToneType::Professional,
                formality_score: 0.5,
                directness_score: 0.5,
                supportiveness: 0.5,
                confidence_indicators: vec![],
                characteristic_phrases: vec![],
                tone_consistency: 0.5,
            };
        }

        // Count formal vs informal indicators
        let formal_count = self
            .formal_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;

        let informal_count = self
            .informal_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;

        let formality_score = if formal_count + informal_count > 0.0 {
            formal_count / (formal_count + informal_count)
        } else {
            0.5 // Neutral if no indicators found
        };

        // Analyze directness (imperative sentences, direct commands)
        let direct_indicators = [
            "must",
            "should",
            "will",
            "do",
            "don't",
            "ensure",
            "make sure",
        ];
        let direct_count = direct_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;

        let directness_score = (direct_count / word_count * 100.0).min(1.0);

        // Analyze supportiveness (encouraging words, helpful phrases)
        let supportive_indicators = [
            "help",
            "support",
            "assist",
            "guide",
            "easy",
            "simple",
            "clear",
            "understand",
            "learn",
            "improve",
            "benefit",
            "advantage",
        ];
        let supportive_count = supportive_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;

        let supportiveness = (supportive_count / word_count * 100.0).min(1.0);

        // Determine primary tone
        let primary_tone = if formality_score > 0.7 {
            if lower_content.contains("research") || lower_content.contains("analysis") {
                ToneType::Academic
            } else {
                ToneType::Formal
            }
        } else if formality_score < 0.3 {
            if supportiveness > 0.3 {
                ToneType::Friendly
            } else {
                ToneType::Informal
            }
        } else if supportiveness > 0.4 {
            ToneType::Instructional
        } else if lower_content.contains("technical") || lower_content.contains("specification") {
            ToneType::Technical
        } else if directness_score > 0.3 {
            ToneType::Professional
        } else {
            ToneType::Conversational
        };

        // Extract confidence indicators
        let confidence_indicators = vec![
            "definitely",
            "certainly",
            "clearly",
            "obviously",
            "undoubtedly",
            "ensure",
            "guarantee",
            "proven",
            "established",
            "confirmed",
        ]
        .into_iter()
        .filter(|indicator| lower_content.contains(indicator))
        .map(|s| s.to_string())
        .collect();

        // Find characteristic phrases (this is simplified)
        let characteristic_phrases = vec![
            "it is important to",
            "please note",
            "for example",
            "in order to",
            "make sure",
            "keep in mind",
            "as you can see",
            "it should be noted",
        ]
        .into_iter()
        .filter(|phrase| lower_content.contains(phrase))
        .map(|s| s.to_string())
        .collect();

        // Calculate tone consistency (simplified - could be enhanced)
        let tone_consistency = if !(0.2..=0.8).contains(&formality_score) {
            0.9 // Very consistent
        } else if !(0.4..=0.6).contains(&formality_score) {
            0.7 // Moderately consistent
        } else {
            0.5 // Mixed tone
        };

        ToneAnalysis {
            primary_tone,
            formality_score,
            directness_score,
            supportiveness,
            confidence_indicators,
            characteristic_phrases,
            tone_consistency,
        }
    }

    fn analyze_structural_patterns(&self, content: &str, _title: &str) -> StructuralPatterns {
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let lower_content = content.to_lowercase();

        if lines.is_empty() {
            return StructuralPatterns {
                typical_flow: vec![],
                heading_style: "Unknown".to_string(),
                list_preferences: "Unknown".to_string(),
                paragraph_length_avg: 0.0,
                section_organization: "Unknown".to_string(),
                explanation_patterns: vec![],
                transition_style: vec![],
                visual_elements_usage: 0.0,
            };
        }

        // Analyze heading patterns
        let heading_style = if content.contains("# ") || content.contains("## ") {
            "Markdown headers".to_string()
        } else if content.contains("1.") && content.contains("2.") {
            "Numbered headers".to_string()
        } else if lines
            .iter()
            .any(|line| *line == line.to_uppercase() && line.len() > 5)
        {
            "ALL CAPS headers".to_string()
        } else {
            "Minimal headers".to_string()
        };

        // Analyze list preferences
        let list_preferences = if content.contains("- ") || content.contains("* ") {
            "Bullet points".to_string()
        } else if content.contains("1. ") || content.contains("2. ") {
            "Numbered lists".to_string()
        } else {
            "Paragraph format".to_string()
        };

        // Calculate paragraph length
        let paragraphs: Vec<&str> = content
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();
        let paragraph_length_avg = if paragraphs.is_empty() {
            0.0
        } else {
            paragraphs
                .iter()
                .map(|p| p.split_whitespace().count())
                .sum::<usize>() as f64
                / paragraphs.len() as f64
        };

        // Determine section organization
        let section_organization = if content.contains("Introduction")
            || content.contains("Overview")
        {
            "Structured with introduction".to_string()
        } else if content.contains("Step") || content.contains("First") || content.contains("Next")
        {
            "Procedural organization".to_string()
        } else if content.contains("Example") || content.contains("For instance") {
            "Example-driven".to_string()
        } else {
            "Linear organization".to_string()
        };

        // Find explanation patterns
        let explanation_patterns = vec![
            "that is",
            "in other words",
            "for example",
            "specifically",
            "namely",
        ]
        .into_iter()
        .filter(|pattern| lower_content.contains(*pattern))
        .map(|s| s.to_string())
        .collect();

        // Find transition style
        let transition_style: Vec<String> = self
            .transition_words
            .iter()
            .filter(|word| lower_content.contains(*word))
            .map(|s| s.to_string())
            .collect();

        // Estimate visual elements usage
        let visual_indicators = [
            "figure",
            "table",
            "image",
            "chart",
            "diagram",
            "see below",
            "above",
        ];
        let visual_count = visual_indicators
            .iter()
            .map(|indicator| content.to_lowercase().matches(indicator).count())
            .sum::<usize>();
        let visual_elements_usage = (visual_count as f64 / content.len() as f64 * 1000.0).min(1.0);

        let typical_flow = if content.contains("Introduction") {
            vec![
                "Introduction".to_string(),
                "Main content".to_string(),
                "Conclusion".to_string(),
            ]
        } else if content.contains("Step") {
            vec![
                "Prerequisites".to_string(),
                "Steps".to_string(),
                "Completion".to_string(),
            ]
        } else {
            vec!["Sequential".to_string(), "Topic-based".to_string()]
        };

        StructuralPatterns {
            typical_flow,
            heading_style,
            list_preferences,
            paragraph_length_avg,
            section_organization,
            explanation_patterns,
            transition_style,
            visual_elements_usage,
        }
    }

    fn analyze_formatting(&self, content: &str) -> FormattingProfile {
        let mut emphasis_patterns = HashMap::new();

        // Count different emphasis types
        emphasis_patterns.insert("bold".to_string(), content.matches("**").count() as i32);
        emphasis_patterns.insert(
            "italic".to_string(),
            content.matches("*").count() as i32 - emphasis_patterns["bold"] * 2,
        );
        emphasis_patterns.insert("underline".to_string(), content.matches("_").count() as i32);
        emphasis_patterns.insert(
            "caps".to_string(),
            content
                .split_whitespace()
                .filter(|word| word == &word.to_uppercase() && word.len() > 2)
                .count() as i32,
        );

        // Analyze heading conventions
        let heading_conventions = if content.contains("# ") {
            vec!["Markdown headings".to_string()]
        } else if content.contains("=====") || content.contains("-----") {
            vec!["Underlined headings".to_string()]
        } else {
            vec!["Minimal heading formatting".to_string()]
        };

        // Analyze list formatting
        let list_formatting = if content.contains("- ") || content.contains("* ") {
            "Bullet lists".to_string()
        } else if content.contains("1. ") {
            "Numbered lists".to_string()
        } else {
            "Paragraph lists".to_string()
        };

        // Analyze spacing preferences
        let double_line_breaks = content.matches("\n\n").count();
        let single_line_breaks = content.matches("\n").count() - double_line_breaks * 2;

        let spacing_preferences = if double_line_breaks > single_line_breaks / 2 {
            "Generous spacing".to_string()
        } else {
            "Compact spacing".to_string()
        };

        // Determine visual density
        let total_chars = content.len();
        let whitespace_chars = content.chars().filter(|c| c.is_whitespace()).count();
        let density_ratio = (total_chars - whitespace_chars) as f64 / total_chars as f64;

        let visual_density = if density_ratio > 0.8 {
            "Dense".to_string()
        } else if density_ratio > 0.6 {
            "Moderate".to_string()
        } else {
            "Sparse".to_string()
        };

        FormattingProfile {
            emphasis_patterns,
            heading_conventions,
            list_formatting,
            spacing_preferences,
            visual_density,
        }
    }

    fn calculate_confidence_score(&self, content: &str) -> f64 {
        let content_length = content.len() as f64;
        let word_count = content.split_whitespace().count() as f64;

        // Confidence based on content volume and variety
        let length_factor = if content_length > 5000.0 {
            1.0
        } else if content_length > 1000.0 {
            0.8
        } else if content_length > 500.0 {
            0.6
        } else {
            0.3
        };

        let word_factor = if word_count > 500.0 {
            1.0
        } else if word_count > 100.0 {
            0.7
        } else {
            0.4
        };

        (length_factor + word_factor) / 2.0
    }

    // Helper methods for style comparison
    fn calculate_vocabulary_similarity(
        &self,
        vocab1: &VocabularyProfile,
        vocab2: &VocabularyProfile,
    ) -> f64 {
        let complexity_match = match (&vocab1.complexity, &vocab2.complexity) {
            (VocabularyComplexity::Basic, VocabularyComplexity::Basic) => 1.0,
            (VocabularyComplexity::Intermediate, VocabularyComplexity::Intermediate) => 1.0,
            (VocabularyComplexity::Advanced, VocabularyComplexity::Advanced) => 1.0,
            (VocabularyComplexity::Expert, VocabularyComplexity::Expert) => 1.0,
            _ => 0.5,
        };

        let length_diff = (vocab1.average_word_length - vocab2.average_word_length).abs();
        let length_similarity = (5.0 - length_diff).max(0.0) / 5.0;

        let tech_diff = (vocab1.technical_terms_ratio - vocab2.technical_terms_ratio).abs();
        let tech_similarity = (0.3 - tech_diff).max(0.0) / 0.3;

        (complexity_match + length_similarity + tech_similarity) / 3.0
    }

    fn calculate_tone_similarity(&self, tone1: &ToneAnalysis, tone2: &ToneAnalysis) -> f64 {
        let tone_match = if std::mem::discriminant(&tone1.primary_tone)
            == std::mem::discriminant(&tone2.primary_tone)
        {
            1.0
        } else {
            0.3
        };

        let formality_diff = (tone1.formality_score - tone2.formality_score).abs();
        let formality_similarity = (1.0 - formality_diff).max(0.0);

        let directness_diff = (tone1.directness_score - tone2.directness_score).abs();
        let directness_similarity = (1.0 - directness_diff).max(0.0);

        (tone_match + formality_similarity + directness_similarity) / 3.0
    }

    fn calculate_structure_similarity(
        &self,
        struct1: &StructuralPatterns,
        struct2: &StructuralPatterns,
    ) -> f64 {
        let heading_match = if struct1.heading_style == struct2.heading_style {
            1.0
        } else {
            0.5
        };
        let list_match = if struct1.list_preferences == struct2.list_preferences {
            1.0
        } else {
            0.5
        };
        let org_match = if struct1.section_organization == struct2.section_organization {
            1.0
        } else {
            0.6
        };

        let paragraph_diff = (struct1.paragraph_length_avg - struct2.paragraph_length_avg).abs();
        let paragraph_similarity = (50.0 - paragraph_diff).max(0.0) / 50.0;

        (heading_match + list_match + org_match + paragraph_similarity) / 4.0
    }

    fn calculate_formatting_similarity(
        &self,
        format1: &FormattingProfile,
        format2: &FormattingProfile,
    ) -> f64 {
        let list_match = if format1.list_formatting == format2.list_formatting {
            1.0
        } else {
            0.5
        };
        let spacing_match = if format1.spacing_preferences == format2.spacing_preferences {
            1.0
        } else {
            0.7
        };
        let density_match = if format1.visual_density == format2.visual_density {
            1.0
        } else {
            0.6
        };

        (list_match + spacing_match + density_match) / 3.0
    }

    fn identify_differences(&self, style1: &StyleProfile, style2: &StyleProfile) -> Vec<String> {
        let mut differences = Vec::new();

        if std::mem::discriminant(&style1.vocabulary.complexity)
            != std::mem::discriminant(&style2.vocabulary.complexity)
        {
            differences.push(format!(
                "Vocabulary complexity differs: {:?} vs {:?}",
                style1.vocabulary.complexity, style2.vocabulary.complexity
            ));
        }

        if std::mem::discriminant(&style1.tone.primary_tone)
            != std::mem::discriminant(&style2.tone.primary_tone)
        {
            differences.push(format!(
                "Tone differs: {:?} vs {:?}",
                style1.tone.primary_tone, style2.tone.primary_tone
            ));
        }

        if (style1.tone.formality_score - style2.tone.formality_score).abs() > 0.3 {
            differences.push(format!(
                "Formality level differs significantly: {:.2} vs {:.2}",
                style1.tone.formality_score, style2.tone.formality_score
            ));
        }

        if style1.structure.heading_style != style2.structure.heading_style {
            differences.push(format!(
                "Heading style differs: {} vs {}",
                style1.structure.heading_style, style2.structure.heading_style
            ));
        }

        differences
    }

    fn generate_recommendations(&self, differences: &[String]) -> Vec<String> {
        let mut recommendations = Vec::new();

        for difference in differences {
            if difference.contains("Vocabulary complexity") {
                recommendations
                    .push("Consider adjusting vocabulary level to match target style".to_string());
            } else if difference.contains("Tone differs") {
                recommendations.push("Adapt tone to match the target document style".to_string());
            } else if difference.contains("Formality level") {
                recommendations.push("Adjust formality level to maintain consistency".to_string());
            } else if difference.contains("Heading style") {
                recommendations
                    .push("Use consistent heading format throughout documents".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Styles are well-aligned".to_string());
        }

        recommendations
    }

    /// Analyze style of raw content string
    pub fn analyze_content_style(&self, content: &str) -> Result<StyleProfile, StyleAnalysisError> {
        if content.trim().is_empty() {
            return Err(StyleAnalysisError::InvalidContent(
                "Content cannot be empty".to_string(),
            ));
        }

        let vocabulary = self.analyze_vocabulary(content);
        let sentence_structure = self.analyze_sentence_structure(content);
        let tone = self.analyze_tone(content);
        let structure = self.analyze_structural_patterns(content, "");
        let formatting = self.analyze_formatting(content);
        let confidence_score = self.calculate_confidence_score(content);

        Ok(StyleProfile {
            vocabulary,
            sentence_structure,
            tone,
            structure,
            formatting,
            confidence_score,
            sample_size: content.split_whitespace().count(),
            document_sources: vec!["raw_content".to_string()],
            analysis_date: chrono::Utc::now().to_rfc3339(),
        })
    }
}

impl Default for StyleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
