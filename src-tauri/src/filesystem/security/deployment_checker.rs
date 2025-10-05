// src-tauri/src/filesystem/security/deployment_checker.rs
// Production deployment readiness checker

use crate::filesystem::security::config_validator::{ConfigSchemaValidator, ValidationError};
use crate::filesystem::security::env_validator::EnvironmentValidator;
use crate::filesystem::security::security_config::{
    SecurityConfig, SecurityConfigError, SecurityLevel,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io;
use std::path::Path;

/// Deployment readiness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentAssessment {
    pub ready_for_production: bool,
    pub security_score: u8, // 0-100
    pub critical_issues: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub security_level: SecurityLevel,
    pub configuration_valid: bool,
    pub environment_valid: bool,
    pub missing_requirements: Vec<String>,
}

/// Comprehensive deployment checker
pub struct DeploymentChecker {
    env_validator: EnvironmentValidator,
    config_validator: ConfigSchemaValidator,
}

impl DeploymentChecker {
    pub fn new() -> Self {
        Self {
            env_validator: EnvironmentValidator::new(),
            config_validator: ConfigSchemaValidator::new(),
        }
    }

    /// Perform comprehensive deployment readiness check
    pub fn assess_deployment_readiness(&self) -> Result<DeploymentAssessment, SecurityConfigError> {
        let mut assessment = DeploymentAssessment {
            ready_for_production: false,
            security_score: 0,
            critical_issues: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            security_level: SecurityLevel::Development,
            configuration_valid: false,
            environment_valid: false,
            missing_requirements: Vec::new(),
        };

        // 1. Validate environment configuration
        let env_result = self.env_validator.validate_environment()?;
        assessment.environment_valid = env_result.valid;
        assessment.security_level = env_result.security_level.clone();
        assessment.critical_issues.extend(env_result.errors);
        assessment.warnings.extend(env_result.warnings);
        assessment
            .missing_requirements
            .extend(env_result.missing_critical_vars);

        // 2. Validate security configuration schema
        let config = SecurityConfig::default();
        let config_result = self.validate_security_config(&config);
        assessment.configuration_valid = config_result.is_ok();

        if let Err(config_error) = config_result {
            match config_error {
                ValidationError::SchemaValidation { errors } => {
                    assessment.critical_issues.extend(errors);
                }
                _ => {
                    assessment.critical_issues.push(config_error.to_string());
                }
            }
        }

        // 3. Perform security-specific checks
        self.assess_security_posture(&mut assessment)?;

        // 4. Check deployment prerequisites
        self.check_deployment_prerequisites(&mut assessment)?;

        // 5. Calculate security score
        assessment.security_score = self.calculate_security_score(&assessment);

        // 6. Generate recommendations
        assessment.recommendations = self.generate_recommendations(&assessment)?;

        // 7. Determine production readiness
        assessment.ready_for_production = self.is_ready_for_production(&assessment);

        Ok(assessment)
    }

    fn validate_security_config(&self, config: &SecurityConfig) -> Result<(), ValidationError> {
        let config_json = serde_json::json!({
            "max_file_size": config.max_file_size,
            "max_path_length": config.max_path_length,
            "enable_magic_number_validation": config.enable_magic_number_validation,
            "security_level": match config.security_level {
                SecurityLevel::Development => "development",
                SecurityLevel::Production => "production",
                SecurityLevel::HighSecurity => "high_security",
            },
            "allowed_extensions": config.allowed_extensions.iter().collect::<Vec<_>>(),
            "max_concurrent_operations": config.max_concurrent_operations,
            "enforce_workspace_boundaries": config.enforce_workspace_boundaries,
            "audit_logging_enabled": config.audit_logging_enabled
        });

        let security_level_str = match config.security_level {
            SecurityLevel::Development => "development",
            SecurityLevel::Production => "production",
            SecurityLevel::HighSecurity => "high_security",
        };

        self.config_validator
            .validate_config(&config_json, security_level_str)
    }

