// src-tauri/src/commands/document_generation_commands.rs
// Tauri commands for document generation

use crate::ai::AIConfig;
use crate::document::{
    convert_parsed_content_to_document, AudienceType, DocumentContent, DocumentGenerator,
    GenerationOptions, OutputFormat, OutputGenerationConfig, OutputGenerationResult,
    OutputGenerator, SourceContent,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentRequest {
    pub content: DocumentContent,
    pub options: GenerationOptions,
    pub output_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentResponse {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub generation_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateFromTextRequest {
    pub title: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub options: GenerationOptions,
    pub output_filename: String,
}

/// Request for unified output generation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOutputRequest {
    pub source_content: SourceContent,
    pub config: OutputGenerationConfig,
}

/// Request for conversation-based document generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateFromConversationRequest {
    pub conversation_content: String,
    pub generation_request: String,
    pub target_audience: AudienceType,
    pub output_format: OutputFormat,
    pub output_filename: String,
}

/// Response for output generation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputGenerationResponse {
    pub success: bool,
    pub result: Option<OutputGenerationResult>,
    pub error: Option<String>,
}

// Document generator state
pub struct DocumentGeneratorState {
    pub generator: Arc<Mutex<Option<DocumentGenerator>>>,
    pub output_generator: Arc<Mutex<Option<OutputGenerator>>>,
    pub output_directory: Arc<Mutex<PathBuf>>,
}

impl Default for DocumentGeneratorState {
    fn default() -> Self {
        let output_dir = std::env::temp_dir().join("fiovana_outputs");
        Self {
            generator: Arc::new(Mutex::new(None)),
            output_generator: Arc::new(Mutex::new(None)),
            output_directory: Arc::new(Mutex::new(output_dir)),
        }
    }
}

pub type DocumentGeneratorAppState = Arc<DocumentGeneratorState>;

#[tauri::command]
pub async fn init_document_generator(
    generator_state: State<'_, DocumentGeneratorAppState>,
    output_directory: Option<String>,
    ai_config: Option<AIConfig>,
) -> Result<bool, String> {
    let output_dir = match output_directory {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir().join("fiovana_outputs"),
    };

    // Update the output directory
    {
        let mut dir_lock = generator_state.output_directory.lock().await;
        *dir_lock = output_dir.clone();
    }

    // Create and store the basic generator
    let generator = DocumentGenerator::new(output_dir.clone());
    {
        let mut gen_lock = generator_state.generator.lock().await;
        *gen_lock = Some(generator);
    }

    // Create and store the output generator if AI config is provided
    if let Some(ai_cfg) = ai_config {
        match OutputGenerator::new(ai_cfg, output_dir) {
            Ok(output_gen) => {
                let mut output_gen_lock = generator_state.output_generator.lock().await;
                *output_gen_lock = Some(output_gen);
                tracing::info!("Output generator initialized with AI configuration");
            }
            Err(e) => {
                tracing::warn!("Failed to initialize output generator: {}", e);
                // Continue without output generator
            }
        }
    }

    tracing::info!("Document generator initialized");
    Ok(true)
}

#[tauri::command]
pub async fn generate_document(
    generator_state: State<'_, DocumentGeneratorAppState>,
    request: GenerateDocumentRequest,
) -> Result<GenerateDocumentResponse, String> {
    let start_time = std::time::Instant::now();

    let generator_lock = generator_state.generator.lock().await;
    let generator = match generator_lock.as_ref() {
        Some(gen) => gen,
        None => {
            return Ok(GenerateDocumentResponse {
                success: false,
                output_path: None,
                error: Some("Document generator not initialized".to_string()),
                generation_time_ms: 0,
            });
        }
    };

    match generator
        .generate_document(&request.content, &request.options, &request.output_filename)
        .await
    {
        Ok(output_path) => {
            let generation_time = start_time.elapsed().as_millis() as u64;
            tracing::info!(
                "Successfully generated document: {} in {}ms",
                output_path.display(),
                generation_time
            );

            Ok(GenerateDocumentResponse {
                success: true,
                output_path: Some(output_path.to_string_lossy().to_string()),
                error: None,
                generation_time_ms: generation_time,
            })
        }
        Err(e) => {
            let error_msg = format!("Document generation failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(GenerateDocumentResponse {
                success: false,
                output_path: None,
                error: Some(error_msg),
                generation_time_ms: start_time.elapsed().as_millis() as u64,
            })
        }
    }
}

