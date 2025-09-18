// src-tauri/src/filesystem/mod.rs

pub mod errors;
pub mod event_persistence;
pub mod event_processor;
pub mod operations;
pub mod security;
pub mod watcher;

pub use errors::SecurityError;
pub use security::path_validator::PathValidator;
pub use security::security_config::SecurityConfig;

#[allow(dead_code)]
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
