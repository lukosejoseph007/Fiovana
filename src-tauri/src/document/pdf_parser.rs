// src-tauri/src/document/pdf_parser.rs
// PDF document parser implementation

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Parsed PDF document content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfContent {
    /// Extracted text content
    pub text: String,
    /// Document structure information
    pub structure: PdfStructure,
    /// Metadata extracted from the document
    pub metadata: PdfMetadata,
}

/// PDF document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfStructure {
    /// Number of pages
    pub page_count: usize,
    /// Estimated paragraph count
    pub paragraph_count: usize,
    /// Detected headings
    pub headings: Vec<PdfHeading>,
}

/// PDF heading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfHeading {
    /// Heading text
    pub text: String,
    /// Page number where heading appears
    pub page: usize,
    /// Estimated heading level
    pub level: u8,
}

/// PDF metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PdfMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
    /// Document creator application
    pub creator: Option<String>,
    /// PDF producer
    pub producer: Option<String>,
}

/// PDF document parser
#[allow(dead_code)]
pub struct PdfParser;

#[allow(dead_code)]
impl PdfParser {
    /// Parse a PDF file and extract content
    pub fn parse<P: AsRef<Path>>(file_path: P) -> Result<PdfContent> {
        let document = lopdf::Document::load(&file_path)
            .with_context(|| format!("Failed to load PDF file: {:?}", file_path.as_ref()))?;

        // Extract text content
        let text = Self::extract_text(&document)?;

        // Extract metadata
        let metadata = Self::extract_metadata(&document)?;

        // Analyze structure
        let structure = Self::analyze_structure(&document, &text)?;

        Ok(PdfContent {
            text,
            structure,
            metadata,
        })
    }

    /// Extract text content from PDF
    fn extract_text(document: &lopdf::Document) -> Result<String> {
        let mut text_content = String::new();
        let page_count = document.get_pages().len();

        for page_num in 1..=page_count {
            match Self::extract_text_from_page(document, page_num as u32) {
                Ok(page_text) => {
                    if !page_text.trim().is_empty() {
                        text_content.push_str(&page_text);
                        text_content.push('\n');
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to extract text from page {}: {}", page_num, e);
                    // Continue processing other pages
                }
            }
        }

        Ok(text_content.trim().to_string())
    }

    /// Extract text from a specific page
    fn extract_text_from_page(document: &lopdf::Document, page_num: u32) -> Result<String> {
        // This is a simplified text extraction
        // In a production system, you'd want more sophisticated text extraction
        // that handles fonts, encoding, and layout properly

        let pages = document.get_pages();
        let page_id = pages
            .get(&page_num)
            .ok_or_else(|| anyhow!("Page {} not found", page_num))?;

        let page = document.get_object(*page_id)?;

        // Try to extract text content - this is a basic implementation
        // Real PDF text extraction is much more complex
        match Self::extract_text_objects(document, page) {
            Ok(text) => Ok(text),
            Err(_) => {
                // Fallback: return empty string for this page
                tracing::debug!("Could not extract text from page {}", page_num);
                Ok(String::new())
            }
        }
    }

    /// Extract text objects from a PDF page object
    fn extract_text_objects(_document: &lopdf::Document, _page: &lopdf::Object) -> Result<String> {
        // Simplified implementation - just return empty for now
        // This is a placeholder for more sophisticated PDF text extraction
        // which would require proper content stream parsing and font handling

        // TODO: Implement proper PDF text extraction
        // This would involve:
        // 1. Parsing content streams
        // 2. Handling text positioning commands
        // 3. Font and encoding management
        // 4. Glyph mapping

        Ok(String::new())
    }

    /// Extract text from PDF content stream (basic implementation)
    fn extract_text_from_content_stream(content: &str) -> String {
        let mut text = String::new();
        let mut in_text_object = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "BT" {
                in_text_object = true;
            } else if trimmed == "ET" {
                in_text_object = false;
                text.push(' '); // Add space between text objects
            } else if in_text_object && (trimmed.starts_with('(') && trimmed.ends_with(") Tj")) {
                // Extract text from Tj commands: (text) Tj
                if let Some(text_part) = Self::extract_text_from_tj_command(trimmed) {
                    text.push_str(&text_part);
                }
            }
        }

        text
    }