#[tauri::command]
pub async fn generate_document_from_text(
    generator_state: State<'_, DocumentGeneratorAppState>,
    request: GenerateFromTextRequest,
) -> Result<GenerateDocumentResponse, String> {
    let start_time = std::time::Instant::now();

    let generator_lock = generator_state.generator.lock().await;
    let generator = match generator_lock.as_ref() {
        Some(gen) => gen,
        None => {
            return Ok(GenerateDocumentResponse {
                success: false,
                output_path: None,
                error: Some("Document generator not initialized".to_string()),
                generation_time_ms: 0,
            });
        }
    };

    // Convert text content to structured document
    let document_content =
        convert_parsed_content_to_document(request.title, &request.content, request.metadata);

    match generator
        .generate_document(
            &document_content,
            &request.options,
            &request.output_filename,
        )
        .await
    {
        Ok(output_path) => {
            let generation_time = start_time.elapsed().as_millis() as u64;
            tracing::info!(
                "Successfully generated document from text: {} in {}ms",
                output_path.display(),
                generation_time
            );

            Ok(GenerateDocumentResponse {
                success: true,
                output_path: Some(output_path.to_string_lossy().to_string()),
                error: None,
                generation_time_ms: generation_time,
            })
        }
        Err(e) => {
            let error_msg = format!("Document generation failed: {}", e);
            tracing::error!("{}", error_msg);

            Ok(GenerateDocumentResponse {
                success: false,
                output_path: None,
                error: Some(error_msg),
                generation_time_ms: start_time.elapsed().as_millis() as u64,
            })
        }
    }
}

#[tauri::command]
pub async fn get_supported_output_formats() -> Result<Vec<String>, String> {
    let formats = DocumentGenerator::get_supported_formats();
    let format_strings: Vec<String> = formats
        .into_iter()
        .map(|f| format!("{:?}", f).to_lowercase())
        .collect();

    Ok(format_strings)
}

