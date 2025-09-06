use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::filesystem::errors::ValidationError;

#[allow(dead_code)]
pub struct MagicNumberValidator {
    signatures: HashMap<String, Vec<u8>>,
}

#[allow(dead_code)]
impl MagicNumberValidator {
    pub fn new() -> Self {
        let mut signatures = HashMap::new();

        // PDF signature
        signatures.insert("pdf".to_string(), vec![0x25, 0x50, 0x44, 0x46]); // "%PDF"

        // DOCX signature (ZIP format)
        signatures.insert("docx".to_string(), vec![0x50, 0x4B, 0x03, 0x04]); // "PK.."

        Self { signatures }
    }

    pub fn validate_file_type(
        &self,
        path: &Path,
        expected_ext: &str,
    ) -> Result<(), ValidationError> {
        let mut file = File::open(path).map_err(|_| ValidationError::FileType {
            reason: "Unable to open file".into(),
        })?;

        let mut buffer = [0u8; 8];
        let bytes_read = file.read(&mut buffer).unwrap_or(0);

        if let Some(signature) = self.signatures.get(&expected_ext.to_lowercase()) {
            if bytes_read < signature.len() {
                return Err(ValidationError::FileType {
                    reason: "File too short for magic number check".into(),
                });
            }

            if buffer[..signature.len()] != signature[..] {
                return Err(ValidationError::MagicNumber {
                    expected: format!("{:?}", signature),
                    actual: format!("{:?}", &buffer[..signature.len()]),
                });
            }
        }

        Ok(())
    }
}

// 👇 Must be outside the above impl
impl Default for MagicNumberValidator {
    fn default() -> Self {
        Self::new()
    }
}
