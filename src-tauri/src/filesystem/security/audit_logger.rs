use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::log_rotation;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use uuid::Uuid;

// Global log rotation manager
static LOG_ROTATION_MANAGER: OnceCell<Arc<Mutex<log_rotation::LogRotationManager>>> =
    OnceCell::new();

/// Standardized security levels for audit logging
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Security event types for audit logging
#[derive(Debug, Clone, Serialize)]
pub enum SecurityEventType {
    FileAccessGranted,
    FileAccessDenied,
    #[allow(dead_code)]
    SecurityViolation,
    #[allow(dead_code)]
    ConfigurationChange,
    #[allow(dead_code)]
    EnvironmentOverride,
    #[allow(dead_code)]
    SchemaValidationFailed,
    PermissionEscalationAttempt,
    ResourceExhaustion,
}

/// Detailed security event structure for comprehensive audit logging
#[derive(Debug, Clone, Serialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub file_path: Option<PathBuf>,
    pub operation: Option<String>,
    pub user: Option<String>,
    pub security_level: SecurityLevel,
    pub error_details: Option<String>,
    pub error_code: Option<String>,
    pub metadata: serde_json::Value,
    pub correlation_id: Uuid,
}

pub struct SecurityAuditor;

impl SecurityAuditor {
    /// Initialize the log rotation system for audit logging
    pub fn init_log_rotation(log_dir: Option<std::path::PathBuf>) -> Result<(), std::io::Error> {
        let manager = log_rotation::init_file_logging(log_dir)?;
        LOG_ROTATION_MANAGER
            .set(Arc::new(Mutex::new(manager)))
            .map_err(|_| std::io::Error::other("Log rotation manager already initialized"))?;
        Ok(())
    }

    /// Check if log rotation is initialized
    #[allow(dead_code)]
    pub fn is_log_rotation_initialized() -> bool {
        LOG_ROTATION_MANAGER.get().is_some()
    }

    /// Helper function to convert string security level to SecurityLevel enum
    pub fn parse_security_level(level: &str) -> SecurityLevel {
        match level.to_uppercase().as_str() {
            "LOW" => SecurityLevel::Low,
            "MEDIUM" => SecurityLevel::Medium,
            "HIGH" => SecurityLevel::High,
            "CRITICAL" => SecurityLevel::Critical,
            _ => SecurityLevel::Medium, // Default to Medium for unknown levels
        }
    }

    /// Creates a new correlation ID for tracking security incidents
    pub fn new_correlation_id() -> Uuid {
        Uuid::new_v4()
    }
    /// Logs a file access attempt, successful or denied.
    pub fn log_file_access_attempt(
        path: &Path,
        operation: &str,
        result: &Result<PathBuf, SecurityError>,
        security_level: &str,
        correlation_id: Option<Uuid>,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: match result {
                Ok(_) => SecurityEventType::FileAccessGranted,
                Err(_) => SecurityEventType::FileAccessDenied,
            },
            file_path: Some(path.to_path_buf()),
            operation: Some(operation.to_string()),
            user: Some(Self::get_current_user()),
            security_level: Self::parse_security_level(security_level),
            error_details: result.as_ref().err().map(|e| e.to_string()),
            error_code: result.as_ref().err().map(|e| e.code().to_string()),
            metadata: serde_json::json!({}),
            correlation_id: correlation_id.unwrap_or_else(Self::new_correlation_id),
        };

