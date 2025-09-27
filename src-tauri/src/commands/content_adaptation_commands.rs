// src-tauri/src/commands/content_adaptation_commands.rs
// Tauri commands for content adaptation functionality

use crate::ai::AIConfig;
use crate::document::style_analyzer::StyleProfile;
use crate::document::{
    AdaptationConfig, AdaptationPurpose, AdaptationResult, AudienceType, ComplexityLevel,
    ContentAdapter, ToneAdjustment,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::Mutex;

/// Global content adapter instance
pub type ContentAdapterState = Arc<Mutex<Option<ContentAdapter>>>;

/// Request structure for content adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptContentRequest {
    pub content: String,
    pub target_audience: AudienceType,
    pub purpose: AdaptationPurpose,
    pub complexity_level: ComplexityLevel,
    pub tone_adjustments: Vec<ToneAdjustment>,
    pub preserve_technical_terms: bool,
    pub preserve_structure: bool,
    pub target_length_ratio: f64,
    pub use_examples: bool,
    pub include_definitions: bool,
    pub style_consistency: bool,
    pub cultural_sensitivity: bool,
    // Style-aware generation options
    pub target_style_profile: Option<StyleProfile>,
    pub enforce_style_consistency: bool,
    pub style_matching_strength: f64,
    pub preserve_author_voice: bool,
    pub apply_style_patterns: bool,
}

/// Response for content adaptation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAdaptationResponse {
    pub success: bool,
    pub result: Option<AdaptationResult>,
    pub error: Option<String>,
}

/// Response for batch content adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAdaptationResponse {
    pub success: bool,
    pub results: Vec<AdaptationResult>,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

/// Response for available options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationOptionsResponse {
    pub audiences: Vec<AudienceType>,
    pub purposes: Vec<AdaptationPurpose>,
    pub complexity_levels: Vec<ComplexityLevel>,
    pub tone_adjustments: Vec<ToneAdjustment>,
}

/// Response for adaptation suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationSuggestionsResponse {
    pub suggestions: Vec<String>,
    pub recommended_config: AdaptationConfig,
    pub quality_score: f64,
}

/// Initialize the content adapter with AI configuration
#[command]
pub async fn initialize_content_adapter(
    ai_config: AIConfig,
    adapter_state: State<'_, ContentAdapterState>,
) -> Result<bool, String> {
    let adapter = ContentAdapter::new(ai_config);
    let mut state = adapter_state.lock().await;
    *state = Some(adapter);
    Ok(true)
}

