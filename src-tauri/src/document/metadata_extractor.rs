// src-tauri/src/document/metadata_extractor.rs
// Deep file metadata extraction beyond basic properties

use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

/// Enhanced file metadata with deep analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMetadata {
    /// Basic file properties
    pub basic: BasicMetadata,
    /// Content analysis
    pub content: ContentMetadata,
    /// Security and permissions
    pub security: SecurityMetadata,
    /// Technical properties
    pub technical: TechnicalMetadata,
    /// Document-specific metadata (if applicable)
    pub document: Option<DocumentMetadata>,
}

/// Basic file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicMetadata {
    pub file_name: String,
    pub file_extension: Option<String>,
    pub file_size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
}

/// Content analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    /// MIME type detected from content
    pub detected_mime_type: Option<String>,
    /// Character encoding (for text files)
    pub encoding: Option<String>,
    /// Line ending style (for text files)
    pub line_endings: Option<String>,
    /// Content preview (first few characters)
    pub preview: Option<String>,
    /// Language detection (for text content)
    pub language: Option<String>,
    /// Content statistics
    pub stats: ContentStats,
}

/// Content statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStats {
    /// Total character count (for text)
    pub char_count: Option<usize>,
    /// Word count (for text)
    pub word_count: Option<usize>,
    /// Line count (for text)
    pub line_count: Option<usize>,
    /// Paragraph count (for text)
    pub paragraph_count: Option<usize>,
    /// Binary data ratio (0.0 to 1.0)
    pub binary_ratio: f64,
}

/// Security-related metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetadata {
    /// File permissions (Unix-style)
    pub permissions: Option<u32>,
    /// Whether file is executable
    pub is_executable: bool,
    /// Whether file is hidden
    pub is_hidden: bool,
    /// File has extended attributes
    pub has_extended_attributes: bool,
    /// Security scan results
    pub security_flags: Vec<String>,
}

/// Technical file properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalMetadata {
    /// File entropy (randomness measure)
    pub entropy: f64,
    /// Compression ratio estimate
    pub compression_ratio: Option<f64>,
    /// Hash checksums
    pub checksums: HashMap<String, String>,
    /// File structure analysis
    pub structure: FileStructure,
}

/// File structure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStructure {
    /// Whether file has recognizable structure
    pub has_structure: bool,
    /// Detected file format version
    pub format_version: Option<String>,
    /// Embedded resources count
    pub embedded_resources: usize,
    /// File sections/chunks
    pub sections: Vec<FileSection>,
}

/// File section information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSection {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub section_type: String,
}

/// Document-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document keywords
    pub keywords: Vec<String>,
    /// Creation application
    pub creator: Option<String>,
    /// Producer/converter
    pub producer: Option<String>,
    /// Document creation date
    pub creation_date: Option<SystemTime>,
    /// Document modification date
    pub modification_date: Option<SystemTime>,
    /// Page/slide count
    pub page_count: Option<usize>,
    /// Document language
    pub document_language: Option<String>,
    /// Document format specific properties
    pub format_properties: HashMap<String, String>,
}

