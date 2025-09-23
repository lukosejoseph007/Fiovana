// src-tauri/src/document/file_processor.rs
// File processing with corruption detection using header validation

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::document::{DocxParser, EnhancedMetadata, MetadataExtractor, PdfParser};

/// File corruption check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorruptionCheckResult {
    /// Whether the file appears to be corrupted
    pub is_corrupted: bool,
    /// Detected file type from magic numbers
    pub detected_type: Option<String>,
    /// Expected file type from extension
    pub expected_type: Option<String>,
    /// Whether the detected type matches the expected type
    pub type_mismatch: bool,
    /// Detailed corruption analysis
    pub corruption_details: Vec<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

impl CorruptionCheckResult {
    /// Create a new result indicating no corruption
    pub fn clean(detected_type: Option<String>, expected_type: Option<String>) -> Self {
        let type_mismatch = match (&detected_type, &expected_type) {
            (Some(detected), Some(expected)) => detected != expected,
            _ => false,
        };

        Self {
            is_corrupted: false,
            detected_type,
            expected_type,
            type_mismatch,
            corruption_details: Vec::new(),
            confidence: 1.0,
        }
    }

    /// Create a new result indicating corruption
    pub fn corrupted(
        detected_type: Option<String>,
        expected_type: Option<String>,
        details: Vec<String>,
        confidence: f64,
    ) -> Self {
        let type_mismatch = match (&detected_type, &expected_type) {
            (Some(detected), Some(expected)) => detected != expected,
            _ => false,
        };

        Self {
            is_corrupted: true,
            detected_type,
            expected_type,
            type_mismatch,
            corruption_details: details,
            confidence,
        }
    }
}

/// Enhanced document processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDocumentResult {
    /// Enhanced metadata
    pub metadata: Option<EnhancedMetadata>,
    /// Document content
    pub content: Option<DocumentContent>,
    /// Document structure
    pub structure: Option<DocumentStructure>,
    /// Corruption check result
    pub corruption_check: CorruptionCheckResult,
    /// Processing status
    pub processing_status: ProcessingStatus,
}

/// Document content information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    /// Full text content
    pub text: String,
    /// Document title
    pub title: Option<String>,
    /// Document sections
    pub sections: Vec<DocumentSection>,
    /// Key terms extracted
    pub key_terms: Vec<String>,
    /// Word count
    pub word_count: usize,
    /// Detected language
    pub language: Option<String>,
}

/// Document section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    /// Section title
    pub title: String,
    /// Section level (0 = top level)
    pub level: u32,
    /// Section content
    pub content: String,
    /// Starting line number
    pub line_start: usize,
    /// Ending line number
    pub line_end: usize,
}

/// Document structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructure {
    /// Document type classification
    pub document_type: DocumentType,
    /// Extracted headings
    pub headings: Vec<DocumentHeading>,
    /// Extracted lists
    pub lists: Vec<DocumentList>,
    /// Extracted tables
    pub tables: Vec<DocumentTable>,
    /// Extracted images
    pub images: Vec<DocumentImage>,
    /// Page count (if available)
    pub page_count: Option<usize>,
    /// Has table of contents
    pub has_toc: bool,
}

/// Document type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    Manual,
    Guide,
    Procedure,
    Reference,
    Training,
    Policy,
    Template,
    Report,
    Article,
    Other(String),
}