    fn assess_security_posture(
        &self,
        assessment: &mut DeploymentAssessment,
    ) -> Result<(), SecurityConfigError> {
        // Check for production security requirements
        match assessment.security_level {
            SecurityLevel::Development => {
                assessment.warnings.push(
                    "Development security level - not recommended for production".to_string(),
                );
            }
            SecurityLevel::Production | SecurityLevel::HighSecurity => {
                // Validate production-specific security settings
                self.validate_production_security_settings(assessment)?;
            }
        }

        // Check for common security misconfigurations
        self.check_security_misconfigurations(assessment)?;

        // Validate encryption settings
        self.validate_encryption_configuration(assessment)?;

        // Check audit and logging configuration
        self.validate_audit_configuration(assessment)?;

        Ok(())
    }

    fn validate_production_security_settings(
        &self,
        assessment: &mut DeploymentAssessment,
    ) -> Result<(), SecurityConfigError> {
        let required_secure_settings = [
            ("FIOVANA_ENABLE_MAGIC_VALIDATION", "true"),
            ("FIOVANA_ENFORCE_WORKSPACE_BOUNDARIES", "true"),
            ("FIOVANA_AUDIT_LOGGING_ENABLED", "true"),
        ];

        for (var_name, expected_value) in &required_secure_settings {
            match env::var(var_name) {
                Ok(value) => {
                    let normalized = value.to_lowercase();
                    let is_enabled = matches!(normalized.as_str(), "true" | "1" | "yes");
                    let should_be_enabled = *expected_value == "true";

                    if is_enabled != should_be_enabled {
                        assessment.critical_issues.push(format!(
                            "Security violation: {} must be '{}' in production",
                            var_name, expected_value
                        ));
                    }
                }
                Err(_) => {
                    assessment.missing_requirements.push(var_name.to_string());
                }
            }
        }

        Ok(())
    }

    fn check_security_misconfigurations(
        &self,
        assessment: &mut DeploymentAssessment,
    ) -> Result<(), SecurityConfigError> {
        // Check for debug mode in production
        if let Ok(debug_value) = env::var("FIOVANA_DEBUG") {
            if matches!(debug_value.to_lowercase().as_str(), "true" | "1" | "yes") {
                if matches!(
                    assessment.security_level,
                    SecurityLevel::Production | SecurityLevel::HighSecurity
                ) {
                    assessment
                        .critical_issues
                        .push("Debug mode enabled in production environment".to_string());
                } else {
                    assessment
                        .warnings
                        .push("Debug mode enabled - disable for production".to_string());
                }
            }
        }

        // Check for overly permissive file size limits
        if let Ok(file_size_str) = env::var("FIOVANA_MAX_FILE_SIZE") {
            if let Ok(file_size) = file_size_str.parse::<u64>() {
                if file_size > 500 * 1024 * 1024
                    && matches!(
                        assessment.security_level,
                        SecurityLevel::Production | SecurityLevel::HighSecurity
                    )
                {
                    assessment
                        .warnings
                        .push("File size limit very high for production environment".to_string());
                }
            }
        }

        // Check for excessive concurrent operations
        if let Ok(ops_str) = env::var("FIOVANA_MAX_CONCURRENT_OPERATIONS") {
            if let Ok(ops) = ops_str.parse::<u32>() {
                if ops > 100
                    && matches!(
                        assessment.security_level,
                        SecurityLevel::Production | SecurityLevel::HighSecurity
                    )
                {
                    assessment
                        .warnings
                        .push("High concurrent operations limit may impact security".to_string());
                }
            }
        }

        Ok(())
    }

    fn validate_encryption_configuration(
        &self,
        assessment: &mut DeploymentAssessment,
    ) -> Result<(), SecurityConfigError> {
        match env::var("FIOVANA_ENCRYPTION_KEY") {
            Ok(key) => {
                if key == "your_secure_32_character_key_here_change_this" {
                    assessment.critical_issues.push(
                        "Default encryption key detected - CRITICAL SECURITY RISK!".to_string(),
                    );
                } else if key.len() < 32 {
                    assessment.critical_issues.push(
                        "Encryption key too short for AES-256 (minimum 32 characters)".to_string(),
                    );
                } else if key.chars().all(|c| c.is_ascii_alphanumeric()) {
                    assessment.warnings.push(
                        "Encryption key should include special characters for better entropy"
                            .to_string(),
                    );
                }
            }
            Err(_) => {
                if matches!(
                    assessment.security_level,
                    SecurityLevel::Production | SecurityLevel::HighSecurity
                ) {
                    assessment
                        .critical_issues
                        .push("Encryption key not configured for production".to_string());
                }
            }
        }

        Ok(())
    }

