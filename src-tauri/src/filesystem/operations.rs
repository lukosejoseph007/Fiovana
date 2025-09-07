use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;
use std::path::Path;

#[allow(dead_code)]
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let config = SecurityConfig::default();
    let allowed_paths = vec![
        dirs::desktop_dir().unwrap(),
        dirs::document_dir().unwrap(),
        dirs::download_dir().unwrap(),
    ];

    let validator = PathValidator::new(config, allowed_paths);
    let validated = validator.validate_import_path(Path::new(path))?;
    Ok(validated.to_string_lossy().to_string())
}
