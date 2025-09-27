// src-tauri/src/commands/format_conversion_commands.rs
// Tauri commands for document format conversion functionality

use crate::document::{
    ConversionOptions, ConversionResult, DocumentFormat, FormatConverter, QualitySettings,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::command;

/// Request structure for document format conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertDocumentRequest {
    pub input_path: String,
    pub output_path: String,
    pub source_format: DocumentFormat,
    pub target_format: DocumentFormat,
    pub options: Option<ConversionOptionsJson>,
}

/// JSON-serializable version of ConversionOptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionOptionsJson {
    pub preserve_formatting: bool,
    pub include_metadata: bool,
    pub template_path: Option<String>,
    pub style_options: HashMap<String, String>,
    pub quality_settings: QualitySettingsJson,
}

/// JSON-serializable version of QualitySettings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettingsJson {
    pub image_quality: u8,
    pub compression_level: u8,
    pub preserve_hyperlinks: bool,
    pub preserve_images: bool,
    pub preserve_tables: bool,
    pub font_embedding: bool,
}

/// Response for format conversion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConversionResponse {
    pub success: bool,
    pub result: Option<ConversionResult>,
    pub error: Option<String>,
}

/// Response for supported formats query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedFormatsResponse {
    pub input_formats: Vec<DocumentFormat>,
    pub output_formats: Vec<DocumentFormat>,
}

/// Response for conversion capability check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionCapabilityResponse {
    pub supported: bool,
    pub capability_level: String,
    pub estimated_quality: String,
    pub warnings: Vec<String>,
}