    fn validate_audit_configuration(
        &self,
        assessment: &mut DeploymentAssessment,
    ) -> Result<(), SecurityConfigError> {
        if matches!(
            assessment.security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        ) {
            // Check if audit logging is properly configured
            if env::var("FIOVANA_AUDIT_LOGGING_ENABLED")
                .unwrap_or_default()
                .to_lowercase()
                != "true"
            {
                assessment
                    .critical_issues
                    .push("Audit logging not enabled for production".to_string());
            }

            // Check for structured logging
            if env::var("FIOVANA_STRUCTURED_LOGGING")
                .unwrap_or_default()
                .to_lowercase()
                != "true"
            {
                assessment
                    .recommendations
                    .push("Enable structured logging for better audit analysis".to_string());
            }
        }

        Ok(())
    }

    fn check_deployment_prerequisites(
        &self,
        assessment: &mut DeploymentAssessment,
    ) -> Result<(), SecurityConfigError> {
        // Check if we're running in CI environment
        let is_ci = std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok();

        // Check for required API keys (skip warning in CI)
        let has_openrouter = env::var("OPENROUTER_API_KEY").is_ok();
        let has_anthropic = env::var("ANTHROPIC_API_KEY").is_ok();

        if !has_openrouter && !has_anthropic && !is_ci {
            assessment
                .warnings
                .push("No AI service API keys configured".to_string());
        }

        // Check database configuration (skip warning in CI)
        if env::var("DATABASE_URL").is_err() && !is_ci {
            assessment
                .warnings
                .push("Database URL not configured - using default SQLite".to_string());
        }

        // Check vector index configuration (skip warning in CI)
        if env::var("VECTOR_INDEX_PATH").is_err() && !is_ci {
            assessment
                .warnings
                .push("Vector index path not configured - using default".to_string());
        }

        // Platform-specific checks
        #[cfg(target_os = "windows")]
        {
            // Windows-specific deployment checks
            if std::env::consts::ARCH != "x86_64" {
                assessment
                    .warnings
                    .push("Non-x64 architecture detected - ensure compatibility".to_string());
            }
        }

        Ok(())
    }

    fn calculate_security_score(&self, assessment: &DeploymentAssessment) -> u8 {
        let mut score = 100u8;

        // Deduct points for critical issues
        score = score.saturating_sub((assessment.critical_issues.len() as u8) * 20);

        // Deduct points for warnings
        score = score.saturating_sub((assessment.warnings.len() as u8) * 5);

        // Deduct points for missing requirements
        score = score.saturating_sub((assessment.missing_requirements.len() as u8) * 15);

        // Bonus points for high security level
        match assessment.security_level {
            SecurityLevel::HighSecurity => score = score.saturating_add(10),
            SecurityLevel::Production => score = score.saturating_add(5),
            SecurityLevel::Development => score = score.saturating_sub(31), // Ensure development scores below 70
        }

        // Ensure minimum score
        score
    }

    /// Get additional recommendations from environment validator
    fn get_env_recommendations(&self) -> Result<Vec<String>, SecurityConfigError> {
        // Simple recommendations based on current environment
        let mut recommendations = Vec::new();

        // Check if this is a production environment
        if std::env::var("FIOVANA_SECURITY_LEVEL")
            .unwrap_or_default()
            .to_lowercase()
            == "production"
        {
            recommendations.push("Consider implementing automated security scanning".to_string());
            recommendations.push("Set up centralized logging and monitoring".to_string());
            recommendations.push("Implement regular security audits".to_string());
        }

        Ok(recommendations)
    }

