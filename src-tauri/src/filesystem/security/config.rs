use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub allowed_extensions: HashSet<String>,
    pub max_path_length: usize,
    pub max_file_size: u64,
    pub allowed_workspace_paths: Vec<PathBuf>,
    pub temp_directory: PathBuf,
    pub prohibited_filename_chars: HashSet<char>,
    pub enable_magic_number_validation: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut allowed_extensions = HashSet::new();
        allowed_extensions.insert(".docx".to_string());
        allowed_extensions.insert(".pdf".to_string());
        allowed_extensions.insert(".md".to_string());
        allowed_extensions.insert(".txt".to_string());

        let mut prohibited_chars = HashSet::new();
        for c in "<>:\"|?*\0".chars() {
            prohibited_chars.insert(c);
        }

        Self {
            allowed_extensions,
            max_path_length: 260,             // Windows MAX_PATH
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_workspace_paths: vec![],
            temp_directory: std::env::temp_dir(),
            prohibited_filename_chars: prohibited_chars,
            enable_magic_number_validation: true,
        }
    }
}
