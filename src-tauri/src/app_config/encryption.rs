// src-tauri/src/app_config/encryption.rs
use crate::app_config::errors::{ConfigError, ConfigResult};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use hex;
use rand::rngs::ThreadRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::env;

pub struct ConfigEncryption;

impl ConfigEncryption {
    /// Generate a 32-byte AES key from environment variables or machine ID
    pub fn get_encryption_key() -> ConfigResult<[u8; 32]> {
        if let Ok(key_str) = env::var("FIOVANA_ENCRYPTION_KEY") {
            if key_str.len() >= 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_str.as_bytes()[..32]);
                return Ok(key);
            }
        }

        let machine_id = Self::get_machine_identifier()?;
        let combined = format!("{}{}", machine_id, "fiovana_config_v1");
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        Ok(key)
    }

    fn get_machine_identifier() -> ConfigResult<String> {
        if let Ok(hostname) = env::var("COMPUTERNAME") {
            return Ok(hostname);
        }
        if let Ok(hostname) = env::var("HOSTNAME") {
            return Ok(hostname);
        }
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "default_user".to_string());
        Ok(format!("{}_{}", username, env::consts::OS))
    }

    /// AES-256-GCM encryption with random nonce
    pub fn encrypt_value(value: &str, key: &[u8; 32]) -> String {
        let cipher = Aes256Gcm::new_from_slice(key).unwrap();
        let mut nonce_bytes = [0u8; 12];
        let mut rng = ThreadRng::default();
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, value.as_bytes()).unwrap();

        // Prepend nonce to ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        hex::encode(combined)
    }

    /// AES-256-GCM decryption with nonce extraction
    pub fn decrypt_value(encrypted_hex: &str, key: &[u8; 32]) -> ConfigResult<String> {
        let combined = hex::decode(encrypted_hex).map_err(|_| ConfigError::EncryptionError {
            message: "Invalid hex encoding".to_string(),
        })?;

        if combined.len() < 12 {
            return Err(ConfigError::EncryptionError {
                message: "Ciphertext too short".to_string(),
            });
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ConfigError::EncryptionError {
            message: "Failed to initialize AES cipher".to_string(),
        })?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted_bytes = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
            ConfigError::EncryptionError {
                message: "AES decryption failed".to_string(),
            }
        })?;
        String::from_utf8(decrypted_bytes).map_err(|_| ConfigError::EncryptionError {
            message: "Decrypted data not valid UTF-8".to_string(),
        })
    }

    /// Legacy XOR decryption (for migration)
    fn decrypt_xor_legacy(value: &str, key: &[u8; 32]) -> ConfigResult<String> {
        let encrypted_bytes = hex::decode(value).map_err(|_| ConfigError::EncryptionError {
            message: "Invalid hex in legacy XOR value".to_string(),
        })?;
        let mut decrypted = Vec::with_capacity(encrypted_bytes.len());
        for (i, &byte) in encrypted_bytes.iter().enumerate() {
            decrypted.push(byte ^ key[i % key.len()]);
        }
        String::from_utf8(decrypted).map_err(|_| ConfigError::EncryptionError {
            message: "Legacy XOR decrypted data not valid UTF-8".to_string(),
        })
    }

    /// Check if string is likely encrypted (hex)
    pub fn is_encrypted_value(value: &str) -> bool {
        value.len() > 16 && value.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Encrypt sensitive fields
    #[allow(dead_code)]
    pub fn encrypt_sensitive_config(
        config: &mut crate::app_config::types::FiovanaConfig,
    ) -> ConfigResult<()> {
        let key = Self::get_encryption_key()?;
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
        if (config.database.url.contains("password=") || config.database.url.contains("://"))
            && !Self::is_encrypted_value(&config.database.url)
        {
            config.database.url = Self::encrypt_value(&config.database.url, &key);
        }
        Ok(())
    }

    /// Decrypt sensitive fields with migration (XOR → AES)
    pub fn decrypt_sensitive_config(
        config: &mut crate::app_config::types::FiovanaConfig,
    ) -> ConfigResult<()> {
        let key = Self::get_encryption_key()?;

        let decrypt_migrate = |value: &str| -> ConfigResult<String> {
            if Self::is_encrypted_value(value) {
                match Self::decrypt_value(value, &key) {
                    Ok(aes_val) => Ok(aes_val),
                    Err(_) => Self::decrypt_xor_legacy(value, &key),
                }
            } else {
                Ok(value.to_string())
            }
        };

        if let Some(ref mut api_key) = config.ai.openrouter_api_key {
            let decrypted = decrypt_migrate(api_key)?;
            *api_key = Self::encrypt_value(&decrypted, &key);
        }
        if let Some(ref mut api_key) = config.ai.anthropic_api_key {
            let decrypted = decrypt_migrate(api_key)?;
            *api_key = Self::encrypt_value(&decrypted, &key);
        }
        if !config.database.url.is_empty() {
            let decrypted = decrypt_migrate(&config.database.url)?;
            config.database.url = Self::encrypt_value(&decrypted, &key);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_roundtrip() {
        let key = ConfigEncryption::get_encryption_key().unwrap();
        let original = "test_api_key_12345";

        // Encrypt twice to check random nonce produces different ciphertexts
        let encrypted1 = ConfigEncryption::encrypt_value(original, &key);
        let encrypted2 = ConfigEncryption::encrypt_value(original, &key);

        // Ciphertexts should not be equal due to random nonce
        assert_ne!(encrypted1, encrypted2);

        // Decrypt both
        let decrypted1 = ConfigEncryption::decrypt_value(&encrypted1, &key).unwrap();
        let decrypted2 = ConfigEncryption::decrypt_value(&encrypted2, &key).unwrap();

        assert_eq!(original, decrypted1);
        assert_eq!(original, decrypted2);

        assert!(ConfigEncryption::is_encrypted_value(&encrypted1));
        assert!(ConfigEncryption::is_encrypted_value(&encrypted2));
    }

    #[test]
    fn test_xor_legacy_roundtrip() {
        let key = ConfigEncryption::get_encryption_key().unwrap();
        let original = "legacy_api_key_12345";

        // Manually encrypt with XOR
        let encrypted = {
            let mut bytes = Vec::with_capacity(original.len());
            for (i, &b) in original.as_bytes().iter().enumerate() {
                bytes.push(b ^ key[i % key.len()]);
            }
            hex::encode(bytes)
        };

        let decrypted = ConfigEncryption::decrypt_xor_legacy(&encrypted, &key).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_migration_xor_to_aes() {
        let key = ConfigEncryption::get_encryption_key().unwrap();
        let original = "legacy_migration_key";

        // Legacy XOR encryption
        let legacy_encrypted = {
            let mut bytes = Vec::with_capacity(original.len());
            for (i, &b) in original.as_bytes().iter().enumerate() {
                bytes.push(b ^ key[i % key.len()]);
            }
            hex::encode(bytes)
        };

        // Decrypt with migration logic (should fallback to XOR if AES fails)
        let decrypted = {
            let decrypt_migrate = |value: &str| -> ConfigResult<String> {
                if ConfigEncryption::is_encrypted_value(value) {
                    match ConfigEncryption::decrypt_value(value, &key) {
                        Ok(aes_val) => Ok(aes_val),
                        Err(_) => ConfigEncryption::decrypt_xor_legacy(value, &key),
                    }
                } else {
                    Ok(value.to_string())
                }
            };
            decrypt_migrate(&legacy_encrypted).unwrap()
        };

        assert_eq!(original, decrypted);

        // Re-encrypt using AES after migration
        let re_encrypted = ConfigEncryption::encrypt_value(&decrypted, &key);
        let final_decrypted = ConfigEncryption::decrypt_value(&re_encrypted, &key).unwrap();
        assert_eq!(original, final_decrypted);
    }

    #[test]
    fn test_multiple_random_nonces() {
        let key = ConfigEncryption::get_encryption_key().unwrap();
        let original = "random_nonce_test";

        let mut ciphertexts = Vec::new();
        for _ in 0..5 {
            ciphertexts.push(ConfigEncryption::encrypt_value(original, &key));
        }

        // Ensure all ciphertexts are unique due to random nonce
        for i in 0..ciphertexts.len() {
            for j in (i + 1)..ciphertexts.len() {
                assert_ne!(ciphertexts[i], ciphertexts[j]);
            }
        }

        // All ciphertexts should decrypt correctly
        for encrypted in ciphertexts {
            let decrypted = ConfigEncryption::decrypt_value(&encrypted, &key).unwrap();
            assert_eq!(decrypted, original);
        }
    }
}
