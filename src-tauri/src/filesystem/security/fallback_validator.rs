use anyhow::Result;
use infer;
use mime_guess::MimeGuess;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum ValidationResult {
    Approved(()),     // File type with confidence
    Fallback(()),     // Fallback validation used
    Rejected(String), // Validation failed
}

#[derive(Debug, Clone)]
pub struct FallbackValidator {
    allowed_extensions: Vec<String>,
    fallback_enabled: bool,
}

impl Default for FallbackValidator {
    fn default() -> Self {
        Self {
            allowed_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "csv".to_string(),
                "log".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "docx".to_string(),
                "pdf".to_string(),
                "xlsx".to_string(),
            ],
            fallback_enabled: true,
        }
    }
}

impl FallbackValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_file(&self, path: &Path) -> Result<ValidationResult> {
        // Primary validation: Magic number detection
        match self.validate_by_magic_number(path) {
            Ok(_file_type) => return Ok(ValidationResult::Approved(())),
            Err(e) => {
                log::warn!("Magic number validation failed for {:?}: {}", path, e);

                if !self.fallback_enabled {
                    return Ok(ValidationResult::Rejected(
                        "Magic number validation failed and fallbacks disabled".to_string(),
                    ));
                }
            }
        }

        // Fallback 1: Extension-based validation
        if let Ok(_result) = self.validate_by_extension(path) {
            return Ok(ValidationResult::Fallback(()));
        }

        // Fallback 2: MIME type guessing
        if let Ok(_result) = self.validate_by_mime_guess(path) {
            return Ok(ValidationResult::Fallback(()));
        }

        // Fallback 3: Content pattern matching for text files
        if let Ok(_result) = self.validate_by_content_patterns(path) {
            return Ok(ValidationResult::Fallback(()));
        }

        // Fallback 4: File size and header heuristics
        if let Ok(_result) = self.validate_by_heuristics(path) {
            return Ok(ValidationResult::Fallback(()));
        }

        // Final fallback: Safe mode for known text extensions
        if self.is_safe_text_extension(path) {
            return Ok(ValidationResult::Fallback(()));
        }

        Ok(ValidationResult::Rejected(
            "All validation methods failed".to_string(),
        ))
    }

    fn validate_by_magic_number(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path)?;

        if let Some(file_type) = infer::get(&bytes) {
            let mime_type = file_type.mime_type();

            // Check if detected type is in our allowed list
            if self.is_mime_allowed(mime_type) {
                return Ok(file_type.extension().to_string());
            } else {
                return Err(anyhow::anyhow!(
                    "Detected file type '{}' not allowed",
                    mime_type
                ));
            }
        }

        Err(anyhow::anyhow!("No magic number detected"))
    }

    fn validate_by_extension(&self, path: &Path) -> Result<String> {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = extension.to_lowercase();

            if self.allowed_extensions.contains(&ext_lower) {
                return Ok(ext_lower);
            }
        }

        Err(anyhow::anyhow!("Extension not in allowed list"))
    }

    fn validate_by_mime_guess(&self, path: &Path) -> Result<String> {
        let guess = MimeGuess::from_path(path);

        if let Some(mime_type) = guess.first() {
            let mime_str = mime_type.as_ref();

            if self.is_mime_allowed(mime_str) {
                return Ok(mime_str.to_string());
            }
        }

        Err(anyhow::anyhow!("MIME guess not allowed"))
    }

    fn validate_by_content_patterns(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)
            .map_err(|_| anyhow::anyhow!("File is not readable as text"))?;

        // Check for common text file patterns
        if content.starts_with("# ") || content.contains("## ") {
            return Ok("markdown".to_string());
        }

        if content.starts_with("{") && content.ends_with("}") {
            return Ok("json".to_string());
        }

        if content.contains("---\n") || content.starts_with("---\n") {
            return Ok("yaml".to_string());
        }

        // CSV pattern detection
        let lines: Vec<&str> = content.lines().take(5).collect();
        if lines.len() > 1 && lines.iter().all(|line| line.contains(',')) {
            return Ok("csv".to_string());
        }

        // Generic text file if it's readable and has reasonable content
        if content.chars().all(|c| c.is_ascii() || c.is_whitespace()) {
            return Ok("text".to_string());
        }

        Err(anyhow::anyhow!("Content patterns don't match known types"))
    }

    fn validate_by_heuristics(&self, path: &Path) -> Result<String> {
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        // Very small files might be empty or test files
        if file_size < 10 {
            return Err(anyhow::anyhow!("File too small for reliable detection"));
        }

        // Read first few bytes for header analysis
        let mut buffer = [0u8; 64];
        let file = fs::File::open(path)?;
        use std::io::Read;
        let bytes_read = std::io::BufReader::new(file).read(&mut buffer)?;

        // Check for ASCII text (heuristic)
        let ascii_ratio = buffer[..bytes_read]
            .iter()
            .filter(|&&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
            .count() as f32
            / bytes_read as f32;

        if ascii_ratio > 0.9 {
            return Ok("text".to_string());
        }

        // Check for UTF-8 BOM
        if buffer.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return Ok("utf8-text".to_string());
        }

        Err(anyhow::anyhow!("Heuristics inconclusive"))
    }

    fn is_safe_text_extension(&self, path: &Path) -> bool {
        let safe_text_extensions = ["txt", "md", "csv", "log", "json", "yaml", "yml"];

        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            return safe_text_extensions.contains(&extension.to_lowercase().as_str());
        }

        false
    }

    fn is_mime_allowed(&self, mime_type: &str) -> bool {
        let allowed_mimes = [
            "text/plain",
            "text/markdown",
            "text/csv",
            "application/json",
            "application/yaml",
            "text/yaml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/pdf",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ];

        allowed_mimes.contains(&mime_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_fallback_validation() {
        let validator = FallbackValidator::new();

        // Create a test file that will fail magic number detection
        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        write!(temp_file, "This is a simple text file").unwrap();

        let result = validator.validate_file(temp_file.path()).unwrap();

        match result {
            ValidationResult::Fallback(_) => {} // Expected fallback validation
            _ => panic!("Expected fallback validation"),
        }
    }
}
