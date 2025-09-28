// src-tauri/src/document/document_comparison.rs
// Document comparison functionality using vector similarity and content analysis

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    DocxContent, PdfContent, StyleAnalyzer, StyleProfile, StyleTransfer, StyleTransferConfig,
    StyleTransferMode, StyleTransferTarget,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentComparisonRequest {
    pub document_a: DocumentForComparison,
    pub document_b: DocumentForComparison,
    pub comparison_type: ComparisonType,
    pub options: ComparisonOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentForComparison {
    pub file_path: String,
    pub content: Option<ParsedDocumentContent>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParsedDocumentContent {
    Docx { content: DocxContent },
    Pdf { content: PdfContent },
    Text { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonType {
    /// Basic text-based comparison
    TextDiff,
    /// Semantic comparison using vector embeddings
    SemanticSimilarity,
    /// Structural comparison (headings, sections, etc.)
    StructuralDiff,
    /// Combined comparison with all methods
    Comprehensive,
    /// Style-preserving comparison for document updates
    StylePreserving,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonOptions {
    pub include_metadata: bool,
    pub similarity_threshold: f64,
    pub max_differences: Option<usize>,
    pub ignore_formatting: bool,
    pub focus_on_content_changes: bool,
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            similarity_threshold: 0.8,
            max_differences: Some(100),
            ignore_formatting: true,
            focus_on_content_changes: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentComparisonResult {
    pub comparison_id: String,
    pub comparison_type: ComparisonType,
    pub summary: ComparisonSummary,
    pub differences: Vec<DocumentDifference>,
    pub similarity_score: f64,
    pub processing_time_ms: u64,
    pub metadata_comparison: Option<MetadataComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    pub total_differences: usize,
    pub major_changes: usize,
    pub minor_changes: usize,
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub overall_similarity: f64,
    pub content_similarity: f64,
    pub structural_similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDifference {
    pub diff_type: DifferenceType,
    pub severity: DifferenceSeverity,
    pub location: DifferenceLocation,
    pub description: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceType {
    Addition,
    Deletion,
    Modification,
    Structural,
    Formatting,
    Semantic,
    StyleChange,
    ToneShift,
    VocabularyChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceSeverity {
    Critical, // Major content changes
    Major,    // Significant changes
    Minor,    // Small changes
    Cosmetic, // Formatting only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferenceLocation {
    pub section: Option<String>,
    pub paragraph: Option<usize>,
    pub line: Option<usize>,
    pub page: Option<usize>,
    pub character_offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataComparison {
    pub title_changed: bool,
    pub author_changed: bool,
    pub creation_date_changed: bool,
    pub modification_date_changed: bool,
    pub other_changes: HashMap<String, (Option<String>, Option<String>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePreservingResult {
    pub original_style: StyleProfile,
    pub updated_style: StyleProfile,
    pub style_differences: Vec<StyleDifference>,
    pub style_preservation_score: f64,
    pub suggested_updates: Vec<StyleConsistentUpdate>,
    pub preserved_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDifference {
    pub difference_type: StyleDifferenceType,
    pub severity: DifferenceSeverity,
    pub description: String,
    pub original_value: String,
    pub updated_value: String,
    pub impact_score: f64,
    pub preservation_suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleDifferenceType {
    ToneChange,
    VocabularyComplexityShift,
    FormalityChange,
    VoiceChange,
    SentenceStructureChange,
    TerminologyInconsistency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConsistentUpdate {
    pub update_type: UpdateType,
    pub target_section: String,
    pub original_text: String,
    pub suggested_text: String,
    pub reasoning: String,
    pub confidence: f64,
    pub style_preservation_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    ContentUpdate,
    StyleAdjustment,
    VocabularyReplacement,
    ToneCorrection,
    StructuralAlignment,
}

pub struct DocumentComparator {
    #[allow(dead_code)]
    vector_similarity_threshold: f64,
}

impl DocumentComparator {
    pub fn new() -> Self {
        Self {
            vector_similarity_threshold: 0.8,
        }
    }

    pub async fn compare_documents(
        &self,
        request: DocumentComparisonRequest,
    ) -> Result<DocumentComparisonResult> {
        let start_time = std::time::Instant::now();
        let comparison_id = uuid::Uuid::new_v4().to_string();

        // Extract content from both documents
        let content_a = self.extract_content(&request.document_a)?;
        let content_b = self.extract_content(&request.document_b)?;

        // Perform comparison based on type
        let (mut differences, similarity_score) = match request.comparison_type {
            ComparisonType::TextDiff => self.text_based_comparison(&content_a, &content_b).await?,
            ComparisonType::SemanticSimilarity => {
                self.semantic_comparison(&content_a, &content_b).await?
            }
            ComparisonType::StructuralDiff => {
                self.structural_comparison(&content_a, &content_b).await?
            }
            ComparisonType::Comprehensive => {
                self.comprehensive_comparison(&content_a, &content_b)
                    .await?
            }
            ComparisonType::StylePreserving => {
                self.style_preserving_comparison(&content_a, &content_b)
                    .await?
            }
        };

        // Enhance differences with AI-powered analysis
        if !differences.is_empty() {
            if let Err(e) = self
                .enhance_differences_with_ai(&mut differences, &content_a, &content_b)
                .await
            {
                tracing::warn!("Failed to enhance differences with AI analysis: {}", e);
                // Continue without AI enhancement
            }
        }

        // Compare metadata if requested
        let metadata_comparison = if request.options.include_metadata {
            Some(self.compare_metadata(&request.document_a, &request.document_b)?)
        } else {
            None
        };

        // Generate enhanced summary
        let summary = self
            .generate_enhanced_summary(&differences, similarity_score, &content_a, &content_b)
            .await;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(DocumentComparisonResult {
            comparison_id,
            comparison_type: request.comparison_type,
            summary,
            differences,
            similarity_score,
            processing_time_ms: processing_time,
            metadata_comparison,
        })
    }

    fn extract_content(&self, document: &DocumentForComparison) -> Result<String> {
        match &document.content {
            Some(ParsedDocumentContent::Docx { content }) => Ok(content.text.clone()),
            Some(ParsedDocumentContent::Pdf { content }) => Ok(content.text.clone()),
            Some(ParsedDocumentContent::Text { content }) => Ok(content.clone()),
            None => Err(anyhow::anyhow!("No content available for document")),
        }
    }

    async fn text_based_comparison(
        &self,
        content_a: &str,
        content_b: &str,
    ) -> Result<(Vec<DocumentDifference>, f64)> {
        let mut differences = Vec::new();

        // Simple line-by-line comparison
        let lines_a: Vec<&str> = content_a.lines().collect();
        let lines_b: Vec<&str> = content_b.lines().collect();

        // Calculate basic similarity
        let similarity = self.calculate_text_similarity(content_a, content_b);

        // Use a simple diff algorithm
        let changes = self.compute_diff(&lines_a, &lines_b)?;

        for (i, change) in changes.iter().enumerate() {
            differences.push(DocumentDifference {
                diff_type: change.diff_type.clone(),
                severity: self.assess_change_severity(change),
                location: DifferenceLocation {
                    section: None,
                    paragraph: None,
                    line: Some(i),
                    page: None,
                    character_offset: None,
                },
                description: change.description.clone(),
                before: change.before.clone(),
                after: change.after.clone(),
                confidence: 0.9, // High confidence for text diff
            });
        }

        Ok((differences, similarity))
    }

    async fn semantic_comparison(
        &self,
        content_a: &str,
        content_b: &str,
    ) -> Result<(Vec<DocumentDifference>, f64)> {
        // For now, fall back to text comparison
        // In a full implementation, this would use the vector search system
        // to compare document embeddings and find semantic differences

        let similarity = self
            .calculate_semantic_similarity(content_a, content_b)
            .await?;
        let (mut differences, _) = self.text_based_comparison(content_a, content_b).await?;

        // Mark differences as semantic
        for diff in &mut differences {
            diff.diff_type = DifferenceType::Semantic;
            diff.confidence = similarity;
        }

        Ok((differences, similarity))
    }

    async fn structural_comparison(
        &self,
        content_a: &str,
        content_b: &str,
    ) -> Result<(Vec<DocumentDifference>, f64)> {
        let mut differences = Vec::new();

        // Extract structural elements
        let structure_a = self.extract_structure(content_a);
        let structure_b = self.extract_structure(content_b);

        // Compare structures
        let structural_diff = self.compare_structures(&structure_a, &structure_b);
        let similarity = self.calculate_structural_similarity(&structure_a, &structure_b);

        for diff in structural_diff {
            differences.push(DocumentDifference {
                diff_type: DifferenceType::Structural,
                severity: DifferenceSeverity::Major,
                location: DifferenceLocation {
                    section: diff.section,
                    paragraph: None,
                    line: None,
                    page: None,
                    character_offset: None,
                },
                description: diff.description,
                before: diff.before,
                after: diff.after,
                confidence: 0.85,
            });
        }

        Ok((differences, similarity))
    }

    async fn comprehensive_comparison(
        &self,
        content_a: &str,
        content_b: &str,
    ) -> Result<(Vec<DocumentDifference>, f64)> {
        // Combine all comparison methods
        let (text_diffs, text_sim) = self.text_based_comparison(content_a, content_b).await?;
        let (semantic_diffs, semantic_sim) = self.semantic_comparison(content_a, content_b).await?;
        let (structural_diffs, structural_sim) =
            self.structural_comparison(content_a, content_b).await?;

        let mut all_differences = Vec::new();
        all_differences.extend(text_diffs);
        all_differences.extend(semantic_diffs);
        all_differences.extend(structural_diffs);

        // Calculate weighted average similarity
        let overall_similarity = text_sim * 0.4 + semantic_sim * 0.4 + structural_sim * 0.2;

        Ok((all_differences, overall_similarity))
    }

    async fn style_preserving_comparison(
        &self,
        content_a: &str,
        content_b: &str,
    ) -> Result<(Vec<DocumentDifference>, f64)> {
        let mut differences = Vec::new();

        // Analyze style of both documents
        let style_analyzer = StyleAnalyzer::new();
        let style_a = style_analyzer.analyze_content_style(content_a)?;
        let style_b = style_analyzer.analyze_content_style(content_b)?;

        // Calculate style similarity
        let style_similarity = self.calculate_style_similarity(&style_a, &style_b)?;

        // Detect style differences
        let style_differences = self.detect_style_differences(&style_a, &style_b)?;

        // Convert style differences to document differences
        for style_diff in style_differences {
            let diff_type = match style_diff.difference_type {
                StyleDifferenceType::ToneChange => DifferenceType::ToneShift,
                StyleDifferenceType::VocabularyComplexityShift => DifferenceType::VocabularyChange,
                StyleDifferenceType::FormalityChange => DifferenceType::StyleChange,
                StyleDifferenceType::VoiceChange => DifferenceType::StyleChange,
                StyleDifferenceType::SentenceStructureChange => DifferenceType::Structural,
                StyleDifferenceType::TerminologyInconsistency => DifferenceType::VocabularyChange,
            };

            differences.push(DocumentDifference {
                diff_type,
                severity: style_diff.severity,
                location: DifferenceLocation {
                    section: None,
                    paragraph: None,
                    line: None,
                    page: None,
                    character_offset: None,
                },
                description: style_diff.description,
                before: Some(style_diff.original_value),
                after: Some(style_diff.updated_value),
                confidence: style_diff.impact_score,
            });
        }

        Ok((differences, style_similarity))
    }

    fn compare_metadata(
        &self,
        doc_a: &DocumentForComparison,
        doc_b: &DocumentForComparison,
    ) -> Result<MetadataComparison> {
        let mut other_changes = HashMap::new();

        // Compare common metadata fields
        for (key, value_a) in &doc_a.metadata {
            let value_b = doc_b.metadata.get(key);
            if value_b != Some(value_a) {
                other_changes.insert(key.clone(), (Some(value_a.clone()), value_b.cloned()));
            }
        }

        // Check for new fields in doc_b
        for (key, value_b) in &doc_b.metadata {
            if !doc_a.metadata.contains_key(key) {
                other_changes.insert(key.clone(), (None, Some(value_b.clone())));
            }
        }

        Ok(MetadataComparison {
            title_changed: doc_a.metadata.get("title") != doc_b.metadata.get("title"),
            author_changed: doc_a.metadata.get("author") != doc_b.metadata.get("author"),
            creation_date_changed: doc_a.metadata.get("creation_date")
                != doc_b.metadata.get("creation_date"),
            modification_date_changed: doc_a.metadata.get("modification_date")
                != doc_b.metadata.get("modification_date"),
            other_changes,
        })
    }

    fn generate_summary(
        &self,
        differences: &[DocumentDifference],
        similarity_score: f64,
    ) -> ComparisonSummary {
        let total_differences = differences.len();
        let major_changes = differences
            .iter()
            .filter(|d| {
                matches!(
                    d.severity,
                    DifferenceSeverity::Critical | DifferenceSeverity::Major
                )
            })
            .count();
        let minor_changes = differences
            .iter()
            .filter(|d| {
                matches!(
                    d.severity,
                    DifferenceSeverity::Minor | DifferenceSeverity::Cosmetic
                )
            })
            .count();

        let additions = differences
            .iter()
            .filter(|d| matches!(d.diff_type, DifferenceType::Addition))
            .count();
        let deletions = differences
            .iter()
            .filter(|d| matches!(d.diff_type, DifferenceType::Deletion))
            .count();
        let modifications = differences
            .iter()
            .filter(|d| matches!(d.diff_type, DifferenceType::Modification))
            .count();

        ComparisonSummary {
            total_differences,
            major_changes,
            minor_changes,
            additions,
            deletions,
            modifications,
            overall_similarity: similarity_score,
            content_similarity: similarity_score,
            structural_similarity: similarity_score, // Simplified for now
        }
    }

    fn calculate_text_similarity(&self, text_a: &str, text_b: &str) -> f64 {
        // Simple Jaccard similarity for now
        let words_a: std::collections::HashSet<&str> = text_a.split_whitespace().collect();
        let words_b: std::collections::HashSet<&str> = text_b.split_whitespace().collect();

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }

    async fn calculate_semantic_similarity(&self, text_a: &str, text_b: &str) -> Result<f64> {
        use crate::vector::{EmbeddingConfig, EmbeddingEngine};

        // Use the vector system to calculate semantic similarity
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await?;

        // Generate embeddings for both texts
        let embedding_a = engine.embed_text(text_a).await?;
        let embedding_b = engine.embed_text(text_b).await?;

        // Calculate cosine similarity
        let similarity = self.cosine_similarity(&embedding_a, &embedding_b);
        Ok(similarity as f64)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn extract_structure(&self, content: &str) -> DocumentStructure {
        let mut headings = Vec::new();
        let mut sections = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_section = String::new();
        let mut section_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Enhanced heuristics for structure detection
            if self.looks_like_heading(trimmed) {
                // Save previous section if any
                if !current_section.is_empty() {
                    sections.push(StructuralElement {
                        element_type: "section".to_string(),
                        content: current_section.clone(),
                        level: 1,
                        line_number: section_start,
                    });
                }

                // Add heading
                let level = self.estimate_heading_level(trimmed);
                headings.push(StructuralElement {
                    element_type: "heading".to_string(),
                    content: trimmed.to_string(),
                    level,
                    line_number: i,
                });

                // Start new section
                current_section.clear();
                section_start = i;
            } else if !trimmed.is_empty() {
                // Add to current section
                if !current_section.is_empty() {
                    current_section.push('\n');
                }
                current_section.push_str(trimmed);
            }
        }

        // Add final section
        if !current_section.is_empty() {
            sections.push(StructuralElement {
                element_type: "section".to_string(),
                content: current_section,
                level: 1,
                line_number: section_start,
            });
        }

        DocumentStructure { headings, sections }
    }

    fn looks_like_heading(&self, line: &str) -> bool {
        if line.len() > 150 || line.is_empty() {
            return false;
        }

        // Check for common heading patterns
        let line_lower = line.to_lowercase();

        // Numbered headings (1., 2., etc.)
        if line.matches('.').count() == 1 && line.chars().take(5).any(|c| c.is_ascii_digit()) {
            return true;
        }

        // All caps (but not too long)
        if line.len() < 80 && line == line.to_uppercase() && line.chars().any(|c| c.is_alphabetic())
        {
            return true;
        }

        // Ends with colon
        if line.ends_with(':') && line.len() < 100 {
            return true;
        }

        // Common heading words
        let heading_indicators = [
            "introduction",
            "conclusion",
            "summary",
            "overview",
            "background",
            "methodology",
            "results",
            "discussion",
            "references",
            "appendix",
            "chapter",
            "section",
            "abstract",
            "executive summary",
        ];

        for indicator in &heading_indicators {
            if line_lower.contains(indicator) && line.len() < 100 {
                return true;
            }
        }

        // Short line that doesn't end with punctuation (except colon)
        line.len() < 60
            && !line.ends_with('.')
            && !line.ends_with(',')
            && !line.ends_with(';')
            && line.chars().any(|c| c.is_alphabetic())
            && !line.contains("http")
    }

    fn estimate_heading_level(&self, line: &str) -> usize {
        // Numbered headings
        if let Some(first_char) = line.chars().next() {
            if first_char.is_ascii_digit() {
                // Try to determine level from numbering
                let number_part: String = line
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                let dot_count = number_part.matches('.').count();
                return dot_count.clamp(1, 6);
            }
        }

        // All caps likely level 1
        if line.len() < 50 && line == line.to_uppercase() {
            return 1;
        }

        // Common high-level sections
        let line_lower = line.to_lowercase();
        let level_1_words = [
            "introduction",
            "conclusion",
            "summary",
            "abstract",
            "executive summary",
        ];
        let level_2_words = ["background", "methodology", "results", "discussion"];

        for word in &level_1_words {
            if line_lower.contains(word) {
                return 1;
            }
        }

        for word in &level_2_words {
            if line_lower.contains(word) {
                return 2;
            }
        }

        // Default to level 2
        2
    }

    fn compare_structures(
        &self,
        structure_a: &DocumentStructure,
        structure_b: &DocumentStructure,
    ) -> Vec<StructuralDiff> {
        let mut diffs = Vec::new();

        // Compare headings
        if structure_a.headings.len() != structure_b.headings.len() {
            diffs.push(StructuralDiff {
                section: Some("Document Structure".to_string()),
                description: format!(
                    "Number of headings changed from {} to {}",
                    structure_a.headings.len(),
                    structure_b.headings.len()
                ),
                before: Some(structure_a.headings.len().to_string()),
                after: Some(structure_b.headings.len().to_string()),
            });
        }

        // Compare individual headings
        for (i, heading_a) in structure_a.headings.iter().enumerate() {
            if let Some(heading_b) = structure_b.headings.get(i) {
                if heading_a.content != heading_b.content {
                    diffs.push(StructuralDiff {
                        section: Some(format!("Heading {}", i + 1)),
                        description: "Heading content changed".to_string(),
                        before: Some(heading_a.content.clone()),
                        after: Some(heading_b.content.clone()),
                    });
                }
            }
        }

        diffs
    }

    fn calculate_structural_similarity(
        &self,
        structure_a: &DocumentStructure,
        structure_b: &DocumentStructure,
    ) -> f64 {
        if structure_a.headings.is_empty() && structure_b.headings.is_empty() {
            return 1.0;
        }

        let common_headings = structure_a
            .headings
            .iter()
            .filter(|h_a| {
                structure_b
                    .headings
                    .iter()
                    .any(|h_b| h_a.content == h_b.content)
            })
            .count();

        let total_headings = std::cmp::max(structure_a.headings.len(), structure_b.headings.len());

        if total_headings == 0 {
            1.0
        } else {
            common_headings as f64 / total_headings as f64
        }
    }

    fn compute_diff(&self, lines_a: &[&str], lines_b: &[&str]) -> Result<Vec<DiffChange>> {
        // Use Longest Common Subsequence (LCS) algorithm for better diff quality
        let lcs_matrix = self.compute_lcs_matrix(lines_a, lines_b);
        self.extract_changes_from_lcs(&lcs_matrix, lines_a, lines_b)
    }

    fn compute_lcs_matrix(&self, lines_a: &[&str], lines_b: &[&str]) -> Vec<Vec<usize>> {
        let m = lines_a.len();
        let n = lines_b.len();
        let mut lcs = vec![vec![0; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if lines_a[i - 1] == lines_b[j - 1] {
                    lcs[i][j] = lcs[i - 1][j - 1] + 1;
                } else {
                    lcs[i][j] = std::cmp::max(lcs[i - 1][j], lcs[i][j - 1]);
                }
            }
        }

        lcs
    }

    fn extract_changes_from_lcs(
        &self,
        lcs_matrix: &[Vec<usize>],
        lines_a: &[&str],
        lines_b: &[&str],
    ) -> Result<Vec<DiffChange>> {
        let mut changes = Vec::new();
        let mut i = lines_a.len();
        let mut j = lines_b.len();

        while i > 0 || j > 0 {
            if i > 0 && j > 0 && lines_a[i - 1] == lines_b[j - 1] {
                // Lines are the same, move diagonally
                i -= 1;
                j -= 1;
            } else if j > 0 && (i == 0 || lcs_matrix[i][j - 1] >= lcs_matrix[i - 1][j]) {
                // Addition
                changes.push(DiffChange {
                    diff_type: DifferenceType::Addition,
                    description: format!("Added line: {}", lines_b[j - 1]),
                    before: None,
                    after: Some(lines_b[j - 1].to_string()),
                });
                j -= 1;
            } else if i > 0 && (j == 0 || lcs_matrix[i][j - 1] < lcs_matrix[i - 1][j]) {
                // Deletion
                changes.push(DiffChange {
                    diff_type: DifferenceType::Deletion,
                    description: format!("Deleted line: {}", lines_a[i - 1]),
                    before: Some(lines_a[i - 1].to_string()),
                    after: None,
                });
                i -= 1;
            }
        }

        // Reverse changes since we built them backwards
        changes.reverse();

        // Post-process to identify modifications (adjacent deletions and additions)
        self.consolidate_modifications(changes)
    }

    fn consolidate_modifications(&self, changes: Vec<DiffChange>) -> Result<Vec<DiffChange>> {
        let mut consolidated = Vec::new();
        let mut i = 0;

        while i < changes.len() {
            if i + 1 < changes.len() {
                let current = &changes[i];
                let next = &changes[i + 1];

                // Check if we have a deletion followed by an addition (modification)
                if matches!(current.diff_type, DifferenceType::Deletion)
                    && matches!(next.diff_type, DifferenceType::Addition)
                    && self.are_similar_lines(
                        current.before.as_deref().unwrap_or(""),
                        next.after.as_deref().unwrap_or(""),
                    )
                {
                    // Combine into a modification
                    consolidated.push(DiffChange {
                        diff_type: DifferenceType::Modification,
                        description: "Line modified".to_string(),
                        before: current.before.clone(),
                        after: next.after.clone(),
                    });
                    i += 2; // Skip both changes
                } else {
                    consolidated.push(current.clone());
                    i += 1;
                }
            } else {
                consolidated.push(changes[i].clone());
                i += 1;
            }
        }

        Ok(consolidated)
    }

    fn are_similar_lines(&self, line_a: &str, line_b: &str) -> bool {
        // Simple similarity check - could be enhanced
        let words_a: std::collections::HashSet<&str> = line_a.split_whitespace().collect();
        let words_b: std::collections::HashSet<&str> = line_b.split_whitespace().collect();

        if words_a.is_empty() && words_b.is_empty() {
            return true;
        }

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            false
        } else {
            (intersection as f64 / union as f64) > 0.5
        }
    }

    async fn enhance_differences_with_ai(
        &self,
        differences: &mut [DocumentDifference],
        _content_a: &str,
        _content_b: &str,
    ) -> Result<()> {
        // For now, enhance descriptions with static analysis until AI integration is ready
        // This provides better explanations without requiring AI connection

        for diff in differences.iter_mut() {
            if matches!(
                diff.severity,
                DifferenceSeverity::Critical | DifferenceSeverity::Major
            ) {
                let enhanced_description = match diff.diff_type {
                    DifferenceType::Addition => {
                        if let Some(content) = &diff.after {
                            format!(
                                "{}\n\nAnalysis: New content added - '{}'",
                                diff.description,
                                content.chars().take(100).collect::<String>()
                            )
                        } else {
                            diff.description.clone()
                        }
                    }
                    DifferenceType::Deletion => {
                        if let Some(content) = &diff.before {
                            format!(
                                "{}\n\nAnalysis: Content removed - '{}'",
                                diff.description,
                                content.chars().take(100).collect::<String>()
                            )
                        } else {
                            diff.description.clone()
                        }
                    }
                    DifferenceType::Modification => {
                        let before_preview = diff
                            .before
                            .as_deref()
                            .unwrap_or("(none)")
                            .chars()
                            .take(50)
                            .collect::<String>();
                        let after_preview = diff
                            .after
                            .as_deref()
                            .unwrap_or("(none)")
                            .chars()
                            .take(50)
                            .collect::<String>();
                        format!(
                            "{}\n\nAnalysis: Content changed from '{}' to '{}'",
                            diff.description, before_preview, after_preview
                        )
                    }
                    _ => diff.description.clone(),
                };

                diff.description = enhanced_description;
                diff.confidence = (diff.confidence * 0.9 + 0.85 * 0.1).min(1.0);
            }
        }

        Ok(())
    }

    async fn generate_enhanced_summary(
        &self,
        differences: &[DocumentDifference],
        similarity_score: f64,
        content_a: &str,
        content_b: &str,
    ) -> ComparisonSummary {
        // Generate basic summary with enhanced structural analysis
        let summary = self.generate_summary(differences, similarity_score);

        // Enhance with content analysis insights
        let content_insights = self.analyze_content_changes(content_a, content_b, differences);

        // Log insights for now (could be stored in summary later)
        if !content_insights.is_empty() {
            tracing::info!("Document comparison insights: {}", content_insights);
        }

        // Enhance structural similarity calculation
        let enhanced_structural_similarity =
            self.calculate_enhanced_structural_similarity(content_a, content_b);

        ComparisonSummary {
            structural_similarity: enhanced_structural_similarity,
            ..summary
        }
    }

    fn analyze_content_changes(
        &self,
        content_a: &str,
        content_b: &str,
        differences: &[DocumentDifference],
    ) -> String {
        let mut insights = Vec::new();

        // Analyze word count changes
        let words_a = content_a.split_whitespace().count();
        let words_b = content_b.split_whitespace().count();
        let word_change = words_b as i32 - words_a as i32;

        if word_change.abs() > 50 {
            insights.push(format!(
                "Significant word count change: {} words {}",
                word_change.abs(),
                if word_change > 0 { "added" } else { "removed" }
            ));
        }

        // Analyze change distribution
        let major_changes = differences
            .iter()
            .filter(|d| {
                matches!(
                    d.severity,
                    DifferenceSeverity::Critical | DifferenceSeverity::Major
                )
            })
            .count();

        if major_changes > differences.len() / 2 {
            insights.push("Document has undergone substantial revision".to_string());
        } else if major_changes < differences.len() / 10 {
            insights.push("Changes are mostly minor refinements".to_string());
        }

        // Analyze change types
        let additions = differences
            .iter()
            .filter(|d| matches!(d.diff_type, DifferenceType::Addition))
            .count();
        let deletions = differences
            .iter()
            .filter(|d| matches!(d.diff_type, DifferenceType::Deletion))
            .count();
        let modifications = differences
            .iter()
            .filter(|d| matches!(d.diff_type, DifferenceType::Modification))
            .count();

        if additions > deletions * 2 {
            insights.push("Document has been significantly expanded".to_string());
        } else if deletions > additions * 2 {
            insights.push("Document has been significantly condensed".to_string());
        } else if modifications > (additions + deletions) {
            insights.push("Changes are primarily edits to existing content".to_string());
        }

        insights.join("; ")
    }

    fn calculate_enhanced_structural_similarity(&self, content_a: &str, content_b: &str) -> f64 {
        let structure_a = self.extract_structure(content_a);
        let structure_b = self.extract_structure(content_b);

        // Basic heading similarity
        let heading_similarity = self.calculate_structural_similarity(&structure_a, &structure_b);

        // Section count similarity
        let section_count_a = structure_a.sections.len();
        let section_count_b = structure_b.sections.len();
        let section_similarity = if section_count_a == 0 && section_count_b == 0 {
            1.0
        } else {
            let max_sections = section_count_a.max(section_count_b) as f64;
            let min_sections = section_count_a.min(section_count_b) as f64;
            min_sections / max_sections
        };

        // Weighted combination
        heading_similarity * 0.7 + section_similarity * 0.3
    }

    fn assess_change_severity(&self, change: &DiffChange) -> DifferenceSeverity {
        match &change.diff_type {
            DifferenceType::Addition | DifferenceType::Deletion => {
                if let Some(content) = change.before.as_ref().or(change.after.as_ref()) {
                    if content.len() > 100 {
                        DifferenceSeverity::Major
                    } else {
                        DifferenceSeverity::Minor
                    }
                } else {
                    DifferenceSeverity::Minor
                }
            }
            DifferenceType::Modification => DifferenceSeverity::Major,
            _ => DifferenceSeverity::Minor,
        }
    }

    fn calculate_style_similarity(
        &self,
        style_a: &StyleProfile,
        style_b: &StyleProfile,
    ) -> Result<f64> {
        // Calculate similarity based on various style dimensions
        let tone_similarity = if style_a.tone.primary_tone == style_b.tone.primary_tone {
            0.8 + (1.0 - (style_a.tone.formality_score - style_b.tone.formality_score).abs()) * 0.2
        } else {
            0.3
        };

        let vocab_similarity = if style_a.vocabulary.complexity == style_b.vocabulary.complexity {
            1.0
        } else {
            0.5
        };

        let voice_similarity = if style_a.tone.voice_type == style_b.tone.voice_type {
            1.0
        } else {
            0.4
        };

        // Weighted average
        let overall_similarity =
            tone_similarity * 0.4 + vocab_similarity * 0.3 + voice_similarity * 0.3;
        Ok(overall_similarity)
    }

    fn detect_style_differences(
        &self,
        style_a: &StyleProfile,
        style_b: &StyleProfile,
    ) -> Result<Vec<StyleDifference>> {
        let mut differences = Vec::new();

        // Check tone differences
        if style_a.tone.primary_tone != style_b.tone.primary_tone {
            differences.push(StyleDifference {
                difference_type: StyleDifferenceType::ToneChange,
                severity: DifferenceSeverity::Major,
                description: format!(
                    "Tone changed from {:?} to {:?}",
                    style_a.tone.primary_tone, style_b.tone.primary_tone
                ),
                original_value: format!("{:?}", style_a.tone.primary_tone),
                updated_value: format!("{:?}", style_b.tone.primary_tone),
                impact_score: 0.8,
                preservation_suggestion: format!(
                    "Consider maintaining the original {:?} tone for consistency",
                    style_a.tone.primary_tone
                ),
            });
        }

        // Check formality differences
        let formality_diff = (style_a.tone.formality_score - style_b.tone.formality_score).abs();
        if formality_diff > 0.3 {
            differences.push(StyleDifference {
                difference_type: StyleDifferenceType::FormalityChange,
                severity: if formality_diff > 0.5 {
                    DifferenceSeverity::Major
                } else {
                    DifferenceSeverity::Minor
                },
                description: format!(
                    "Formality level changed from {:.2} to {:.2}",
                    style_a.tone.formality_score, style_b.tone.formality_score
                ),
                original_value: format!("{:.2}", style_a.tone.formality_score),
                updated_value: format!("{:.2}", style_b.tone.formality_score),
                impact_score: formality_diff,
                preservation_suggestion: "Consider adjusting formality to match original document"
                    .to_string(),
            });
        }

        // Check vocabulary complexity differences
        if style_a.vocabulary.complexity != style_b.vocabulary.complexity {
            differences.push(StyleDifference {
                difference_type: StyleDifferenceType::VocabularyComplexityShift,
                severity: DifferenceSeverity::Major,
                description: format!(
                    "Vocabulary complexity changed from {:?} to {:?}",
                    style_a.vocabulary.complexity, style_b.vocabulary.complexity
                ),
                original_value: format!("{:?}", style_a.vocabulary.complexity),
                updated_value: format!("{:?}", style_b.vocabulary.complexity),
                impact_score: 0.7,
                preservation_suggestion: "Maintain consistent vocabulary complexity level"
                    .to_string(),
            });
        }

        // Check voice differences
        if style_a.tone.voice_type != style_b.tone.voice_type {
            differences.push(StyleDifference {
                difference_type: StyleDifferenceType::VoiceChange,
                severity: DifferenceSeverity::Major,
                description: format!(
                    "Voice type changed from {:?} to {:?}",
                    style_a.tone.voice_type, style_b.tone.voice_type
                ),
                original_value: format!("{:?}", style_a.tone.voice_type),
                updated_value: format!("{:?}", style_b.tone.voice_type),
                impact_score: 0.6,
                preservation_suggestion: "Consider preserving the original voice type".to_string(),
            });
        }

        Ok(differences)
    }

    #[allow(dead_code)]
    pub async fn generate_style_consistent_updates(
        &self,
        original_content: &str,
        updated_content: &str,
        target_style: &StyleProfile,
    ) -> Result<Vec<StyleConsistentUpdate>> {
        let mut updates = Vec::new();

        // Use style transfer to apply target style to updated content
        let style_transfer = StyleTransfer::new();
        let transfer_config = StyleTransferConfig {
            mode: StyleTransferMode::Conservative,
            preserve_structure: true,
            preserve_technical_terms: true,
            target_formality: Some(target_style.tone.formality_score),
            target_tone: Some(target_style.tone.primary_tone.clone()),
            target_voice: Some(target_style.tone.voice_type.clone()),
            target_complexity: Some(target_style.vocabulary.complexity.clone()),
            ai_assistance: false, // Conservative approach for document updates
            confidence_threshold: 0.7,
        };

        let transfer_request = crate::document::StyleTransferRequest {
            content: updated_content.to_string(),
            target_style: StyleTransferTarget::StyleProfile(Box::new(target_style.clone())),
            config: transfer_config,
        };

        match style_transfer.transfer_style(transfer_request) {
            Ok(transfer_result) => {
                for change in transfer_result.applied_changes {
                    let update_type = match change.change_type {
                        crate::document::StyleChangeType::ToneAdjustment => {
                            UpdateType::ToneCorrection
                        }
                        crate::document::StyleChangeType::VocabularyReplacement => {
                            UpdateType::VocabularyReplacement
                        }
                        crate::document::StyleChangeType::FormalityAdjustment => {
                            UpdateType::StyleAdjustment
                        }
                        crate::document::StyleChangeType::VoiceModification => {
                            UpdateType::StyleAdjustment
                        }
                        crate::document::StyleChangeType::StructuralReorganization => {
                            UpdateType::StructuralAlignment
                        }
                        _ => UpdateType::ContentUpdate,
                    };

                    updates.push(StyleConsistentUpdate {
                        update_type,
                        target_section: "main".to_string(),
                        original_text: change.original_text,
                        suggested_text: change.modified_text,
                        reasoning: change.reason,
                        confidence: change.confidence,
                        style_preservation_impact: 0.8,
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Style transfer failed during update generation: {}", e);
                // Fallback to basic suggestions
                updates.push(StyleConsistentUpdate {
                    update_type: UpdateType::ContentUpdate,
                    target_section: "main".to_string(),
                    original_text: original_content.to_string(),
                    suggested_text: updated_content.to_string(),
                    reasoning: "Manual review recommended for style consistency".to_string(),
                    confidence: 0.5,
                    style_preservation_impact: 0.6,
                });
            }
        }

        Ok(updates)
    }
}

impl Default for DocumentComparator {
    fn default() -> Self {
        Self::new()
    }
}

// Helper structures
#[derive(Debug, Clone)]
struct DocumentStructure {
    headings: Vec<StructuralElement>,
    #[allow(dead_code)]
    sections: Vec<StructuralElement>,
}

#[derive(Debug, Clone)]
struct StructuralElement {
    #[allow(dead_code)]
    element_type: String,
    content: String,
    #[allow(dead_code)]
    level: usize,
    #[allow(dead_code)]
    line_number: usize,
}

#[derive(Debug, Clone)]
struct StructuralDiff {
    section: Option<String>,
    description: String,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Debug, Clone)]
struct DiffChange {
    diff_type: DifferenceType,
    description: String,
    before: Option<String>,
    after: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_text_similarity() {
        let comparator = DocumentComparator::new();
        let text_a = "Hello world this is a test";
        let text_b = "Hello world this is a test";

        let similarity = comparator.calculate_text_similarity(text_a, text_b);
        assert_eq!(similarity, 1.0);
    }

    #[tokio::test]
    async fn test_basic_comparison() {
        let comparator = DocumentComparator::new();

        let doc_a = DocumentForComparison {
            file_path: "doc_a.txt".to_string(),
            content: Some(ParsedDocumentContent::Text {
                content: "Hello world".to_string(),
            }),
            metadata: HashMap::new(),
        };

        let doc_b = DocumentForComparison {
            file_path: "doc_b.txt".to_string(),
            content: Some(ParsedDocumentContent::Text {
                content: "Hello universe".to_string(),
            }),
            metadata: HashMap::new(),
        };

        let request = DocumentComparisonRequest {
            document_a: doc_a,
            document_b: doc_b,
            comparison_type: ComparisonType::TextDiff,
            options: ComparisonOptions::default(),
        };

        let result = comparator.compare_documents(request).await;
        assert!(result.is_ok());

        let comparison = result.unwrap();
        assert!(comparison.differences.len() > 0);
        assert!(comparison.similarity_score > 0.0);
        assert!(comparison.similarity_score < 1.0);
    }
}
