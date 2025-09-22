// src-tauri/src/commands/document_comparison_commands.rs
// Tauri commands for document comparison functionality

use crate::document::{
    ComparisonOptions, ComparisonType, DocumentComparator, DocumentComparisonRequest,
    DocumentComparisonResult, DocumentForComparison, ParsedDocumentContent,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDocumentsRequest {
    pub file_path_a: String,
    pub file_path_b: String,
    pub comparison_type: String, // "text", "semantic", "structural", "comprehensive"
    pub options: Option<ComparisonOptionsInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonOptionsInput {
    pub include_metadata: Option<bool>,
    pub similarity_threshold: Option<f64>,
    pub max_differences: Option<usize>,
    pub ignore_formatting: Option<bool>,
    pub focus_on_content_changes: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDocumentsResponse {
    pub success: bool,
    pub result: Option<DocumentComparisonResult>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareTextRequest {
    pub text_a: String,
    pub text_b: String,
    pub title_a: Option<String>,
    pub title_b: Option<String>,
    pub comparison_type: String,
    pub options: Option<ComparisonOptionsInput>,
}

// Document comparison state
pub struct DocumentComparisonState {
    pub comparator: Arc<Mutex<DocumentComparator>>,
    pub comparison_history: Arc<Mutex<Vec<DocumentComparisonResult>>>,
}

impl Default for DocumentComparisonState {
    fn default() -> Self {
        Self {
            comparator: Arc::new(Mutex::new(DocumentComparator::new())),
            comparison_history: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

pub type DocumentComparisonAppState = Arc<DocumentComparisonState>;

#[tauri::command]
pub async fn init_document_comparison(
    comparison_state: State<'_, DocumentComparisonAppState>,
) -> Result<bool, String> {
    // Initialize the comparator
    {
        let mut comparator_lock = comparison_state.comparator.lock().await;
        *comparator_lock = DocumentComparator::new();
    }

    tracing::info!("Document comparison system initialized");
    Ok(true)
}

#[tauri::command]
pub async fn compare_documents_by_path(
    comparison_state: State<'_, DocumentComparisonAppState>,
    request: CompareDocumentsRequest,
) -> Result<CompareDocumentsResponse, String> {
    let start_time = std::time::Instant::now();

    // Parse the documents first
    let doc_a_result = parse_document_from_path(&request.file_path_a).await?;
    let doc_b_result = parse_document_from_path(&request.file_path_b).await?;

    // Convert to comparison request
    let comparison_request = DocumentComparisonRequest {
        document_a: doc_a_result,
        document_b: doc_b_result,
        comparison_type: parse_comparison_type(&request.comparison_type)?,
        options: request.options.map(convert_options).unwrap_or_default(),
    };

    let comparator_lock = comparison_state.comparator.lock().await;
    match comparator_lock.compare_documents(comparison_request).await {
        Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Store in history
            {
                let mut history_lock = comparison_state.comparison_history.lock().await;
                history_lock.push(result.clone());

                // Keep only the last 100 comparisons
                if history_lock.len() > 100 {
                    let excess = history_lock.len() - 100;
                    history_lock.drain(0..excess);
                }
            }

            tracing::info!(
                "Document comparison completed: {} vs {} in {}ms",
                request.file_path_a,
                request.file_path_b,
                processing_time
            );

            Ok(CompareDocumentsResponse {
                success: true,
                result: Some(result),
                error: None,
                processing_time_ms: processing_time,
            })
        }
        Err(e) => {
            let error_msg = format!("Document comparison failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(CompareDocumentsResponse {
                success: false,
                result: None,
                error: Some(error_msg),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            })
        }
    }
}

#[tauri::command]
pub async fn compare_text_content(
    comparison_state: State<'_, DocumentComparisonAppState>,
    request: CompareTextRequest,
) -> Result<CompareDocumentsResponse, String> {
    let start_time = std::time::Instant::now();

    // Create documents from text
    let doc_a = DocumentForComparison {
        file_path: request
            .title_a
            .clone()
            .unwrap_or_else(|| "Document A".to_string()),
        content: Some(ParsedDocumentContent::Text {
            content: request.text_a,
        }),
        metadata: {
            let mut metadata = HashMap::new();
            if let Some(title) = request.title_a {
                metadata.insert("title".to_string(), title);
            }
            metadata
        },
    };

    let doc_b = DocumentForComparison {
        file_path: request
            .title_b
            .clone()
            .unwrap_or_else(|| "Document B".to_string()),
        content: Some(ParsedDocumentContent::Text {
            content: request.text_b,
        }),
        metadata: {
            let mut metadata = HashMap::new();
            if let Some(title) = request.title_b {
                metadata.insert("title".to_string(), title);
            }
            metadata
        },
    };

    let comparison_request = DocumentComparisonRequest {
        document_a: doc_a,
        document_b: doc_b,
        comparison_type: parse_comparison_type(&request.comparison_type)?,
        options: request.options.map(convert_options).unwrap_or_default(),
    };

    let comparator_lock = comparison_state.comparator.lock().await;
    match comparator_lock.compare_documents(comparison_request).await {
        Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Store in history
            {
                let mut history_lock = comparison_state.comparison_history.lock().await;
                history_lock.push(result.clone());

                if history_lock.len() > 100 {
                    let excess = history_lock.len() - 100;
                    history_lock.drain(0..excess);
                }
            }

            tracing::info!("Text comparison completed in {}ms", processing_time);

            Ok(CompareDocumentsResponse {
                success: true,
                result: Some(result),
                error: None,
                processing_time_ms: processing_time,
            })
        }
        Err(e) => {
            let error_msg = format!("Text comparison failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(CompareDocumentsResponse {
                success: false,
                result: None,
                error: Some(error_msg),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            })
        }
    }
}

#[tauri::command]
pub async fn get_comparison_history(
    comparison_state: State<'_, DocumentComparisonAppState>,
    limit: Option<usize>,
) -> Result<Vec<DocumentComparisonResult>, String> {
    let history_lock = comparison_state.comparison_history.lock().await;
    let limit = limit.unwrap_or(20);

    let results = if history_lock.len() > limit {
        history_lock[(history_lock.len() - limit)..].to_vec()
    } else {
        history_lock.clone()
    };

    Ok(results)
}

#[tauri::command]
pub async fn clear_comparison_history(
    comparison_state: State<'_, DocumentComparisonAppState>,
) -> Result<bool, String> {
    let mut history_lock = comparison_state.comparison_history.lock().await;
    history_lock.clear();
    tracing::info!("Comparison history cleared");
    Ok(true)
}

#[tauri::command]
pub async fn get_supported_comparison_types() -> Result<Vec<String>, String> {
    Ok(vec![
        "text".to_string(),
        "semantic".to_string(),
        "structural".to_string(),
        "comprehensive".to_string(),
    ])
}

// Test command for development
#[tauri::command]
pub async fn test_document_comparison(
    comparison_state: State<'_, DocumentComparisonAppState>,
) -> Result<String, String> {
    let test_request = CompareTextRequest {
        text_a: "Hello world\nThis is the first document\nIt has some content.".to_string(),
        text_b: "Hello universe\nThis is the second document\nIt has different content."
            .to_string(),
        title_a: Some("Test Document A".to_string()),
        title_b: Some("Test Document B".to_string()),
        comparison_type: "comprehensive".to_string(),
        options: Some(ComparisonOptionsInput {
            include_metadata: Some(true),
            similarity_threshold: Some(0.8),
            max_differences: Some(50),
            ignore_formatting: Some(true),
            focus_on_content_changes: Some(true),
        }),
    };

    match compare_text_content(comparison_state, test_request).await {
        Ok(response) => {
            if response.success {
                if let Some(result) = response.result {
                    Ok(format!(
                        "Test comparison completed successfully!\nSimilarity: {:.2}\nDifferences: {}\nTime: {}ms",
                        result.similarity_score,
                        result.summary.total_differences,
                        response.processing_time_ms
                    ))
                } else {
                    Ok("Test completed but no results generated".to_string())
                }
            } else {
                Err(response.error.unwrap_or("Unknown error".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

// Helper functions
async fn parse_document_from_path(file_path: &str) -> Result<DocumentForComparison, String> {
    use crate::commands::document_commands::parse_document;

    match parse_document(file_path.to_string()).await {
        Ok(response) => match response {
            crate::commands::document_commands::DocumentParseResponse::Docx {
                content,
                file_path,
            } => Ok(DocumentForComparison {
                file_path,
                content: Some(ParsedDocumentContent::Docx { content }),
                metadata: HashMap::new(),
            }),
            crate::commands::document_commands::DocumentParseResponse::Pdf {
                content,
                file_path,
            } => Ok(DocumentForComparison {
                file_path,
                content: Some(ParsedDocumentContent::Pdf { content }),
                metadata: HashMap::new(),
            }),
            crate::commands::document_commands::DocumentParseResponse::Error {
                message,
                file_path,
            } => Err(format!(
                "Failed to parse document {}: {}",
                file_path, message
            )),
        },
        Err(e) => Err(format!("Document parsing error: {}", e)),
    }
}

fn parse_comparison_type(type_str: &str) -> Result<ComparisonType, String> {
    match type_str.to_lowercase().as_str() {
        "text" => Ok(ComparisonType::TextDiff),
        "semantic" => Ok(ComparisonType::SemanticSimilarity),
        "structural" => Ok(ComparisonType::StructuralDiff),
        "comprehensive" => Ok(ComparisonType::Comprehensive),
        _ => Err(format!("Unsupported comparison type: {}", type_str)),
    }
}

fn convert_options(input: ComparisonOptionsInput) -> ComparisonOptions {
    ComparisonOptions {
        include_metadata: input.include_metadata.unwrap_or(true),
        similarity_threshold: input.similarity_threshold.unwrap_or(0.8),
        max_differences: input.max_differences,
        ignore_formatting: input.ignore_formatting.unwrap_or(true),
        focus_on_content_changes: input.focus_on_content_changes.unwrap_or(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_type_parsing() {
        assert!(matches!(
            parse_comparison_type("text").unwrap(),
            ComparisonType::TextDiff
        ));
        assert!(matches!(
            parse_comparison_type("semantic").unwrap(),
            ComparisonType::SemanticSimilarity
        ));
        assert!(matches!(
            parse_comparison_type("comprehensive").unwrap(),
            ComparisonType::Comprehensive
        ));
        assert!(parse_comparison_type("invalid").is_err());
    }

    #[test]
    fn test_options_conversion() {
        let input = ComparisonOptionsInput {
            include_metadata: Some(false),
            similarity_threshold: Some(0.9),
            max_differences: Some(10),
            ignore_formatting: Some(false),
            focus_on_content_changes: Some(false),
        };

        let options = convert_options(input);
        assert_eq!(options.include_metadata, false);
        assert_eq!(options.similarity_threshold, 0.9);
        assert_eq!(options.max_differences, Some(10));
        assert_eq!(options.ignore_formatting, false);
        assert_eq!(options.focus_on_content_changes, false);
    }

    #[tokio::test]
    async fn test_comparison_state() {
        let state = DocumentComparisonState::default();
        assert_eq!(state.comparison_history.lock().await.len(), 0);
    }
}
