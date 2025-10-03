pub mod ai;
pub mod app_config;
pub mod app_state;
pub mod collaboration;
pub mod commands;
pub mod document;
pub mod filesystem;
pub mod memory_monitor;
pub mod notifications;
pub mod resource_monitor;
pub mod vector;
pub mod workspace;

pub use app_state::{AppState, SecurityState};
pub use commands::*;
pub use vector::{EmbeddingEngine, VectorStore};

#[cfg(test)]
pub mod test_utils {
    use std::env;
    use std::path::PathBuf;

    pub fn get_safe_test_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Check if running in CI
        if env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok() {
            // Use environment-provided paths in CI
            if let Ok(desktop) = env::var("XDG_DESKTOP_DIR") {
                paths.push(PathBuf::from(desktop));
            }
            if let Ok(documents) = env::var("XDG_DOCUMENTS_DIR") {
                paths.push(PathBuf::from(documents));
            }
            if let Ok(downloads) = env::var("XDG_DOWNLOAD_DIR") {
                paths.push(PathBuf::from(downloads));
            }
        } else {
            // Use real directories locally
            if let Some(desktop) = dirs::desktop_dir() {
                paths.push(desktop);
            }
            if let Some(documents) = dirs::document_dir() {
                paths.push(documents);
            }
            if let Some(downloads) = dirs::download_dir() {
                paths.push(downloads);
            }
        }

        // Always ensure we have at least one path
        if paths.is_empty() {
            paths.push(env::temp_dir());
        }

        paths
    }
}
