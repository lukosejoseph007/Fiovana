// src-tauri/src/app_config/encryption.rs
use crate::app_config::errors::{ConfigError, ConfigResult};
use sha2::{Digest, Sha256};
use std::env;

pub struct ConfigEncryption;

impl ConfigEncryption {
    /// Generate a simple encryption key from environment variables
    /// In production, this should use proper key management
    pub fn get_encryption_key() -> ConfigResult<[u8; 32]> {
        // Try to get encryption key from environment
        if let Ok(key_str) = env::var("PROXEMIC_ENCRYPTION_KEY") {
            if key_str.len() >= 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_str.as_bytes()[..32]);
                return Ok(key);
            }
        }

        // Fallback: Generate key from machine identifier and app name
        let machine_id = Self::get_machine_identifier()?;
        let app_salt = "proxemic_config_v1";
        let combined = format!("{}{}", machine_id, app_salt);

        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        Ok(key)
    }

    /// Get a machine identifier for key derivation
    fn get_machine_identifier() -> ConfigResult<String> {
        // Try multiple methods to get a stable machine identifier

        // Method 1: Environment variables
        if let Ok(hostname) = env::var("COMPUTERNAME") {
            return Ok(hostname);
        }
        if let Ok(hostname) = env::var("HOSTNAME") {
            return Ok(hostname);
        }

        // Method 2: Username + OS
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "default_user".to_string());

        let os = env::consts::OS;
        Ok(format!("{}_{}", username, os))
    }

    /// Simple XOR encryption for configuration values
    /// Note: This is basic encryption - production should use proper crypto libraries
    pub fn encrypt_value(value: &str, key: &[u8; 32]) -> String {
        let value_bytes = value.as_bytes();
        let mut encrypted = Vec::with_capacity(value_bytes.len());

        for (i, &byte) in value_bytes.iter().enumerate() {
            let key_byte = key[i % key.len()];
            encrypted.push(byte ^ key_byte);
        }

        hex::encode(encrypted)
    }

    /// Simple XOR decryption for configuration values
    pub fn decrypt_value(encrypted_hex: &str, key: &[u8; 32]) -> ConfigResult<String> {
        let encrypted_bytes =
            hex::decode(encrypted_hex).map_err(|_| ConfigError::EncryptionError {
                message: "Invalid hex encoding in encrypted value".to_string(),
            })?;

        let mut decrypted = Vec::with_capacity(encrypted_bytes.len());

        for (i, &byte) in encrypted_bytes.iter().enumerate() {
            let key_byte = key[i % key.len()];
            decrypted.push(byte ^ key_byte);
        }

        String::from_utf8(decrypted).map_err(|_| ConfigError::EncryptionError {
            message: "Decrypted data is not valid UTF-8".to_string(),
        })
    }

    /// Check if a value looks like it's encrypted (hex string)
    pub fn is_encrypted_value(value: &str) -> bool {
        value.len() > 16 && value.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Encrypt sensitive configuration fields
    pub fn encrypt_sensitive_config(
        config: &mut crate::app_config::types::ProxemicConfig,
    ) -> ConfigResult<()> {
        let key = Self::get_encryption_key()?;

        // Encrypt API keys if they exist and aren't already encrypted
        if let Some(ref api_key) = config.ai.openrouter_api_key {
            if !Self::is_encrypted_value(api_key) {
                config.ai.openrouter_api_key = Some(Self::encrypt_value(api_key, &key));
            }
        }

        if let Some(ref api_key) = config.ai.anthropic_api_key {
            if !Self::is_encrypted_value(api_key) {
                config.ai.anthropic_api_key = Some(Self::encrypt_value(api_key, &key));
            }
        }

        // Encrypt database URL if it contains sensitive information
        if (config.database.url.contains("password=") || config.database.url.contains("://"))
            && !Self::is_encrypted_value(&config.database.url)
        {
            config.database.url = Self::encrypt_value(&config.database.url, &key);
        }

        Ok(())
    }

    /// Decrypt sensitive configuration fields
    pub fn decrypt_sensitive_config(
        config: &mut crate::app_config::types::ProxemicConfig,
    ) -> ConfigResult<()> {
        let key = Self::get_encryption_key()?;

        // Decrypt API keys if they're encrypted
        if let Some(ref api_key) = config.ai.openrouter_api_key {
            if Self::is_encrypted_value(api_key) {
                config.ai.openrouter_api_key = Some(Self::decrypt_value(api_key, &key)?);
            }
        }

        if let Some(ref api_key) = config.ai.anthropic_api_key {
            if Self::is_encrypted_value(api_key) {
                config.ai.anthropic_api_key = Some(Self::decrypt_value(api_key, &key)?);
            }
        }

        // Decrypt database URL if it's encrypted
        if Self::is_encrypted_value(&config.database.url) {
            config.database.url = Self::decrypt_value(&config.database.url, &key)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = ConfigEncryption::get_encryption_key().unwrap();
        let original = "test_api_key_12345";

        let encrypted = ConfigEncryption::encrypt_value(original, &key);
        let decrypted = ConfigEncryption::decrypt_value(&encrypted, &key).unwrap();

        assert_eq!(original, decrypted);
        assert!(ConfigEncryption::is_encrypted_value(&encrypted));
        assert!(!ConfigEncryption::is_encrypted_value(original));
    }

    #[test]
    fn test_machine_identifier() {
        let id1 = ConfigEncryption::get_machine_identifier().unwrap();
        let id2 = ConfigEncryption::get_machine_identifier().unwrap();

        assert_eq!(id1, id2); // Should be consistent
        assert!(!id1.is_empty());
    }
}
