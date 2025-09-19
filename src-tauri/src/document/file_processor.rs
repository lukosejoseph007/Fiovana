// src-tauri/src/document/file_processor.rs
// File processing with corruption detection using header validation

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

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
