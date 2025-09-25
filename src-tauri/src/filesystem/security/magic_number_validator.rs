use infer::Infer;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::filesystem::errors::ValidationError;
use crate::filesystem::security::security_config::SecurityConfig;

#[allow(dead_code)]
pub struct MagicNumberValidator {
    signatures: HashMap<String, Vec<Vec<u8>>>,
    allowed_mime_types: HashSet<String>,
    max_file_size: u64,
}

#[allow(dead_code)]
impl MagicNumberValidator {
    pub fn new(config: &SecurityConfig) -> Self {
        Self {
            signatures: config.magic_number_map.clone(),
            allowed_mime_types: config.allowed_mime_types.clone(),
            max_file_size: config.max_file_size,
        }
    }

    pub fn validate_file_size(&self, path: &Path) -> Result<(), ValidationError> {
        let metadata = std::fs::metadata(path).map_err(|_| ValidationError::FileType {
            reason: "Unable to read file metadata".into(),
        })?;

        if metadata.len() > self.max_file_size {
            return Err(ValidationError::FileSize {
                size: metadata.len(),
                max: self.max_file_size,
            });
        }
        Ok(())
    }

    pub fn validate_mime_type(&self, path: &Path) -> Result<(), ValidationError> {
        let mut file = File::open(path).map_err(|_| ValidationError::FileType {
            reason: "Unable to open file".into(),
        })?;

        let mut buffer = [0; 256];
        let _ = file.read(&mut buffer).unwrap_or(0);

        let mut mime = Infer::new()
            .get(&buffer)
            .map(|info| info.mime_type())
            .unwrap_or("application/octet-stream");

        // Handle Office documents that are detected as ZIP files
        if mime == "application/zip" {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                match extension.to_lowercase().as_str() {
                    "docx" => mime =
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                    "xlsx" => {
                        mime = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    }
                    "pptx" => mime =
                        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                    _ => {} // Keep as application/zip
                }
            }
        }

        if !self.allowed_mime_types.contains(mime) {
            return Err(ValidationError::MimeType {
                mime: mime.to_string(),
            });
        }
        Ok(())
    }

    pub fn validate_file_type(
        &self,
        path: &Path,
        expected_ext: &str,
    ) -> Result<(), ValidationError> {
        let expected_ext = expected_ext.to_lowercase();

        // Handle text files with multiple possible magic number patterns
        if ["md", "csv", "log"].contains(&&*expected_ext) {
            return Ok(());
        }

        let mut file = File::open(path).map_err(|_| ValidationError::FileType {
            reason: "Unable to open file".into(),
        })?;

        let mut buffer = [0u8; 8];
        let bytes_read = file.read(&mut buffer).unwrap_or(0);

        if let Some(signatures) = self.signatures.get(&expected_ext) {
            let mut matched = false;
            for signature in signatures {
                if bytes_read >= signature.len() && buffer[..signature.len()] == signature[..] {
                    matched = true;
                    break;
                }
            }

            if !matched {
                return Err(ValidationError::MagicNumber {
                    expected: format!("Any of: {:?}", signatures),
                    actual: format!(
                        "{:?}",
                        &buffer[..signatures.iter().map(|s| s.len()).max().unwrap_or(0)]
                    ),
                });
            }
        }

        Ok(())
    }
}

// 👇 Must be outside the above impl
impl Default for MagicNumberValidator {
    fn default() -> Self {
        Self::new(&SecurityConfig::default())
    }
}
