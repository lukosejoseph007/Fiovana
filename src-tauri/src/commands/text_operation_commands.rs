// src-tauri/src/commands/text_operation_commands.rs
use crate::ai::text_operations::{
    DocumentContext, TextOperation, TextOperationProcessor, TextOperationResult,
};
use crate::commands::ai_commands::AIState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Request structure for text operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOperationRequest {
    pub text: String,
    pub operation: TextOperation,
    pub context: Option<DocumentContext>,
}

/// Execute a text operation with AI
#[tauri::command]
pub async fn execute_text_operation(
    request: TextOperationRequest,
    ai_state: State<'_, AIState>,
) -> Result<TextOperationResult, String> {
    // Get AI orchestrator
    let state_guard = ai_state.lock().await;
    let orchestrator = state_guard.as_ref().ok_or_else(|| {
        "AI system not initialized. Please initialize AI in settings first.".to_string()
    })?;

    // Create processor
    let processor = TextOperationProcessor::new();

    // Execute operation
    processor
        .execute(
            request.text,
            request.operation,
            request.context,
            orchestrator,
        )
        .await
}

/// Get list of available text operations
#[tauri::command]
pub fn get_available_text_operations() -> Result<Vec<String>, String> {
    Ok(TextOperationProcessor::get_available_operations())
}

/// Get description for a specific operation
#[tauri::command]
pub fn get_text_operation_description(operation_name: String) -> Result<Option<String>, String> {
    Ok(TextOperationProcessor::get_operation_description(
        &operation_name,
    ))
}

/// Get all operations with their descriptions
#[tauri::command]
pub fn get_text_operations_info() -> Result<Vec<OperationInfo>, String> {
    let operations = TextOperationProcessor::get_available_operations();
    let mut result = Vec::new();

    for op in operations {
        if let Some(desc) = TextOperationProcessor::get_operation_description(&op) {
            result.push(OperationInfo {
                name: op.clone(),
                description: desc,
                requires_params: matches!(op.as_str(), "Rewrite" | "Translate" | "Custom"),
            });
        }
    }

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationInfo {
    pub name: String,
    pub description: String,
    pub requires_params: bool,
}

/// Test text operations with sample text
#[tauri::command]
pub async fn test_text_operations(
    sample_text: Option<String>,
    ai_state: State<'_, AIState>,
) -> Result<Vec<TextOperationResult>, String> {
    let text =
        sample_text.unwrap_or_else(|| "The quick brown fox jumps over the lazy dog.".to_string());

    let operations = vec![
        TextOperation::Define,
        TextOperation::Explain,
        TextOperation::Simplify,
        TextOperation::Improve,
    ];

    let mut results = Vec::new();

    for operation in operations {
        let request = TextOperationRequest {
            text: text.clone(),
            operation,
            context: None,
        };

        match execute_text_operation(request, ai_state.clone()).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::warn!("Test operation failed: {}", e);
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_text_operations() {
        let result = get_available_text_operations();
        assert!(result.is_ok());
        let operations = result.unwrap();
        assert!(!operations.is_empty());
    }

    #[test]
    fn test_get_text_operation_description() {
        let result = get_text_operation_description("Define".to_string());
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_get_text_operations_info() {
        let result = get_text_operations_info();
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(!info.is_empty());
        assert!(info.iter().any(|op| op.name == "Define"));
    }
}
