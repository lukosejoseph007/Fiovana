use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SafeModeLevel {
    #[default]
    Disabled, // Normal operation
    Restricted, // Limited file types
    Paranoid,   // Only text files
    Emergency,  // No file operations
}

impl std::str::FromStr for SafeModeLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "disabled" => Ok(SafeModeLevel::Disabled),
            "restricted" => Ok(SafeModeLevel::Restricted),
            "paranoid" => Ok(SafeModeLevel::Paranoid),
            "emergency" => Ok(SafeModeLevel::Emergency),
            _ => Err(anyhow::anyhow!("Invalid safe mode level: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeModeConfig {
    pub level: SafeModeLevel,
    pub allowed_extensions: HashSet<String>,
    pub allowed_mime_types: HashSet<String>,
    pub max_file_size_mb: u64,
    pub require_magic_number: bool,
    pub disable_script_execution: bool,
    pub quarantine_suspicious_files: bool,
    pub auto_enable_on_threats: bool,
    pub allowed_directories: Vec<String>,
    pub blocked_directories: Vec<String>,
}

impl Default for SafeModeConfig {
    fn default() -> Self {
        Self {
            level: SafeModeLevel::Disabled,
            allowed_extensions: Self::default_allowed_extensions(),
            allowed_mime_types: Self::default_allowed_mime_types(),
            max_file_size_mb: 50,
            require_magic_number: false,
            disable_script_execution: true,
            quarantine_suspicious_files: true,
            auto_enable_on_threats: true,
            allowed_directories: vec![], // Empty means all allowed
            blocked_directories: Self::default_blocked_directories(),
        }
    }
}

impl SafeModeConfig {
    fn default_allowed_extensions() -> HashSet<String> {
        [
            "txt", "md", "csv", "log", "json", "yaml", "yml", "docx", "pdf", "xlsx", "pptx", "png",
            "jpg", "jpeg", "gif", "svg",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn default_allowed_mime_types() -> HashSet<String> {
        [
            "text/plain",
            "text/markdown",
            "text/csv",
            "application/json",
            "application/yaml",
            "text/yaml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/pdf",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "image/png",
            "image/jpeg",
            "image/gif",
            "image/svg+xml",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn default_blocked_directories() -> Vec<String> {
        vec![
            "System32".to_string(),
            "Windows".to_string(),
            "/bin".to_string(),
            "/sbin".to_string(),
            "/usr/bin".to_string(),
            "/System".to_string(),
            "/Applications".to_string(),
        ]
    }

    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Load from environment variables
        if let Ok(level_str) = std::env::var("PROXEMIC_SAFE_MODE_LEVEL") {
            config.level = level_str.parse()?;
        }

        if let Ok(max_size) = std::env::var("PROXEMIC_MAX_FILE_SIZE_MB") {
            config.max_file_size_mb = max_size.parse()?;
        }

        if let Ok(require_magic) = std::env::var("PROXEMIC_REQUIRE_MAGIC_NUMBER") {
            config.require_magic_number = require_magic.parse().unwrap_or(false);
        }

        if let Ok(auto_enable) = std::env::var("PROXEMIC_AUTO_ENABLE_SAFE_MODE") {
            config.auto_enable_on_threats = auto_enable.parse().unwrap_or(true);
        }

        // Load allowed extensions from env
        if let Ok(extensions) = std::env::var("PROXEMIC_ALLOWED_EXTENSIONS") {
            config.allowed_extensions = extensions
                .split(',')
                .map(str::trim)
                .map(str::to_lowercase)
                .collect();
        }

        // Load blocked directories from env
        if let Ok(blocked_dirs) = std::env::var("PROXEMIC_BLOCKED_DIRECTORIES") {
            config.blocked_directories = blocked_dirs
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        Ok(config)
    }
}

// Global safe mode manager
pub struct SafeModeManager {
    config: Arc<RwLock<SafeModeConfig>>,
}

static SAFE_MODE_MANAGER: Lazy<SafeModeManager> = Lazy::new(SafeModeManager::new);

impl Default for SafeModeManager {
    fn default() -> Self {
        let config = SafeModeConfig::from_env().unwrap_or_default();

        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }
}

impl SafeModeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn global() -> &'static SafeModeManager {
        &SAFE_MODE_MANAGER
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> SafeModeConfig {
        self.config.read().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn set_level(&self, level: SafeModeLevel) -> Result<()> {
        let mut config = self.config.write().unwrap();
        let level_clone = level.clone();
        config.level = level;

        log::info!("Safe mode level changed to: {:?}", level_clone);

        Ok(())
    }

    pub fn is_file_allowed(&self, path: &std::path::Path) -> Result<bool> {
        let config = self.config.read().unwrap();

        match config.level {
            SafeModeLevel::Emergency => return Ok(false),
            SafeModeLevel::Disabled => return Ok(true),
            _ => {}
        }

        // Check file extension
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = extension.to_lowercase();
            if !config.allowed_extensions.contains(&ext_lower) {
                log::warn!("File extension '{}' not allowed in safe mode", ext_lower);
                return Ok(false);
            }
        } else {
            log::warn!("File has no extension, blocked in safe mode");
            return Ok(false);
        }

        // Check file size
        if let Ok(metadata) = std::fs::metadata(path) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > config.max_file_size_mb {
                log::warn!(
                    "File size {}MB exceeds limit of {}MB",
                    size_mb,
                    config.max_file_size_mb
                );
                return Ok(false);
            }
        }

        // Check directory restrictions
        if self.is_path_blocked(path)? {
            return Ok(false);
        }

        Ok(true)
    }

    fn is_path_blocked(&self, path: &std::path::Path) -> Result<bool> {
        let config = self.config.read().unwrap();
        let path_str = path.to_string_lossy();

        // Check blocked directories
        for blocked_dir in &config.blocked_directories {
            if blocked_dir == "*" || path_str.contains(blocked_dir) {
                log::warn!("Path blocked by safe mode: {}", path_str);
                return Ok(true);
            }
        }

        // If allowed directories are specified, path must be in one of them
        if !config.allowed_directories.is_empty() {
            let mut allowed = false;
            for allowed_dir in &config.allowed_directories {
                if path_str.starts_with(allowed_dir) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                log::warn!("Path not in allowed directories: {}", path_str);
                return Ok(true);
            }
        }

        Ok(false)
    }
}

// Tauri command for runtime safe mode control
#[tauri::command]
#[allow(dead_code)]
pub async fn set_safe_mode_level(level: String) -> Result<String, String> {
    let safe_mode_level: SafeModeLevel = level
        .parse()
        .map_err(|e| format!("Invalid safe mode level: {}", e))?;

    SafeModeManager::global()
        .set_level(safe_mode_level)
        .map_err(|e| format!("Failed to set safe mode: {}", e))?;

    Ok(format!("Safe mode set to: {}", level))
}

#[tauri::command]
#[allow(dead_code)]
pub async fn get_safe_mode_status() -> Result<SafeModeConfig, String> {
    Ok(SafeModeManager::global().get_config())
}

// Enhanced file guard with safe mode integration
// Note: FileGuard integration commented out due to missing dependency
// use crate::filesystem::security::file_guard::FileGuard;

// impl FileGuard {
//     pub fn validate_with_safe_mode(&self, path: &std::path::Path) -> Result<bool> {
//         // First check safe mode restrictions
//         if !SafeModeManager::global().is_file_allowed(path)? {
//             return Ok(false);
//         }
//
//         // Then apply normal security validations
//         self.validate_file_access(path)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_safe_mode_config_from_env() {
        std::env::set_var("PROXEMIC_SAFE_MODE_LEVEL", "restricted");
        std::env::set_var("PROXEMIC_MAX_FILE_SIZE_MB", "25");
        std::env::set_var("PROXEMIC_REQUIRE_MAGIC_NUMBER", "true");

        let config = SafeModeConfig::from_env().unwrap();

        assert!(matches!(config.level, SafeModeLevel::Restricted));
        assert_eq!(config.max_file_size_mb, 25);
        assert!(config.require_magic_number);

        // Cleanup
        std::env::remove_var("PROXEMIC_SAFE_MODE_LEVEL");
        std::env::remove_var("PROXEMIC_MAX_FILE_SIZE_MB");
        std::env::remove_var("PROXEMIC_REQUIRE_MAGIC_NUMBER");
    }

    #[test]
    fn test_safe_mode_file_restrictions() {
        let manager = SafeModeManager::new();
        manager.set_level(SafeModeLevel::Restricted).unwrap();

        // Create test files
        let mut allowed_file = NamedTempFile::with_suffix(".txt").unwrap();
        let mut blocked_file = NamedTempFile::with_suffix(".exe").unwrap();

        write!(allowed_file, "test content").unwrap();
        write!(blocked_file, "binary content").unwrap();

        assert!(manager.is_file_allowed(allowed_file.path()).unwrap());
        assert!(!manager.is_file_allowed(blocked_file.path()).unwrap());
    }
}