        Self::log_event(event);
    }

    /// Logs a security violation with detailed information
    #[allow(dead_code)]
    pub fn log_security_violation(
        violation_type: &str,
        details: &str,
        file_path: Option<&Path>,
        operation: Option<&str>,
        security_level: &str,
        correlation_id: Option<Uuid>,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::SecurityViolation,
            file_path: file_path.map(|p| p.to_path_buf()),
            operation: operation.map(|s| s.to_string()),
            user: Some(Self::get_current_user()),
            security_level: Self::parse_security_level(security_level),
            error_details: Some(details.to_string()),
            error_code: Some(violation_type.to_string()),
            metadata: serde_json::json!({
                "violation_type": violation_type,
                "details": details
            }),
            correlation_id: correlation_id.unwrap_or_else(Self::new_correlation_id),
        };

        Self::log_event(event);
    }

    /// Logs configuration changes for audit trail
    #[allow(dead_code)]
    pub fn log_configuration_change(
        config_field: &str,
        old_value: &str,
        new_value: &str,
        security_level: &str,
        correlation_id: Option<Uuid>,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::ConfigurationChange,
            file_path: None,
            operation: Some("configuration_update".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Self::parse_security_level(security_level),
            error_details: None,
            error_code: None,
            metadata: serde_json::json!({
                "field": config_field,
                "old_value": old_value,
                "new_value": new_value
            }),
            correlation_id: correlation_id.unwrap_or_else(Self::new_correlation_id),
        };

        Self::log_event(event);
    }

    /// Logs environment variable overrides
    #[allow(dead_code)]
    pub fn log_environment_override(
        env_var: &str,
        value: &str,
        security_level: &str,
        correlation_id: Option<Uuid>,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::EnvironmentOverride,
            file_path: None,
            operation: Some("environment_override".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Self::parse_security_level(security_level),
            error_details: None,
            error_code: None,
            metadata: serde_json::json!({
                "environment_variable": env_var,
                "value": value
            }),
            correlation_id: correlation_id.unwrap_or_else(Self::new_correlation_id),
        };

        Self::log_event(event);
    }

    /// Logs schema validation failures
    #[allow(dead_code)]
    pub fn log_schema_validation_failure(
        errors: &[String],
        config: &serde_json::Value,
        security_level: &str,
        correlation_id: Option<Uuid>,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::SchemaValidationFailed,
            file_path: None,
            operation: Some("schema_validation".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Self::parse_security_level(security_level),
            error_details: Some(errors.join("; ")),
            error_code: None,
            metadata: serde_json::json!({
                "validation_errors": errors,
                "config_snapshot": config
            }),
            correlation_id: correlation_id.unwrap_or_else(Self::new_correlation_id),
        };

        Self::log_event(event);
    }

    /// Internal method to log events with appropriate severity
    fn log_event(event: SecurityEvent) {
        let event_json = serde_json::to_string(&event)
            .unwrap_or_else(|_| "Failed to serialize event".to_string());

        // Try to log to file first, fall back to console if file logging fails
        if let Some(manager) = LOG_ROTATION_MANAGER.get() {
            if let Ok(mut manager) = manager.lock() {
                if let Err(e) = manager.write_log(&event_json) {
                    // Fall back to console logging if file logging fails
                    tracing::error!("Failed to write to audit log file: {}", e);
                } else {
                    // Successfully logged to file, also log to console for visibility
                    match event.event_type {
                        SecurityEventType::FileAccessGranted => info!(
                            security_event = %event_json,
                            "Security event: File access granted"
                        ),
                        SecurityEventType::FileAccessDenied => warn!(
                            security_event = %event_json,
                            "Security event: File access denied"
                        ),
                        SecurityEventType::SecurityViolation => error!(
                            security_event = %event_json,
                            "Security event: Security violation"
                        ),
                        SecurityEventType::ConfigurationChange => info!(
                            security_event = %event_json,
                            "Security event: Configuration change"
                        ),
                        SecurityEventType::EnvironmentOverride => info!(
                            security_event = %event_json,
                            "Security event: Environment override"
                        ),
                        SecurityEventType::SchemaValidationFailed => error!(
                            security_event = %event_json,
                            "Security event: Schema validation failed"
                        ),
                        SecurityEventType::PermissionEscalationAttempt => error!(
                            security_event = %event_json,
                            "Security event: Permission escalation attempt"
                        ),
                        SecurityEventType::ResourceExhaustion => warn!(
                            security_event = %event_json,
                            "Security event: Resource exhaustion"
                        ),
                    }
                    return;
                }
            }
        }

        // Fallback: log only to console if file logging is not available
        match event.event_type {
            SecurityEventType::FileAccessGranted => info!(
                security_event = %event_json,
                "Security event: File access granted"
            ),
            SecurityEventType::FileAccessDenied => warn!(
                security_event = %event_json,
                "Security event: File access denied"
            ),
            SecurityEventType::SecurityViolation => error!(
                security_event = %event_json,
                "Security event: Security violation"
            ),
            SecurityEventType::ConfigurationChange => info!(
                security_event = %event_json,
                "Security event: Configuration change"
            ),
            SecurityEventType::EnvironmentOverride => info!(
                security_event = %event_json,
                "Security event: Environment override"
            ),
            SecurityEventType::SchemaValidationFailed => error!(
                security_event = %event_json,
                "Security event: Schema validation failed"
            ),
            SecurityEventType::PermissionEscalationAttempt => error!(
                security_event = %event_json,
                "Security event: Permission escalation attempt"
            ),
            SecurityEventType::ResourceExhaustion => warn!(
                security_event = %event_json,
                "Security event: Resource exhaustion"
            ),
        }
    }

    /// Gets the current user for audit logging
    fn get_current_user() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Logs permission escalation attempts
    #[allow(dead_code)]
    pub fn log_permission_escalation_attempt(
        operation: &str,
        target_path: &Path,
        security_level: &str,
        correlation_id: Option<Uuid>,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::PermissionEscalationAttempt,
            file_path: Some(target_path.to_path_buf()),
            operation: Some(operation.to_string()),
            user: Some(Self::get_current_user()),
            security_level: Self::parse_security_level(security_level),
            error_details: Some("Permission escalation attempt detected".to_string()),
            error_code: Some("PERMISSION_ESCALATION".to_string()),
            metadata: serde_json::json!({}),
            correlation_id: correlation_id.unwrap_or_else(Self::new_correlation_id),
        };

        Self::log_event(event);
    }

    /// Logs resource exhaustion events
    #[allow(dead_code)]
    pub fn log_resource_exhaustion(
        resource_type: &str,
        current_usage: u64,
        max_limit: u64,
        security_level: &str,
        correlation_id: Option<Uuid>,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::ResourceExhaustion,
            file_path: None,
            operation: Some("resource_usage".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Self::parse_security_level(security_level),
            error_details: Some(format!(
                "{} usage: {}/{}",
                resource_type, current_usage, max_limit
            )),
            error_code: Some("RESOURCE_EXHAUSTION".to_string()),
            metadata: serde_json::json!({
                "resource_type": resource_type,
                "current_usage": current_usage,
                "max_limit": max_limit
            }),
            correlation_id: correlation_id.unwrap_or_else(Self::new_correlation_id),
        };

        Self::log_event(event);
    }
}