impl DocumentType {
    /// Classify document type from content and filename
    pub fn from_content(content: &str, path: &Path) -> Self {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();

        let content_lower = content.to_lowercase();

        // Classification based on filename
        if filename.contains("manual")
            || content_lower.contains("user manual")
            || content_lower.contains("instruction manual")
        {
            DocumentType::Manual
        } else if filename.contains("guide")
            || content_lower.contains("guide")
            || content_lower.contains("how to")
        {
            DocumentType::Guide
        } else if filename.contains("procedure")
            || content_lower.contains("procedure")
            || content_lower.contains("step by step")
        {
            DocumentType::Procedure
        } else if filename.contains("reference")
            || content_lower.contains("reference")
            || content_lower.contains("documentation")
        {
            DocumentType::Reference
        } else if filename.contains("training")
            || content_lower.contains("training")
            || content_lower.contains("course")
        {
            DocumentType::Training
        } else if filename.contains("policy")
            || content_lower.contains("policy")
            || content_lower.contains("regulation")
        {
            DocumentType::Policy
        } else if filename.contains("template") || content_lower.contains("template") {
            DocumentType::Template
        } else if filename.contains("report")
            || content_lower.contains("report")
            || content_lower.contains("analysis")
        {
            DocumentType::Report
        } else if filename.contains("article") || content_lower.contains("article") {
            DocumentType::Article
        } else {
            DocumentType::Other(filename)
        }
    }
}

/// Document heading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHeading {
    /// Heading text
    pub text: String,
    /// Heading level (0 = H1, 1 = H2, etc.)
    pub level: u32,
    /// Position in document
    pub position: usize,
}

/// Document list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentList {
    /// List type
    pub list_type: ListType,
    /// List items
    pub items: Vec<String>,
}

/// List type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ListType {
    Ordered,
    Unordered,
}

/// Document table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTable {
    /// Table rows (each row is a vector of cell contents)
    pub rows: Vec<Vec<String>>,
    /// Whether first row is header
    pub has_header: bool,
}

/// Document image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentImage {
    /// Alt text
    pub alt_text: String,
    /// Image source
    pub src: String,
    /// Image type
    pub image_type: ImageType,
}

/// Image type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageType {
    Embedded,
    Referenced,
    Inline,
}

/// Processing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Success,
    PartialSuccess,
    CorruptedFile,
    UnsupportedFormat,
    ProcessingError(String),
}

/// File magic number patterns for common document types
pub struct MagicNumbers;

impl MagicNumbers {
    /// Get magic number patterns for file type detection
    pub fn get_patterns() -> Vec<(Vec<u8>, &'static str, &'static str)> {
        vec![
            // PDF files
            (vec![0x25, 0x50, 0x44, 0x46], "pdf", "PDF document"),
            // Microsoft Office formats (ZIP-based)
            (
                vec![0x50, 0x4B, 0x03, 0x04],
                "zip",
                "ZIP archive (may be Office doc)",
            ),
            (vec![0x50, 0x4B, 0x05, 0x06], "zip", "ZIP archive (empty)"),
            (vec![0x50, 0x4B, 0x07, 0x08], "zip", "ZIP archive (spanned)"),
            // Plain text files
            (vec![0xEF, 0xBB, 0xBF], "txt", "UTF-8 BOM text"),
            (vec![0xFF, 0xFE], "txt", "UTF-16 LE BOM text"),
            (vec![0xFE, 0xFF], "txt", "UTF-16 BE BOM text"),
            // Rich Text Format
            (
                vec![0x7B, 0x5C, 0x72, 0x74, 0x66],
                "rtf",
                "Rich Text Format",
            ),
            // JPEG images (sometimes embedded in documents)
            (vec![0xFF, 0xD8, 0xFF], "jpg", "JPEG image"),
            // PNG images
            (
                vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
                "png",
                "PNG image",
            ),
        ]
    }

    /// Detect file type from magic numbers
    pub fn detect_type(header: &[u8]) -> Option<(String, String)> {
        let patterns = Self::get_patterns();

        for (pattern, file_type, description) in patterns {
            if header.len() >= pattern.len() && header.starts_with(&pattern) {
                return Some((file_type.to_string(), description.to_string()));
            }
        }

        // Check if it looks like text
        if Self::is_likely_text(header) {
            return Some(("txt".to_string(), "Plain text".to_string()));
        }

        None
    }

