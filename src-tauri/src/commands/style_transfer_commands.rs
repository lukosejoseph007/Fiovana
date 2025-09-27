use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::document::indexer::DocumentIndexer;
use crate::document::style_analyzer::{ToneType, VocabularyComplexity, VoiceType};
use crate::document::style_learner::{OrganizationalStyle, StyleLearner};
use crate::document::style_transfer::{
    StyleTransfer, StyleTransferConfig, StyleTransferMode, StyleTransferRequest,
    StyleTransferResult, StyleTransferTarget,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferStyleRequest {
    pub content: String,
    pub target_style_type: String, // "organizational", "custom", or "document_profile"
    pub target_style_data: serde_json::Value,
    pub config: StyleTransferConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferStyleResponse {
    pub success: bool,
    pub result: Option<StyleTransferResult>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyOrganizationalStyleRequest {
    pub content: String,
    pub organization_id: String,
    pub document_paths: Vec<String>, // Documents to learn style from
    pub config: Option<StyleTransferConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyOrganizationalStyleResponse {
    pub success: bool,
    pub result: Option<StyleTransferResult>,
    pub learned_style: Option<OrganizationalStyle>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomStyleRequest {
    pub content: String,
    pub tone: String,
    pub voice: String,
    pub formality: f64,
    pub complexity: String,
    pub terminology_preferences: std::collections::HashMap<String, String>,
    pub config: Option<StyleTransferConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StylePreviewRequest {
    pub content: String,
    pub target_style_type: String,
    pub target_style_data: serde_json::Value,
    pub preview_changes_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StylePreviewResponse {
    pub success: bool,
    pub changes: Vec<crate::document::style_transfer::StyleChange>,
    pub confidence_score: f64,
    pub summary: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateStyleConfigRequest {
    pub config: StyleTransferConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateStyleConfigResponse {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Transfer content style using specified target style
#[tauri::command]
pub async fn transfer_content_style(
    request: TransferStyleRequest,
) -> Result<TransferStyleResponse, String> {
    println!(
        "Transferring content style with type: {}",
        request.target_style_type
    );

    let style_transfer = StyleTransfer::new();

    // Parse target style based on type
    let target_style =
        match parse_target_style(&request.target_style_type, &request.target_style_data) {
            Ok(style) => style,
            Err(e) => {
                return Ok(TransferStyleResponse {
                    success: false,
                    result: None,
                    error: Some(format!("Invalid target style: {}", e)),
                });
            }
        };

    let transfer_request = StyleTransferRequest {
        content: request.content,
        target_style,
        config: request.config,
    };

    match style_transfer.transfer_style(transfer_request) {
        Ok(result) => {
            println!(
                "Style transfer completed with {} changes",
                result.applied_changes.len()
            );
            Ok(TransferStyleResponse {
                success: true,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => {
            println!("Style transfer failed: {}", e);
            Ok(TransferStyleResponse {
                success: false,
                result: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Apply organizational style to content by learning from document corpus
#[tauri::command]
pub async fn apply_organizational_style(
    request: ApplyOrganizationalStyleRequest,
    indexer: State<'_, Mutex<DocumentIndexer>>,
) -> Result<ApplyOrganizationalStyleResponse, String> {
    println!(
        "Applying organizational style for organization: {}",
        request.organization_id
    );

    let indexer = indexer
        .lock()
        .map_err(|e| format!("Failed to lock indexer: {}", e))?;
    let style_learner = StyleLearner::new();
    let style_transfer = StyleTransfer::new();

    // Get documents from paths
    let mut documents = Vec::new();
    let all_docs = indexer.get_all_documents();
    for path in &request.document_paths {
        if let Some(doc) = all_docs.iter().find(|d| d.path.to_string_lossy() == *path) {
            documents.push((*doc).clone());
        }
    }

    if documents.is_empty() {
        return Ok(ApplyOrganizationalStyleResponse {
            success: false,
            result: None,
            learned_style: None,
            error: Some("No valid documents found for style learning".to_string()),
        });
    }

    // Learn organizational style
    let learning_result =
        match style_learner.learn_from_corpus(&documents, request.organization_id.clone()) {
            Ok(result) => result,
            Err(e) => {
                return Ok(ApplyOrganizationalStyleResponse {
                    success: false,
                    result: None,
                    learned_style: None,
                    error: Some(format!("Failed to learn organizational style: {}", e)),
                });
            }
        };

    // Apply learned style to content
    let config = request.config.unwrap_or_default();
    let transfer_request = StyleTransferRequest {
        content: request.content,
        target_style: StyleTransferTarget::OrganizationalStyle(
            learning_result.organizational_style.clone(),
        ),
        config,
    };

    match style_transfer.transfer_style(transfer_request) {
        Ok(result) => {
            println!(
                "Organizational style applied with {} changes",
                result.applied_changes.len()
            );
            Ok(ApplyOrganizationalStyleResponse {
                success: true,
                result: Some(result),
                learned_style: Some(learning_result.organizational_style),
                error: None,
            })
        }
        Err(e) => {
            println!("Failed to apply organizational style: {}", e);
            Ok(ApplyOrganizationalStyleResponse {
                success: false,
                result: None,
                learned_style: Some(learning_result.organizational_style),
                error: Some(e.to_string()),
            })
        }
    }
}

/// Apply custom style to content
#[tauri::command]
pub async fn apply_custom_style(
    request: CustomStyleRequest,
) -> Result<TransferStyleResponse, String> {
    println!(
        "Applying custom style with tone: {} and voice: {}",
        request.tone, request.voice
    );

    let style_transfer = StyleTransfer::new();

    // Parse tone and voice from strings
    let tone = parse_tone_type(&request.tone)?;
    let voice = parse_voice_type(&request.voice)?;
    let complexity = parse_vocabulary_complexity(&request.complexity)?;

    let target_style = StyleTransferTarget::CustomStyle {
        tone,
        voice,
        formality: request.formality,
        complexity,
        terminology_preferences: request.terminology_preferences,
    };

    let config = request.config.unwrap_or_default();
    let transfer_request = StyleTransferRequest {
        content: request.content,
        target_style,
        config,
    };

    match style_transfer.transfer_style(transfer_request) {
        Ok(result) => {
            println!(
                "Custom style applied with {} changes",
                result.applied_changes.len()
            );
            Ok(TransferStyleResponse {
                success: true,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => {
            println!("Failed to apply custom style: {}", e);
            Ok(TransferStyleResponse {
                success: false,
                result: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Preview style changes without applying them
#[tauri::command]
pub async fn preview_style_changes(
    request: StylePreviewRequest,
) -> Result<StylePreviewResponse, String> {
    println!(
        "Previewing style changes for type: {}",
        request.target_style_type
    );

    let style_transfer = StyleTransfer::new();

    // Parse target style
    let target_style =
        match parse_target_style(&request.target_style_type, &request.target_style_data) {
            Ok(style) => style,
            Err(e) => {
                return Ok(StylePreviewResponse {
                    success: false,
                    changes: vec![],
                    confidence_score: 0.0,
                    summary: String::new(),
                    error: Some(format!("Invalid target style: {}", e)),
                });
            }
        };

    let config = StyleTransferConfig {
        mode: StyleTransferMode::Conservative, // Use conservative mode for preview
        ..StyleTransferConfig::default()
    };

    let transfer_request = StyleTransferRequest {
        content: request.content,
        target_style,
        config,
    };

    match style_transfer.transfer_style(transfer_request) {
        Ok(result) => {
            let summary = if request.preview_changes_only {
                format!(
                    "Preview: {} potential changes identified",
                    result.applied_changes.len()
                )
            } else {
                result.transfer_summary.clone()
            };

            println!(
                "Style preview generated with {} potential changes",
                result.applied_changes.len()
            );
            Ok(StylePreviewResponse {
                success: true,
                changes: result.applied_changes,
                confidence_score: result.confidence_score,
                summary,
                error: None,
            })
        }
        Err(e) => {
            println!("Style preview failed: {}", e);
            Ok(StylePreviewResponse {
                success: false,
                changes: vec![],
                confidence_score: 0.0,
                summary: String::new(),
                error: Some(e.to_string()),
            })
        }
    }
}

/// Validate style transfer configuration
#[tauri::command]
pub async fn validate_style_config(
    request: ValidateStyleConfigRequest,
) -> Result<ValidateStyleConfigResponse, String> {
    println!("Validating style transfer configuration");

    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    // Validate configuration parameters
    if request.config.confidence_threshold < 0.0 || request.config.confidence_threshold > 1.0 {
        warnings.push("Confidence threshold should be between 0.0 and 1.0".to_string());
    }

    if request.config.target_formality.is_some() {
        let formality = request.config.target_formality.unwrap();
        if !(0.0..=1.0).contains(&formality) {
            warnings.push("Target formality should be between 0.0 and 1.0".to_string());
        }
    }

    // Generate recommendations
    if matches!(request.config.mode, StyleTransferMode::Aggressive) && !request.config.ai_assistance
    {
        recommendations
            .push("Consider enabling AI assistance for aggressive style transfer mode".to_string());
    }

    if !request.config.preserve_structure && !request.config.preserve_technical_terms {
        recommendations.push(
            "Consider preserving either structure or technical terms to maintain content integrity"
                .to_string(),
        );
    }

    if request.config.confidence_threshold < 0.5 {
        recommendations.push(
            "Consider using a higher confidence threshold (≥0.5) for reliable results".to_string(),
        );
    }

    let is_valid = warnings.is_empty();

    println!(
        "Configuration validation completed - Valid: {}, Warnings: {}",
        is_valid,
        warnings.len()
    );
    Ok(ValidateStyleConfigResponse {
        valid: is_valid,
        warnings,
        recommendations,
    })
}

/// Get available style transfer options and capabilities
#[tauri::command]
pub async fn get_style_transfer_capabilities() -> Result<serde_json::Value, String> {
    println!("Getting style transfer capabilities");

    Ok(serde_json::json!({
        "supported_target_types": [
            "organizational",
            "custom",
            "document_profile"
        ],
        "supported_modes": [
            "Conservative",
            "Moderate",
            "Aggressive"
        ],
        "supported_tones": [
            "Formal",
            "Informal",
            "Friendly",
            "Professional",
            "Academic",
            "Conversational",
            "Instructional",
            "Technical",
            "Authoritative",
            "Empathetic",
            "Persuasive",
            "Objective",
            "Enthusiastic",
            "Cautious"
        ],
        "supported_voices": [
            "Authoritative",
            "Friendly",
            "Neutral",
            "Encouraging",
            "Explanatory",
            "Conversational",
            "Expert",
            "Empathetic"
        ],
        "supported_complexity_levels": [
            "Basic",
            "Intermediate",
            "Advanced",
            "Expert"
        ],
        "features": [
            "formality_adjustment",
            "tone_modification",
            "voice_transformation",
            "vocabulary_complexity_adjustment",
            "terminology_standardization",
            "ai_assisted_rewriting",
            "style_preview",
            "confidence_scoring",
            "change_tracking"
        ],
        "ai_assistance": true,
        "preview_support": true,
        "batch_processing": false,
        "version": "1.0.0"
    }))
}

// Helper functions for parsing style parameters
fn parse_target_style(
    style_type: &str,
    style_data: &serde_json::Value,
) -> Result<StyleTransferTarget, String> {
    match style_type {
        "organizational" => {
            let org_style: OrganizationalStyle = serde_json::from_value(style_data.clone())
                .map_err(|e| format!("Failed to parse organizational style: {}", e))?;
            Ok(StyleTransferTarget::OrganizationalStyle(org_style))
        }
        "custom" => {
            let custom_data = style_data
                .as_object()
                .ok_or("Custom style data must be an object")?;

            let tone = parse_tone_type(
                custom_data
                    .get("tone")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid tone in custom style")?,
            )?;

            let voice = parse_voice_type(
                custom_data
                    .get("voice")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid voice in custom style")?,
            )?;

            let formality = custom_data
                .get("formality")
                .and_then(|v| v.as_f64())
                .ok_or("Missing or invalid formality in custom style")?;

            let complexity = parse_vocabulary_complexity(
                custom_data
                    .get("complexity")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid complexity in custom style")?,
            )?;

            let terminology_preferences = custom_data
                .get("terminology_preferences")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();

            Ok(StyleTransferTarget::CustomStyle {
                tone,
                voice,
                formality,
                complexity,
                terminology_preferences,
            })
        }
        "document_profile" => {
            // For document profile, we would analyze a specific document
            // This is a simplified implementation
            Err("Document profile style transfer not yet implemented".to_string())
        }
        _ => Err(format!("Unsupported style type: {}", style_type)),
    }
}

fn parse_tone_type(tone_str: &str) -> Result<ToneType, String> {
    match tone_str {
        "Formal" => Ok(ToneType::Formal),
        "Informal" => Ok(ToneType::Informal),
        "Friendly" => Ok(ToneType::Friendly),
        "Professional" => Ok(ToneType::Professional),
        "Academic" => Ok(ToneType::Academic),
        "Conversational" => Ok(ToneType::Conversational),
        "Instructional" => Ok(ToneType::Instructional),
        "Technical" => Ok(ToneType::Technical),
        "Authoritative" => Ok(ToneType::Authoritative),
        "Empathetic" => Ok(ToneType::Empathetic),
        "Persuasive" => Ok(ToneType::Persuasive),
        "Objective" => Ok(ToneType::Objective),
        "Enthusiastic" => Ok(ToneType::Enthusiastic),
        "Cautious" => Ok(ToneType::Cautious),
        _ => Err(format!("Invalid tone type: {}", tone_str)),
    }
}

fn parse_voice_type(voice_str: &str) -> Result<VoiceType, String> {
    match voice_str {
        "Authoritative" => Ok(VoiceType::Authoritative),
        "Friendly" => Ok(VoiceType::Friendly),
        "Neutral" => Ok(VoiceType::Neutral),
        "Encouraging" => Ok(VoiceType::Encouraging),
        "Explanatory" => Ok(VoiceType::Explanatory),
        "Conversational" => Ok(VoiceType::Conversational),
        "Expert" => Ok(VoiceType::Expert),
        "Empathetic" => Ok(VoiceType::Empathetic),
        _ => Err(format!("Invalid voice type: {}", voice_str)),
    }
}

fn parse_vocabulary_complexity(complexity_str: &str) -> Result<VocabularyComplexity, String> {
    match complexity_str {
        "Basic" => Ok(VocabularyComplexity::Basic),
        "Intermediate" => Ok(VocabularyComplexity::Intermediate),
        "Advanced" => Ok(VocabularyComplexity::Advanced),
        "Expert" => Ok(VocabularyComplexity::Expert),
        _ => Err(format!("Invalid vocabulary complexity: {}", complexity_str)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_style_transfer_capabilities() {
        let result = get_style_transfer_capabilities().await;
        assert!(result.is_ok());

        let capabilities = result.unwrap();
        assert!(capabilities.get("supported_target_types").is_some());
        assert!(capabilities.get("supported_modes").is_some());
        assert!(capabilities.get("features").is_some());
    }

    #[tokio::test]
    async fn test_validate_style_config_valid() {
        let request = ValidateStyleConfigRequest {
            config: StyleTransferConfig::default(),
        };

        let result = validate_style_config(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.valid);
        assert!(response.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_validate_style_config_invalid_confidence() {
        let request = ValidateStyleConfigRequest {
            config: StyleTransferConfig {
                confidence_threshold: 1.5, // Invalid: > 1.0
                ..StyleTransferConfig::default()
            },
        };

        let result = validate_style_config(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.valid);
        assert!(!response.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_parse_tone_type_valid() {
        let result = parse_tone_type("Formal");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ToneType::Formal));
    }

    #[tokio::test]
    async fn test_parse_tone_type_invalid() {
        let result = parse_tone_type("InvalidTone");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_voice_type_valid() {
        let result = parse_voice_type("Authoritative");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VoiceType::Authoritative));
    }

    #[tokio::test]
    async fn test_parse_vocabulary_complexity_valid() {
        let result = parse_vocabulary_complexity("Advanced");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VocabularyComplexity::Advanced));
    }

    #[test]
    fn test_transfer_style_request_serialization() {
        let request = TransferStyleRequest {
            content: "Test content".to_string(),
            target_style_type: "custom".to_string(),
            target_style_data: serde_json::json!({
                "tone": "Formal",
                "voice": "Authoritative",
                "formality": 0.8,
                "complexity": "Advanced",
                "terminology_preferences": {}
            }),
            config: StyleTransferConfig::default(),
        };

        let serialized = serde_json::to_string(&request);
        assert!(serialized.is_ok());
    }
}
