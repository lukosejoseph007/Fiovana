/// Defines allowed access scopes for file operations
#[derive(Debug, Clone)]
pub struct ScopeRestrictions {
    /// Allowed directories or base paths
    pub allowed_paths: Vec<String>,
    /// Maximum depth of subdirectories accessible
    pub max_subdirectory_depth: usize,
}

impl Default for ScopeRestrictions {
    fn default() -> Self {
        Self {
            allowed_paths: vec!["C:/Users/test".to_string()],
            max_subdirectory_depth: 5,
        }
    }
}

impl ScopeRestrictions {
    /// Check if a given path is within allowed scope
    pub fn is_within_scope(&self, path: &str) -> bool {
        self.allowed_paths.iter().any(|base| path.starts_with(base))
        // Additional depth checks can be added here
    }
}