    /// Check if the header suggests this is likely a text file
    fn is_likely_text(header: &[u8]) -> bool {
        if header.is_empty() {
            return false;
        }

        // Count printable ASCII characters
        let printable_count = header.iter()
            .take(512) // Check first 512 bytes
            .filter(|&&b| (32..=126).contains(&b) || b == 9 || b == 10 || b == 13) // printable + tab + CR + LF
            .count();

        let total_checked = header.len().min(512);
        let printable_ratio = printable_count as f64 / total_checked as f64;

        // If more than 95% are printable characters, likely text
        printable_ratio > 0.95
    }
}

/// File processor for corruption detection and validation
pub struct FileProcessor;

#[allow(dead_code)]
impl FileProcessor {
    /// Check file for corruption using header validation
    pub fn check_corruption<P: AsRef<Path>>(path: P) -> Result<CorruptionCheckResult> {
        let path = path.as_ref();

        // Get expected type from file extension
        let expected_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        // Read file header for magic number detection
        let mut file =
            File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

        let mut header = vec![0u8; 512]; // Read first 512 bytes
        let bytes_read = file
            .read(&mut header)
            .with_context(|| format!("Failed to read file header: {}", path.display()))?;

        if bytes_read == 0 {
            return Ok(CorruptionCheckResult::corrupted(
                None,
                expected_type,
                vec!["File is empty".to_string()],
                0.9,
            ));
        }

        header.truncate(bytes_read);

        // Detect file type from magic numbers
        let (detected_type, _description) = MagicNumbers::detect_type(&header)
            .map(|(t, d)| (Some(t), d))
            .unwrap_or((None, String::new()));

        // Perform specific validation based on detected/expected type
        let mut corruption_details = Vec::new();
        let mut confidence = 1.0;
        let mut is_corrupted = false;

        // PDF-specific validation
        if detected_type.as_deref() == Some("pdf") || expected_type.as_deref() == Some("pdf") {
            if let Err(e) = Self::validate_pdf_structure(&mut file) {
                corruption_details.push(format!("PDF validation failed: {}", e));
                is_corrupted = true;
                confidence = 0.8;
            }
        }

        // ZIP-based Office document validation
        if detected_type.as_deref() == Some("zip")
            && expected_type
                .as_deref()
                .is_some_and(|ext| ["docx", "xlsx", "pptx"].contains(&ext))
        {
            if let Err(e) = Self::validate_office_document(&mut file) {
                corruption_details.push(format!("Office document validation failed: {}", e));
                is_corrupted = true;
                confidence = 0.8;
            }
        }

        // Check for type mismatch
        if let (Some(ref detected), Some(ref expected)) = (&detected_type, &expected_type) {
            if detected != expected && !Self::is_compatible_type(detected, expected) {
                corruption_details.push(format!(
                    "File type mismatch: detected '{}', expected '{}'",
                    detected, expected
                ));
                confidence *= 0.9;
            }
        }

        // Generic corruption indicators
        if let Some(generic_issues) = Self::check_generic_corruption(&header) {
            corruption_details.extend(generic_issues);
            is_corrupted = true;
            confidence *= 0.8;
        }

        if is_corrupted || !corruption_details.is_empty() {
            Ok(CorruptionCheckResult::corrupted(
                detected_type,
                expected_type,
                corruption_details,
                confidence,
            ))
        } else {
            Ok(CorruptionCheckResult::clean(detected_type, expected_type))
        }
    }

    /// Validate PDF file structure
    fn validate_pdf_structure(file: &mut File) -> Result<()> {
        // Check PDF header
        file.seek(SeekFrom::Start(0))?;
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;

        if !header.starts_with(b"%PDF-") {
            bail!("Invalid PDF header");
        }

        // Check for PDF trailer (basic check)
        let file_size = file.metadata()?.len();
        if file_size > 1024 {
            let mut trailer_area = vec![0u8; 1024.min(file_size as usize)];
            file.seek(SeekFrom::End(-(trailer_area.len() as i64)))?;
            file.read_exact(&mut trailer_area)?;

            let trailer_str = String::from_utf8_lossy(&trailer_area);
            if !trailer_str.contains("%%EOF") {
                bail!("PDF trailer not found or malformed");
            }
        }

        Ok(())
    }

