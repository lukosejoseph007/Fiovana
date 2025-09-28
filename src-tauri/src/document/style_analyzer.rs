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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VocabularyComplexity {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadabilityLevel {
    VeryEasy,        // 90-100 Flesch score
    Easy,            // 80-90
    FairlyEasy,      // 70-80
    Standard,        // 60-70
    FairlyDifficult, // 50-60
    Difficult,       // 30-50
    VeryDifficult,   // 0-30
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
    // Advanced vocabulary analysis fields
    pub flesch_reading_score: f64,
    pub flesch_kincaid_grade: f64,
    pub technical_density: f64,
    pub formality_index: f64,
    pub syllable_complexity: f64,
    pub jargon_ratio: f64,
    pub readability_level: ReadabilityLevel,
    pub domain_specific_terms: HashMap<String, Vec<String>>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToneType {
    Formal,
    Informal,
    Friendly,
    Professional,
    Academic,
    Conversational,
    Instructional,
    Technical,
    Authoritative,
    Empathetic,
    Persuasive,
    Objective,
    Enthusiastic,
    Cautious,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoiceType {
    Authoritative,  // Commands, directives, expertise
    Friendly,       // Warm, approachable, personal
    Neutral,        // Balanced, professional
    Encouraging,    // Supportive, motivational
    Explanatory,    // Educational, detailed
    Conversational, // Casual, informal dialogue
    Expert,         // Technical, specialized knowledge
    Empathetic,     // Understanding, caring
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmotionalTone {
    Positive,     // Optimistic, upbeat
    Negative,     // Critical, pessimistic
    Neutral,      // Balanced, objective
    Enthusiastic, // Excited, energetic
    Cautious,     // Careful, reserved
    Confident,    // Assured, certain
    Uncertain,    // Hesitant, questioning
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneAnalysis {
    pub primary_tone: ToneType,
    pub voice_type: VoiceType,
    pub emotional_tone: EmotionalTone,
    pub formality_score: f64,  // 0.0 = very informal, 1.0 = very formal
    pub directness_score: f64, // 0.0 = indirect, 1.0 = direct
    pub supportiveness: f64,   // 0.0 = unsupportive, 1.0 = very supportive
    pub authority_score: f64,  // 0.0 = submissive, 1.0 = authoritative
    pub empathy_score: f64,    // 0.0 = cold, 1.0 = very empathetic
    pub enthusiasm_score: f64, // 0.0 = neutral, 1.0 = very enthusiastic
    pub certainty_score: f64,  // 0.0 = uncertain, 1.0 = very certain
    pub politeness_score: f64, // 0.0 = rude, 1.0 = very polite
    pub confidence_indicators: Vec<String>,
    pub authority_indicators: Vec<String>,
    pub empathy_indicators: Vec<String>,
    pub enthusiasm_indicators: Vec<String>,
    pub characteristic_phrases: Vec<String>,
    pub tone_consistency: f64,
    pub voice_consistency: f64,
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
    // Advanced vocabulary analysis patterns
    academic_terms: Vec<&'static str>,
    business_jargon: Vec<&'static str>,
    scientific_terms: Vec<&'static str>,
    legal_terms: Vec<&'static str>,
    complex_connectors: Vec<&'static str>,
    // Advanced tone and voice detection patterns
    authority_indicators: Vec<&'static str>,
    empathy_indicators: Vec<&'static str>,
    enthusiasm_indicators: Vec<&'static str>,
    politeness_indicators: Vec<&'static str>,
    certainty_indicators: Vec<&'static str>,
    uncertainty_indicators: Vec<&'static str>,
    positive_emotion_words: Vec<&'static str>,
    negative_emotion_words: Vec<&'static str>,
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
            academic_terms: vec![
                "hypothesis",
                "methodology",
                "empirical",
                "theoretical",
                "paradigm",
                "conceptual",
                "framework",
                "analysis",
                "synthesis",
                "correlation",
                "implications",
                "phenomena",
                "discourse",
                "substantive",
                "comprehensive",
                "systematic",
                "prevalent",
                "inherent",
                "pertinent",
                "subsequently",
            ],
            business_jargon: vec![
                "synergy",
                "leverage",
                "paradigm",
                "optimization",
                "stakeholder",
                "deliverable",
                "actionable",
                "scalable",
                "benchmark",
                "strategize",
                "monetize",
                "streamline",
                "proactive",
                "mindshare",
                "bandwidth",
                "ideation",
                "holistic",
                "disruptive",
                "innovative",
                "ecosystem",
            ],
            scientific_terms: vec![
                "hypothesis",
                "methodology",
                "empirical",
                "statistical",
                "correlation",
                "regression",
                "variables",
                "coefficient",
                "algorithm",
                "protocol",
                "specimen",
                "parameters",
                "quantitative",
                "qualitative",
                "systematic",
                "experimental",
                "observational",
                "analytical",
                "diagnostic",
                "therapeutic",
            ],
            legal_terms: vec![
                "pursuant",
                "whereas",
                "herein",
                "thereof",
                "aforementioned",
                "hereafter",
                "notwithstanding",
                "heretofore",
                "provision",
                "stipulation",
                "compliance",
                "jurisdiction",
                "liability",
                "indemnification",
                "arbitration",
                "litigation",
                "statutory",
                "regulatory",
                "contractual",
                "fiduciary",
            ],
            complex_connectors: vec![
                "nevertheless",
                "furthermore",
                "consequently",
                "notwithstanding",
                "subsequently",
                "additionally",
                "specifically",
                "particularly",
                "accordingly",
                "alternatively",
                "meanwhile",
                "likewise",
                "otherwise",
                "conversely",
                "similarly",
                "ultimately",
                "essentially",
                "fundamentally",
                "predominantly",
                "substantially",
            ],
            authority_indicators: vec![
                "command",
                "require",
                "must",
                "shall",
                "mandatory",
                "directive",
                "insist",
                "demand",
                "instruct",
                "order",
                "decree",
                "dictate",
                "enforce",
                "compel",
                "establish",
                "determine",
                "decide",
                "rule",
                "declare",
                "assert",
            ],
            empathy_indicators: vec![
                "understand",
                "appreciate",
                "recognize",
                "acknowledge",
                "empathize",
                "sympathize",
                "care",
                "concern",
                "support",
                "comfort",
                "reassure",
                "encourage",
                "share",
                "feel",
                "relate",
                "experience",
                "struggle",
                "challenge",
                "difficulty",
                "sorry",
            ],
            enthusiasm_indicators: vec![
                "excited",
                "amazing",
                "fantastic",
                "wonderful",
                "excellent",
                "outstanding",
                "incredible",
                "awesome",
                "brilliant",
                "superb",
                "great",
                "love",
                "enjoy",
                "thrilled",
                "delighted",
                "pleased",
                "happy",
                "enthusiastic",
                "passionate",
                "eager",
            ],
            politeness_indicators: vec![
                "please",
                "thank you",
                "thanks",
                "appreciate",
                "grateful",
                "kindly",
                "would you",
                "could you",
                "may i",
                "excuse me",
                "sorry",
                "pardon",
                "welcome",
                "glad",
                "honor",
                "privilege",
                "respect",
                "consider",
                "suggest",
                "recommend",
            ],
            certainty_indicators: vec![
                "definitely",
                "certainly",
                "absolutely",
                "undoubtedly",
                "clearly",
                "obviously",
                "without doubt",
                "sure",
                "confident",
                "certain",
                "positive",
                "guarantee",
                "ensure",
                "assure",
                "confirm",
                "establish",
                "prove",
                "demonstrate",
                "verify",
                "validate",
            ],
            uncertainty_indicators: vec![
                "maybe",
                "perhaps",
                "possibly",
                "might",
                "could",
                "may",
                "seems",
                "appears",
                "likely",
                "probably",
                "suppose",
                "assume",
                "guess",
                "think",
                "believe",
                "suspect",
                "doubt",
                "uncertain",
                "unclear",
                "unsure",
            ],
            positive_emotion_words: vec![
                "happy",
                "joy",
                "pleased",
                "satisfied",
                "content",
                "delighted",
                "excited",
                "enthusiastic",
                "optimistic",
                "hopeful",
                "grateful",
                "thankful",
                "appreciate",
                "love",
                "enjoy",
                "wonderful",
                "excellent",
                "great",
                "amazing",
                "fantastic",
            ],
            negative_emotion_words: vec![
                "sad",
                "angry",
                "frustrated",
                "disappointed",
                "upset",
                "annoyed",
                "worried",
                "concerned",
                "anxious",
                "stressed",
                "difficult",
                "problem",
                "issue",
                "trouble",
                "challenge",
                "struggle",
                "fail",
                "error",
                "mistake",
                "wrong",
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
                flesch_reading_score: 0.0,
                flesch_kincaid_grade: 0.0,
                technical_density: 0.0,
                formality_index: 0.0,
                syllable_complexity: 0.0,
                jargon_ratio: 0.0,
                readability_level: ReadabilityLevel::Standard,
                domain_specific_terms: HashMap::new(),
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

        // Calculate advanced vocabulary metrics
        let flesch_reading_score = self.calculate_flesch_reading_score(content);
        let flesch_kincaid_grade = self.calculate_flesch_kincaid_grade(content);
        let technical_density = self.calculate_technical_density(content);
        let formality_index = self.calculate_formality_index(content);
        let syllable_complexity = self.calculate_syllable_complexity(content);
        let jargon_ratio = self.calculate_jargon_ratio(content);
        let readability_level = self.determine_readability_level(flesch_reading_score);
        let domain_specific_terms = self.identify_domain_specific_terms(content);

        VocabularyProfile {
            complexity,
            average_word_length,
            technical_terms_ratio,
            unique_words_ratio,
            common_terms: common_terms_list,
            technical_terms: technical_terms.into_iter().take(10).collect(),
            preferred_terms,
            avoided_terms: vec![], // Could be enhanced with more analysis
            flesch_reading_score,
            flesch_kincaid_grade,
            technical_density,
            formality_index,
            syllable_complexity,
            jargon_ratio,
            readability_level,
            domain_specific_terms,
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
                voice_type: VoiceType::Neutral,
                emotional_tone: EmotionalTone::Neutral,
                formality_score: 0.5,
                directness_score: 0.5,
                supportiveness: 0.5,
                authority_score: 0.5,
                empathy_score: 0.5,
                enthusiasm_score: 0.5,
                certainty_score: 0.5,
                politeness_score: 0.5,
                confidence_indicators: vec![],
                authority_indicators: vec![],
                empathy_indicators: vec![],
                enthusiasm_indicators: vec![],
                characteristic_phrases: vec![],
                tone_consistency: 0.5,
                voice_consistency: 0.5,
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

        // Calculate authority score
        let authority_count = self
            .authority_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;
        let authority_score = (authority_count / word_count * 100.0).min(1.0);

        // Calculate empathy score
        let empathy_count = self
            .empathy_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;
        let empathy_score = (empathy_count / word_count * 100.0).min(1.0);

        // Calculate enthusiasm score
        let enthusiasm_count = self
            .enthusiasm_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;
        let enthusiasm_score = (enthusiasm_count / word_count * 100.0).min(1.0);

        // Calculate politeness score
        let politeness_count = self
            .politeness_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;
        let politeness_score = (politeness_count / word_count * 100.0).min(1.0);

        // Calculate certainty score
        let certainty_count = self
            .certainty_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;
        let uncertainty_count = self
            .uncertainty_indicators
            .iter()
            .map(|indicator| lower_content.matches(indicator).count())
            .sum::<usize>() as f64;

        let certainty_score = if certainty_count + uncertainty_count > 0.0 {
            certainty_count / (certainty_count + uncertainty_count)
        } else {
            0.5
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

        // Calculate emotional tone
        let positive_emotion_count = self
            .positive_emotion_words
            .iter()
            .map(|word| lower_content.matches(word).count())
            .sum::<usize>() as f64;

        let negative_emotion_count = self
            .negative_emotion_words
            .iter()
            .map(|word| lower_content.matches(word).count())
            .sum::<usize>() as f64;

        let emotional_tone = if enthusiasm_score > 0.3 {
            EmotionalTone::Enthusiastic
        } else if positive_emotion_count > negative_emotion_count && positive_emotion_count > 0.0 {
            EmotionalTone::Positive
        } else if negative_emotion_count > positive_emotion_count && negative_emotion_count > 0.0 {
            EmotionalTone::Negative
        } else if certainty_score > 0.7 {
            EmotionalTone::Confident
        } else if certainty_score < 0.3 {
            EmotionalTone::Uncertain
        } else if authority_score > 0.2 && directness_score < 0.3 {
            EmotionalTone::Cautious
        } else {
            EmotionalTone::Neutral
        };

        // Determine voice type
        let voice_type = if authority_score > 0.3 {
            VoiceType::Authoritative
        } else if empathy_score > 0.3 {
            VoiceType::Empathetic
        } else if supportiveness > 0.3 {
            VoiceType::Encouraging
        } else if enthusiasm_score > 0.2 || positive_emotion_count > 0.0 {
            VoiceType::Friendly
        } else if lower_content.contains("technical") || lower_content.contains("specification") {
            VoiceType::Expert
        } else if formality_score < 0.4 {
            VoiceType::Conversational
        } else if lower_content.contains("explain")
            || lower_content.contains("describe")
            || lower_content.contains("understand")
        {
            VoiceType::Explanatory
        } else {
            VoiceType::Neutral
        };

        // Determine primary tone (enhanced)
        let primary_tone = if formality_score > 0.7 {
            if lower_content.contains("research") || lower_content.contains("analysis") {
                ToneType::Academic
            } else if authority_score > 0.3 {
                ToneType::Authoritative
            } else {
                ToneType::Formal
            }
        } else if formality_score < 0.3 {
            if enthusiasm_score > 0.3 {
                ToneType::Enthusiastic
            } else if supportiveness > 0.3 {
                ToneType::Friendly
            } else {
                ToneType::Informal
            }
        } else if empathy_score > 0.3 {
            ToneType::Empathetic
        } else if supportiveness > 0.4 {
            ToneType::Instructional
        } else if lower_content.contains("consider") || lower_content.contains("suggest") {
            ToneType::Persuasive
        } else if certainty_score < 0.3 && authority_score < 0.2 {
            ToneType::Cautious
        } else if lower_content.contains("technical") || lower_content.contains("specification") {
            ToneType::Technical
        } else if directness_score > 0.3 {
            ToneType::Professional
        } else if positive_emotion_count == 0.0
            && negative_emotion_count == 0.0
            && formality_score > 0.4
        {
            ToneType::Objective
        } else {
            ToneType::Conversational
        };

        // Extract indicators for different aspects
        let confidence_indicators = self
            .certainty_indicators
            .iter()
            .filter(|indicator| lower_content.contains(*indicator))
            .map(|s| s.to_string())
            .collect();

        let authority_indicators = self
            .authority_indicators
            .iter()
            .filter(|indicator| lower_content.contains(*indicator))
            .map(|s| s.to_string())
            .collect();

        let empathy_indicators = self
            .empathy_indicators
            .iter()
            .filter(|indicator| lower_content.contains(*indicator))
            .map(|s| s.to_string())
            .collect();

        let enthusiasm_indicators = self
            .enthusiasm_indicators
            .iter()
            .filter(|indicator| lower_content.contains(*indicator))
            .map(|s| s.to_string())
            .collect();

        // Find characteristic phrases (enhanced)
        let characteristic_phrases = vec![
            "it is important to",
            "please note",
            "for example",
            "in order to",
            "make sure",
            "keep in mind",
            "as you can see",
            "it should be noted",
            "we believe",
            "in our opinion",
            "you might consider",
            "it is recommended",
            "feel free to",
            "don't hesitate",
        ]
        .into_iter()
        .filter(|phrase| lower_content.contains(phrase))
        .map(|s| s.to_string())
        .collect();

        // Calculate tone consistency
        let tone_scores = [
            formality_score,
            authority_score,
            empathy_score,
            enthusiasm_score,
        ];
        let max_score = tone_scores.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_score = tone_scores.iter().fold(1.0f64, |a, &b| a.min(b));
        let tone_consistency = 1.0 - (max_score - min_score);

        // Calculate voice consistency
        let voice_scores = [
            authority_score,
            empathy_score,
            supportiveness,
            enthusiasm_score,
        ];
        let voice_max = voice_scores.iter().fold(0.0f64, |a, &b| a.max(b));
        let voice_min = voice_scores.iter().fold(1.0f64, |a, &b| a.min(b));
        let voice_consistency = 1.0 - (voice_max - voice_min);

        ToneAnalysis {
            primary_tone,
            voice_type,
            emotional_tone,
            formality_score,
            directness_score,
            supportiveness,
            authority_score,
            empathy_score,
            enthusiasm_score,
            certainty_score,
            politeness_score,
            confidence_indicators,
            authority_indicators,
            empathy_indicators,
            enthusiasm_indicators,
            characteristic_phrases,
            tone_consistency,
            voice_consistency,
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

    // Advanced vocabulary analysis methods

    /// Calculate Flesch Reading Ease score
    /// Formula: 206.835 - (1.015 × ASL) - (84.6 × ASW)
    /// ASL = average sentence length, ASW = average syllables per word
    fn calculate_flesch_reading_score(&self, content: &str) -> f64 {
        let sentences = self.count_sentences(content);
        let words = self.count_words(content);
        let syllables = self.count_syllables(content);

        if sentences == 0 || words == 0 {
            return 0.0;
        }

        let asl = words as f64 / sentences as f64; // Average sentence length
        let asw = syllables as f64 / words as f64; // Average syllables per word

        let score = 206.835 - (1.015 * asl) - (84.6 * asw);
        score.clamp(0.0, 100.0)
    }

    /// Calculate Flesch-Kincaid Grade Level
    /// Formula: (0.39 × ASL) + (11.8 × ASW) - 15.59
    fn calculate_flesch_kincaid_grade(&self, content: &str) -> f64 {
        let sentences = self.count_sentences(content);
        let words = self.count_words(content);
        let syllables = self.count_syllables(content);

        if sentences == 0 || words == 0 {
            return 0.0;
        }

        let asl = words as f64 / sentences as f64;
        let asw = syllables as f64 / words as f64;

        let grade = (0.39 * asl) + (11.8 * asw) - 15.59;
        grade.max(0.0)
    }

    /// Calculate technical density as ratio of technical terms to total words
    fn calculate_technical_density(&self, content: &str) -> f64 {
        let words = self.get_words(content);
        if words.is_empty() {
            return 0.0;
        }

        let technical_count = words
            .iter()
            .filter(|word| {
                self.technical_patterns
                    .iter()
                    .any(|pattern| word.to_lowercase().contains(pattern))
                    || self
                        .scientific_terms
                        .iter()
                        .any(|term| word.to_lowercase().contains(term))
            })
            .count();

        technical_count as f64 / words.len() as f64
    }

    /// Calculate formality index based on formal vs informal indicators
    fn calculate_formality_index(&self, content: &str) -> f64 {
        let lower_content = content.to_lowercase();
        let word_count = self.count_words(content) as f64;

        if word_count == 0.0 {
            return 0.5;
        }

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

        let academic_count = self
            .academic_terms
            .iter()
            .map(|term| lower_content.matches(term).count())
            .sum::<usize>() as f64;

        let legal_count = self
            .legal_terms
            .iter()
            .map(|term| lower_content.matches(term).count())
            .sum::<usize>() as f64;

        let complex_connector_count = self
            .complex_connectors
            .iter()
            .map(|connector| lower_content.matches(connector).count())
            .sum::<usize>() as f64;

        // Weight different indicators
        let formal_score = formal_count * 1.0
            + academic_count * 1.2
            + legal_count * 1.5
            + complex_connector_count * 0.8;
        let informal_score = informal_count * 1.0;

        let total_indicators = formal_score + informal_score;
        if total_indicators == 0.0 {
            return 0.5; // Neutral
        }

        (formal_score / total_indicators).min(1.0)
    }

    /// Calculate syllable complexity (average syllables per word)
    fn calculate_syllable_complexity(&self, content: &str) -> f64 {
        let words = self.count_words(content);
        let syllables = self.count_syllables(content);

        if words == 0 {
            return 0.0;
        }

        syllables as f64 / words as f64
    }

    /// Calculate jargon ratio (domain-specific terms)
    fn calculate_jargon_ratio(&self, content: &str) -> f64 {
        let words = self.get_words(content);
        if words.is_empty() {
            return 0.0;
        }

        let jargon_count = words
            .iter()
            .filter(|word| {
                let lower_word = word.to_lowercase();
                self.business_jargon
                    .iter()
                    .any(|jargon| lower_word.contains(jargon))
                    || self
                        .academic_terms
                        .iter()
                        .any(|term| lower_word.contains(term))
                    || self
                        .legal_terms
                        .iter()
                        .any(|term| lower_word.contains(term))
            })
            .count();

        jargon_count as f64 / words.len() as f64
    }

    /// Determine readability level from Flesch reading score
    fn determine_readability_level(&self, flesch_score: f64) -> ReadabilityLevel {
        match flesch_score {
            90.0..=100.0 => ReadabilityLevel::VeryEasy,
            80.0..=89.9 => ReadabilityLevel::Easy,
            70.0..=79.9 => ReadabilityLevel::FairlyEasy,
            60.0..=69.9 => ReadabilityLevel::Standard,
            50.0..=59.9 => ReadabilityLevel::FairlyDifficult,
            30.0..=49.9 => ReadabilityLevel::Difficult,
            _ => ReadabilityLevel::VeryDifficult,
        }
    }

    /// Identify domain-specific terms categorized by domain
    fn identify_domain_specific_terms(&self, content: &str) -> HashMap<String, Vec<String>> {
        let lower_content = content.to_lowercase();
        let mut domain_terms = HashMap::new();

        // Academic terms
        let academic_found: Vec<String> = self
            .academic_terms
            .iter()
            .filter(|term| lower_content.contains(*term))
            .map(|s| s.to_string())
            .collect();
        if !academic_found.is_empty() {
            domain_terms.insert("academic".to_string(), academic_found);
        }

        // Business jargon
        let business_found: Vec<String> = self
            .business_jargon
            .iter()
            .filter(|term| lower_content.contains(*term))
            .map(|s| s.to_string())
            .collect();
        if !business_found.is_empty() {
            domain_terms.insert("business".to_string(), business_found);
        }

        // Scientific terms
        let scientific_found: Vec<String> = self
            .scientific_terms
            .iter()
            .filter(|term| lower_content.contains(*term))
            .map(|s| s.to_string())
            .collect();
        if !scientific_found.is_empty() {
            domain_terms.insert("scientific".to_string(), scientific_found);
        }

        // Legal terms
        let legal_found: Vec<String> = self
            .legal_terms
            .iter()
            .filter(|term| lower_content.contains(*term))
            .map(|s| s.to_string())
            .collect();
        if !legal_found.is_empty() {
            domain_terms.insert("legal".to_string(), legal_found);
        }

        // Technical terms
        let technical_found: Vec<String> = self
            .technical_patterns
            .iter()
            .filter(|term| lower_content.contains(*term))
            .map(|s| s.to_string())
            .collect();
        if !technical_found.is_empty() {
            domain_terms.insert("technical".to_string(), technical_found);
        }

        domain_terms
    }

    // Helper methods for advanced analysis

    fn count_sentences(&self, content: &str) -> usize {
        content
            .chars()
            .filter(|&c| c == '.' || c == '!' || c == '?')
            .count()
            .max(1) // Ensure at least 1 sentence for calculation safety
    }

    fn count_words(&self, content: &str) -> usize {
        content.split_whitespace().filter(|w| !w.is_empty()).count()
    }

    fn get_words(&self, content: &str) -> Vec<String> {
        content
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
            .filter(|w| !w.is_empty())
            .map(|w| w.to_string())
            .collect()
    }

    fn count_syllables(&self, content: &str) -> usize {
        let words = self.get_words(content);
        words
            .iter()
            .map(|word| self.count_syllables_in_word(word))
            .sum()
    }

    /// Simple syllable counting algorithm
    fn count_syllables_in_word(&self, word: &str) -> usize {
        let word = word.to_lowercase();
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        let mut syllable_count = 0;
        let mut prev_was_vowel = false;

        for ch in word.chars() {
            let is_vowel = vowels.contains(&ch);
            if is_vowel && !prev_was_vowel {
                syllable_count += 1;
            }
            prev_was_vowel = is_vowel;
        }

        // Handle silent 'e'
        if word.ends_with('e') && syllable_count > 1 {
            syllable_count -= 1;
        }

        // Ensure at least 1 syllable per word
        syllable_count.max(1)
    }
}

impl Default for StyleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
