// src-tauri/src/document/document_comparison.rs
// Document comparison functionality using vector similarity and content analysis

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{DocxContent, PdfContent};

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
        let (differences, similarity_score) = match request.comparison_type {
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
        };

        // Compare metadata if requested
        let metadata_comparison = if request.options.include_metadata {
            Some(self.compare_metadata(&request.document_a, &request.document_b)?)
        } else {
            None
        };

        // Generate summary
        let summary = self.generate_summary(&differences, similarity_score);

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

    async fn calculate_semantic_similarity(&self, _text_a: &str, _text_b: &str) -> Result<f64> {
        // Placeholder for semantic similarity using vector embeddings
        // In a full implementation, this would use the embedding engine
        Ok(0.8)
    }

    fn extract_structure(&self, content: &str) -> DocumentStructure {
        let mut headings = Vec::new();
        let sections = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Simple heuristics for structure detection
            if self.looks_like_heading(trimmed) {
                headings.push(StructuralElement {
                    element_type: "heading".to_string(),
                    content: trimmed.to_string(),
                    level: self.estimate_heading_level(trimmed),
                    line_number: i,
                });
            }
        }

        DocumentStructure { headings, sections }
    }

    fn looks_like_heading(&self, line: &str) -> bool {
        // Simple heuristics
        line.len() < 100
            && !line.contains('.')
            && !line.starts_with(' ')
            && line.chars().any(|c| c.is_alphabetic())
    }

    fn estimate_heading_level(&self, _line: &str) -> usize {
        // Simple implementation - could be enhanced with font size analysis
        1
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
        let mut changes = Vec::new();

        // Simple diff implementation - could be enhanced with Myers algorithm
        let mut i = 0;
        let mut j = 0;

        while i < lines_a.len() || j < lines_b.len() {
            if i >= lines_a.len() {
                // Addition
                changes.push(DiffChange {
                    diff_type: DifferenceType::Addition,
                    description: format!("Added line: {}", lines_b[j]),
                    before: None,
                    after: Some(lines_b[j].to_string()),
                });
                j += 1;
            } else if j >= lines_b.len() {
                // Deletion
                changes.push(DiffChange {
                    diff_type: DifferenceType::Deletion,
                    description: format!("Deleted line: {}", lines_a[i]),
                    before: Some(lines_a[i].to_string()),
                    after: None,
                });
                i += 1;
            } else if lines_a[i] == lines_b[j] {
                // No change
                i += 1;
                j += 1;
            } else {
                // Modification
                changes.push(DiffChange {
                    diff_type: DifferenceType::Modification,
                    description: "Line modified".to_string(),
                    before: Some(lines_a[i].to_string()),
                    after: Some(lines_b[j].to_string()),
                });
                i += 1;
                j += 1;
            }
        }

        Ok(changes)
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
