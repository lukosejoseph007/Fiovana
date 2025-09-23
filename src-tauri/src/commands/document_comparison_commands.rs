// src-tauri/src/commands/document_comparison_commands.rs
// Tauri commands for document comparison functionality

use crate::commands::document_indexing_commands::DocumentIndexerState;
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
pub struct CompareDocumentsByIdRequest {
    pub document_a_id: String,
    pub document_b_id: String,
    pub comparison_type: String, // "TextDiff", "StructuralDiff", "SemanticSimilarity", "Comprehensive"
    pub include_metadata: bool,
    pub similarity_threshold: f64,
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

#[tauri::command]
pub async fn compare_documents(
    comparison_state: State<'_, DocumentComparisonAppState>,
    indexer_state: State<'_, DocumentIndexerState>,
    request: CompareDocumentsByIdRequest,
) -> Result<CompareDocumentsResponse, String> {
    let start_time = std::time::Instant::now();

    // Get documents from indexer by ID
    let indexer_lock = indexer_state.lock().await;
    let indexer = match indexer_lock.as_ref() {
        Some(indexer) => indexer,
        None => {
            return Ok(CompareDocumentsResponse {
                success: false,
                result: None,
                error: Some("Document indexer not initialized".to_string()),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }
    };

    let doc_a_entry = match indexer.get_document(&request.document_a_id) {
        Some(entry) => entry,
        None => {
            return Ok(CompareDocumentsResponse {
                success: false,
                result: None,
                error: Some(format!(
                    "Document A with ID '{}' not found",
                    request.document_a_id
                )),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }
    };

    let doc_b_entry = match indexer.get_document(&request.document_b_id) {
        Some(entry) => entry,
        None => {
            return Ok(CompareDocumentsResponse {
                success: false,
                result: None,
                error: Some(format!(
                    "Document B with ID '{}' not found",
                    request.document_b_id
                )),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }
    };

    // Convert paths to strings
    let path_a = doc_a_entry.path.to_string_lossy().to_string();
    let path_b = doc_b_entry.path.to_string_lossy().to_string();

    // Release the indexer lock
    drop(indexer_lock);

    // Parse the documents
    let doc_a = match parse_document_from_path(&path_a).await {
        Ok(doc) => doc,
        Err(e) => {
            return Ok(CompareDocumentsResponse {
                success: false,
                result: None,
                error: Some(format!("Failed to parse document A: {}", e)),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }
    };

    let doc_b = match parse_document_from_path(&path_b).await {
        Ok(doc) => doc,
        Err(e) => {
            return Ok(CompareDocumentsResponse {
                success: false,
                result: None,
                error: Some(format!("Failed to parse document B: {}", e)),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }
    };

    // Convert comparison type from frontend format to backend format
    let comparison_type = match request.comparison_type.as_str() {
        "TextDiff" => ComparisonType::TextDiff,
        "StructuralDiff" => ComparisonType::StructuralDiff,
        "SemanticSimilarity" => ComparisonType::SemanticSimilarity,
        "Comprehensive" => ComparisonType::Comprehensive,
        _ => ComparisonType::Comprehensive, // Default fallback
    };

    let options = ComparisonOptions {
        include_metadata: request.include_metadata,
        similarity_threshold: request.similarity_threshold,
        max_differences: None,
        ignore_formatting: false,
        focus_on_content_changes: true,
    };

    let comparison_request = DocumentComparisonRequest {
        document_a: doc_a,
        document_b: doc_b,
        comparison_type,
        options,
    };

    let comparator_lock = comparison_state.comparator.lock().await;
    match comparator_lock.compare_documents(comparison_request).await {
        Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Store in history
            let mut history_lock = comparison_state.comparison_history.lock().await;
            history_lock.push(result.clone());
            if history_lock.len() > 100 {
                history_lock.remove(0);
            }

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

// Helper functions
async fn parse_document_from_path(file_path: &str) -> Result<DocumentForComparison, String> {
    use crate::commands::document_commands::parse_document;
    use std::path::Path;

    // First try the structured document parser for supported formats
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
            } => {
                // If structured parsing fails, try to read as plain text for common text formats
                let path = Path::new(&file_path);
                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase());

                match extension.as_deref() {
                    Some("md") | Some("txt") | Some("markdown") | Some("text") | Some("rst") => {
                        // Read as plain text
                        parse_text_document(&file_path).await
                    }
                    _ => Err(format!(
                        "Failed to parse document {}: {}",
                        file_path, message
                    )),
                }
            }
        },
        Err(e) => {
            // If the main parser fails entirely, try text parsing for supported formats
            let path = Path::new(file_path);
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase());

            match extension.as_deref() {
                Some("md") | Some("txt") | Some("markdown") | Some("text") | Some("rst") => {
                    parse_text_document(file_path).await
                }
                _ => Err(format!("Document parsing error: {}", e)),
            }
        }
    }
}

async fn parse_text_document(file_path: &str) -> Result<DocumentForComparison, String> {
    use std::fs;

    match fs::read_to_string(file_path) {
        Ok(content) => {
            let mut metadata = HashMap::new();

            // Extract filename as title
            if let Some(filename) = std::path::Path::new(file_path).file_stem() {
                if let Some(title) = filename.to_str() {
                    metadata.insert("title".to_string(), title.to_string());
                }
            }

            // Determine file type
            let extension = std::path::Path::new(file_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("txt");
            metadata.insert("format".to_string(), extension.to_string());

            Ok(DocumentForComparison {
                file_path: file_path.to_string(),
                content: Some(ParsedDocumentContent::Text { content }),
                metadata,
            })
        }
        Err(e) => Err(format!("Failed to read text file {}: {}", file_path, e)),
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
