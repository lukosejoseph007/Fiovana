// src-tauri/src/document/docx_parser.rs
// DOCX document parser implementation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Parsed DOCX document content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxContent {
    /// Extracted plain text content
    pub text: String,
    /// Document structure information
    pub structure: DocxStructure,
    /// Metadata extracted from the document
    pub metadata: DocxMetadata,
}

/// Document structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxStructure {
    /// Paragraph count
    pub paragraph_count: usize,
    /// Heading hierarchy
    pub headings: Vec<Heading>,
    /// Tables found in the document
    pub tables: Vec<TableInfo>,
}

/// Heading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    /// Heading level (1-6)
    pub level: u8,
    /// Heading text
    pub text: String,
    /// Position in document
    pub position: usize,
}

/// Table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// Number of rows
    pub rows: usize,
    /// Number of columns
    pub columns: usize,
    /// Position in document
    pub position: usize,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocxMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document creator
    pub creator: Option<String>,
    /// Creation date
    pub created: Option<String>,
    /// Last modified date
    pub modified: Option<String>,
}

/// DOCX document parser
#[allow(dead_code)]
pub struct DocxParser;

#[allow(dead_code)]
impl DocxParser {
    /// Parse a DOCX file and extract content
    pub fn parse<P: AsRef<Path>>(file_path: P) -> Result<DocxContent> {
        let file = std::fs::File::open(&file_path)
            .with_context(|| format!("Failed to open DOCX file: {:?}", file_path.as_ref()))?;

        let mut archive =
            ZipArchive::new(file).context("Failed to read DOCX file as ZIP archive")?;

        // Extract main document content
        let text = Self::extract_document_text(&mut archive)?;

        // Extract metadata
        let metadata = Self::extract_metadata(&mut archive)?;

        // Analyze structure
        let structure = Self::analyze_structure(&text);

        Ok(DocxContent {
            text,
            structure,
            metadata,
        })
    }

    /// Extract main document text from document.xml
    fn extract_document_text(archive: &mut ZipArchive<std::fs::File>) -> Result<String> {
        // Read document.xml file
        let mut document_file = archive
            .by_name("word/document.xml")
            .context("Failed to find document.xml in DOCX archive")?;

        let mut content = String::new();
        document_file
            .read_to_string(&mut content)
            .context("Failed to read document.xml content")?;

        // Parse XML and extract text content
        Self::extract_text_from_xml(&content)
    }

    /// Extract text content from XML
    fn extract_text_from_xml(xml_content: &str) -> Result<String> {
        use std::io::Cursor;
        use xml::reader::{EventReader, XmlEvent};

        let cursor = Cursor::new(xml_content);
        let parser = EventReader::new(cursor);
        let mut text_content = String::new();
        let mut in_text_element = false;

        for event in parser {
            match event.context("Failed to parse XML")? {
                XmlEvent::StartElement { name, .. } => {
                    if name.local_name == "t" {
                        in_text_element = true;
                    }
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "t" {
                        in_text_element = false;
                    } else if name.local_name == "p" {
                        // Add paragraph break
                        text_content.push('\n');
                    }
                }
                XmlEvent::Characters(text) => {
                    if in_text_element {
                        text_content.push_str(&text);
                    }
                }
                XmlEvent::Whitespace(ws) => {
                    if in_text_element {
                        text_content.push_str(&ws);
                    }
                }
                _ => {}
            }
        }

        Ok(text_content.trim().to_string())
    }

    /// Extract document metadata
    fn extract_metadata(archive: &mut ZipArchive<std::fs::File>) -> Result<DocxMetadata> {
        // Try to read core.xml for metadata
        let metadata = if let Ok(mut core_file) = archive.by_name("docProps/core.xml") {
            let mut content = String::new();
            if core_file.read_to_string(&mut content).is_ok() {
                Self::parse_core_metadata(&content).unwrap_or_default()
            } else {
                DocxMetadata::default()
            }
        } else {
            DocxMetadata::default()
        };

        Ok(metadata)
    }

    /// Parse core metadata from core.xml
    fn parse_core_metadata(xml_content: &str) -> Result<DocxMetadata> {
        use std::io::Cursor;
        use xml::reader::{EventReader, XmlEvent};

        let cursor = Cursor::new(xml_content);
        let parser = EventReader::new(cursor);
        let mut metadata = DocxMetadata::default();
        let mut current_element = String::new();

        for event in parser {
            match event.context("Failed to parse core metadata XML")? {
                XmlEvent::StartElement { name, .. } => {
                    current_element = name.local_name.clone();
                }
                XmlEvent::Characters(text) => match current_element.as_str() {
                    "title" => metadata.title = Some(text),
                    "creator" => metadata.creator = Some(text),
                    "created" => metadata.created = Some(text),
                    "modified" => metadata.modified = Some(text),
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(metadata)
    }

    /// Analyze document structure
    fn analyze_structure(text: &str) -> DocxStructure {
        let lines: Vec<&str> = text.lines().collect();
        let paragraph_count = lines.len();

        // Basic heading detection (could be improved with actual style analysis)
        let mut headings = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if Self::is_likely_heading(line) {
                headings.push(Heading {
                    level: Self::detect_heading_level(line),
                    text: line.trim().to_string(),
                    position: index,
                });
            }
        }

        DocxStructure {
            paragraph_count,
            headings,
            tables: Vec::new(), // TODO: Implement table detection
        }
    }

    /// Simple heuristic to detect if a line is likely a heading
    fn is_likely_heading(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }

        // Heuristics for heading detection
        // - Short lines (less than 100 characters)
        // - Doesn't end with period
        // - May start with numbers (1. 2. etc.)
        trimmed.len() < 100
            && !trimmed.ends_with('.')
            && (trimmed.chars().next().unwrap_or(' ').is_uppercase()
                || trimmed.starts_with(char::is_numeric))
    }

    /// Detect heading level based on content
    fn detect_heading_level(line: &str) -> u8 {
        let trimmed = line.trim();

        // Simple heuristic based on numbering - check most specific first
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
        assert!(DocxParser::is_likely_heading("Introduction"));
        assert!(DocxParser::is_likely_heading("1. Overview"));
        assert!(DocxParser::is_likely_heading("Chapter 1: Getting Started"));

        assert!(!DocxParser::is_likely_heading(
            "This is a long sentence that ends with a period."
        ));
        assert!(!DocxParser::is_likely_heading(""));
    }

    #[test]
    fn test_heading_level_detection() {
        assert_eq!(DocxParser::detect_heading_level("1. Introduction"), 1);
        assert_eq!(DocxParser::detect_heading_level("1.1 Getting Started"), 2); // Now correctly returns 2
        assert_eq!(DocxParser::detect_heading_level("1.1.1 Installation"), 3);
        assert_eq!(DocxParser::detect_heading_level("Overview"), 1);
    }

    #[test]
    fn test_xml_text_extraction() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:body>
                <w:p>
                    <w:r>
                        <w:t>Hello World</w:t>
                    </w:r>
                </w:p>
                <w:p>
                    <w:r>
                        <w:t>Second paragraph</w:t>
                    </w:r>
                </w:p>
            </w:body>
        </w:document>"#;

        let result = DocxParser::extract_text_from_xml(xml).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("Second paragraph"));
    }
}
