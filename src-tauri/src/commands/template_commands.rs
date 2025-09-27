// src-tauri/src/commands/template_commands.rs
// Tauri commands for template management system

use crate::document::{
    AudienceLevel, OutputTemplate, TemplateDefinition, TemplateManager, TemplateOutputFormat,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

/// Template operation result
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOperationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Template generation request
#[derive(Debug, Deserialize)]
pub struct TemplateGenerationRequest {
    #[allow(dead_code)]
    pub template_type: OutputTemplate,
    pub variables: HashMap<String, String>,
    pub output_format: TemplateOutputFormat,
    #[allow(dead_code)]
    pub audience_level: AudienceLevel,
}

/// Template search criteria
#[derive(Debug, Deserialize)]
pub struct TemplateSearchCriteria {
    pub template_type: Option<OutputTemplate>,
    pub audience_level: Option<AudienceLevel>,
    pub output_format: Option<TemplateOutputFormat>,
    pub search_term: Option<String>,
}

// Global template manager state
pub type TemplateManagerState = Mutex<TemplateManager>;

/// Initialize template manager
#[tauri::command]
pub async fn initialize_template_manager(
    state: State<'_, TemplateManagerState>,
) -> Result<TemplateOperationResult, String> {
    let mut manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    match manager.initialize_default_templates() {
        Ok(_) => Ok(TemplateOperationResult {
            success: true,
            message: "Template manager initialized successfully".to_string(),
            data: None,
        }),
        Err(e) => Ok(TemplateOperationResult {
            success: false,
            message: format!("Failed to initialize template manager: {}", e),
            data: None,
        }),
    }
}

/// Create a new template
#[tauri::command]
pub async fn create_template(
    state: State<'_, TemplateManagerState>,
    template: TemplateDefinition,
) -> Result<TemplateOperationResult, String> {
    let mut manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    match manager.create_template(template) {
        Ok(template_id) => Ok(TemplateOperationResult {
            success: true,
            message: "Template created successfully".to_string(),
            data: Some(serde_json::json!({ "template_id": template_id })),
        }),
        Err(e) => Ok(TemplateOperationResult {
            success: false,
            message: format!("Failed to create template: {}", e),
            data: None,
        }),
    }
}

/// Get a template by ID
#[tauri::command]
pub async fn get_template(
    state: State<'_, TemplateManagerState>,
    template_id: String,
) -> Result<TemplateOperationResult, String> {
    let manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    match manager.get_template(&template_id) {
        Some(template) => Ok(TemplateOperationResult {
            success: true,
            message: "Template retrieved successfully".to_string(),
            data: Some(
                serde_json::to_value(template)
                    .map_err(|e| format!("Serialization error: {}", e))?,
            ),
        }),
        None => Ok(TemplateOperationResult {
            success: false,
            message: "Template not found".to_string(),
            data: None,
        }),
    }
}

/// Update an existing template
#[tauri::command]
pub async fn update_template(
    state: State<'_, TemplateManagerState>,
    template_id: String,
    template: TemplateDefinition,
) -> Result<TemplateOperationResult, String> {
    let mut manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    match manager.update_template(&template_id, template) {
        Ok(_) => Ok(TemplateOperationResult {
            success: true,
            message: "Template updated successfully".to_string(),
            data: None,
        }),
        Err(e) => Ok(TemplateOperationResult {
            success: false,
            message: format!("Failed to update template: {}", e),
            data: None,
        }),
    }
}

/// Delete a template
#[tauri::command]
pub async fn delete_template(
    state: State<'_, TemplateManagerState>,
    template_id: String,
) -> Result<TemplateOperationResult, String> {
    let mut manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    match manager.delete_template(&template_id) {
        Ok(_) => Ok(TemplateOperationResult {
            success: true,
            message: "Template deleted successfully".to_string(),
            data: None,
        }),
        Err(e) => Ok(TemplateOperationResult {
            success: false,
            message: format!("Failed to delete template: {}", e),
            data: None,
        }),
    }
}

/// List all available templates
#[tauri::command]
pub async fn list_templates(
    state: State<'_, TemplateManagerState>,
) -> Result<TemplateOperationResult, String> {
    let manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    let templates = manager.list_templates();
    Ok(TemplateOperationResult {
        success: true,
        message: format!("Retrieved {} templates", templates.len()),
        data: Some(
            serde_json::to_value(templates).map_err(|e| format!("Serialization error: {}", e))?,
        ),
    })
}

/// Search templates by criteria
#[tauri::command]
pub async fn search_templates(
    state: State<'_, TemplateManagerState>,
    criteria: TemplateSearchCriteria,
) -> Result<TemplateOperationResult, String> {
    let manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Convert criteria to search parameters
    let mut results = manager.list_templates();

    // Filter by template type
    if let Some(template_type) = criteria.template_type {
        results.retain(|template| template.template_type == template_type);
    }

    // Filter by audience level
    if let Some(audience_level) = criteria.audience_level {
        results.retain(|template| template.audience_level == audience_level);
    }

    // Filter by output format
    if let Some(output_format) = criteria.output_format {
        results.retain(|template| template.supported_formats.contains(&output_format));
    }

    // Filter by search term (case-insensitive search in name and description)
    if let Some(search_term) = criteria.search_term {
        let search_term = search_term.to_lowercase();
        results.retain(|template| {
            template.name.to_lowercase().contains(&search_term)
                || template.description.to_lowercase().contains(&search_term)
        });
    }

    Ok(TemplateOperationResult {
        success: true,
        message: format!("Found {} matching templates", results.len()),
        data: Some(
            serde_json::to_value(results).map_err(|e| format!("Serialization error: {}", e))?,
        ),
    })
}

/// Generate content from template
#[tauri::command]
pub async fn generate_from_template(
    state: State<'_, TemplateManagerState>,
    template_id: String,
    request: TemplateGenerationRequest,
) -> Result<TemplateOperationResult, String> {
    let manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    match manager.generate_from_template(&template_id, &request.variables, &request.output_format) {
        Ok(generated_content) => Ok(TemplateOperationResult {
            success: true,
            message: "Content generated successfully".to_string(),
            data: Some(serde_json::json!({
                "content": generated_content,
                "format": request.output_format
            })),
        }),
        Err(e) => Ok(TemplateOperationResult {
            success: false,
            message: format!("Failed to generate content: {}", e),
            data: None,
        }),
    }
}

/// Validate template definition
#[tauri::command]
pub async fn validate_template(
    template: TemplateDefinition,
) -> Result<TemplateOperationResult, String> {
    let temp_manager = TemplateManager::new_default();
    match temp_manager.validate_template(&template) {
        Ok(_) => Ok(TemplateOperationResult {
            success: true,
            message: "Template is valid".to_string(),
            data: None,
        }),
        Err(e) => Ok(TemplateOperationResult {
            success: false,
            message: format!("Template validation failed: {}", e),
            data: None,
        }),
    }
}

/// Get template statistics
#[tauri::command]
pub async fn get_template_statistics(
    state: State<'_, TemplateManagerState>,
) -> Result<TemplateOperationResult, String> {
    let manager = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    let stats = manager.get_statistics();
    Ok(TemplateOperationResult {
        success: true,
        message: "Statistics retrieved successfully".to_string(),
        data: Some(serde_json::to_value(stats).map_err(|e| format!("Serialization error: {}", e))?),
    })
}

/// Get available template types
#[tauri::command]
pub async fn get_template_types() -> Result<TemplateOperationResult, String> {
    let template_type_names = vec![
        "TrainingManual",
        "QuickReference",
        "Presentation",
        "Assessment",
        "Report",
        "Documentation",
        "Summary",
        "Checklist",
    ];

    Ok(TemplateOperationResult {
        success: true,
        message: "Template types retrieved successfully".to_string(),
        data: Some(
            serde_json::to_value(template_type_names)
                .map_err(|e| format!("Serialization error: {}", e))?,
        ),
    })
}

/// Get supported output formats
#[tauri::command]
pub async fn get_output_formats() -> Result<TemplateOperationResult, String> {
    let output_formats = vec![
        TemplateOutputFormat::Markdown,
        TemplateOutputFormat::HTML,
        TemplateOutputFormat::LaTeX,
        TemplateOutputFormat::Word,
        TemplateOutputFormat::PDF,
        TemplateOutputFormat::PowerPoint,
        TemplateOutputFormat::JSON,
    ];

    Ok(TemplateOperationResult {
        success: true,
        message: "Output formats retrieved successfully".to_string(),
        data: Some(
            serde_json::to_value(output_formats)
                .map_err(|e| format!("Serialization error: {}", e))?,
        ),
    })
}

/// Get audience levels
#[tauri::command]
pub async fn get_audience_levels() -> Result<TemplateOperationResult, String> {
    let audience_levels = vec![
        AudienceLevel::Beginner,
        AudienceLevel::Intermediate,
        AudienceLevel::Advanced,
        AudienceLevel::Expert,
        AudienceLevel::Mixed,
    ];

    Ok(TemplateOperationResult {
        success: true,
        message: "Audience levels retrieved successfully".to_string(),
        data: Some(
            serde_json::to_value(audience_levels)
                .map_err(|e| format!("Serialization error: {}", e))?,
        ),
    })
}
