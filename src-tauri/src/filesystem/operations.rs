use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;
use std::path::Path;

#[allow(dead_code)]
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let config = SecurityConfig::default();
    let validator = PathValidator::new(config);
    let validated = validator.validate_import_path(Path::new(path))?;
    Ok(validated.to_string_lossy().to_string())
}