/// Convert a document from one format to another
#[command]
pub async fn convert_document_format(
    request: ConvertDocumentRequest,
) -> Result<FormatConversionResponse, String> {
    async fn inner(request: ConvertDocumentRequest) -> Result<FormatConversionResponse> {
        // Create temporary directory for conversion operations
        let temp_dir = std::env::temp_dir().join("proxemic_conversions");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create temp directory: {}", e))?;

        let converter = FormatConverter::new(temp_dir);

        // Convert JSON options to internal format
        let conversion_options = if let Some(json_options) = request.options {
            Some(ConversionOptions {
                source_format: request.source_format.clone(),
                target_format: request.target_format.clone(),
                preserve_formatting: json_options.preserve_formatting,
                include_metadata: json_options.include_metadata,
                template_path: json_options.template_path.map(PathBuf::from),
                style_options: json_options.style_options,
                quality_settings: QualitySettings {
                    image_quality: json_options.quality_settings.image_quality,
                    compression_level: json_options.quality_settings.compression_level,
                    preserve_hyperlinks: json_options.quality_settings.preserve_hyperlinks,
                    preserve_images: json_options.quality_settings.preserve_images,
                    preserve_tables: json_options.quality_settings.preserve_tables,
                    font_embedding: json_options.quality_settings.font_embedding,
                },
            })
        } else {
            None
        };

        // Perform the conversion
        let result = converter
            .convert_document(
                &PathBuf::from(&request.input_path),
                &PathBuf::from(&request.output_path),
                conversion_options,
            )
            .await?;

        Ok(FormatConversionResponse {
            success: true,
            result: Some(result),
            error: None,
        })
    }

    match inner(request).await {
        Ok(response) => Ok(response),
        Err(e) => Ok(FormatConversionResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get list of supported input and output formats
#[command]
pub async fn get_supported_formats() -> Result<SupportedFormatsResponse, String> {
    let input_formats = FormatConverter::get_supported_input_formats();
    let output_formats = FormatConverter::get_supported_output_formats();

    Ok(SupportedFormatsResponse {
        input_formats,
        output_formats,
    })
}

/// Check if a specific format conversion is supported
#[command]
pub async fn check_conversion_capability(
    source_format: DocumentFormat,
    target_format: DocumentFormat,
) -> Result<ConversionCapabilityResponse, String> {
    let temp_dir = std::env::temp_dir().join("proxemic_conversions");
    let converter = FormatConverter::new(temp_dir);

    let supported = converter.is_conversion_supported(&source_format, &target_format);

    if !supported {
        return Ok(ConversionCapabilityResponse {
            supported: false,
            capability_level: "Not Supported".to_string(),
            estimated_quality: "N/A".to_string(),
            warnings: vec!["This conversion is not currently supported".to_string()],
        });
    }

    // Determine capability level and quality
    let (capability_level, estimated_quality, warnings) = match (&source_format, &target_format) {
        // Same format
        (a, b) if a == b => ("Full Support".to_string(), "Perfect".to_string(), vec![]),

        // High-quality conversions
        (DocumentFormat::Markdown, DocumentFormat::Html) => {
            ("Full Support".to_string(), "Excellent".to_string(), vec![])
        }
        (DocumentFormat::Json, DocumentFormat::Html) => {
            ("Full Support".to_string(), "Excellent".to_string(), vec![])
        }

        // Good conversions with some limitations
        (DocumentFormat::Html, DocumentFormat::Markdown) => (
            "Partial Support".to_string(),
            "Good".to_string(),
            vec!["Some HTML-specific formatting may be lost".to_string()],
        ),

        // Basic text-only conversions
        (_, DocumentFormat::PlainText) => (
            "Basic Support".to_string(),
            "Fair".to_string(),
            vec!["All formatting will be removed".to_string()],
        ),

        // PDF conversions (limited)
        (DocumentFormat::Pdf, _) => (
            "Basic Support".to_string(),
            "Fair".to_string(),
            vec!["PDF text extraction may not preserve layout".to_string()],
        ),
        (_, DocumentFormat::Pdf) => (
            "Partial Support".to_string(),
            "Good".to_string(),
            vec!["Generated PDF may have basic styling".to_string()],
        ),

        // Office document conversions
        (DocumentFormat::Docx, _) => (
            "Partial Support".to_string(),
            "Good".to_string(),
            vec!["Complex formatting may be simplified".to_string()],
        ),
        (_, DocumentFormat::Docx) => (
            "Partial Support".to_string(),
            "Good".to_string(),
            vec!["Generated DOCX will have basic formatting".to_string()],
        ),

        // PowerPoint conversions
        (DocumentFormat::PowerPoint, _) => (
            "Basic Support".to_string(),
            "Fair".to_string(),
            vec!["Slide layouts will be converted to linear text".to_string()],
        ),
        (_, DocumentFormat::PowerPoint) => (
            "Basic Support".to_string(),
            "Fair".to_string(),
            vec!["Generated presentation will have simple slide layout".to_string()],
        ),

        // Default
        _ => (
            "Partial Support".to_string(),
            "Good".to_string(),
            vec!["Conversion may require multiple steps".to_string()],
        ),
    };

    Ok(ConversionCapabilityResponse {
        supported: true,
        capability_level,
        estimated_quality,
        warnings,
    })
}

/// Batch convert multiple documents
#[command]
pub async fn batch_convert_documents(
    requests: Vec<ConvertDocumentRequest>,
) -> Result<Vec<FormatConversionResponse>, String> {
    let mut results = Vec::new();

    for request in requests {
        let result = convert_document_format(request).await?;
        results.push(result);
    }

    Ok(results)
}

/// Get default conversion options for a format pair
#[command]
pub async fn get_default_conversion_options(
    _source_format: DocumentFormat,
    target_format: DocumentFormat,
) -> Result<ConversionOptionsJson, String> {
    let default_options = match (&_source_format, &target_format) {
        // High-fidelity conversions
        (DocumentFormat::Markdown, DocumentFormat::Html) => ConversionOptionsJson {
            preserve_formatting: true,
            include_metadata: true,
            template_path: None,
            style_options: HashMap::new(),
            quality_settings: QualitySettingsJson {
                image_quality: 95,
                compression_level: 3,
                preserve_hyperlinks: true,
                preserve_images: true,
                preserve_tables: true,
                font_embedding: false,
            },
        },

        // Document to PDF
        (_, DocumentFormat::Pdf) => ConversionOptionsJson {
            preserve_formatting: true,
            include_metadata: true,
            template_path: None,
            style_options: HashMap::new(),
            quality_settings: QualitySettingsJson {
                image_quality: 85,
                compression_level: 6,
                preserve_hyperlinks: true,
                preserve_images: true,
                preserve_tables: true,
                font_embedding: true,
            },
        },

        // To plain text
        (_, DocumentFormat::PlainText) => ConversionOptionsJson {
            preserve_formatting: false,
            include_metadata: false,
            template_path: None,
            style_options: HashMap::new(),
            quality_settings: QualitySettingsJson {
                image_quality: 50,
                compression_level: 8,
                preserve_hyperlinks: false,
                preserve_images: false,
                preserve_tables: false,
                font_embedding: false,
            },
        },

        // Default settings
        _ => ConversionOptionsJson {
            preserve_formatting: true,
            include_metadata: true,
            template_path: None,
            style_options: HashMap::new(),
            quality_settings: QualitySettingsJson {
                image_quality: 85,
                compression_level: 6,
                preserve_hyperlinks: true,
                preserve_images: true,
                preserve_tables: true,
                font_embedding: false,
            },
        },
    };

    Ok(default_options)
}

/// Validate conversion options
#[command]
pub async fn validate_conversion_options(
    _source_format: DocumentFormat,
    target_format: DocumentFormat,
    options: ConversionOptionsJson,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    // Check quality settings
    if options.quality_settings.image_quality > 100 {
        warnings.push("Image quality cannot exceed 100%".to_string());
    }

    if options.quality_settings.compression_level > 10 {
        warnings.push("Compression level cannot exceed 10".to_string());
    }

    // Check format-specific options
    match target_format {
        DocumentFormat::PlainText => {
            if options.preserve_formatting {
                warnings.push(
                    "Formatting cannot be preserved when converting to plain text".to_string(),
                );
            }
            if options.quality_settings.preserve_images {
                warnings
                    .push("Images cannot be preserved when converting to plain text".to_string());
            }
        }
        DocumentFormat::Pdf => {
            if options.quality_settings.image_quality < 50 {
                warnings.push("Low image quality may result in poor PDF appearance".to_string());
            }
        }
        _ => {}
    }

    // Check template path if provided
    if let Some(template_path) = &options.template_path {
        if !std::path::Path::new(template_path).exists() {
            warnings.push(format!("Template file does not exist: {}", template_path));
        }
    }

    Ok(warnings)
}

/// Get conversion statistics for a completed conversion
#[command]
pub async fn get_conversion_statistics(
    conversion_result: ConversionResult,
) -> Result<HashMap<String, String>, String> {
    let mut stats = HashMap::new();

    stats.insert(
        "conversion_time_ms".to_string(),
        conversion_result.conversion_time_ms.to_string(),
    );
    stats.insert(
        "file_size_bytes".to_string(),
        conversion_result.file_size_bytes.to_string(),
    );
    stats.insert(
        "target_format".to_string(),
        format!("{:?}", conversion_result.format_info.format),
    );
    stats.insert(
        "features_preserved".to_string(),
        conversion_result
            .format_info
            .features_preserved
            .len()
            .to_string(),
    );
    stats.insert(
        "features_lost".to_string(),
        conversion_result
            .format_info
            .features_lost
            .len()
            .to_string(),
    );
    stats.insert(
        "warnings_count".to_string(),
        conversion_result.warnings.len().to_string(),
    );

    // Calculate file size in human-readable format
    let file_size_mb = conversion_result.file_size_bytes as f64 / (1024.0 * 1024.0);
    if file_size_mb > 1.0 {
        stats.insert(
            "file_size_human".to_string(),
            format!("{:.2} MB", file_size_mb),
        );
    } else {
        let file_size_kb = conversion_result.file_size_bytes as f64 / 1024.0;
        stats.insert(
            "file_size_human".to_string(),
            format!("{:.2} KB", file_size_kb),
        );
    }

    // Calculate conversion speed
    if conversion_result.conversion_time_ms > 0 {
        let mb_per_second = (file_size_mb * 1000.0) / conversion_result.conversion_time_ms as f64;
        stats.insert(
            "conversion_speed_mbps".to_string(),
            format!("{:.2}", mb_per_second),
        );
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_supported_formats() {
        let result = get_supported_formats().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.input_formats.is_empty());
        assert!(!response.output_formats.is_empty());
        assert!(response.input_formats.contains(&DocumentFormat::Markdown));
        assert!(response.output_formats.contains(&DocumentFormat::Html));
    }

    #[tokio::test]
    async fn test_check_conversion_capability() {
        let result =
            check_conversion_capability(DocumentFormat::Markdown, DocumentFormat::Html).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.supported);
        assert_eq!(response.capability_level, "Full Support");
    }

    #[tokio::test]
    async fn test_get_default_conversion_options() {
        let result =
            get_default_conversion_options(DocumentFormat::Markdown, DocumentFormat::Html).await;

        assert!(result.is_ok());
        let options = result.unwrap();
        assert!(options.preserve_formatting);
        assert!(options.include_metadata);
    }

    #[tokio::test]
    async fn test_validate_conversion_options() {
        let options = ConversionOptionsJson {
            preserve_formatting: true,
            include_metadata: true,
            template_path: None,
            style_options: HashMap::new(),
            quality_settings: QualitySettingsJson {
                image_quality: 150, // Invalid value
                compression_level: 5,
                preserve_hyperlinks: true,
                preserve_images: true,
                preserve_tables: true,
                font_embedding: false,
            },
        };

        let result =
            validate_conversion_options(DocumentFormat::Markdown, DocumentFormat::Html, options)
                .await;

        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(!warnings.is_empty());
        assert!(warnings
            .iter()
            .any(|w| w.contains("Image quality cannot exceed 100%")));
    }
}