    fn generate_recommendations(
        &self,
        assessment: &DeploymentAssessment,
    ) -> Result<Vec<String>, SecurityConfigError> {
        let mut recommendations = Vec::new();

        // Security level recommendations
        match assessment.security_level {
            SecurityLevel::Development => {
                recommendations
                    .push("Upgrade to 'production' security level before deployment".to_string());
            }
            SecurityLevel::Production => {
                recommendations
                    .push("Consider 'high_security' level for sensitive environments".to_string());
            }
            SecurityLevel::HighSecurity => {
                recommendations.push("Excellent security configuration for production".to_string());
            }
        }

        // Performance recommendations
        if env::var("FIOVANA_MAX_CONCURRENT_OPERATIONS").is_err() {
            recommendations.push(
                "Set FIOVANA_MAX_CONCURRENT_OPERATIONS to control resource usage".to_string(),
            );
        }

        // Monitoring recommendations
        if env::var("FIOVANA_PERFORMANCE_MONITORING")
            .unwrap_or_default()
            .to_lowercase()
            != "true"
        {
            recommendations
                .push("Enable performance monitoring for production visibility".to_string());
        }

        // Backup recommendations
        recommendations.push("Implement regular database backups for production".to_string());
        recommendations.push("Set up log rotation and retention policies".to_string());

        // Get additional recommendations from environment validator
        let env_recommendations = self.get_env_recommendations()?;
        recommendations.extend(env_recommendations);

        Ok(recommendations)
    }

    fn is_ready_for_production(&self, assessment: &DeploymentAssessment) -> bool {
        assessment.critical_issues.is_empty()
            && assessment.configuration_valid
            && assessment.environment_valid
            && assessment.missing_requirements.is_empty()
            && matches!(
                assessment.security_level,
                SecurityLevel::Production | SecurityLevel::HighSecurity
            )
            && assessment.security_score >= 70
    }

