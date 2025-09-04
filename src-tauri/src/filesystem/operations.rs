use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;
use std::path::Path;

pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    // Use default config for now (later you can load from settings)
    let config = SecurityConfig::default();

    let validated = PathValidator::validate(Path::new(path), &config)?;
    Ok(validated.to_string_lossy().to_string())
}
