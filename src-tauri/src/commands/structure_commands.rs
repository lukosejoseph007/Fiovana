// src-tauri/src/commands/structure_commands.rs
//! Tauri commands for document structure analysis

use crate::document::{DocumentStructureAnalysis, StructureAnalyzer};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::RwLock;

/// Structure analysis service state
pub struct StructureService {
    analyzer: StructureAnalyzer,
}

impl StructureService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            analyzer: StructureAnalyzer::new()?,
        })
    }

    pub async fn analyze_file_structure(
        &self,
        file_path: &str,
    ) -> Result<DocumentStructureAnalysis> {
        let content = tokio::fs::read_to_string(file_path).await?;
        self.analyzer.analyze_structure(&content)
    }

    pub async fn analyze_content_structure(
        &self,
        content: &str,
    ) -> Result<DocumentStructureAnalysis> {
        self.analyzer.analyze_structure(content)
    }
}

/// Request for analyzing document structure from file
#[derive(Debug, Deserialize)]
pub struct AnalyzeFileStructureRequest {
    /// File path to analyze
    pub file_path: String,
}

/// Request for analyzing document structure from content
#[derive(Debug, Deserialize)]
pub struct AnalyzeContentStructureRequest {
    /// Content to analyze
    pub content: String,
}

/// Structure analysis response
#[derive(Debug, Serialize)]
pub struct StructureAnalysisResponse {
    /// Success status
    pub success: bool,
    /// Analysis result
    pub analysis: Option<DocumentStructureAnalysis>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Analyze structure of a document file
#[tauri::command]
pub async fn analyze_document_structure(
    request: AnalyzeFileStructureRequest,
    service: State<'_, RwLock<StructureService>>,
) -> Result<StructureAnalysisResponse, String> {
    let service = service.read().await;

    match service.analyze_file_structure(&request.file_path).await {
        Ok(analysis) => Ok(StructureAnalysisResponse {
            success: true,
            analysis: Some(analysis),
            error: None,
        }),
        Err(e) => Ok(StructureAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Failed to analyze structure: {}", e)),
        }),
    }
}

/// Analyze structure of content directly
#[tauri::command]
pub async fn analyze_content_structure(
    request: AnalyzeContentStructureRequest,
    service: State<'_, RwLock<StructureService>>,
) -> Result<StructureAnalysisResponse, String> {
    let service = service.read().await;

    match service.analyze_content_structure(&request.content).await {
        Ok(analysis) => Ok(StructureAnalysisResponse {
            success: true,
            analysis: Some(analysis),
            error: None,
        }),
        Err(e) => Ok(StructureAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("Failed to analyze structure: {}", e)),
        }),
    }
}

/// Get structure analysis statistics for multiple files
#[tauri::command]
pub async fn batch_analyze_structure(
    file_paths: Vec<String>,
    service: State<'_, RwLock<StructureService>>,
) -> Result<Vec<StructureAnalysisResponse>, String> {
    let service = service.read().await;
    let mut results = Vec::new();

    for file_path in file_paths {
        let result = match service.analyze_file_structure(&file_path).await {
            Ok(analysis) => StructureAnalysisResponse {
                success: true,
                analysis: Some(analysis),
                error: None,
            },
            Err(e) => StructureAnalysisResponse {
                success: false,
                analysis: None,
                error: Some(format!("Failed to analyze {}: {}", file_path, e)),
            },
        };
        results.push(result);
    }

    Ok(results)
}

/// Compare structure between two documents
#[tauri::command]
pub async fn compare_document_structures(
    file_path_a: String,
    file_path_b: String,
    service: State<'_, RwLock<StructureService>>,
) -> Result<StructureComparisonResult, String> {
    let service = service.read().await;

    let analysis_a = service
        .analyze_file_structure(&file_path_a)
        .await
        .map_err(|e| format!("Failed to analyze {}: {}", file_path_a, e))?;

    let analysis_b = service
        .analyze_file_structure(&file_path_b)
        .await
        .map_err(|e| format!("Failed to analyze {}: {}", file_path_b, e))?;

    let comparison = compare_structures(&analysis_a, &analysis_b);

    Ok(StructureComparisonResult {
        success: true,
        comparison: Some(comparison),
        error: None,
    })
}

