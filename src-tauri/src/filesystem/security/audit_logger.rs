use crate::filesystem::errors::SecurityError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

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
    pub security_level: Option<String>,
    pub error_details: Option<String>,
    pub error_code: Option<String>,
    pub metadata: serde_json::Value,
}

pub struct SecurityAuditor;

impl SecurityAuditor {
    /// Logs a file access attempt, successful or denied.
    pub fn log_file_access_attempt(
        path: &Path,
        operation: &str,
        result: &Result<PathBuf, SecurityError>,
        security_level: &str,
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
            security_level: Some(security_level.to_string()),
            error_details: result.as_ref().err().map(|e| e.to_string()),
            error_code: result.as_ref().err().map(|e| e.code().to_string()),
            metadata: serde_json::json!({}),
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
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::SecurityViolation,
            file_path: file_path.map(|p| p.to_path_buf()),
            operation: operation.map(|s| s.to_string()),
            user: Some(Self::get_current_user()),
            security_level: Some(security_level.to_string()),
            error_details: Some(details.to_string()),
            error_code: Some(violation_type.to_string()),
            metadata: serde_json::json!({
                "violation_type": violation_type,
                "details": details
            }),
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
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::ConfigurationChange,
            file_path: None,
            operation: Some("configuration_update".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Some(security_level.to_string()),
            error_details: None,
            error_code: None,
            metadata: serde_json::json!({
                "field": config_field,
                "old_value": old_value,
                "new_value": new_value
            }),
        };

        Self::log_event(event);
    }

    /// Logs environment variable overrides
    #[allow(dead_code)]
    pub fn log_environment_override(env_var: &str, value: &str, security_level: &str) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::EnvironmentOverride,
            file_path: None,
            operation: Some("environment_override".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Some(security_level.to_string()),
            error_details: None,
            error_code: None,
            metadata: serde_json::json!({
                "environment_variable": env_var,
                "value": value
            }),
        };

        Self::log_event(event);
    }

    /// Logs schema validation failures
    #[allow(dead_code)]
    pub fn log_schema_validation_failure(
        errors: &[String],
        config: &serde_json::Value,
        security_level: &str,
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::SchemaValidationFailed,
            file_path: None,
            operation: Some("schema_validation".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Some(security_level.to_string()),
            error_details: Some(errors.join("; ")),
            error_code: None,
            metadata: serde_json::json!({
                "validation_errors": errors,
                "config_snapshot": config
            }),
        };

        Self::log_event(event);
    }

    /// Internal method to log events with appropriate severity
    fn log_event(event: SecurityEvent) {
        let event_json = serde_json::to_string(&event)
            .unwrap_or_else(|_| "Failed to serialize event".to_string());

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
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::PermissionEscalationAttempt,
            file_path: Some(target_path.to_path_buf()),
            operation: Some(operation.to_string()),
            user: Some(Self::get_current_user()),
            security_level: Some(security_level.to_string()),
            error_details: Some("Permission escalation attempt detected".to_string()),
            error_code: Some("PERMISSION_ESCALATION".to_string()),
            metadata: serde_json::json!({}),
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
    ) {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::ResourceExhaustion,
            file_path: None,
            operation: Some("resource_usage".to_string()),
            user: Some(Self::get_current_user()),
            security_level: Some(security_level.to_string()),
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
        };

        Self::log_event(event);
    }
}
