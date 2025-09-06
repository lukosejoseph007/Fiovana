use crate::magic_number_validator::MagicNumberValidator;
use crate::path_validator::PathValidator;
use std::fs::File;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn allowed_paths_should_pass() {
        let allowed_paths = vec![
            dirs::desktop_dir().unwrap(),
            dirs::document_dir().unwrap(),
            dirs::download_dir().unwrap(),
        ];

        for path in allowed_paths {
            assert!(
                PathValidator::validate(&path).is_ok(),
                "Allowed path {:?} should pass",
                path
            );
        }
    }

    #[test]
    fn restricted_paths_should_fail() {
        let restricted_path = Path::new("C:/Windows/System32");
        assert!(
            PathValidator::validate(restricted_path).is_err(),
            "Restricted path should fail"
        );
    }

    #[test]
    fn file_type_validation() {
        let valid_file = "tests/test_files/example.png";
        let invalid_file = "tests/test_files/malicious.png"; // actually .exe

        assert!(
            MagicNumberValidator::validate(valid_file).is_ok(),
            "Valid PNG should pass"
        );
        assert!(
            MagicNumberValidator::validate(invalid_file).is_err(),
            "Mismatched magic number should fail"
        );
    }

    #[test]
    fn edge_cases() {
        let long_path = Path::new(&"C:/".to_string() + &"a".repeat(300));
        assert!(
            PathValidator::validate(long_path).is_err(),
            "Long path should fail"
        );

        let traversal_path = Path::new("../secret/file.txt");
        assert!(
            PathValidator::validate(traversal_path).is_err(),
            "Path traversal should fail"
        );
    }

    #[test]
    fn concurrent_access_should_respect_rules() {
        let handles: Vec<_> = (0..5)
            .map(|_| thread::spawn(|| PathValidator::validate(dirs::desktop_dir().unwrap())))
            .collect();

        for handle in handles {
            assert!(
                handle.join().unwrap().is_ok(),
                "Concurrent access should respect rules"
            );
        }
    }
}