/// Structure comparison result
#[derive(Debug, Serialize)]
pub struct StructureComparisonResult {
    pub success: bool,
    pub comparison: Option<StructureComparison>,
    pub error: Option<String>,
}

/// Comparison between two document structures
#[derive(Debug, Serialize)]
pub struct StructureComparison {
    /// Similarity score (0.0-1.0)
    pub similarity_score: f64,
    /// Heading structure comparison
    pub heading_comparison: HeadingComparison,
    /// Section type comparison
    pub section_comparison: SectionComparison,
    /// Organization comparison
    pub organization_comparison: OrganizationComparison,
    /// Key differences
    pub differences: Vec<StructureDifference>,
}

#[derive(Debug, Serialize)]
pub struct HeadingComparison {
    pub structure_a_count: usize,
    pub structure_b_count: usize,
    pub common_headings: Vec<String>,
    pub unique_to_a: Vec<String>,
    pub unique_to_b: Vec<String>,
    pub hierarchy_similarity: f64,
}

#[derive(Debug, Serialize)]
pub struct SectionComparison {
    pub structure_a_sections: usize,
    pub structure_b_sections: usize,
    pub common_section_types: Vec<String>,
    pub unique_to_a: Vec<String>,
    pub unique_to_b: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OrganizationComparison {
    pub flow_type_match: bool,
    pub structure_quality_diff: f64,
    pub complexity_diff: f64,
}

#[derive(Debug, Serialize)]
pub struct StructureDifference {
    pub category: String,
    pub description: String,
    pub significance: f64,
}

/// Compare two document structure analyses
fn compare_structures(
    analysis_a: &DocumentStructureAnalysis,
    analysis_b: &DocumentStructureAnalysis,
) -> StructureComparison {
    // Heading comparison
    let headings_a: Vec<String> = extract_heading_texts(&analysis_a.heading_hierarchy);
    let headings_b: Vec<String> = extract_heading_texts(&analysis_b.heading_hierarchy);

    let common_headings: Vec<String> = headings_a
        .iter()
        .filter(|h| headings_b.contains(h))
        .cloned()
        .collect();

    let unique_to_a: Vec<String> = headings_a
        .iter()
        .filter(|h| !headings_b.contains(h))
        .cloned()
        .collect();

    let unique_to_b: Vec<String> = headings_b
        .iter()
        .filter(|h| !headings_a.contains(h))
        .cloned()
        .collect();

    let hierarchy_similarity = calculate_hierarchy_similarity(
        &analysis_a.heading_hierarchy,
        &analysis_b.heading_hierarchy,
    );

    let heading_comparison = HeadingComparison {
        structure_a_count: headings_a.len(),
        structure_b_count: headings_b.len(),
        common_headings,
        unique_to_a,
        unique_to_b,
        hierarchy_similarity,
    };

    // Section comparison
    let sections_a: Vec<String> = analysis_a
        .sections
        .iter()
        .map(|s| format!("{:?}", s.section_type))
        .collect();

    let sections_b: Vec<String> = analysis_b
        .sections
        .iter()
        .map(|s| format!("{:?}", s.section_type))
        .collect();

    let common_section_types: Vec<String> = sections_a
        .iter()
        .filter(|s| sections_b.contains(s))
        .cloned()
        .collect();

    let section_comparison = SectionComparison {
        structure_a_sections: sections_a.len(),
        structure_b_sections: sections_b.len(),
        common_section_types,
        unique_to_a: sections_a
            .iter()
            .filter(|s| !sections_b.contains(s))
            .cloned()
            .collect(),
        unique_to_b: sections_b
            .iter()
            .filter(|s| !sections_a.contains(s))
            .cloned()
            .collect(),
    };

    // Organization comparison
    let organization_comparison = OrganizationComparison {
        flow_type_match: analysis_a.organization.flow_type == analysis_b.organization.flow_type,
        structure_quality_diff: (analysis_a.organization.structure_quality
            - analysis_b.organization.structure_quality)
            .abs(),
        complexity_diff: (analysis_a.statistics.complexity_score
            - analysis_b.statistics.complexity_score)
            .abs(),
    };

    // Calculate overall similarity
    let similarity_score = calculate_overall_similarity(analysis_a, analysis_b);

    // Generate differences
    let differences = generate_structure_differences(analysis_a, analysis_b);

    StructureComparison {
        similarity_score,
        heading_comparison,
        section_comparison,
        organization_comparison,
        differences,
    }
}

/// Extract heading texts recursively
fn extract_heading_texts(headings: &[crate::document::HeadingNode]) -> Vec<String> {
    let mut texts = Vec::new();
    for heading in headings {
        texts.push(heading.text.clone());
        texts.extend(extract_heading_texts(&heading.children));
    }
    texts
}

/// Calculate hierarchy similarity between two heading structures
fn calculate_hierarchy_similarity(
    headings_a: &[crate::document::HeadingNode],
    headings_b: &[crate::document::HeadingNode],
) -> f64 {
    if headings_a.is_empty() && headings_b.is_empty() {
        return 1.0;
    }
    if headings_a.is_empty() || headings_b.is_empty() {
        return 0.0;
    }

    // Compare level distributions
    let levels_a = get_level_distribution(headings_a);
    let levels_b = get_level_distribution(headings_b);

    let max_level = *levels_a.keys().chain(levels_b.keys()).max().unwrap_or(&1);
    let mut similarity = 0.0;
    let mut total_weight = 0.0;

    for level in 1..=max_level {
        let count_a = *levels_a.get(&level).unwrap_or(&0) as f64;
        let count_b = *levels_b.get(&level).unwrap_or(&0) as f64;
        let max_count = count_a.max(count_b);

        if max_count > 0.0 {
            let level_similarity = 1.0 - (count_a - count_b).abs() / max_count;
            similarity += level_similarity * max_count;
            total_weight += max_count;
        }
    }

    if total_weight > 0.0 {
        similarity / total_weight
    } else {
        0.0
    }
}

/// Get level distribution from headings
fn get_level_distribution(
    headings: &[crate::document::HeadingNode],
) -> std::collections::HashMap<usize, usize> {
    let mut distribution = std::collections::HashMap::new();
    for heading in headings {
        *distribution.entry(heading.level).or_insert(0) += 1;
        let child_dist = get_level_distribution(&heading.children);
        for (level, count) in child_dist {
            *distribution.entry(level).or_insert(0) += count;
        }
    }
    distribution
}

/// Calculate overall similarity between two analyses
fn calculate_overall_similarity(
    analysis_a: &DocumentStructureAnalysis,
    analysis_b: &DocumentStructureAnalysis,
) -> f64 {
    let mut similarity = 0.0;
    let mut weight = 0.0;

    // Heading similarity (30% weight)
    let heading_sim = calculate_hierarchy_similarity(
        &analysis_a.heading_hierarchy,
        &analysis_b.heading_hierarchy,
    );
    similarity += heading_sim * 0.3;
    weight += 0.3;

    // Section type similarity (25% weight)
    let section_types_a: std::collections::HashSet<_> = analysis_a
        .sections
        .iter()
        .map(|s| &s.section_type)
        .collect();
    let section_types_b: std::collections::HashSet<_> = analysis_b
        .sections
        .iter()
        .map(|s| &s.section_type)
        .collect();
    let common_types = section_types_a.intersection(&section_types_b).count();
    let total_unique_types = section_types_a.union(&section_types_b).count();
    let section_sim = if total_unique_types > 0 {
        common_types as f64 / total_unique_types as f64
    } else {
        1.0
    };
    similarity += section_sim * 0.25;
    weight += 0.25;

    // Flow type similarity (20% weight)
    let flow_sim = if analysis_a.organization.flow_type == analysis_b.organization.flow_type {
        1.0
    } else {
        0.0
    };
    similarity += flow_sim * 0.2;
    weight += 0.2;

    // Structure quality similarity (15% weight)
    let quality_diff = (analysis_a.organization.structure_quality
        - analysis_b.organization.structure_quality)
        .abs();
    let quality_sim = 1.0 - quality_diff.min(1.0);
    similarity += quality_sim * 0.15;
    weight += 0.15;

    // Complexity similarity (10% weight)
    let complexity_diff =
        (analysis_a.statistics.complexity_score - analysis_b.statistics.complexity_score).abs();
    let complexity_sim = 1.0 - complexity_diff.min(1.0);
    similarity += complexity_sim * 0.1;
    weight += 0.1;

    similarity / weight
}

/// Generate list of structural differences
fn generate_structure_differences(
    analysis_a: &DocumentStructureAnalysis,
    analysis_b: &DocumentStructureAnalysis,
) -> Vec<StructureDifference> {
    let mut differences = Vec::new();

    // Heading count differences
    let heading_diff = (analysis_a.statistics.heading_count as i32
        - analysis_b.statistics.heading_count as i32)
        .abs();
    if heading_diff > 2 {
        differences.push(StructureDifference {
            category: "Headings".to_string(),
            description: format!(
                "Heading count differs significantly: {} vs {}",
                analysis_a.statistics.heading_count, analysis_b.statistics.heading_count
            ),
            significance: (heading_diff as f64
                / (analysis_a.statistics.heading_count + analysis_b.statistics.heading_count).max(1)
                    as f64)
                .min(1.0),
        });
    }

    // Flow type differences
    if analysis_a.organization.flow_type != analysis_b.organization.flow_type {
        differences.push(StructureDifference {
            category: "Organization".to_string(),
            description: format!(
                "Different document flows: {:?} vs {:?}",
                analysis_a.organization.flow_type, analysis_b.organization.flow_type
            ),
            significance: 0.8,
        });
    }

    // Section count differences
    let section_diff = (analysis_a.statistics.section_count as i32
        - analysis_b.statistics.section_count as i32)
        .abs();
    if section_diff > 1 {
        differences.push(StructureDifference {
            category: "Sections".to_string(),
            description: format!(
                "Section count differs: {} vs {}",
                analysis_a.statistics.section_count, analysis_b.statistics.section_count
            ),
            significance: (section_diff as f64
                / (analysis_a.statistics.section_count + analysis_b.statistics.section_count).max(1)
                    as f64)
                .min(1.0),
        });
    }

    // Complexity differences
    let complexity_diff =
        (analysis_a.statistics.complexity_score - analysis_b.statistics.complexity_score).abs();
    if complexity_diff > 0.3 {
        differences.push(StructureDifference {
            category: "Complexity".to_string(),
            description: format!(
                "Complexity differs significantly: {:.2} vs {:.2}",
                analysis_a.statistics.complexity_score, analysis_b.statistics.complexity_score
            ),
            significance: complexity_diff,
        });
    }

    differences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_structure_service_creation() {
        let service = StructureService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_content_structure_analysis() {
        let service = StructureService::new().unwrap();
        let content = r#"
# Test Document

## Introduction
This is a test document.

## Procedures
1. First step
2. Second step

## Conclusion
End of document.
"#;

        let result = service.analyze_content_structure(content).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(!analysis.heading_hierarchy.is_empty());
        assert!(!analysis.sections.is_empty());
        assert_eq!(analysis.heading_hierarchy[0].text, "Test Document");
    }
}