/// Adapt content for a specific audience and purpose
#[command]
pub async fn adapt_content(
    request: AdaptContentRequest,
    adapter_state: State<'_, ContentAdapterState>,
) -> Result<ContentAdaptationResponse, String> {
    async fn inner(
        request: AdaptContentRequest,
        adapter_state: State<'_, ContentAdapterState>,
    ) -> Result<ContentAdaptationResponse> {
        let state = adapter_state.lock().await;
        let adapter = state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Content adapter not initialized"))?;

        let config = AdaptationConfig {
            target_audience: request.target_audience,
            purpose: request.purpose,
            complexity_level: request.complexity_level,
            tone_adjustments: request.tone_adjustments,
            preserve_technical_terms: request.preserve_technical_terms,
            preserve_structure: request.preserve_structure,
            target_length_ratio: request.target_length_ratio,
            use_examples: request.use_examples,
            include_definitions: request.include_definitions,
            style_consistency: request.style_consistency,
            cultural_sensitivity: request.cultural_sensitivity,
            target_style_profile: request.target_style_profile,
            enforce_style_consistency: request.enforce_style_consistency,
            style_matching_strength: request.style_matching_strength,
            preserve_author_voice: request.preserve_author_voice,
            apply_style_patterns: request.apply_style_patterns,
        };

        let result = adapter
            .adapt_content(&request.content, Some(config))
            .await?;

        Ok(ContentAdaptationResponse {
            success: true,
            result: Some(result),
            error: None,
        })
    }

    match inner(request, adapter_state).await {
        Ok(response) => Ok(response),
        Err(e) => Ok(ContentAdaptationResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Batch adapt multiple content pieces
#[command]
pub async fn batch_adapt_content(
    content_pieces: Vec<String>,
    config: AdaptContentRequest,
    adapter_state: State<'_, ContentAdapterState>,
) -> Result<BatchAdaptationResponse, String> {
    async fn inner(
        content_pieces: Vec<String>,
        config: AdaptContentRequest,
        adapter_state: State<'_, ContentAdapterState>,
    ) -> Result<BatchAdaptationResponse> {
        let state = adapter_state.lock().await;
        let adapter = state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Content adapter not initialized"))?;

        let adaptation_config = AdaptationConfig {
            target_audience: config.target_audience,
            purpose: config.purpose,
            complexity_level: config.complexity_level,
            tone_adjustments: config.tone_adjustments,
            preserve_technical_terms: config.preserve_technical_terms,
            preserve_structure: config.preserve_structure,
            target_length_ratio: config.target_length_ratio,
            use_examples: config.use_examples,
            include_definitions: config.include_definitions,
            style_consistency: config.style_consistency,
            cultural_sensitivity: config.cultural_sensitivity,
            target_style_profile: config.target_style_profile,
            enforce_style_consistency: config.enforce_style_consistency,
            style_matching_strength: config.style_matching_strength,
            preserve_author_voice: config.preserve_author_voice,
            apply_style_patterns: config.apply_style_patterns,
        };

        let content_refs: Vec<&str> = content_pieces.iter().map(|s| s.as_str()).collect();
        let results = adapter
            .batch_adapt_content(content_refs, adaptation_config)
            .await?;

        Ok(BatchAdaptationResponse {
            success: true,
            results,
            failed_count: 0,
            errors: vec![],
        })
    }

    match inner(content_pieces, config, adapter_state).await {
        Ok(response) => Ok(response),
        Err(e) => Ok(BatchAdaptationResponse {
            success: false,
            results: vec![],
            failed_count: 1,
            errors: vec![e.to_string()],
        }),
    }
}

/// Get available adaptation options
#[command]
pub async fn get_adaptation_options() -> Result<AdaptationOptionsResponse, String> {
    Ok(AdaptationOptionsResponse {
        audiences: ContentAdapter::get_available_audiences(),
        purposes: ContentAdapter::get_available_purposes(),
        complexity_levels: vec![
            ComplexityLevel::Elementary,
            ComplexityLevel::Basic,
            ComplexityLevel::Intermediate,
            ComplexityLevel::Advanced,
            ComplexityLevel::Expert,
        ],
        tone_adjustments: vec![
            ToneAdjustment::MoreFormal,
            ToneAdjustment::LessFormal,
            ToneAdjustment::MoreFriendly,
            ToneAdjustment::MoreProfessional,
            ToneAdjustment::MoreDirective,
            ToneAdjustment::MoreSupportive,
            ToneAdjustment::MoreEngaging,
            ToneAdjustment::MoreNeutral,
        ],
    })
}

/// Get adaptation suggestions for specific content and audience
#[command]
pub async fn get_adaptation_suggestions(
    content: String,
    target_audience: AudienceType,
    adapter_state: State<'_, ContentAdapterState>,
) -> Result<AdaptationSuggestionsResponse, String> {
    async fn inner(
        content: String,
        target_audience: AudienceType,
        adapter_state: State<'_, ContentAdapterState>,
    ) -> Result<AdaptationSuggestionsResponse> {
        let state = adapter_state.lock().await;
        let adapter = state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Content adapter not initialized"))?;

        // Analyze content to get metrics
        let metrics = adapter.analyze_content_metrics(&content)?;

        // Get suggestions for the target audience
        let suggestions = adapter.get_adaptation_suggestions(&target_audience, &metrics);

        // Create recommended configuration
        let recommended_config = AdaptationConfig {
            target_audience: target_audience.clone(),
            purpose: AdaptationPurpose::Simplify, // Default, could be smarter
            complexity_level: match target_audience {
                AudienceType::Expert | AudienceType::Technical => ComplexityLevel::Advanced,
                AudienceType::Executive | AudienceType::Academic => ComplexityLevel::Intermediate,
                AudienceType::Student => ComplexityLevel::Intermediate,
                AudienceType::Beginner | AudienceType::Child => ComplexityLevel::Elementary,
                _ => ComplexityLevel::Basic,
            },
            tone_adjustments: vec![],
            preserve_technical_terms: matches!(
                target_audience,
                AudienceType::Technical | AudienceType::Expert
            ),
            preserve_structure: true,
            target_length_ratio: match target_audience {
                AudienceType::Executive => 0.7, // Shorter for executives
                AudienceType::Child | AudienceType::Beginner => 0.8, // Simpler, shorter
                AudienceType::Academic | AudienceType::Expert => 1.2, // More detail
                _ => 1.0,
            },
            use_examples: !matches!(target_audience, AudienceType::Executive),
            include_definitions: matches!(
                target_audience,
                AudienceType::Student | AudienceType::Beginner | AudienceType::General
            ),
            style_consistency: true,
            cultural_sensitivity: true,
            target_style_profile: None,
            enforce_style_consistency: true,
            style_matching_strength: 0.7,
            preserve_author_voice: false,
            apply_style_patterns: false,
        };

        // Calculate quality score (simplified)
        let quality_score = adapter.calculate_audience_appropriateness(
            &metrics,
            &adapter.get_audience_profile(&target_audience),
        );

        Ok(AdaptationSuggestionsResponse {
            suggestions,
            recommended_config,
            quality_score,
        })
    }

    match inner(content, target_audience, adapter_state).await {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}

/// Preview adaptation changes without actually performing the adaptation
#[command]
pub async fn preview_adaptation(
    content: String,
    config: AdaptContentRequest,
    adapter_state: State<'_, ContentAdapterState>,
) -> Result<AdaptationSuggestionsResponse, String> {
    async fn inner(
        content: String,
        config: AdaptContentRequest,
        adapter_state: State<'_, ContentAdapterState>,
    ) -> Result<AdaptationSuggestionsResponse> {
        let state = adapter_state.lock().await;
        let adapter = state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Content adapter not initialized"))?;

        let metrics = adapter.analyze_content_metrics(&content)?;
        let suggestions = adapter.get_adaptation_suggestions(&config.target_audience, &metrics);
        let target_audience_clone = config.target_audience.clone();

        let adaptation_config = AdaptationConfig {
            target_audience: config.target_audience,
            purpose: config.purpose,
            complexity_level: config.complexity_level,
            tone_adjustments: config.tone_adjustments,
            preserve_technical_terms: config.preserve_technical_terms,
            preserve_structure: config.preserve_structure,
            target_length_ratio: config.target_length_ratio,
            use_examples: config.use_examples,
            include_definitions: config.include_definitions,
            style_consistency: config.style_consistency,
            cultural_sensitivity: config.cultural_sensitivity,
            target_style_profile: config.target_style_profile,
            enforce_style_consistency: config.enforce_style_consistency,
            style_matching_strength: config.style_matching_strength,
            preserve_author_voice: config.preserve_author_voice,
            apply_style_patterns: config.apply_style_patterns,
        };

        let audience_profile = adapter.get_audience_profile(&target_audience_clone);
        let quality_score = adapter.calculate_audience_appropriateness(&metrics, &audience_profile);

        Ok(AdaptationSuggestionsResponse {
            suggestions,
            recommended_config: adaptation_config,
            quality_score,
        })
    }

    match inner(content, config, adapter_state).await {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}

/// Get content adaptation status and statistics
#[command]
pub async fn get_adaptation_status(
    adapter_state: State<'_, ContentAdapterState>,
) -> Result<bool, String> {
    let state = adapter_state.lock().await;
    Ok(state.is_some())
}

/// Validate adaptation configuration
#[command]
pub async fn validate_adaptation_config(
    config: AdaptContentRequest,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    // Validate target length ratio
    if config.target_length_ratio < 0.1 || config.target_length_ratio > 5.0 {
        warnings.push("Target length ratio should be between 0.1 and 5.0".to_string());
    }

    // Check for conflicting settings
    if config.preserve_technical_terms
        && matches!(
            config.target_audience,
            AudienceType::Child | AudienceType::Beginner
        )
    {
        warnings.push(
            "Preserving technical terms may not be appropriate for child/beginner audiences"
                .to_string(),
        );
    }

    if config.complexity_level == ComplexityLevel::Expert
        && matches!(
            config.target_audience,
            AudienceType::Child | AudienceType::Beginner
        )
    {
        warnings.push("Expert complexity level conflicts with beginner/child audience".to_string());
    }

    if config.target_length_ratio < 0.5 && matches!(config.purpose, AdaptationPurpose::Elaborate) {
        warnings.push("Short target length conflicts with elaboration purpose".to_string());
    }

    if config.target_length_ratio > 1.5
        && matches!(
            config.purpose,
            AdaptationPurpose::Summarize | AdaptationPurpose::Simplify
        )
    {
        warnings.push(
            "Long target length conflicts with summarization/simplification purpose".to_string(),
        );
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_adaptation_options() {
        let result = get_adaptation_options().await;
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(!options.audiences.is_empty());
        assert!(!options.purposes.is_empty());
        assert!(!options.complexity_levels.is_empty());
        assert!(!options.tone_adjustments.is_empty());
    }

    #[tokio::test]
    async fn test_validate_adaptation_config() {
        let config = AdaptContentRequest {
            content: "Test content".to_string(),
            target_audience: AudienceType::Child,
            purpose: AdaptationPurpose::Simplify,
            complexity_level: ComplexityLevel::Expert, // This should trigger a warning
            tone_adjustments: vec![],
            preserve_technical_terms: true, // This should trigger a warning
            preserve_structure: true,
            target_length_ratio: 1.0,
            use_examples: true,
            include_definitions: true,
            style_consistency: true,
            cultural_sensitivity: true,
            target_style_profile: None,
            enforce_style_consistency: true,
            style_matching_strength: 0.7,
            preserve_author_voice: false,
            apply_style_patterns: false,
        };

        let result = validate_adaptation_config(config).await;
        assert!(result.is_ok());

        let warnings = result.unwrap();
        assert!(!warnings.is_empty()); // Should have warnings for the conflicting settings
    }

    #[test]
    fn test_adaptation_request_serialization() {
        let request = AdaptContentRequest {
            content: "Test content".to_string(),
            target_audience: AudienceType::Technical,
            purpose: AdaptationPurpose::Formalize,
            complexity_level: ComplexityLevel::Advanced,
            tone_adjustments: vec![ToneAdjustment::MoreProfessional],
            preserve_technical_terms: true,
            preserve_structure: false,
            target_length_ratio: 1.2,
            use_examples: false,
            include_definitions: false,
            style_consistency: true,
            cultural_sensitivity: true,
            target_style_profile: None,
            enforce_style_consistency: true,
            style_matching_strength: 0.8,
            preserve_author_voice: true,
            apply_style_patterns: true,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: AdaptContentRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.content, deserialized.content);
        assert_eq!(request.target_audience, deserialized.target_audience);
        assert_eq!(request.purpose, deserialized.purpose);
    }
}
