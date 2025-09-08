pub mod errors;
pub mod operations;
pub mod security;

pub use errors::SecurityError;
pub use security::config::SecurityConfig;
pub use security::path_validator::PathValidator;

pub fn init_security_subsystem(config: SecurityConfig) -> Result<PathValidator, SecurityError> {
    let allowed_paths = vec![
        dirs::desktop_dir().unwrap_or_default(),
        dirs::document_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
    ];

    let validator = PathValidator::new(config, allowed_paths);

    // Stub for validate_system_requirements if not yet implemented
    // validator.validate_system_requirements()?;

    tracing::info!("Filesystem security subsystem initialized");
    Ok(validator)
}
