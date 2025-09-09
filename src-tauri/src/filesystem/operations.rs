use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use mime_guess::from_path;
use std::path::Path; // Add `mime_guess` crate to Cargo.toml

#[allow(dead_code)]
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let mut config = SecurityConfig::default();

    // Add allowed extensions for development
    config.allowed_extensions.extend(vec![
        ".txt".to_string(),
        ".md".to_string(),
        ".pdf".to_string(),
        ".csv".to_string(),
        ".docx".to_string(),
        ".json".to_string(),
        ".zip".to_string(),
        ".tar.gz".to_string(),
    ]);

    let allowed_paths = vec![
        dirs::desktop_dir().unwrap_or_default(),
        dirs::document_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
        std::env::temp_dir(), // Ensure temp dir is always included
    ];

    // Validate the path first
    let validator = PathValidator::new(config.clone(), allowed_paths);
    let validated_path = validator.validate_import_path(Path::new(path))?;

    // File name extraction
    let file_name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| SecurityError::ProhibitedCharacters {
            filename: path.to_string(),
        })?;

    // --- Option 2: Validate extension first ---
    let file_extension = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !config
        .allowed_extensions
        .contains(&format!(".{}", file_extension))
    {
        return Err(SecurityError::FileTypeViolation(file_name.to_string()));
    }

    // --- Validate MIME type ---
    let mime_type = from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    if !config.allowed_mime_types.contains(&mime_type) {
        return Err(SecurityError::MimeTypeViolation(mime_type));
    }

    Ok(validated_path.to_string_lossy().to_string())
}