#[tauri::command]
pub async fn get_output_directory(
    generator_state: State<'_, DocumentGeneratorAppState>,
) -> Result<String, String> {
    let dir_lock = generator_state.output_directory.lock().await;
    Ok(dir_lock.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn set_output_directory(
    generator_state: State<'_, DocumentGeneratorAppState>,
    directory_path: String,
) -> Result<bool, String> {
    let new_path = PathBuf::from(directory_path);

    // Validate the directory
    if let Some(parent) = new_path.parent() {
        if !parent.exists() {
            return Err("Parent directory does not exist".to_string());
        }
    }

    // Create directory if it doesn't exist
    if !new_path.exists() {
        std::fs::create_dir_all(&new_path)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Update the stored directory
    {
        let mut dir_lock = generator_state.output_directory.lock().await;
        *dir_lock = new_path.clone();
    }

    // Reinitialize the generator with new directory
    let generator = DocumentGenerator::new(new_path);
    {
        let mut gen_lock = generator_state.generator.lock().await;
        *gen_lock = Some(generator);
    }

    tracing::info!("Output directory updated");
    Ok(true)
}

// Test command for development
#[tauri::command]
pub async fn test_document_generation(
    generator_state: State<'_, DocumentGeneratorAppState>,
) -> Result<String, String> {
    let test_content = "# Test Document\n\nThis is a test document for generation.\n\n## Section 1\n\nSome content here.\n\n- Item 1\n- Item 2\n- Item 3\n\n## Section 2\n\nMore content.\n\n```\nCode example\n```";

    let request = GenerateFromTextRequest {
        title: "Test Document".to_string(),
        content: test_content.to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("author".to_string(), "Fiovana Test".to_string());
            meta.insert("date".to_string(), "2024-01-01".to_string());
            meta
        },
        options: GenerationOptions {
            format: OutputFormat::Html,
            template: None,
            style_options: HashMap::new(),
            include_metadata: true,
        },
        output_filename: "test_document.html".to_string(),
    };

    match generate_document_from_text(generator_state, request).await {
        Ok(response) => {
            if response.success {
                Ok(format!(
                    "Test document generated successfully!\nPath: {}\nTime: {}ms",
                    response.output_path.unwrap_or("Unknown".to_string()),
                    response.generation_time_ms
                ))
            } else {
                Err(response.error.unwrap_or("Unknown error".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

/// Generate output using the unified pipeline
#[tauri::command]
pub async fn generate_output_unified(
    generator_state: State<'_, DocumentGeneratorAppState>,
    request: GenerateOutputRequest,
) -> Result<OutputGenerationResponse, String> {
    let output_gen_lock = generator_state.output_generator.lock().await;
    let output_generator = match output_gen_lock.as_ref() {
        Some(gen) => gen,
        None => {
            return Ok(OutputGenerationResponse {
                success: false,
                result: None,
                error: Some("Output generator not initialized. Please call init_document_generator with AI configuration first.".to_string()),
            });
        }
    };

    match output_generator
        .generate_output(request.source_content, request.config)
        .await
    {
        Ok(result) => Ok(OutputGenerationResponse {
            success: result.success,
            result: Some(result),
            error: None,
        }),
        Err(e) => Ok(OutputGenerationResponse {
            success: false,
            result: None,
            error: Some(format!("Output generation failed: {}", e)),
        }),
    }
}

/// Generate document from conversation using the unified pipeline
#[tauri::command]
pub async fn generate_from_conversation_unified(
    generator_state: State<'_, DocumentGeneratorAppState>,
    request: GenerateFromConversationRequest,
) -> Result<OutputGenerationResponse, String> {
    let output_gen_lock = generator_state.output_generator.lock().await;
    let output_generator = match output_gen_lock.as_ref() {
        Some(gen) => gen,
        None => {
            return Ok(OutputGenerationResponse {
                success: false,
                result: None,
                error: Some("Output generator not initialized. Please call init_document_generator with AI configuration first.".to_string()),
            });
        }
    };

    match output_generator
        .generate_from_conversation(
            &request.conversation_content,
            &request.generation_request,
            request.target_audience,
            request.output_format,
            &request.output_filename,
        )
        .await
    {
        Ok(result) => Ok(OutputGenerationResponse {
            success: result.success,
            result: Some(result),
            error: None,
        }),
        Err(e) => Ok(OutputGenerationResponse {
            success: false,
            result: None,
            error: Some(format!("Conversation-based generation failed: {}", e)),
        }),
    }
}

/// Get the status of the output generator
#[tauri::command]
pub async fn get_output_generator_status(
    generator_state: State<'_, DocumentGeneratorAppState>,
) -> Result<bool, String> {
    let output_gen_lock = generator_state.output_generator.lock().await;
    Ok(output_gen_lock.is_some())
}

/// Get available audience types for content adaptation
#[tauri::command]
pub async fn get_available_audience_types() -> Result<Vec<String>, String> {
    let audiences = vec![
        "General",
        "Technical",
        "Executive",
        "Academic",
        "Student",
        "Expert",
        "Beginner",
        "Child",
        "International",
        "Legal",
        "Medical",
        "Business",
    ];
    Ok(audiences.into_iter().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_document_generator_state() {
        let state = DocumentGeneratorState::default();
        assert!(state.output_directory.lock().await.is_absolute());
    }

    #[test]
    fn test_supported_formats() {
        let formats = DocumentGenerator::get_supported_formats();
        assert!(!formats.is_empty());
        assert!(formats.len() >= 3); // Should have at least Markdown, HTML, PlainText
    }
}
