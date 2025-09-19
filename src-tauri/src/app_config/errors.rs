// src-tauri/src/app_config/errors.rs
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found at path: {path}")]
    #[allow(dead_code)]
    FileNotFound { path: PathBuf },

    #[error("Failed to read configuration file: {source}")]
    ReadError {
        #[from]
        source: std::io::Error,
    },

    #[error("Configuration parsing failed: {message}")]
    ParseError { message: String },

    #[error("Configuration validation failed: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Environment variable error: {name} - {message}")]
    #[allow(dead_code)]
    EnvironmentError { name: String, message: String },

    #[error("Invalid environment: {message}")]
    #[allow(dead_code)]
    InvalidEnvironment { message: String },

    #[error("Encryption/decryption failed: {message}")]
    EncryptionError { message: String },

    #[error("Configuration serialization failed: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
}

pub type ConfigResult<T> = Result<T, ConfigError>;