    /// Validate Office document (ZIP-based) structure
    fn validate_office_document(file: &mut File) -> Result<()> {
        // Basic ZIP validation - check local file header signature
        file.seek(SeekFrom::Start(0))?;
        let mut signature = [0u8; 4];
        file.read_exact(&mut signature)?;

        if signature != [0x50, 0x4B, 0x03, 0x04]
            && signature != [0x50, 0x4B, 0x05, 0x06]
            && signature != [0x50, 0x4B, 0x07, 0x08]
        {
            bail!("Invalid ZIP/Office document signature");
        }

        // TODO: Could add more sophisticated Office document validation
        // - Check for required Office document structure
        // - Validate content types
        // - Check relationships

        Ok(())
    }

    /// Check for generic corruption indicators
    fn check_generic_corruption(header: &[u8]) -> Option<Vec<String>> {
        let mut issues = Vec::new();

        // Check for null-byte patterns that might indicate corruption
        let null_count = header.iter().filter(|&&b| b == 0).count();
        let null_ratio = null_count as f64 / header.len() as f64;

        if null_ratio > 0.5 {
            issues.push("High ratio of null bytes detected".to_string());
        }

        // Check for repeating patterns (might indicate data corruption)
        if Self::has_excessive_repetition(header) {
            issues.push("Excessive byte repetition detected".to_string());
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }

    /// Check if file has excessive byte repetition
    fn has_excessive_repetition(data: &[u8]) -> bool {
        if data.len() < 64 {
            return false;
        }

        let mut byte_counts = [0u32; 256];
        for &byte in data.iter().take(512) {
            byte_counts[byte as usize] += 1;
        }

        // Check if any single byte makes up more than 80% of the data
        let max_count = *byte_counts.iter().max().unwrap_or(&0);
        let total_bytes = data.len().min(512) as u32;

        max_count > (total_bytes * 4) / 5 // More than 80%
    }

    /// Check if detected and expected types are compatible
    fn is_compatible_type(detected: &str, expected: &str) -> bool {
        match (detected, expected) {
            // ZIP files can be Office documents
            ("zip", "docx") | ("zip", "xlsx") | ("zip", "pptx") => true,
            // Text files might not have BOM
            ("txt", "md") | ("txt", "csv") => true,
            _ => false,
        }
    }

    /// Perform comprehensive file validation
    pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<FileValidationResult> {
        let path = path.as_ref();

        // Check if file exists and is readable
        if !path.exists() {
            return Ok(FileValidationResult::invalid(
                "File does not exist".to_string(),
            ));
        }

        if !path.is_file() {
            return Ok(FileValidationResult::invalid(
                "Path is not a file".to_string(),
            ));
        }

        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to get file metadata: {}", path.display()))?;

        // Check file size (0 bytes is suspicious for most document types)
        if metadata.len() == 0 {
            return Ok(FileValidationResult::invalid("File is empty".to_string()));
        }

        // Check for corruption
        let corruption_result = Self::check_corruption(path)?;

        if corruption_result.is_corrupted {
            return Ok(FileValidationResult::corrupted(corruption_result));
        }

        Ok(FileValidationResult::valid(corruption_result))
    }

    /// Enhanced document processing with rich metadata extraction
    pub fn process_document<P: AsRef<Path>>(path: P) -> Result<ProcessedDocumentResult> {
        let path = path.as_ref();

        // First check for corruption
        let corruption_check = Self::check_corruption(path)?;
        if corruption_check.is_corrupted {
            return Ok(ProcessedDocumentResult {
                metadata: None,
                content: None,
                structure: None,
                corruption_check,
                processing_status: ProcessingStatus::CorruptedFile,
            });
        }

        // Extract enhanced metadata
        let metadata = match MetadataExtractor::extract(path) {
            Ok(meta) => Some(meta),
            Err(e) => {
                tracing::warn!("Failed to extract metadata from {}: {}", path.display(), e);
                None
            }
        };

        // Extract content and structure based on file type
        let (content, structure) = Self::extract_content_and_structure(path)?;

        Ok(ProcessedDocumentResult {
            metadata,
            content,
            structure,
            corruption_check,
            processing_status: ProcessingStatus::Success,
        })
    }

    /// Extract content and structural information from document
    fn extract_content_and_structure(
        path: &Path,
    ) -> Result<(Option<DocumentContent>, Option<DocumentStructure>)> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "docx" => {
                let docx_content = DocxParser::parse(path).context("Failed to parse DOCX file")?;

                let content = DocumentContent {
                    text: docx_content.text.clone(),
                    title: Self::extract_title_from_content(&docx_content.text, path),
                    sections: Self::extract_sections_from_text(&docx_content.text),
                    key_terms: Self::extract_key_terms(&docx_content.text),
                    word_count: Self::count_words(&docx_content.text),
                    language: Self::detect_language(&docx_content.text),
                };

                let structure = DocumentStructure {
                    document_type: DocumentType::from_content(&docx_content.text, path),
                    headings: Self::extract_headings_from_text(&docx_content.text),
                    lists: Self::extract_lists_from_text(&docx_content.text),
                    tables: Self::extract_tables_from_text(&docx_content.text),
                    images: Vec::new(), // DOCX parser doesn't extract images yet
                    page_count: None,
                    has_toc: Self::has_table_of_contents(&docx_content.text),
                };

                Ok((Some(content), Some(structure)))
            }
            "pdf" => {
                let pdf_content = PdfParser::parse(path).context("Failed to parse PDF file")?;

                let content = DocumentContent {
                    text: pdf_content.text.clone(),
                    title: Self::extract_title_from_content(&pdf_content.text, path),
                    sections: Self::extract_sections_from_text(&pdf_content.text),
                    key_terms: Self::extract_key_terms(&pdf_content.text),
                    word_count: Self::count_words(&pdf_content.text),
                    language: Self::detect_language(&pdf_content.text),
                };

                let structure = DocumentStructure {
                    document_type: DocumentType::from_content(&pdf_content.text, path),
                    headings: pdf_content
                        .structure
                        .headings
                        .iter()
                        .map(|h| DocumentHeading {
                            text: h.text.clone(),
                            level: h.level as u32,
                            position: h.page * 1000, // Approximate position based on page
                        })
                        .collect(),
                    lists: Self::extract_lists_from_text(&pdf_content.text),
                    tables: Self::extract_tables_from_text(&pdf_content.text),
                    images: Vec::new(), // PDF parser doesn't extract images yet
                    page_count: Some(pdf_content.structure.page_count),
                    has_toc: Self::has_table_of_contents(&pdf_content.text),
                };

                Ok((Some(content), Some(structure)))
            }
            "txt" | "md" | "markdown" => {
                let text = std::fs::read_to_string(path).context("Failed to read text file")?;

                let content = DocumentContent {
                    text: text.clone(),
                    title: Self::extract_title_from_content(&text, path),
                    sections: Self::extract_sections_from_text(&text),
                    key_terms: Self::extract_key_terms(&text),
                    word_count: Self::count_words(&text),
                    language: Self::detect_language(&text),
                };

                let structure = DocumentStructure {
                    document_type: DocumentType::from_content(&text, path),
                    headings: Self::extract_headings_from_text(&text),
                    lists: Self::extract_lists_from_text(&text),
                    tables: Self::extract_tables_from_text(&text),
                    images: Self::extract_images_from_text(&text),
                    page_count: None,
                    has_toc: Self::has_table_of_contents(&text),
                };

                Ok((Some(content), Some(structure)))
            }
            _ => {
                // Unsupported file type
                Ok((None, None))
            }
        }
    }

    /// Extract title from document content
    fn extract_title_from_content(content: &str, path: &Path) -> Option<String> {
        // Try to find title in first few lines
        for line in content.lines().take(10) {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed.len() < 200 {
                // Markdown heading
                if trimmed.starts_with("# ") {
                    return Some(trimmed.trim_start_matches("# ").to_string());
                }
                // HTML heading
                if trimmed.starts_with("<h1>") && trimmed.ends_with("</h1>") {
                    return Some(
                        trimmed
                            .trim_start_matches("<h1>")
                            .trim_end_matches("</h1>")
                            .to_string(),
                    );
                }
                // First significant line as title
                if trimmed.chars().next().is_some_and(|c| c.is_uppercase())
                    && !trimmed.ends_with('.')
                {
                    return Some(trimmed.to_string());
                }
            }
        }

        // Fallback to filename
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string())
    }

    /// Extract sections from text content
    fn extract_sections_from_text(content: &str) -> Vec<DocumentSection> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_section: Option<DocumentSection> = None;
        let mut line_number = 0;

        for line in lines {
            line_number += 1;
            let trimmed = line.trim();

            // Detect headings
            if let Some((level, title)) = Self::detect_heading_in_line(trimmed) {
                // Save previous section
                if let Some(section) = current_section.take() {
                    sections.push(section);
                }

                // Start new section
                current_section = Some(DocumentSection {
                    title: title.to_string(),
                    level,
                    content: String::new(),
                    line_start: line_number,
                    line_end: line_number,
                });
            } else if let Some(ref mut section) = current_section {
                // Add content to current section
                if !section.content.is_empty() {
                    section.content.push('\n');
                }
                section.content.push_str(line);
                section.line_end = line_number;
            }
        }

        // Save last section
        if let Some(section) = current_section {
            sections.push(section);
        }

        // If no sections found, create one main section
        if sections.is_empty() {
            sections.push(DocumentSection {
                title: "Main Content".to_string(),
                level: 0,
                content: content.to_string(),
                line_start: 1,
                line_end: line_number,
            });
        }

        sections
    }

    /// Detect heading in a line of text
    fn detect_heading_in_line(line: &str) -> Option<(u32, &str)> {
        // Markdown-style headings
        if line.starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count() as u32;
            let title = line.trim_start_matches('#').trim();
            if !title.is_empty() {
                return Some((level - 1, title)); // 0-based level
            }
        }

        // Simple heuristic: short lines that start with uppercase and don't end with period
        if line.len() < 100
            && line.chars().next().is_some_and(|c| c.is_uppercase())
            && !line.ends_with('.')
            && !line.contains(':')
        {
            return Some((0, line));
        }

        None
    }

    /// Extract key terms from text
    fn extract_key_terms(content: &str) -> Vec<String> {
        let mut term_counts = HashMap::new();

        // Simple tokenization and counting
        for word in content.to_lowercase().split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphabetic());
            if cleaned.len() > 3 && !Self::is_stop_word(cleaned) {
                *term_counts.entry(cleaned.to_string()).or_insert(0) += 1;
            }
        }

        // Sort by frequency and take top terms
        let mut terms: Vec<_> = term_counts.into_iter().collect();
        terms.sort_by(|a, b| b.1.cmp(&a.1));

        terms
            .into_iter()
            .take(20)
            .map(|(term, _count)| term)
            .collect()
    }

    /// Check if word is a stop word
    fn is_stop_word(word: &str) -> bool {
        const STOP_WORDS: &[&str] = &[
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
            "one", "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new",
            "now", "old", "see", "two", "who", "boy", "did", "she", "use", "air", "men", "way",
            "come", "could", "show", "also", "after", "back", "other", "many", "than", "then",
            "them", "these", "some", "what", "make", "like", "into", "time", "very", "when",
            "much", "know", "take", "people", "just", "first", "well", "water", "been", "call",
            "find", "long", "down", "made", "part",
        ];
        STOP_WORDS.contains(&word)
    }

    /// Count words in text
    fn count_words(content: &str) -> usize {
        content.split_whitespace().count()
    }

    /// Detect language of text (simple heuristic)
    fn detect_language(content: &str) -> Option<String> {
        // Very simple language detection based on common words
        let text_lower = content.to_lowercase();

        if text_lower.contains(" the ")
            && text_lower.contains(" and ")
            && text_lower.contains(" of ")
        {
            Some("en".to_string())
        } else if text_lower.contains(" el ")
            && text_lower.contains(" de ")
            && text_lower.contains(" la ")
        {
            Some("es".to_string())
        } else if text_lower.contains(" le ")
            && text_lower.contains(" de ")
            && text_lower.contains(" la ")
        {
            Some("fr".to_string())
        } else {
            None
        }
    }

    /// Extract headings from text
    fn extract_headings_from_text(content: &str) -> Vec<DocumentHeading> {
        let mut headings = Vec::new();
        let mut position = 0;

        for line in content.lines() {
            if let Some((level, title)) = Self::detect_heading_in_line(line.trim()) {
                headings.push(DocumentHeading {
                    text: title.to_string(),
                    level,
                    position,
                });
            }
            position += line.len() + 1; // +1 for newline
        }

        headings
    }

    /// Extract lists from text
    fn extract_lists_from_text(content: &str) -> Vec<DocumentList> {
        let mut lists = Vec::new();
        let mut current_list: Option<DocumentList> = None;
        let mut _item_number = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Detect list items
            if trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.chars().next().is_some_and(|c| c.is_numeric())
            {
                let item_text = if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                    trimmed[2..].trim().to_string()
                } else {
                    // Numbered list - remove number and period/parenthesis
                    trimmed
                        .split_once(['.', ')'])
                        .map(|(_, rest)| rest.trim().to_string())
                        .unwrap_or_else(|| trimmed.to_string())
                };

                let list_type = if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                    ListType::Unordered
                } else {
                    ListType::Ordered
                };

                // Start new list or continue existing one
                if let Some(ref mut list) = current_list {
                    if list.list_type == list_type {
                        list.items.push(item_text);
                    } else {
                        // Different list type, save current and start new
                        lists.push(current_list.take().unwrap());
                        current_list = Some(DocumentList {
                            list_type,
                            items: vec![item_text],
                        });
                    }
                } else {
                    current_list = Some(DocumentList {
                        list_type,
                        items: vec![item_text],
                    });
                }
                _item_number += 1;
            } else if current_list.is_some() && !trimmed.is_empty() {
                // Non-list line, save current list
                if let Some(list) = current_list.take() {
                    lists.push(list);
                }
            }
        }

        // Save last list
        if let Some(list) = current_list {
            lists.push(list);
        }

        lists
    }

    /// Extract tables from text
    fn extract_tables_from_text(content: &str) -> Vec<DocumentTable> {
        let mut tables = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            // Detect table rows (simple heuristic: lines with pipe characters)
            if line.contains('|') && line.matches('|').count() >= 2 {
                let mut table_rows = Vec::new();
                let mut j = i;

                // Collect consecutive table rows
                while j < lines.len() {
                    let table_line = lines[j].trim();
                    if table_line.contains('|') && table_line.matches('|').count() >= 2 {
                        let cells: Vec<String> = table_line
                            .split('|')
                            .map(|cell| cell.trim().to_string())
                            .filter(|cell| !cell.is_empty())
                            .collect();
                        table_rows.push(cells);
                        j += 1;
                    } else {
                        break;
                    }
                }

                if table_rows.len() > 1 {
                    tables.push(DocumentTable {
                        rows: table_rows,
                        has_header: true, // Assume first row is header
                    });
                }

                i = j;
            } else {
                i += 1;
            }
        }

        tables
    }

    /// Extract images from text (markdown/HTML format)
    fn extract_images_from_text(content: &str) -> Vec<DocumentImage> {
        let mut images = Vec::new();

        // Markdown images: ![alt](src)
        for line in content.lines() {
            if line.contains("![") && line.contains("](") {
                // Simple regex-like parsing
                if let Some(start) = line.find("![") {
                    if let Some(alt_end) = line[start..].find("](") {
                        if let Some(src_end) = line[start + alt_end + 2..].find(')') {
                            let alt = &line[start + 2..start + alt_end];
                            let src = &line[start + alt_end + 2..start + alt_end + 2 + src_end];

                            images.push(DocumentImage {
                                alt_text: alt.to_string(),
                                src: src.to_string(),
                                image_type: ImageType::Embedded,
                            });
                        }
                    }
                }
            }
        }

        images
    }

    /// Check if content has table of contents
    fn has_table_of_contents(content: &str) -> bool {
        let lower_content = content.to_lowercase();
        lower_content.contains("table of contents")
            || lower_content.contains("contents") && lower_content.contains("page")
            || lower_content.contains("toc")
    }
}