/// Metadata extractor
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract enhanced metadata from file
    pub fn extract<P: AsRef<Path>>(path: P) -> Result<EnhancedMetadata> {
        let path = path.as_ref();

        // Extract basic metadata
        let basic = Self::extract_basic_metadata(path)?;

        // Extract content metadata
        let content = Self::extract_content_metadata(path)?;

        // Extract security metadata
        let security = Self::extract_security_metadata(path, &basic)?;

        // Extract technical metadata
        let technical = Self::extract_technical_metadata(path)?;

        // Extract document metadata (if applicable)
        let document = Self::extract_document_metadata(path).ok();

        Ok(EnhancedMetadata {
            basic,
            content,
            security,
            technical,
            document,
        })
    }

    /// Extract basic file metadata
    fn extract_basic_metadata(path: &Path) -> Result<BasicMetadata> {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        Ok(BasicMetadata {
            file_name,
            file_extension,
            file_size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            is_symlink: metadata.is_symlink(),
        })
    }

    /// Extract content metadata
    fn extract_content_metadata(path: &Path) -> Result<ContentMetadata> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;

        // Read sample of file content for analysis
        let mut buffer = vec![0u8; 8192]; // 8KB sample
        let bytes_read = file.read(&mut buffer)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        buffer.truncate(bytes_read);

        // Detect MIME type from content
        let detected_mime_type = Self::detect_mime_type(&buffer);

        // Analyze content if it appears to be text
        let (encoding, line_endings, preview, language, stats) = if Self::is_text_content(&buffer) {
            let encoding = Self::detect_encoding(&buffer);
            let text_content = String::from_utf8_lossy(&buffer);
            let line_endings = Self::detect_line_endings(&text_content);
            let preview = Some(Self::create_preview(&text_content));
            let language = Self::detect_language(&text_content);
            let stats = Self::analyze_text_content(&text_content);
            (encoding, line_endings, preview, language, stats)
        } else {
            let stats = ContentStats {
                char_count: None,
                word_count: None,
                line_count: None,
                paragraph_count: None,
                binary_ratio: Self::calculate_binary_ratio(&buffer),
            };
            (None, None, None, None, stats)
        };

        Ok(ContentMetadata {
            detected_mime_type,
            encoding,
            line_endings,
            preview,
            language,
            stats,
        })
    }

    /// Extract security metadata
    fn extract_security_metadata(path: &Path, basic: &BasicMetadata) -> Result<SecurityMetadata> {
        #[cfg(unix)]
        let permissions = {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(path)
                .ok()
                .map(|m| m.permissions().mode())
        };

        #[cfg(not(unix))]
        let permissions = None;

        let is_executable = Self::is_executable(path);
        let is_hidden = Self::is_hidden(path);
        let has_extended_attributes = Self::has_extended_attributes(path);
        let security_flags = Self::analyze_security_flags(path, basic);

        Ok(SecurityMetadata {
            permissions,
            is_executable,
            is_hidden,
            has_extended_attributes,
            security_flags,
        })
    }

    /// Extract technical metadata
    fn extract_technical_metadata(path: &Path) -> Result<TechnicalMetadata> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();

        // For large files, only read first 1MB for analysis
        let file_size = file.metadata()?.len();
        let read_size = std::cmp::min(file_size, 1024 * 1024) as usize;
        buffer.resize(read_size, 0);
        file.read_exact(&mut buffer)?;

        let entropy = Self::calculate_entropy(&buffer);
        let compression_ratio = Self::estimate_compression_ratio(&buffer);
        let checksums = Self::calculate_checksums(&buffer);
        let structure = Self::analyze_file_structure(&buffer, path);

        Ok(TechnicalMetadata {
            entropy,
            compression_ratio,
            checksums,
            structure,
        })
    }

    /// Extract document-specific metadata
    fn extract_document_metadata(path: &Path) -> Result<DocumentMetadata> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("pdf") => Self::extract_pdf_metadata(path),
            Some("docx") | Some("xlsx") | Some("pptx") => Self::extract_office_metadata(path),
            Some("txt") | Some("md") => Self::extract_text_metadata(path),
            _ => {
                // Generic document metadata
                Ok(DocumentMetadata {
                    title: None,
                    author: None,
                    subject: None,
                    keywords: Vec::new(),
                    creator: None,
                    producer: None,
                    creation_date: None,
                    modification_date: None,
                    page_count: None,
                    document_language: None,
                    format_properties: HashMap::new(),
                })
            }
        }
    }

    // Helper methods

    fn detect_mime_type(buffer: &[u8]) -> Option<String> {
        // Basic MIME type detection based on file signatures
        if buffer.starts_with(b"%PDF") {
            Some("application/pdf".to_string())
        } else if buffer.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            Some("application/zip".to_string())
        } else if buffer.starts_with(&[0xFF, 0xD8, 0xFF]) {
            Some("image/jpeg".to_string())
        } else if buffer.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            Some("image/png".to_string())
        } else if Self::is_text_content(buffer) {
            Some("text/plain".to_string())
        } else {
            Some("application/octet-stream".to_string())
        }
    }

    fn is_text_content(buffer: &[u8]) -> bool {
        if buffer.is_empty() {
            return false;
        }

        let text_chars = buffer.iter()
            .take(1024)
            .filter(|&&b| (b >= 32 && b <= 126) || b == 9 || b == 10 || b == 13)
            .count();

        let sample_size = buffer.len().min(1024);
        text_chars as f64 / sample_size as f64 > 0.95
    }

    fn detect_encoding(buffer: &[u8]) -> Option<String> {
        if buffer.starts_with(&[0xEF, 0xBB, 0xBF]) {
            Some("UTF-8".to_string())
        } else if buffer.starts_with(&[0xFF, 0xFE]) {
            Some("UTF-16LE".to_string())
        } else if buffer.starts_with(&[0xFE, 0xFF]) {
            Some("UTF-16BE".to_string())
        } else if String::from_utf8(buffer.to_vec()).is_ok() {
            Some("UTF-8".to_string())
        } else {
            Some("Unknown".to_string())
        }
    }

    fn detect_line_endings(text: &str) -> Option<String> {
        let crlf_count = text.matches("\r\n").count();
        let lf_count = text.matches('\n').count() - crlf_count;
        let cr_count = text.matches('\r').count() - crlf_count;

        if crlf_count > lf_count && crlf_count > cr_count {
            Some("CRLF".to_string())
        } else if lf_count > cr_count {
            Some("LF".to_string())
        } else if cr_count > 0 {
            Some("CR".to_string())
        } else {
            None
        }
    }

    fn create_preview(text: &str) -> String {
        text.chars().take(200).collect::<String>()
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn detect_language(text: &str) -> Option<String> {
        // Simple language detection based on common words
        let text_lower = text.to_lowercase();

        if text_lower.contains("the ") && text_lower.contains("and ") {
            Some("English".to_string())
        } else {
            Some("Unknown".to_string())
        }
    }

    fn analyze_text_content(text: &str) -> ContentStats {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = text.lines().count();
        let paragraph_count = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();

        ContentStats {
            char_count: Some(char_count),
            word_count: Some(word_count),
            line_count: Some(line_count),
            paragraph_count: Some(paragraph_count),
            binary_ratio: 0.0,
        }
    }

    fn calculate_binary_ratio(buffer: &[u8]) -> f64 {
        if buffer.is_empty() {
            return 0.0;
        }

        let binary_bytes = buffer.iter()
            .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
            .count();

        binary_bytes as f64 / buffer.len() as f64
    }

    fn is_executable(path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(path)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }

        #[cfg(not(unix))]
        {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "com"))
                .unwrap_or(false)
        }
    }

    fn is_hidden(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    fn has_extended_attributes(_path: &Path) -> bool {
        // Simplified - in a real implementation, this would check for extended attributes
        false
    }

    fn analyze_security_flags(path: &Path, _basic: &BasicMetadata) -> Vec<String> {
        let mut flags = Vec::new();

        // Check for potentially dangerous extensions
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "exe" | "bat" | "cmd" | "scr" | "pif" => {
                    flags.push("executable_file".to_string());
                }
                "js" | "vbs" | "ps1" => {
                    flags.push("script_file".to_string());
                }
                _ => {}
            }
        }

        flags
    }

    fn calculate_entropy(buffer: &[u8]) -> f64 {
        if buffer.is_empty() {
            return 0.0;
        }

        let mut counts = [0u32; 256];
        for &byte in buffer {
            counts[byte as usize] += 1;
        }

        let len = buffer.len() as f64;
        let mut entropy = 0.0;

        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    fn estimate_compression_ratio(buffer: &[u8]) -> Option<f64> {
        // Simple compression ratio estimation using run-length encoding
        if buffer.is_empty() {
            return None;
        }

        let mut compressed_size = 0;
        let mut i = 0;

        while i < buffer.len() {
            let current = buffer[i];
            let mut count = 1;

            while i + count < buffer.len() && buffer[i + count] == current && count < 255 {
                count += 1;
            }

            compressed_size += if count > 1 { 2 } else { 1 }; // Run-length encoding
            i += count;
        }

        Some(buffer.len() as f64 / compressed_size as f64)
    }

    fn calculate_checksums(buffer: &[u8]) -> HashMap<String, String> {
        use sha2::{Sha256, Digest};

        let mut checksums = HashMap::new();

        // Calculate SHA-256
        let mut hasher = Sha256::new();
        hasher.update(buffer);
        let sha256 = format!("{:x}", hasher.finalize());
        checksums.insert("sha256".to_string(), sha256);

        checksums
    }

    fn analyze_file_structure(buffer: &[u8], path: &Path) -> FileStructure {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("pdf") => Self::analyze_pdf_structure(buffer),
            Some("zip") | Some("docx") | Some("xlsx") | Some("pptx") => Self::analyze_zip_structure(buffer),
            _ => FileStructure {
                has_structure: false,
                format_version: None,
                embedded_resources: 0,
                sections: Vec::new(),
            }
        }
    }

    fn analyze_pdf_structure(buffer: &[u8]) -> FileStructure {
        let mut sections = Vec::new();

        if buffer.starts_with(b"%PDF-") {
            if let Some(version_end) = buffer[5..].iter().position(|&b| b == b'\n' || b == b'\r') {
                let version = String::from_utf8_lossy(&buffer[5..5 + version_end]).to_string();

                sections.push(FileSection {
                    name: "Header".to_string(),
                    offset: 0,
                    size: (5 + version_end) as u64,
                    section_type: "header".to_string(),
                });

                return FileStructure {
                    has_structure: true,
                    format_version: Some(version),
                    embedded_resources: 0, // Would need proper PDF parsing
                    sections,
                };
            }
        }

        FileStructure {
            has_structure: false,
            format_version: None,
            embedded_resources: 0,
            sections: Vec::new(),
        }
    }

    fn analyze_zip_structure(buffer: &[u8]) -> FileStructure {
        if buffer.len() >= 4 && buffer.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            FileStructure {
                has_structure: true,
                format_version: Some("2.0".to_string()), // ZIP 2.0 format
                embedded_resources: 1, // Simplified - would need proper ZIP parsing
                sections: vec![
                    FileSection {
                        name: "Local File Header".to_string(),
                        offset: 0,
                        size: 30, // Minimum local file header size
                        section_type: "header".to_string(),
                    }
                ],
            }
        } else {
            FileStructure {
                has_structure: false,
                format_version: None,
                embedded_resources: 0,
                sections: Vec::new(),
            }
        }
    }

    fn extract_pdf_metadata(_path: &Path) -> Result<DocumentMetadata> {
        // Placeholder - would need proper PDF library
        Ok(DocumentMetadata {
            title: None,
            author: None,
            subject: None,
            keywords: Vec::new(),
            creator: Some("PDF Document".to_string()),
            producer: None,
            creation_date: None,
            modification_date: None,
            page_count: None,
            document_language: None,
            format_properties: HashMap::new(),
        })
    }

    fn extract_office_metadata(_path: &Path) -> Result<DocumentMetadata> {
        // Placeholder - would need proper Office document library
        Ok(DocumentMetadata {
            title: None,
            author: None,
            subject: None,
            keywords: Vec::new(),
            creator: Some("Microsoft Office".to_string()),
            producer: None,
            creation_date: None,
            modification_date: None,
            page_count: None,
            document_language: None,
            format_properties: HashMap::new(),
        })
    }

    fn extract_text_metadata(path: &Path) -> Result<DocumentMetadata> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Try to extract title from first line
        let title = content.lines().next()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string());

        Ok(DocumentMetadata {
            title,
            author: None,
            subject: None,
            keywords: Vec::new(),
            creator: Some("Text Editor".to_string()),
            producer: None,
            creation_date: None,
            modification_date: None,
            page_count: Some(content.lines().count()),
            document_language: None,
            format_properties: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_basic_metadata_extraction() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test content")?;

        let metadata = MetadataExtractor::extract(temp_file.path())?;

        assert!(metadata.basic.is_file);
        assert!(!metadata.basic.is_dir);
        assert_eq!(metadata.basic.file_size, 12);

        Ok(())
    }

    #[test]
    fn test_text_content_analysis() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Hello world\nThis is a test file\nWith multiple lines")?;

        let metadata = MetadataExtractor::extract(temp_file.path())?;

        assert_eq!(metadata.content.detected_mime_type, Some("text/plain".to_string()));
        assert!(metadata.content.stats.char_count.is_some());
        assert!(metadata.content.stats.word_count.is_some());
        assert!(metadata.content.stats.line_count.is_some());

        Ok(())
    }

    #[test]
    fn test_binary_ratio_calculation() {
        let text_buffer = b"This is plain text content";
        let binary_buffer = &[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE];

        assert!(MetadataExtractor::calculate_binary_ratio(text_buffer) < 0.1);
        assert!(MetadataExtractor::calculate_binary_ratio(binary_buffer) > 0.5);
    }

    #[test]
    fn test_entropy_calculation() {
        let uniform_buffer = vec![0x41; 1000]; // All 'A's
        let random_buffer: Vec<u8> = (0..256).cycle().take(1000).map(|x| x as u8).collect();

        let uniform_entropy = MetadataExtractor::calculate_entropy(&uniform_buffer);
        let random_entropy = MetadataExtractor::calculate_entropy(&random_buffer);

        assert!(uniform_entropy < 1.0); // Low entropy
        assert!(random_entropy > 7.0); // High entropy
    }
}