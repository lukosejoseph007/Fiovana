use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum SecurityError {
    #[error("Path traversal attempt detected: {path}")]
    PathTraversal { path: String },

    #[error("Invalid file extension: {extension}")]
    InvalidExtension { extension: String },

    #[error("Path too long: {length} exceeds maximum {max}")]
    PathTooLong { length: usize, max: usize },

    #[error("Filename contains prohibited characters: {filename}")]
    ProhibitedCharacters { filename: String },

    #[error("Access denied to path: {path}")]
    AccessDenied { path: String },

    #[error("File size {size} exceeds limit {limit}")]
    FileSizeExceeded { size: u64, limit: u64 },

    #[error("Path is outside the allowed workspace: {path}")]
    PathOutsideWorkspace { path: String },

    #[error("I/O error occurred: {0}")]
    IoError(String),
}

#[allow(dead_code)]
#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum ValidationError {
    #[error("File type validation failed: {reason}")]
    FileType { reason: String },

    #[error("Magic number mismatch for file type: {expected} vs {actual}")]
    MagicNumber { expected: String, actual: String },

    #[error("File corruption detected: {details}")]
    Corruption { details: String },
}

// 🔽 Error code getters
#[allow(dead_code)]
impl SecurityError {
    pub fn code(&self) -> &'static str {
        match self {
            SecurityError::PathTraversal { .. } => "SEC_PATH_TRAVERSAL",
            SecurityError::InvalidExtension { .. } => "SEC_INVALID_EXTENSION",
            SecurityError::PathTooLong { .. } => "SEC_PATH_TOO_LONG",
            SecurityError::ProhibitedCharacters { .. } => "SEC_PROHIBITED_CHARS",
            SecurityError::AccessDenied { .. } => "SEC_ACCESS_DENIED",
            SecurityError::FileSizeExceeded { .. } => "SEC_FILE_TOO_LARGE",
            SecurityError::PathOutsideWorkspace { .. } => "SEC_PATH_OUTSIDE_WORKSPACE",
            SecurityError::IoError(_) => "SEC_IO_ERROR",
        }
    }
}

#[allow(dead_code)]
impl ValidationError {
    pub fn code(&self) -> &'static str {
        match self {
            ValidationError::FileType { .. } => "VAL_FILE_TYPE",
            ValidationError::MagicNumber { .. } => "VAL_MAGIC_MISMATCH",
            ValidationError::Corruption { .. } => "VAL_CORRUPTION",
        }
    }
}

impl From<std::io::Error> for SecurityError {
    fn from(err: std::io::Error) -> SecurityError {
        SecurityError::IoError(err.to_string())
    }
}

// 🔽 Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_error_display() {
        let err = SecurityError::PathTraversal {
            path: "/etc/passwd".into(),
        };
        assert_eq!(
            format!("{}", err),
            "Path traversal attempt detected: /etc/passwd"
        );
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::MagicNumber {
            expected: "PDF".into(),
            actual: "TXT".into(),
        };
        assert_eq!(
            format!("{}", err),
            "Magic number mismatch for file type: PDF vs TXT"
        );
    }

    #[test]
    fn test_security_error_code() {
        let err = SecurityError::AccessDenied {
            path: "C:/restricted.txt".into(),
        };
        assert_eq!(err.code(), "SEC_ACCESS_DENIED");
    }

    #[test]
    fn test_validation_error_code() {
        let err = ValidationError::Corruption {
            details: "checksum mismatch".into(),
        };
        assert_eq!(err.code(), "VAL_CORRUPTION");
    }

    #[test]
    fn test_path_outside_workspace_error() {
        let err = SecurityError::PathOutsideWorkspace {
            path: "C:/unauthorized/file.txt".into(),
        };
        assert_eq!(
            format!("{}", err),
            "Path is outside the allowed workspace: C:/unauthorized/file.txt"
        );
        assert_eq!(err.code(), "SEC_PATH_OUTSIDE_WORKSPACE");
    }
}