/// Overall file validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValidationResult {
    /// Whether the file is valid
    pub is_valid: bool,
    /// Validation status
    pub status: ValidationStatus,
    /// Corruption check details
    pub corruption_check: Option<CorruptionCheckResult>,
    /// Validation message
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    Corrupted,
    Invalid,
}

impl FileValidationResult {
    /// Create a valid file result
    pub fn valid(corruption_check: CorruptionCheckResult) -> Self {
        Self {
            is_valid: true,
            status: ValidationStatus::Valid,
            corruption_check: Some(corruption_check),
            message: "File validation passed".to_string(),
        }
    }

    /// Create a corrupted file result
    pub fn corrupted(corruption_check: CorruptionCheckResult) -> Self {
        Self {
            is_valid: false,
            status: ValidationStatus::Corrupted,
            corruption_check: Some(corruption_check),
            message: "File appears to be corrupted".to_string(),
        }
    }

    /// Create an invalid file result
    pub fn invalid(message: String) -> Self {
        Self {
            is_valid: false,
            status: ValidationStatus::Invalid,
            corruption_check: None,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_magic_number_detection() {
        // Test PDF detection
        let pdf_header = b"%PDF-1.4\n";
        let result = MagicNumbers::detect_type(pdf_header);
        assert_eq!(
            result,
            Some(("pdf".to_string(), "PDF document".to_string()))
        );

        // Test ZIP detection
        let zip_header = &[0x50, 0x4B, 0x03, 0x04];
        let result = MagicNumbers::detect_type(zip_header);
        assert_eq!(
            result,
            Some((
                "zip".to_string(),
                "ZIP archive (may be Office doc)".to_string()
            ))
        );

        // Test text detection
        let text_header = b"This is plain text content";
        let result = MagicNumbers::detect_type(text_header);
        assert_eq!(result, Some(("txt".to_string(), "Plain text".to_string())));
    }

    #[test]
    fn test_pdf_validation() -> Result<()> {
        // Create a simple valid PDF-like file
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"%PDF-1.4\nSome PDF content here\n%%EOF")?;

        let result = FileProcessor::check_corruption(temp_file.path())?;
        assert!(!result.is_corrupted);
        assert_eq!(result.detected_type, Some("pdf".to_string()));

        Ok(())
    }

    #[test]
    fn test_corrupted_pdf_detection() -> Result<()> {
        // Create an invalid PDF file
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"NOT A PDF FILE")?;

        // Rename to have .pdf extension
        let pdf_path = temp_file.path().with_extension("pdf");
        std::fs::copy(temp_file.path(), &pdf_path)?;

        let result = FileProcessor::check_corruption(&pdf_path)?;
        assert!(result.type_mismatch || !result.corruption_details.is_empty());

        // Cleanup
        std::fs::remove_file(pdf_path)?;

        Ok(())
    }

    #[test]
    fn test_empty_file_detection() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        // File is empty by default

        let result = FileProcessor::validate_file(temp_file.path())?;
        assert!(!result.is_valid);
        assert!(matches!(result.status, ValidationStatus::Invalid));

        Ok(())
    }

    #[test]
    fn test_text_file_validation() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"This is a normal text file with regular content.")?;

        let result = FileProcessor::validate_file(temp_file.path())?;
        assert!(result.is_valid);
        assert!(matches!(result.status, ValidationStatus::Valid));

        Ok(())
    }
}
