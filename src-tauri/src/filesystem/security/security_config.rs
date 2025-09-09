// src-tauri/src/filesystem/security/security_config.rs
// Legacy adapter for the new configuration system

use crate::app_config::types::SecurityConfig as AppSecurityConfig;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Legacy SecurityConfig structure for backward compatibility
/// Converts from the new AppSecurityConfig (types.rs)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub allowed_extensions: HashSet<String>,
    pub max_path_length: usize,
    pub max_file_size: u64,
    pub allowed_mime_types: HashSet<String>,
    pub allowed_workspace_paths: Vec<PathBuf>,
    pub temp_directory: PathBuf,
    pub prohibited_filename_chars: HashSet<char>,
    pub enable_magic_number_validation: bool,
    pub magic_number_map: HashMap<String, Vec<Vec<u8>>>,
}

impl SecurityConfig {
    /// Create a SecurityConfig from the new AppSecurityConfig
    pub fn from_app_config(app_config: &AppSecurityConfig) -> Self {
        let allowed_extensions: HashSet<String> =
            app_config.allowed_extensions.iter().cloned().collect();
        let allowed_mime_types: HashSet<String> =
            app_config.allowed_mime_types.iter().cloned().collect();
        let prohibited_chars: HashSet<char> = app_config
            .prohibited_filename_chars
            .iter()
            .cloned()
            .collect();

        Self {
            allowed_extensions,
            max_path_length: app_config.max_path_length,
            max_file_size: app_config.max_file_size,
            allowed_mime_types,
            allowed_workspace_paths: Self::get_default_workspace_paths(),
            temp_directory: std::env::temp_dir(),
            prohibited_filename_chars: prohibited_chars,
            enable_magic_number_validation: app_config.enable_magic_number_validation,
            magic_number_map: Self::create_magic_number_map(),
        }
    }

    /// Get default workspace paths (Desktop, Documents, Downloads, etc.)
    fn get_default_workspace_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home_dir) = dirs::home_dir() {
            // Add common user directories
            paths.push(home_dir.join("Desktop"));
            paths.push(home_dir.join("Documents"));
            paths.push(home_dir.join("Downloads"));

            // Platform-specific additions
            #[cfg(target_os = "windows")]
            {
                paths.push(home_dir.join("OneDrive"));
                paths.push(home_dir.join("OneDrive - Personal"));
            }

            #[cfg(target_os = "macos")]
            {
                paths.push(home_dir.join("iCloud Drive"));
            }
        }

        // Always include temp directory
        paths.push(std::env::temp_dir());

        // Filter to only existing directories
        paths
            .into_iter()
            .filter(|p| p.exists() && p.is_dir())
            .collect()
    }

    /// Create the magic number mapping for file type detection
    fn create_magic_number_map() -> HashMap<String, Vec<Vec<u8>>> {
        let mut magic_numbers = HashMap::new();

        // PDF files
        magic_numbers.insert(
            "pdf".to_string(),
            vec![vec![0x25, 0x50, 0x44, 0x46]], // %PDF
        );

        // Microsoft Office files (ZIP-based)
        magic_numbers.insert(
            "docx".to_string(),
            vec![
                vec![0x50, 0x4B, 0x03, 0x04], // PK.. (ZIP signature)
                vec![0x50, 0x4B, 0x05, 0x06],
                vec![0x50, 0x4B, 0x07, 0x08],
            ],
        );

        // Text/Markdown files (no reliable magic number)
        magic_numbers.insert("txt".to_string(), vec![]);
        magic_numbers.insert("md".to_string(), vec![]);

        // PNG
        magic_numbers.insert(
            "png".to_string(),
            vec![vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]],
        );

        // JPEG
        magic_numbers.insert("jpg".to_string(), vec![vec![0xFF, 0xD8, 0xFF]]);

        magic_numbers
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let app_config = AppSecurityConfig::default();
        Self::from_app_config(&app_config)
    }
}
