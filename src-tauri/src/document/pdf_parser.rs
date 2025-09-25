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

        tracing::debug!("PDF has {} pages", page_count);

        for page_num in 1..=page_count {
            match Self::extract_text_from_page(document, page_num as u32) {
                Ok(page_text) => {
                    tracing::debug!("Page {} extracted {} characters", page_num, page_text.len());
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

        tracing::debug!("Total extracted text length: {}", text_content.len());

        // If no text was extracted, provide helpful information
        if text_content.trim().is_empty() {
            let message = format!("This PDF document contains {} pages but no extractable text was found. This may be a scanned document or use complex formatting that requires OCR (Optical Character Recognition) to extract text.", page_count);
            tracing::info!("PDF text extraction result: {}", message);
            Ok(message)
        } else {
            Ok(text_content.trim().to_string())
        }
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
    fn extract_text_objects(document: &lopdf::Document, page: &lopdf::Object) -> Result<String> {
        let mut text = String::new();

        // Extract content streams from the page
        if let lopdf::Object::Dictionary(page_dict) = page {
            if let Ok(contents_obj) = page_dict.get(b"Contents") {
                // Handle both single content stream and array of streams
                match contents_obj {
                    lopdf::Object::Reference(_reference) => {
                        if let Ok((_, stream_obj)) = document.dereference(contents_obj) {
                            if let Ok(content_text) =
                                Self::extract_text_from_stream(document, stream_obj)
                            {
                                text.push_str(&content_text);
                            }
                        }
                    }
                    lopdf::Object::Array(stream_refs) => {
                        for stream_ref in stream_refs {
                            if let Ok((_, stream_obj)) = document.dereference(stream_ref) {
                                if let Ok(content_text) =
                                    Self::extract_text_from_stream(document, stream_obj)
                                {
                                    text.push_str(&content_text);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(text)
    }

    /// Extract text from a content stream object
    fn extract_text_from_stream(
        _document: &lopdf::Document,
        stream_obj: &lopdf::Object,
    ) -> Result<String> {
        if let lopdf::Object::Stream(stream) = stream_obj {
            tracing::debug!(
                "Processing content stream with {} bytes",
                stream.content.len()
            );

            // Try to decode the stream content
            match stream.decode_content() {
                Ok(decoded_content) => {
                    tracing::debug!("Successfully decoded stream content");
                    // Convert the Content to a string representation
                    let content_str = format!("{:?}", decoded_content);
                    let extracted = Self::extract_text_from_content_stream(&content_str);
                    tracing::debug!(
                        "Extracted {} characters from decoded content",
                        extracted.len()
                    );
                    Ok(extracted)
                }
                Err(e) => {
                    tracing::debug!("Failed to decode stream content: {}, trying raw content", e);
                    // Fallback: try to get raw stream data
                    let content_str = String::from_utf8_lossy(&stream.content);
                    let extracted = Self::extract_text_from_content_stream(&content_str);
                    tracing::debug!("Extracted {} characters from raw content", extracted.len());
                    Ok(extracted)
                }
            }
        } else {
            tracing::debug!("Stream object is not a stream");
            Ok(String::new())
        }
    }

    /// Extract text from PDF content stream (enhanced implementation)
    fn extract_text_from_content_stream(content: &str) -> String {
        let mut text = String::new();
        let mut in_text_object = false;

        // Try multiple parsing approaches

        // Approach 1: Token-based parsing (existing approach, enhanced)
        let tokens: Vec<&str> = content.split_whitespace().collect();
        let mut i = 0;

        while i < tokens.len() {
            let token = tokens[i];

            if token == "BT" {
                in_text_object = true;
            } else if token == "ET" {
                in_text_object = false;
                text.push(' '); // Add space between text objects
            } else if in_text_object {
                // Handle different text-showing commands
                if token == "Tj" || token == "TJ" || token == "'" || token == "\"" {
                    // Look for text strings before this command
                    if i > 0 {
                        let prev_token = tokens[i - 1];
                        if let Some(extracted_text) =
                            Self::extract_text_from_string_literal(prev_token)
                        {
                            text.push_str(&extracted_text);
                            text.push(' '); // Add space between text chunks
                        }
                    }
                } else if token.starts_with('(') && token.ends_with(')') {
                    // Direct text string - might be followed by a text command
                    if let Some(extracted_text) = Self::extract_text_from_string_literal(token) {
                        text.push_str(&extracted_text);
                        text.push(' ');
                    }
                } else if token.starts_with('(') && !token.ends_with(')') {
                    // Multi-token string - collect until closing parenthesis
                    let mut string_parts = vec![token];
                    let mut j = i + 1;
                    while j < tokens.len() && !tokens[j - 1].ends_with(')') {
                        string_parts.push(tokens[j]);
                        j += 1;
                    }
                    let combined_string = string_parts.join(" ");
                    if let Some(extracted_text) =
                        Self::extract_text_from_string_literal(&combined_string)
                    {
                        text.push_str(&extracted_text);
                        text.push(' ');
                    }
                    i = j - 1; // Skip processed tokens
                }
            }

            i += 1;
        }

        // Approach 2: Line-based parsing as fallback
        if text.trim().is_empty() {
            text = Self::extract_text_line_based(content);
        }

        // Clean up the extracted text
        let cleaned_text = text.trim().replace("  ", " ");
        tracing::debug!(
            "Extracted text from content stream: {} characters",
            cleaned_text.len()
        );
        cleaned_text
    }

    /// Fallback line-based text extraction for complex PDFs
    fn extract_text_line_based(content: &str) -> String {
        let mut text = String::new();
        let mut in_text_object = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "BT" {
                in_text_object = true;
            } else if trimmed == "ET" {
                in_text_object = false;
                text.push(' ');
            } else if in_text_object {
                // Look for parenthesized strings in the line
                let mut chars = trimmed.chars().peekable();
                while let Some(ch) = chars.next() {
                    if ch == '(' {
                        let mut string_content = String::new();
                        let mut paren_count = 1;

                        while paren_count > 0 {
                            if let Some(next_ch) = chars.next() {
                                if next_ch == '(' {
                                    paren_count += 1;
                                    string_content.push(next_ch);
                                } else if next_ch == ')' {
                                    paren_count -= 1;
                                    if paren_count > 0 {
                                        string_content.push(next_ch);
                                    }
                                } else {
                                    string_content.push(next_ch);
                                }
                            } else {
                                break;
                            }
                        }

                        if !string_content.is_empty() {
                            // Handle basic escape sequences
                            let unescaped = string_content
                                .replace("\\(", "(")
                                .replace("\\)", ")")
                                .replace("\\\\", "\\")
                                .replace("\\n", "\n")
                                .replace("\\r", "\r")
                                .replace("\\t", "\t");
                            text.push_str(&unescaped);
                            text.push(' ');
                        }
                    }
                }
            }
        }

        text
    }

    /// Extract text from string literal (handles parentheses)
    fn extract_text_from_string_literal(token: &str) -> Option<String> {
        if token.starts_with('(') && token.ends_with(')') {
            let text_part = &token[1..token.len() - 1]; // Remove parentheses
                                                        // Handle basic escape sequences
            let unescaped = text_part
                .replace("\\(", "(")
                .replace("\\)", ")")
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t");
            Some(unescaped)
        } else {
            None
        }
    }

    /// Extract text from Tj command (legacy function, kept for compatibility)
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
