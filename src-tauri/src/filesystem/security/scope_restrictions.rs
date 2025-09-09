use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_outside_workspace() {
        let config = SecurityConfig::default();
        let allowed_paths = vec![
            dirs::desktop_dir().unwrap(),
            dirs::document_dir().unwrap(),
            dirs::download_dir().unwrap(),
        ];
        let validator = PathValidator::new(config, allowed_paths);

        let path = Path::new("C:/Windows/System32/config.sys");
        assert!(validator.validate_import_path(path).is_err());
    }

    #[test]
    fn test_path_inside_workspace() {
        let config = SecurityConfig::default();
        let allowed_paths = vec![
            dirs::desktop_dir().unwrap(),
            dirs::document_dir().unwrap(),
            dirs::download_dir().unwrap(),
        ];
        let validator = PathValidator::new(config, allowed_paths);

        let path = Path::new("C:/Users/test/document.txt");
        assert!(validator.validate_import_path(path).is_ok());
    }
}
