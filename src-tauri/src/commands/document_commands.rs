// src-tauri/src/commands/document_commands.rs
// Tauri commands for document processing

use crate::document::{DocxContent, DocxParser, PdfContent, PdfParser};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Response for document parsing operations
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentParseResponse {
    Docx {
        content: DocxContent,
        file_path: String,
    },
    Pdf {
        content: PdfContent,
        file_path: String,
    },
    Error {
        message: String,
        file_path: String,
    },
}

/// Parse a DOCX document
#[tauri::command]
pub async fn parse_docx_document(file_path: String) -> Result<DocumentParseResponse, String> {
    let path = PathBuf::from(&file_path);

    // Validate file exists and has correct extension
    if !path.exists() {
        return Ok(DocumentParseResponse::Error {
            message: "File does not exist".to_string(),
            file_path,
        });
    }

    if !file_path.to_lowercase().ends_with(".docx") {
        return Ok(DocumentParseResponse::Error {
            message: "File is not a DOCX document".to_string(),
            file_path,
        });
    }

    match DocxParser::parse(&path) {
        Ok(content) => Ok(DocumentParseResponse::Docx { content, file_path }),
        Err(e) => Ok(DocumentParseResponse::Error {
            message: format!("Failed to parse DOCX: {}", e),
            file_path,
        }),
    }
}

/// Parse a PDF document
#[tauri::command]
pub async fn parse_pdf_document(file_path: String) -> Result<DocumentParseResponse, String> {
    let path = PathBuf::from(&file_path);

    // Validate file exists and has correct extension
    if !path.exists() {
        return Ok(DocumentParseResponse::Error {
            message: "File does not exist".to_string(),
            file_path,
        });
    }

    if !file_path.to_lowercase().ends_with(".pdf") {
        return Ok(DocumentParseResponse::Error {
            message: "File is not a PDF document".to_string(),
            file_path,
        });
    }

    match PdfParser::parse(&path) {
        Ok(content) => Ok(DocumentParseResponse::Pdf { content, file_path }),
        Err(e) => Ok(DocumentParseResponse::Error {
            message: format!("Failed to parse PDF: {}", e),
            file_path,
        }),
    }
}

/// Parse any supported document format
#[tauri::command]
pub async fn parse_document(file_path: String) -> Result<DocumentParseResponse, String> {
    let path = PathBuf::from(&file_path);

    // Validate file exists
    if !path.exists() {
        return Ok(DocumentParseResponse::Error {
            message: "File does not exist".to_string(),
            file_path,
        });
    }

    // Determine parser based on file extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "docx" => parse_docx_document(file_path).await,
        "pdf" => parse_pdf_document(file_path).await,
        _ => Ok(DocumentParseResponse::Error {
            message: format!("Unsupported file format: {}", extension),
            file_path,
        }),
    }
}

/// Get supported document formats
#[tauri::command]
pub async fn get_supported_document_formats() -> Result<Vec<String>, String> {
    Ok(vec!["docx".to_string(), "pdf".to_string()])
}

/// Document processing statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentStats {
    pub total_files_processed: u64,
    pub successful_parses: u64,
    pub failed_parses: u64,
    pub supported_formats: Vec<String>,
}

/// Get document processing statistics
#[tauri::command]
pub async fn get_document_processing_stats() -> Result<DocumentStats, String> {
    // For now, return placeholder stats
    // In a real implementation, this would be tracked in application state
    Ok(DocumentStats {
        total_files_processed: 0,
        successful_parses: 0,
        failed_parses: 0,
        supported_formats: vec!["docx".to_string(), "pdf".to_string()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_supported_formats() {
        let formats = get_supported_document_formats().await.unwrap();
        assert!(formats.contains(&"docx".to_string()));
        assert!(formats.contains(&"pdf".to_string()));
    }

    #[tokio::test]
    async fn test_parse_nonexistent_file() {
        let response = parse_document("/nonexistent/file.docx".to_string())
            .await
            .unwrap();
        match response {
            DocumentParseResponse::Error { message, .. } => {
                assert!(message.contains("does not exist"));
            }
            _ => panic!("Expected error response"),
        }
    }

    #[tokio::test]
    async fn test_unsupported_format() {
        // Create a temporary file with unsupported extension
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path_with_unsupported_ext = format!("{}.txt", temp_file.path().display());

        // Write some content to the file to make it exist
        std::fs::write(&path_with_unsupported_ext, "test content").unwrap();

        let response = parse_document(path_with_unsupported_ext).await.unwrap();
        match response {
            DocumentParseResponse::Error { message, .. } => {
                assert!(message.contains("Unsupported file format") || message.contains("txt"));
            }
            _ => panic!("Expected error response"),
        }
    }
}
