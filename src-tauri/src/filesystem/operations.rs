use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use std::path::Path;

#[allow(dead_code)]
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let mut config = SecurityConfig::default();
    config.allowed_extensions.insert(".txt".to_string());

    let allowed_paths = vec![
        dirs::desktop_dir().unwrap_or_default(),
        dirs::document_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
        std::env::temp_dir(), // Ensure temp dir is always included
    ];

    let validator = PathValidator::new(config, allowed_paths);
    let validated = validator.validate_import_path(Path::new(path))?;
    Ok(validated.to_string_lossy().to_string())
}
