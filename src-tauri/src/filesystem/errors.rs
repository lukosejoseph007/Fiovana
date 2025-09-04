use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),

    #[error("Invalid file extension: {0}")]
    InvalidExtension(String),

    #[error("Access denied to path: {0}")]
    AccessDenied(String),

    #[error("File too large: {0} bytes")]
    FileSizeExceeded(u64),
}
