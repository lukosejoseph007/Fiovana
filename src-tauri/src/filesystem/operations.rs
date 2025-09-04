use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::path_validator::PathValidator;
use std::path::Path;

// Exposed to the outside world
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let validated = PathValidator::validate(Path::new(path))?;
    Ok(validated.to_string_lossy().to_string())
}