    /// Extract text from Tj command
    fn extract_text_from_tj_command(command: &str) -> Option<String> {
        if command.starts_with('(') && command.ends_with(") Tj") {
            let text_part = &command[1..command.len() - 4]; // Remove ( and ) Tj
            Some(text_part.to_string())
        } else {
            None
        }
    }

    /// Extract metadata from PDF
    fn extract_metadata(document: &lopdf::Document) -> Result<PdfMetadata> {
        let mut metadata = PdfMetadata::default();

        // Try to extract metadata from document info dictionary
        if let Ok(info_dict) = document.trailer.get(b"Info") {
            if let Ok((_, lopdf::Object::Dictionary(dict))) = document.dereference(info_dict) {
                // Extract common metadata fields
                metadata.title = Self::extract_string_from_dict(dict, b"Title");
                metadata.author = Self::extract_string_from_dict(dict, b"Author");
                metadata.creator = Self::extract_string_from_dict(dict, b"Creator");
                metadata.producer = Self::extract_string_from_dict(dict, b"Producer");
                metadata.creation_date = Self::extract_string_from_dict(dict, b"CreationDate");
                metadata.modification_date = Self::extract_string_from_dict(dict, b"ModDate");
            }
        }

        Ok(metadata)
    }

    /// Extract string value from dictionary
    fn extract_string_from_dict(dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
        dict.get(key).ok().and_then(|obj| match obj {
            lopdf::Object::String(bytes, _) => String::from_utf8(bytes.clone()).ok(),
            _ => None,
        })
    }

    /// Analyze document structure
    fn analyze_structure(document: &lopdf::Document, text: &str) -> Result<PdfStructure> {
        let page_count = document.get_pages().len();
        let lines: Vec<&str> = text.lines().collect();
        let paragraph_count = lines.iter().filter(|line| !line.trim().is_empty()).count();

        // Basic heading detection
        let mut headings = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if Self::is_likely_heading(line) {
                headings.push(PdfHeading {
                    text: line.trim().to_string(),
                    page: (index * page_count / lines.len().max(1)) + 1, // Estimate page
                    level: Self::detect_heading_level(line),
                });
            }
        }

        Ok(PdfStructure {
            page_count,
            paragraph_count,
            headings,
        })
    }

    /// Simple heuristic to detect if a line is likely a heading
    fn is_likely_heading(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.len() > 100 {
            return false;
        }

        // Similar heuristics as DOCX parser
        !trimmed.ends_with('.')
            && (trimmed.chars().next().unwrap_or(' ').is_uppercase()
                || trimmed.starts_with(char::is_numeric))
    }

    /// Detect heading level based on content
    fn detect_heading_level(line: &str) -> u8 {
        let trimmed = line.trim();

        // Check most specific patterns first
        if trimmed.starts_with("1.1.1") || trimmed.starts_with("a.") {
            3
        } else if trimmed.starts_with("1.1") || trimmed.starts_with("A.") {
            2
        } else {
            1 // Default to level 1 (includes "1." and "I." patterns)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_detection() {
        assert!(PdfParser::is_likely_heading("Introduction"));
        assert!(PdfParser::is_likely_heading("1. Overview"));
        assert!(PdfParser::is_likely_heading("Chapter 1"));

        assert!(!PdfParser::is_likely_heading(
            "This is a long sentence that ends with a period."
        ));
        assert!(!PdfParser::is_likely_heading(""));
    }

    #[test]
    fn test_heading_level_detection() {
        assert_eq!(PdfParser::detect_heading_level("1. Introduction"), 1);
        assert_eq!(PdfParser::detect_heading_level("1.1 Getting Started"), 2); // Now correctly returns 2
        assert_eq!(PdfParser::detect_heading_level("1.1.1 Installation"), 3);
    }

    #[test]
    fn test_tj_command_extraction() {
        assert_eq!(
            PdfParser::extract_text_from_tj_command("(Hello World) Tj"),
            Some("Hello World".to_string())
        );
        assert_eq!(
            PdfParser::extract_text_from_tj_command("(Test) Tj"),
            Some("Test".to_string())
        );
        assert_eq!(
            PdfParser::extract_text_from_tj_command("invalid command"),
            None
        );
    }
}
