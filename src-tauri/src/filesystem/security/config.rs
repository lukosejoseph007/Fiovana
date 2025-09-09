use std::collections::HashSet;
use std::path::PathBuf;

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
    pub magic_number_map: std::collections::HashMap<String, Vec<Vec<u8>>>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut allowed_extensions = HashSet::new();
        for ext in &[".docx", ".pdf", ".md", ".txt"] {
            allowed_extensions.insert(ext.to_string().to_lowercase());
        }

        let mut prohibited_chars = HashSet::new();
        for c in "<>:\"/\\\\|?*\0".chars() {
            prohibited_chars.insert(c);
        }

        let mut allowed_mime_types = HashSet::new();
        allowed_mime_types.insert("application/pdf".to_string());
        allowed_mime_types.insert("text/plain".to_string());

        Self {
            allowed_extensions,
            max_path_length: 260,             // Windows MAX_PATH
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_mime_types,
            allowed_workspace_paths: vec![],
            temp_directory: std::env::temp_dir(),
            prohibited_filename_chars: prohibited_chars,
            enable_magic_number_validation: true,
            magic_number_map: std::collections::HashMap::new(),
        }
    }
}
