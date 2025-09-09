// src-tauri/src/filesystem/operations.rs
// Enhanced operations with production-safe security integration

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::{SecurityConfig, SecurityConfigError};
use mime_guess::from_path;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Operations manager with enhanced security and validation
pub struct SecureOperationsManager {
    config: Arc<SecurityConfig>,
    validator: Arc<PathValidator>,
    operation_counter: Arc<Mutex<HashMap<String, u32>>>,
}

impl SecureOperationsManager {
    /// Create a new operations manager with production-safe defaults
    pub fn new() -> Result<Self, SecurityConfigError> {
        let config = Self::load_production_config()?;
        let validator = PathValidator::new(config.clone(), config.allowed_workspace_paths.clone());

        Ok(Self {
            config: Arc::new(config),
            validator: Arc::new(validator),
            operation_counter: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create operations manager from custom configuration
    pub fn from_config(config: SecurityConfig) -> Result<Self, SecurityConfigError> {
        // Validate the configuration before using it
        config.validate()?;

        let validator = PathValidator::new(config.clone(), config.allowed_workspace_paths.clone());

        Ok(Self {
            config: Arc::new(config),
            validator: Arc::new(validator),
            operation_counter: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Load production-safe configuration with environment overrides
    fn load_production_config() -> Result<SecurityConfig, SecurityConfigError> {
        // Start with production defaults or environment-specified level
        let mut config = SecurityConfig::default();

        // Apply any environment overrides
        config.apply_environment_overrides()?;

        // Validate the final configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate file for import with comprehensive security checks
    pub fn validate_file_for_import(&self, path: &str) -> Result<String, SecurityError> {
        // Check operation limits
        self.check_operation_limits("file_import")?;

        // Validate the path using the path validator
        let validated_path = self.validator.validate_import_path(Path::new(path))?;

        // Additional security checks based on configuration level
        self.perform_additional_security_checks(&validated_path)?;

        // Extension validation
        let file_extension = Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let extension_with_dot = format!(".{}", file_extension);
        if !self.config.allowed_extensions.contains(&extension_with_dot) {
            return Err(SecurityError::InvalidExtension {
                extension: extension_with_dot,
            });
        }

        // MIME type validation
        let mime_type = from_path(path)
            .first_or_octet_stream()
            .essence_str()
            .to_string();
        if !self.config.allowed_mime_types.contains(&mime_type) {
            return Err(SecurityError::MimeTypeViolation(mime_type));
        }

        // Audit logging for production environments
        if self.config.audit_logging_enabled {
            self.log_security_event("file_import_validated", path, "success");
        }

        // Increment operation counter
        self.increment_operation_counter("file_import");

        Ok(validated_path.to_string_lossy().to_string())
    }

    /// Check if operation limits are exceeded
    fn check_operation_limits(&self, _operation_type: &str) -> Result<(), SecurityError> {
        if let Ok(counter) = self.operation_counter.lock() {
            let current_operations: u32 = counter.values().sum();
            if current_operations >= self.config.max_concurrent_operations {
                return Err(SecurityError::OperationLimitExceeded {
                    current: current_operations,
                    max: self.config.max_concurrent_operations,
                });
            }
        }
        Ok(())
    }

    /// Increment operation counter
    fn increment_operation_counter(&self, operation_type: &str) {
        if let Ok(mut counter) = self.operation_counter.lock() {
            *counter.entry(operation_type.to_string()).or_insert(0) += 1;
        }
    }

    /// Decrement operation counter (call when operation completes)
    pub fn decrement_operation_counter(&self, operation_type: &str) {
        if let Ok(mut counter) = self.operation_counter.lock() {
            if let Some(count) = counter.get_mut(operation_type) {
                if *count > 0 {
                    *count -= 1;
                }
                if *count == 0 {
                    counter.remove(operation_type);
                }
            }
        }
    }

    /// Perform additional security checks based on configuration level
    fn perform_additional_security_checks(&self, path: &Path) -> Result<(), SecurityError> {
        match self.config.security_level {
            crate::filesystem::security::security_config::SecurityLevel::HighSecurity => {
                self.perform_high_security_checks(path)?;
            }
            crate::filesystem::security::security_config::SecurityLevel::Production => {
                self.perform_production_security_checks(path)?;
            }
            _ => {
                // Development mode - minimal additional checks
            }
        }
        Ok(())
    }

    /// High-security mode additional checks
    fn perform_high_security_checks(&self, path: &Path) -> Result<(), SecurityError> {
        // Check if file has been recently modified (potential indicator of tampering)
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let now = std::time::SystemTime::now();
                if let Ok(duration) = now.duration_since(modified) {
                    // Flag files modified in the last 5 minutes as suspicious in high-security mode
                    if duration.as_secs() < 300 {
                        if self.config.audit_logging_enabled {
                            self.log_security_event(
                                "recently_modified_file",
                                &path.to_string_lossy(),
                                "warning",
                            );
                        }
                    }
                }
            }
        }

        // Additional file content scanning
        self.perform_enhanced_content_validation(path)?;

        Ok(())
    }

    /// Production mode security checks
    fn perform_production_security_checks(&self, path: &Path) -> Result<(), SecurityError> {
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Check common temp directories
        let temp_dirs = [std::env::temp_dir(), Path::new("/tmp").to_path_buf()];

        for temp_dir in &temp_dirs {
            if canonical_path.starts_with(temp_dir) && temp_dir.exists() {
                if self.config.audit_logging_enabled {
                    self.log_security_event(
                        "temp_directory_access",
                        &path.to_string_lossy(),
                        "warning",
                    );
                }
                // Allow but log the access
                break;
            }
        }

        // Check Windows temp directory separately if on Windows
        #[cfg(target_os = "windows")]
        {
            let windows_temp = Path::new("C:\\Windows\\Temp");
            if canonical_path.starts_with(windows_temp) && windows_temp.exists() {
                if self.config.audit_logging_enabled {
                    self.log_security_event(
                        "temp_directory_access",
                        &path.to_string_lossy(),
                        "warning",
                    );
                }
            }
        }

        Ok(())
    }

    /// Enhanced content validation for high-security environments
    fn perform_enhanced_content_validation(&self, path: &Path) -> Result<(), SecurityError> {
        if path.exists() {
            // Check for embedded executables in document files
            if let Some(extension) = path.extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                if matches!(ext_str.as_str(), "docx" | "xlsx" | "pptx" | "pdf") {
                    self.scan_for_embedded_content(path)?;
                }
            }
        }
        Ok(())
    }

    /// Scan for potentially dangerous embedded content
    fn scan_for_embedded_content(&self, path: &Path) -> Result<(), SecurityError> {
        // Read first 1KB of file to check for suspicious patterns
        if let Ok(mut file) = std::fs::File::open(path) {
            use std::io::Read;
            let mut buffer = vec![0; 1024];
            if let Ok(bytes_read) = file.read(&mut buffer) {
                buffer.truncate(bytes_read);

                // Check for executable signatures - use Vec<Vec<u8>> for different sized patterns
                let suspicious_patterns = vec![
                    b"MZ".to_vec(),               // PE executable signature
                    b"\x7fELF".to_vec(),          // ELF executable signature
                    vec![0xcf, 0xfa, 0xed, 0xfe], // Mach-O signature
                ];

                for pattern in &suspicious_patterns {
                    if buffer
                        .windows(pattern.len())
                        .any(|window| window == pattern.as_slice())
                    {
                        if self.config.audit_logging_enabled {
                            self.log_security_event(
                                "suspicious_embedded_content",
                                &path.to_string_lossy(),
                                "error",
                            );
                        }
                        return Err(SecurityError::SuspiciousContent {
                            path: path.to_string_lossy().to_string(),
                            reason: "Potential embedded executable detected".to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Log security events for audit purposes
    fn log_security_event(&self, event_type: &str, path: &str, level: &str) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        eprintln!(
            "[{}] SECURITY_AUDIT: {} - {} - {}",
            timestamp,
            level.to_uppercase(),
            event_type,
            path
        );

        // In a real implementation, this would write to a proper audit log file
        // or send to a logging service
    }

    /// Get current security configuration (read-only)
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Get current operation statistics
    pub fn get_operation_stats(&self) -> HashMap<String, u32> {
        self.operation_counter
            .lock()
            .unwrap_or_else(|_| panic!("Failed to acquire operation counter lock"))
            .clone()
    }

    /// Validate configuration at runtime
    pub fn validate_runtime_config(&self) -> Result<(), SecurityConfigError> {
        self.config.validate()
    }
}

/// Legacy compatibility function - now uses SecureOperationsManager
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let manager = SecureOperationsManager::new()
        .map_err(|e| SecurityError::ConfigValidationFailed(e.to_string()))?;

    let result = manager.validate_file_for_import(path);

    // Clean up operation counter
    manager.decrement_operation_counter("file_import");

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_secure_operations_manager_creation() {
        let manager = SecureOperationsManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_operation_limits() {
        let mut config = SecurityConfig::default();
        config.max_concurrent_operations = 1;

        let manager = SecureOperationsManager::from_config(config).unwrap();

        // Create a temporary file for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // First operation should succeed
        assert!(manager
            .validate_file_for_import(&test_file.to_string_lossy())
            .is_ok());

        // Second operation should fail due to limit
        let result = manager.validate_file_for_import(&test_file.to_string_lossy());
        assert!(result.is_err());
    }

    #[test]
    fn test_production_security_level() {
        std::env::set_var("PROXEMIC_SECURITY_LEVEL", "production");
        let manager = SecureOperationsManager::new().unwrap();

        match manager.config.security_level {
            crate::filesystem::security::security_config::SecurityLevel::Production => {
                assert!(manager.config.enable_magic_number_validation);
                assert!(manager.config.enforce_workspace_boundaries);
            }
            _ => panic!("Expected production security level"),
        }

        std::env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }
}
