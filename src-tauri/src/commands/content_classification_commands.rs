// src-tauri/src/commands/content_classification_commands.rs
//! Tauri commands for content classification functionality

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ai::{AIConfig, AIOrchestrator};
use crate::document::{
    ContentClassification, ContentClassifier, DocumentContentAnalysis, StructureAnalyzer,
};

/// Classify text content using rule-based classification
#[tauri::command]
pub async fn classify_text_content(
    content: String,
    context: Option<String>,
) -> Result<ContentClassification, String> {
    let classifier = ContentClassifier::new(None).map_err(|e| e.to_string())?;

    let classification = classifier.classify_content(&content, context.as_deref());
    Ok(classification)
}

/// Classify text content with AI assistance
#[tauri::command]
pub async fn classify_text_content_with_ai(
    content: String,
    context: Option<String>,
) -> Result<ContentClassification, String> {
    // Use default AI config for now - in production this would come from settings
    let ai_config = AIConfig::default();

    // Initialize AI orchestrator
    let ai_client = AIOrchestrator::new(ai_config)
        .await
        .map_err(|e| e.to_string())?;
    let classifier = ContentClassifier::new(Some(ai_client)).map_err(|e| e.to_string())?;

    let classification = classifier
        .classify_content_with_ai(&content, context.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(classification)
}

/// Classify content from a file
#[tauri::command]
pub async fn classify_file_content(
    file_path: String,
    with_ai: bool,
) -> Result<DocumentContentAnalysis, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    // Read file content
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    // First, analyze document structure
    let structure_analyzer = StructureAnalyzer::new().map_err(|e| e.to_string())?;
    let structure_analysis = structure_analyzer
        .analyze_structure(&content)
        .map_err(|e| e.to_string())?;

    // Then classify content
    let ai_client = if with_ai {
        let ai_config = AIConfig::default();
        Some(
            AIOrchestrator::new(ai_config)
                .await
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    let classifier = ContentClassifier::new(ai_client).map_err(|e| e.to_string())?;

    let content_analysis = classifier
        .analyze_document_content(&structure_analysis, &content)
        .await
        .map_err(|e| e.to_string())?;

    Ok(content_analysis)
}

/// Get content classification statistics for multiple files
#[tauri::command]
pub async fn batch_classify_files(
    file_paths: Vec<String>,
    with_ai: bool,
) -> Result<HashMap<String, DocumentContentAnalysis>, String> {
    let mut results = HashMap::new();

    // Initialize classifier once for efficiency
    let ai_client = if with_ai {
        let ai_config = AIConfig::default();
        Some(
            AIOrchestrator::new(ai_config)
                .await
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    let classifier = ContentClassifier::new(ai_client).map_err(|e| e.to_string())?;
    let structure_analyzer = StructureAnalyzer::new().map_err(|e| e.to_string())?;

    for file_path in file_paths {
        let path = PathBuf::from(&file_path);
        if !path.exists() {
            continue; // Skip missing files
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                // Analyze structure
                if let Ok(structure_analysis) = structure_analyzer.analyze_structure(&content) {
                    // Classify content
                    if let Ok(content_analysis) = classifier
                        .analyze_document_content(&structure_analysis, &content)
                        .await
                    {
                        results.insert(file_path, content_analysis);
                    }
                }
            }
            Err(_) => continue, // Skip files that can't be read
        }
    }

    Ok(results)
}

/// Get available content categories
#[tauri::command]
pub fn get_content_categories() -> Vec<String> {
    vec![
        "Procedures".to_string(),
        "Explanations".to_string(),
        "Examples".to_string(),
        "Definitions".to_string(),
        "QAndA".to_string(),
        "Warnings".to_string(),
        "BestPractices".to_string(),
        "TechnicalSpecs".to_string(),
        "Troubleshooting".to_string(),
        "Background".to_string(),
        "Reference".to_string(),
        "Unknown".to_string(),
    ]
}

/// Compare content classifications between two documents
#[tauri::command]
pub async fn compare_content_classifications(
    file_path_a: String,
    file_path_b: String,
    with_ai: bool,
) -> Result<HashMap<String, serde_json::Value>, String> {
    // Get classifications for both files
    let analysis_a = classify_file_content(file_path_a.clone(), with_ai).await?;
    let analysis_b = classify_file_content(file_path_b.clone(), with_ai).await?;

    // Compare content distributions
    let mut comparison = HashMap::new();
    comparison.insert(
        "file_a".to_string(),
        serde_json::to_value(&analysis_a).unwrap(),
    );
    comparison.insert(
        "file_b".to_string(),
        serde_json::to_value(&analysis_b).unwrap(),
    );

    // Calculate similarity score based on content distribution
    let similarity_score = calculate_content_similarity(&analysis_a, &analysis_b);
    comparison.insert(
        "similarity_score".to_string(),
        serde_json::Value::from(similarity_score),
    );

    // Identify content type differences
    let content_differences = identify_content_differences(&analysis_a, &analysis_b);
    comparison.insert(
        "content_differences".to_string(),
        serde_json::to_value(&content_differences).unwrap(),
    );

    Ok(comparison)
}

/// Helper function to calculate content similarity between two documents
fn calculate_content_similarity(
    analysis_a: &DocumentContentAnalysis,
    analysis_b: &DocumentContentAnalysis,
) -> f64 {
    let dist_a = &analysis_a.content_distribution;
    let dist_b = &analysis_b.content_distribution;

    // Get all unique categories
    let mut all_categories = std::collections::HashSet::new();
    all_categories.extend(dist_a.keys());
    all_categories.extend(dist_b.keys());

    if all_categories.is_empty() {
        return 1.0; // Both empty = similar
    }

    // Calculate cosine similarity
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for category in all_categories {
        let val_a = dist_a.get(category).unwrap_or(&0.0);
        let val_b = dist_b.get(category).unwrap_or(&0.0);

        dot_product += val_a * val_b;
        norm_a += val_a * val_a;
        norm_b += val_b * val_b;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

/// Helper function to identify content differences between two documents
fn identify_content_differences(
    analysis_a: &DocumentContentAnalysis,
    analysis_b: &DocumentContentAnalysis,
) -> HashMap<String, serde_json::Value> {
    let mut differences = HashMap::new();

    // Compare dominant content types
    if analysis_a.dominant_content_type != analysis_b.dominant_content_type {
        differences.insert(
            "dominant_type_difference".to_string(),
            serde_json::json!({
                "file_a": format!("{:?}", analysis_a.dominant_content_type),
                "file_b": format!("{:?}", analysis_b.dominant_content_type)
            }),
        );
    }

    // Compare complexity scores
    let complexity_diff = (analysis_a.complexity_score - analysis_b.complexity_score).abs();
    if complexity_diff > 0.1 {
        // Threshold for significant difference
        differences.insert(
            "complexity_difference".to_string(),
            serde_json::json!({
                "file_a": analysis_a.complexity_score,
                "file_b": analysis_b.complexity_score,
                "difference": complexity_diff
            }),
        );
    }

    // Find categories with significant distribution differences
    let mut category_differences = HashMap::new();
    let dist_a = &analysis_a.content_distribution;
    let dist_b = &analysis_b.content_distribution;

    let mut all_categories = std::collections::HashSet::new();
    all_categories.extend(dist_a.keys());
    all_categories.extend(dist_b.keys());

    for category in all_categories {
        let val_a = dist_a.get(category).unwrap_or(&0.0);
        let val_b = dist_b.get(category).unwrap_or(&0.0);
        let diff = (val_a - val_b).abs();

        if diff > 0.1 {
            // Threshold for significant difference
            category_differences.insert(
                format!("{:?}", category),
                serde_json::json!({
                    "file_a": val_a,
                    "file_b": val_b,
                    "difference": diff
                }),
            );
        }
    }

    if !category_differences.is_empty() {
        differences.insert(
            "category_differences".to_string(),
            serde_json::to_value(category_differences).unwrap(),
        );
    }

    differences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_content_categories() {
        let categories = get_content_categories();
        assert!(categories.contains(&"Procedures".to_string()));
        assert!(categories.contains(&"Explanations".to_string()));
        assert!(categories.len() >= 10);
    }

    #[test]
    fn test_calculate_content_similarity() {
        use crate::document::ContentCategory;

        let mut dist_a = HashMap::new();
        dist_a.insert(ContentCategory::Procedures, 0.5);
        dist_a.insert(ContentCategory::Examples, 0.3);
        dist_a.insert(ContentCategory::Explanations, 0.2);

        let mut dist_b = HashMap::new();
        dist_b.insert(ContentCategory::Procedures, 0.6);
        dist_b.insert(ContentCategory::Examples, 0.2);
        dist_b.insert(ContentCategory::Explanations, 0.2);

        let analysis_a = DocumentContentAnalysis {
            section_classifications: HashMap::new(),
            content_distribution: dist_a,
            dominant_content_type: ContentCategory::Procedures,
            complexity_score: 0.5,
        };

        let analysis_b = DocumentContentAnalysis {
            section_classifications: HashMap::new(),
            content_distribution: dist_b,
            dominant_content_type: ContentCategory::Procedures,
            complexity_score: 0.6,
        };

        let similarity = calculate_content_similarity(&analysis_a, &analysis_b);
        assert!(similarity > 0.8); // Should be quite similar
        assert!(similarity <= 1.0);
    }
}