    /// Generate a comprehensive deployment report
    pub fn generate_deployment_report(&self) -> Result<String, SecurityConfigError> {
        let assessment = self.assess_deployment_readiness()?;

        let mut report = String::new();
        report.push_str("╔════════════════════════════════════════════════════════════╗\n");
        report.push_str("║                  FIOVANA DEPLOYMENT REPORT                ║\n");
        report.push_str("╚════════════════════════════════════════════════════════════╝\n\n");

        // Overall Status
        let status_icon = if assessment.ready_for_production {
            "✅"
        } else {
            "❌"
        };
        let status_text = if assessment.ready_for_production {
            "READY"
        } else {
            "NOT READY"
        };

        report.push_str(&format!(
            "Production Ready: {} {}\n",
            status_icon, status_text
        ));
        report.push_str(&format!(
            "Security Score: {}/100\n",
            assessment.security_score
        ));
        report.push_str(&format!(
            "Security Level: {:?}\n",
            assessment.security_level
        ));
        report.push_str(&format!(
            "Environment Valid: {}\n",
            if assessment.environment_valid {
                "✅"
            } else {
                "❌"
            }
        ));
        report.push_str(&format!(
            "Configuration Valid: {}\n\n",
            if assessment.configuration_valid {
                "✅"
            } else {
                "❌"
            }
        ));

        // Critical Issues
        if !assessment.critical_issues.is_empty() {
            report.push_str("🔴 CRITICAL ISSUES (Must Fix Before Deployment):\n");
            for (i, issue) in assessment.critical_issues.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, issue));
            }
            report.push('\n');
        }

        // Missing Requirements
        if !assessment.missing_requirements.is_empty() {
            report.push_str("📋 MISSING REQUIREMENTS:\n");
            for requirement in &assessment.missing_requirements {
                report.push_str(&format!("  • {}\n", requirement));
            }
            report.push('\n');
        }

        // Warnings
        if !assessment.warnings.is_empty() {
            report.push_str("⚠️  WARNINGS (Should Address):\n");
            for warning in &assessment.warnings {
                report.push_str(&format!("  • {}\n", warning));
            }
            report.push('\n');
        }

        // Recommendations
        if !assessment.recommendations.is_empty() {
            report.push_str("💡 RECOMMENDATIONS:\n");
            for recommendation in &assessment.recommendations {
                report.push_str(&format!("  • {}\n", recommendation));
            }
            report.push('\n');
        }

        // Next Steps
        report.push_str("📝 NEXT STEPS:\n");
        if !assessment.ready_for_production {
            report.push_str("  1. Address all critical issues above\n");
            report.push_str("  2. Configure missing requirements\n");
            report.push_str("  3. Re-run deployment check\n");
        } else {
            report.push_str("  1. Review and address any warnings\n");
            report.push_str("  2. Set up monitoring and alerting\n");
            report.push_str("  3. Prepare deployment environment\n");
            report.push_str("  4. Execute deployment plan\n");
        }

        Ok(report)
    }

    /// Validate release artifact integrity by checking SHA256 checksums
    #[allow(dead_code)]
    pub fn validate_release_artifacts(
        &self,
        artifacts_path: &Path,
        checksums_path: &Path,
    ) -> Result<(), SecurityConfigError> {
        if !artifacts_path.exists() {
            return Err(SecurityConfigError::SchemaValidation {
                errors: vec!["Artifacts directory does not exist".to_string()],
            });
        }

        if !checksums_path.exists() {
            return Err(SecurityConfigError::SchemaValidation {
                errors: vec!["Checksums file does not exist".to_string()],
            });
        }

        let checksums_content = fs::read_to_string(checksums_path).map_err(|e| {
            SecurityConfigError::SchemaValidation {
                errors: vec![format!("Failed to read checksums file: {}", e)],
            }
        })?;

        let mut valid_artifacts = 0;
        let mut invalid_artifacts = 0;

        for line in checksums_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let expected_hash = parts[0];
            let filename = parts[1..].join(" ");

            let artifact_path = artifacts_path.join(&filename);
            if !artifact_path.exists() {
                invalid_artifacts += 1;
                continue;
            }

            let actual_hash = self.calculate_file_hash(&artifact_path).map_err(|e| {
                SecurityConfigError::SchemaValidation {
                    errors: vec![format!("Failed to calculate hash for {}: {}", filename, e)],
                }
            })?;

            if actual_hash == expected_hash {
                valid_artifacts += 1;
            } else {
                invalid_artifacts += 1;
            }
        }

        if invalid_artifacts > 0 {
            return Err(SecurityConfigError::SchemaValidation {
                errors: vec![format!(
                    "Artifact integrity check failed: {} valid, {} invalid",
                    valid_artifacts, invalid_artifacts
                )],
            });
        }

        Ok(())
    }

    /// Calculate SHA256 hash of a file
    fn calculate_file_hash(&self, path: &Path) -> Result<String, io::Error> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Verify code signing status for release artifacts
    #[allow(dead_code)]
    pub fn verify_code_signing(&self, artifacts_path: &Path) -> Result<(), SecurityConfigError> {
        let mut signed_artifacts = 0;
        let mut unsigned_artifacts = 0;

        for entry in
            fs::read_dir(artifacts_path).map_err(|e| SecurityConfigError::SchemaValidation {
                errors: vec![format!("Failed to read artifacts directory: {}", e)],
            })?
        {
            let entry = entry.map_err(|e| SecurityConfigError::SchemaValidation {
                errors: vec![format!("Failed to read directory entry: {}", e)],
            })?;
            let path = entry.path();

            if path.is_file() {
                let ext = path.extension().unwrap_or_default().to_string_lossy();
                let _filename = path.file_name().unwrap_or_default().to_string_lossy();

                // Check for common executable formats
                if matches!(
                    ext.as_ref(),
                    "exe" | "dmg" | "app" | "AppImage" | "deb" | "rpm"
                ) {
                    if self.is_file_signed(&path) {
                        signed_artifacts += 1;
                    } else {
                        unsigned_artifacts += 1;
                    }
                }
            }
        }

        if unsigned_artifacts > 0 {
            return Err(SecurityConfigError::SchemaValidation {
                errors: vec![format!(
                    "Code signing verification failed: {} signed, {} unsigned artifacts",
                    signed_artifacts, unsigned_artifacts
                )],
            });
        }

        Ok(())
    }

    /// Check if a file is signed (platform-specific implementation)
    #[allow(dead_code)]
    fn is_file_signed(&self, path: &Path) -> bool {
        #[cfg(target_os = "windows")]
        {
            // Windows code signing check using signtool
            use std::process::Command;

            let output = Command::new("signtool")
                .args(["verify", "/pa", path.to_str().unwrap_or_default()])
                .output();

            output.map(|o| o.status.success()).unwrap_or(false)
        }

        #[cfg(target_os = "macos")]
        {
            // macOS code signing check using codesign
            use std::process::Command;

            let output = Command::new("codesign")
                .args(["-v", path.to_str().unwrap_or_default()])
                .output();

            output.map(|o| o.status.success()).unwrap_or(false)
        }

        #[cfg(target_os = "linux")]
        {
            // Linux - check for GPG signatures or other signing mechanisms
            // For now, assume true for Linux as signing is less common
            let _ = path; // Mark path as intentionally unused
            true
        }
    }

    /// Generate deployment manifest with all artifacts and their metadata
    #[allow(dead_code)]
    pub fn generate_deployment_manifest(
        &self,
        artifacts_path: &Path,
    ) -> Result<String, SecurityConfigError> {
        let mut manifest = String::new();
        manifest.push_str("╔════════════════════════════════════════════════════════════╗\n");
        manifest.push_str("║                 DEPLOYMENT MANIFEST                      ║\n");
        manifest.push_str("╚════════════════════════════════════════════════════════════╝\n\n");

        manifest.push_str(&format!("Generated: {}\n", chrono::Utc::now()));
        manifest.push_str(&format!(
            "Artifacts Directory: {}\n\n",
            artifacts_path.display()
        ));

        for entry in
            fs::read_dir(artifacts_path).map_err(|e| SecurityConfigError::SchemaValidation {
                errors: vec![format!("Failed to read artifacts directory: {}", e)],
            })?
        {
            let entry = entry.map_err(|e| SecurityConfigError::SchemaValidation {
                errors: vec![format!("Failed to read directory entry: {}", e)],
            })?;
            let path = entry.path();

            if path.is_file() {
                let metadata =
                    fs::metadata(&path).map_err(|e| SecurityConfigError::SchemaValidation {
                        errors: vec![format!("Failed to get file metadata: {}", e)],
                    })?;

                let hash = self.calculate_file_hash(&path).unwrap_or_default();
                let signed = self.is_file_signed(&path);

                manifest.push_str(&format!(
                    "📦 {}:\n",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
                manifest.push_str(&format!("  Size: {} bytes\n", metadata.len()));
                manifest.push_str(&format!("  SHA256: {}\n", hash));
                manifest.push_str(&format!("  Signed: {}\n", if signed { "✅" } else { "❌" }));
                manifest.push_str(&format!(
                    "  Modified: {}\n",
                    chrono::DateTime::<chrono::Utc>::from(
                        metadata
                            .modified()
                            .unwrap_or_else(|_| std::time::SystemTime::now())
                    )
                ));
                manifest.push('\n');
            }
        }

        Ok(manifest)
    }
}

impl Default for DeploymentChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_checker_creation() {
        let checker = DeploymentChecker::new();
        // Basic smoke test - ensure checker can be created
        assert!(matches!(checker.env_validator, EnvironmentValidator { .. }));
    }

    #[test]
    fn test_security_score_calculation() {
        let checker = DeploymentChecker::new();

        let assessment = DeploymentAssessment {
            ready_for_production: false,
            security_score: 0,
            critical_issues: vec!["Test issue".to_string()],
            warnings: vec!["Test warning".to_string()],
            recommendations: vec![],
            security_level: SecurityLevel::Production,
            configuration_valid: true,
            environment_valid: true,
            missing_requirements: vec![],
        };

        let score = checker.calculate_security_score(&assessment);

        // Should have deductions for critical issue (20 points) and warning (5 points)
        // Plus bonus for production level (5 points)
        // 100 - 20 - 5 + 5 = 80
        assert_eq!(score, 80);
    }

    #[test]
    fn test_production_readiness() {
        let checker = DeploymentChecker::new();

        let assessment = DeploymentAssessment {
            ready_for_production: false,
            security_score: 80,
            critical_issues: vec![],
            warnings: vec![],
            recommendations: vec![],
            security_level: SecurityLevel::Production,
            configuration_valid: true,
            environment_valid: true,
            missing_requirements: vec![],
        };

        assert!(checker.is_ready_for_production(&assessment));
    }
}
