// src-tauri/src/document/output_generator.rs
// Unified Output Generation Pipeline: Source content → AI adaptation → template application → format conversion

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::content_adapter::{
    AdaptationConfig, AdaptationResult, AudienceType, ComplexityLevel, ContentAdapter,
};
use super::document_generator::{
    DocumentContent, DocumentGenerator, DocumentSection, GenerationOptions, OutputFormat,
    SectionType,
};
use super::format_converters::{
    ConversionOptions, DocumentFormat, FormatConverter, QualitySettings,
};
use super::templates::{OutputTemplate, TemplateManager};
use crate::ai::AIConfig;

/// Comprehensive configuration for the output generation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputGenerationConfig {
    // Content adaptation settings
    pub adaptation_config: AdaptationConfig,
    // Template configuration
    pub template: OutputTemplate,
    // Output format and quality settings
    pub output_format: OutputFormat,
    pub quality_settings: QualitySettings,
    // File handling
    pub output_filename: String,
    pub preserve_source_metadata: bool,
    // Pipeline behavior
    pub enable_ai_adaptation: bool,
    pub apply_template: bool,
    pub include_generation_metadata: bool,
}

/// Source content input for the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContent {
    pub title: String,
    pub content: String,
    pub content_type: SourceContentType,
    pub metadata: HashMap<String, String>,
    pub source_path: Option<PathBuf>,
}

/// Types of source content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceContentType {
    PlainText,
    Markdown,
    Html,
    DocumentFragment,
    ConversationTranscript,
    MultipleDocuments(Vec<String>),
}

/// Complete pipeline result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputGenerationResult {
    pub success: bool,
    pub output_path: Option<PathBuf>,
    pub generation_summary: GenerationSummary,
    pub adaptation_result: Option<AdaptationResult>,
    pub template_application_log: Vec<String>,
    pub format_conversion_log: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub processing_time_ms: u64,
    pub tokens_used: Option<u32>,
}

/// Summary of the generation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSummary {
    pub original_word_count: usize,
    pub final_word_count: usize,
    pub content_adaptation_applied: bool,
    pub template_applied: String,
    pub output_format: OutputFormat,
    pub style_patterns_applied: Vec<String>,
    pub quality_score: Option<f64>,
}

/// Unified output generation pipeline
pub struct OutputGenerator {
    content_adapter: ContentAdapter,
    #[allow(dead_code)]
    template_manager: TemplateManager,
    document_generator: DocumentGenerator,
    format_converter: FormatConverter,
    #[allow(dead_code)]
    output_directory: PathBuf,
}

impl OutputGenerator {
    /// Create a new output generator with AI configuration
    pub fn new(ai_config: AIConfig, output_directory: PathBuf) -> Result<Self> {
        let templates_dir = output_directory.join("templates");
        Ok(Self {
            content_adapter: ContentAdapter::new(ai_config),
            template_manager: TemplateManager::new(templates_dir)?,
            document_generator: DocumentGenerator::new(output_directory.clone()),
            format_converter: FormatConverter::new(output_directory.join("temp")),
            output_directory,
        })
    }

    /// Execute the complete output generation pipeline
    pub async fn generate_output(
        &self,
        source: SourceContent,
        config: OutputGenerationConfig,
    ) -> Result<OutputGenerationResult> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut template_application_log = Vec::new();
        let mut format_conversion_log = Vec::new();

        // Step 1: Prepare source content
        let original_word_count = source.content.split_whitespace().count();

        // Step 2: AI-powered content adaptation (if enabled)
        let (adapted_content, adaptation_result) = if config.enable_ai_adaptation {
            match self
                .content_adapter
                .adapt_content(&source.content, Some(config.adaptation_config.clone()))
                .await
            {
                Ok(result) => {
                    let adapted = result.adapted_content.clone();
                    template_application_log
                        .push("Content successfully adapted for target audience".to_string());
                    (adapted, Some(result))
                }
                Err(e) => {
                    warnings.push(format!(
                        "Content adaptation failed: {}. Using original content.",
                        e
                    ));
                    (source.content.clone(), None)
                }
            }
        } else {
            template_application_log.push("AI adaptation skipped by configuration".to_string());
            (source.content.clone(), None)
        };

        // Step 3: Template application (if enabled)
        let structured_content = if config.apply_template {
            match self
                .apply_template(&adapted_content, &source, &config)
                .await
            {
                Ok(content) => {
                    template_application_log.push(format!(
                        "Template {:?} successfully applied",
                        config.template
                    ));
                    content
                }
                Err(e) => {
                    warnings.push(format!(
                        "Template application failed: {}. Using basic structure.",
                        e
                    ));
                    self.create_basic_document_structure(&adapted_content, &source)
                }
            }
        } else {
            template_application_log
                .push("Template application skipped by configuration".to_string());
            self.create_basic_document_structure(&adapted_content, &source)
        };

        // Step 4: Document generation
        let generation_options = GenerationOptions {
            format: config.output_format.clone(),
            template: None, // Template already applied
            style_options: HashMap::new(),
            include_metadata: config.include_generation_metadata,
        };

        let output_path = match self
            .document_generator
            .generate_document(
                &structured_content,
                &generation_options,
                &config.output_filename,
            )
            .await
        {
            Ok(path) => {
                format_conversion_log
                    .push("Document generation completed successfully".to_string());
                Some(path)
            }
            Err(e) => {
                errors.push(format!("Document generation failed: {}", e));
                None
            }
        };

        // Step 5: Format conversion (if needed)
        let final_output_path = if let Some(ref path) = output_path {
            match self.apply_format_conversion(path, &config).await {
                Ok(converted_path) => {
                    format_conversion_log
                        .push("Format conversion completed successfully".to_string());
                    Some(converted_path)
                }
                Err(e) => {
                    warnings.push(format!(
                        "Format conversion failed: {}. Using original format.",
                        e
                    ));
                    output_path
                }
            }
        } else {
            None
        };

        // Step 6: Generate result summary
        let final_word_count = adapted_content.split_whitespace().count();
        let style_patterns_applied = adaptation_result
            .as_ref()
            .map(|r| r.style_patterns_applied.clone())
            .unwrap_or_default();

        let generation_summary = GenerationSummary {
            original_word_count,
            final_word_count,
            content_adaptation_applied: config.enable_ai_adaptation && adaptation_result.is_some(),
            template_applied: format!("{:?}", config.template),
            output_format: config.output_format,
            style_patterns_applied,
            quality_score: adaptation_result.as_ref().map(|r| r.quality_score),
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        let tokens_used = adaptation_result.as_ref().and_then(|r| r.tokens_used);

        Ok(OutputGenerationResult {
            success: final_output_path.is_some() && errors.is_empty(),
            output_path: final_output_path,
            generation_summary,
            adaptation_result,
            template_application_log,
            format_conversion_log,
            errors,
            warnings,
            processing_time_ms: processing_time,
            tokens_used,
        })
    }

    /// Apply template to adapted content
    async fn apply_template(
        &self,
        adapted_content: &str,
        source: &SourceContent,
        config: &OutputGenerationConfig,
    ) -> Result<DocumentContent> {
        // Generate template-based document structure based on template type
        let template_result = self
            .generate_template_structure(&config.template, adapted_content, source)
            .context("Failed to apply template")?;

        // Convert template result to DocumentContent
        let mut sections = Vec::new();

        // Parse the template result into structured sections
        let lines: Vec<&str> = template_result.lines().collect();
        let mut current_section: Option<DocumentSection> = None;
        let mut current_content = String::new();

        for line in lines {
            if line.starts_with('#') {
                // Save previous section if exists
                if let Some(section) = current_section.take() {
                    sections.push(DocumentSection {
                        title: section.title,
                        content: current_content.trim().to_string(),
                        level: section.level,
                        section_type: section.section_type,
                    });
                    current_content.clear();
                }

                // Start new section
                let level = line.chars().take_while(|&c| c == '#').count() as u8;
                let title = line.trim_start_matches('#').trim().to_string();

                current_section = Some(DocumentSection {
                    title,
                    content: String::new(),
                    level,
                    section_type: SectionType::Heading,
                });
            } else if !line.trim().is_empty() {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Don't forget the last section
        if let Some(section) = current_section {
            sections.push(DocumentSection {
                title: section.title,
                content: current_content.trim().to_string(),
                level: section.level,
                section_type: SectionType::Heading,
            });
        }

        // If no sections were created, create a basic structure
        if sections.is_empty() {
            sections.push(DocumentSection {
                title: "Content".to_string(),
                content: adapted_content.to_string(),
                level: 1,
                section_type: SectionType::Paragraph,
            });
        }

        // Combine metadata
        let mut metadata = source.metadata.clone();
        if config.preserve_source_metadata {
            if let Some(source_path) = &source.source_path {
                metadata.insert(
                    "source_path".to_string(),
                    source_path.to_string_lossy().to_string(),
                );
            }
            metadata.insert(
                "generation_timestamp".to_string(),
                chrono::Utc::now().to_rfc3339(),
            );
            metadata.insert(
                "template_used".to_string(),
                format!("{:?}", config.template),
            );
        }

        Ok(DocumentContent {
            title: source.title.clone(),
            sections,
            metadata,
        })
    }

    /// Generate template structure based on template type
    fn generate_template_structure(
        &self,
        template: &OutputTemplate,
        content: &str,
        source: &SourceContent,
    ) -> Result<String> {
        let structured_content = match template {
            OutputTemplate::TrainingManual { audience: _ } => {
                format!(
                    "# Training Manual: {}\n\n## Overview\n{}\n\n## Learning Objectives\n\n## Content\n{}\n\n## Summary\n\n## Next Steps\n",
                    source.title,
                    "This training manual provides comprehensive guidance on the topic.",
                    content
                )
            }
            OutputTemplate::QuickReference { format: _ } => {
                format!(
                    "# Quick Reference: {}\n\n## Key Points\n{}\n\n## Quick Actions\n\n## Common Issues\n",
                    source.title,
                    content
                )
            }
            OutputTemplate::Presentation {
                slides: _,
                style: _,
            } => {
                format!(
                    "# Presentation: {}\n\n## Slide 1: Introduction\n\n## Slide 2: Main Content\n{}\n\n## Slide 3: Conclusion\n",
                    source.title,
                    content
                )
            }
            OutputTemplate::Assessment {
                questions: _,
                difficulty: _,
            } => {
                format!(
                    "# Assessment: {}\n\n## Instructions\n\n## Questions\n\n## Content Review\n{}\n",
                    source.title,
                    content
                )
            }
            OutputTemplate::Report {
                report_type: _,
                sections,
            } => {
                let mut report = format!("# Report: {}\n\n", source.title);
                for section in sections {
                    report.push_str(&format!("## {}\n\n", section));
                }
                report.push_str(&format!("## Content\n{}\n", content));
                report
            }
            OutputTemplate::Documentation {
                doc_type: _,
                structure: _,
            } => {
                format!(
                    "# Documentation: {}\n\n## Introduction\n\n## Overview\n{}\n\n## Details\n\n## Examples\n\n## References\n",
                    source.title,
                    content
                )
            }
            OutputTemplate::Summary {
                length: _,
                focus: _,
            } => {
                format!(
                    "# Summary: {}\n\n## Key Points\n{}\n\n## Conclusion\n",
                    source.title, content
                )
            }
            OutputTemplate::Checklist {
                category: _,
                items: _,
            } => {
                format!(
                    "# Checklist: {}\n\n## Overview\n{}\n\n## Checklist Items\n\n- [ ] Item 1\n- [ ] Item 2\n- [ ] Item 3\n",
                    source.title,
                    content
                )
            }
        };

        Ok(structured_content)
    }

    /// Create basic document structure when template application fails
    fn create_basic_document_structure(
        &self,
        content: &str,
        source: &SourceContent,
    ) -> DocumentContent {
        let sections = vec![DocumentSection {
            title: "Content".to_string(),
            content: content.to_string(),
            level: 1,
            section_type: SectionType::Paragraph,
        }];

        DocumentContent {
            title: source.title.clone(),
            sections,
            metadata: source.metadata.clone(),
        }
    }

    /// Apply format conversion if needed
    async fn apply_format_conversion(
        &self,
        input_path: &Path,
        config: &OutputGenerationConfig,
    ) -> Result<PathBuf> {
        // Determine if conversion is needed
        let input_format = self.detect_format_from_extension(input_path)?;
        let target_format = self.output_format_to_document_format(&config.output_format);

        if input_format == target_format {
            // No conversion needed
            return Ok(input_path.to_path_buf());
        }

        // Set up conversion options
        let conversion_options = ConversionOptions {
            source_format: input_format,
            target_format: target_format.clone(),
            preserve_formatting: true,
            include_metadata: config.include_generation_metadata,
            template_path: None,
            style_options: HashMap::new(),
            quality_settings: config.quality_settings.clone(),
        };

        // Generate output path for converted file
        let output_path = input_path.with_extension(self.get_extension_for_format(&target_format));

        // Perform conversion
        let _conversion_result = self
            .format_converter
            .convert_document(input_path, &output_path, Some(conversion_options))
            .await
            .context("Failed to convert document format")?;

        Ok(output_path)
    }

    /// Detect document format from file extension
    fn detect_format_from_extension(&self, path: &Path) -> Result<DocumentFormat> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("No file extension found"))?;

        match extension.to_lowercase().as_str() {
            "docx" => Ok(DocumentFormat::Docx),
            "pdf" => Ok(DocumentFormat::Pdf),
            "html" | "htm" => Ok(DocumentFormat::Html),
            "md" | "markdown" => Ok(DocumentFormat::Markdown),
            "txt" => Ok(DocumentFormat::PlainText),
            "pptx" => Ok(DocumentFormat::PowerPoint),
            "rtf" => Ok(DocumentFormat::Rtf),
            "odt" => Ok(DocumentFormat::OpenDocument),
            "tex" => Ok(DocumentFormat::LaTeX),
            _ => Err(anyhow!("Unsupported file extension: {}", extension)),
        }
    }

    /// Convert OutputFormat to DocumentFormat
    fn output_format_to_document_format(&self, format: &OutputFormat) -> DocumentFormat {
        match format {
            OutputFormat::Docx => DocumentFormat::Docx,
            OutputFormat::Pdf => DocumentFormat::Pdf,
            OutputFormat::Html => DocumentFormat::Html,
            OutputFormat::Markdown => DocumentFormat::Markdown,
            OutputFormat::PlainText => DocumentFormat::PlainText,
        }
    }

    /// Get file extension for document format
    fn get_extension_for_format(&self, format: &DocumentFormat) -> &str {
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

    /// Generate a document from conversation or natural language input
    pub async fn generate_from_conversation(
        &self,
        conversation_content: &str,
        generation_request: &str,
        target_audience: AudienceType,
        output_format: OutputFormat,
        output_filename: &str,
    ) -> Result<OutputGenerationResult> {
        // Parse the generation request to determine template and configuration
        let (template, adaptation_config) =
            self.parse_generation_request(generation_request, target_audience)?;

        // Create source content from conversation
        let source = SourceContent {
            title: self.extract_title_from_request(generation_request),
            content: conversation_content.to_string(),
            content_type: SourceContentType::ConversationTranscript,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "generation_request".to_string(),
                    generation_request.to_string(),
                );
                metadata.insert("source_type".to_string(), "conversation".to_string());
                metadata
            },
            source_path: None,
        };

        // Create generation configuration
        let config = OutputGenerationConfig {
            adaptation_config,
            template,
            output_format,
            quality_settings: QualitySettings::default(),
            output_filename: output_filename.to_string(),
            preserve_source_metadata: true,
            enable_ai_adaptation: true,
            apply_template: true,
            include_generation_metadata: true,
        };

        // Execute the pipeline
        self.generate_output(source, config).await
    }

    /// Parse natural language generation request
    fn parse_generation_request(
        &self,
        request: &str,
        target_audience: AudienceType,
    ) -> Result<(OutputTemplate, AdaptationConfig)> {
        let request_lower = request.to_lowercase();

        // Determine template based on request keywords
        let template = if request_lower.contains("training manual")
            || request_lower.contains("training guide")
        {
            OutputTemplate::TrainingManual {
                audience: self.audience_type_to_audience_level(&target_audience),
            }
        } else if request_lower.contains("quick reference") || request_lower.contains("cheat sheet")
        {
            OutputTemplate::QuickReference {
                format: super::templates::ReferenceFormat::QuickCard,
            }
        } else if request_lower.contains("presentation") || request_lower.contains("slides") {
            OutputTemplate::Presentation {
                slides: 10, // Default
                style: super::templates::PresentationStyle::Corporate,
            }
        } else if request_lower.contains("assessment")
            || request_lower.contains("test")
            || request_lower.contains("quiz")
        {
            OutputTemplate::Assessment {
                questions: 10, // Default
                difficulty: super::templates::Difficulty::Medium,
            }
        } else if request_lower.contains("report") {
            OutputTemplate::Report {
                report_type: super::templates::ReportType::Analysis,
                sections: vec![
                    "Introduction".to_string(),
                    "Analysis".to_string(),
                    "Conclusion".to_string(),
                ],
            }
        } else if request_lower.contains("documentation") || request_lower.contains("docs") {
            OutputTemplate::Documentation {
                doc_type: super::templates::DocumentationType::UserGuide,
                structure: super::templates::DocumentStructure::Hierarchical,
            }
        } else if request_lower.contains("summary") {
            OutputTemplate::Summary {
                length: super::templates::SummaryType::Overview,
                focus: super::templates::SummaryFocus::General,
            }
        } else if request_lower.contains("checklist") {
            OutputTemplate::Checklist {
                category: super::templates::ChecklistCategory::Process,
                items: 10, // Default
            }
        } else {
            // Default to documentation
            OutputTemplate::Documentation {
                doc_type: super::templates::DocumentationType::UserGuide,
                structure: super::templates::DocumentStructure::Sequential,
            }
        };

        // Create adaptation configuration
        let adaptation_config = AdaptationConfig {
            target_audience: target_audience.clone(),
            purpose: super::content_adapter::AdaptationPurpose::Formalize,
            complexity_level: self.audience_type_to_complexity_level(&target_audience),
            tone_adjustments: vec![],
            preserve_technical_terms: matches!(
                target_audience,
                AudienceType::Technical | AudienceType::Expert
            ),
            preserve_structure: true,
            target_length_ratio: 1.0,
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
            apply_style_patterns: true,
        };

        Ok((template, adaptation_config))
    }

    /// Extract title from generation request
    fn extract_title_from_request(&self, request: &str) -> String {
        // Simple heuristic to extract title
        if let Some(title_start) = request.find("\"") {
            if let Some(title_end) = request[title_start + 1..].find("\"") {
                return request[title_start + 1..title_start + 1 + title_end].to_string();
            }
        }

        // Fallback: use first few words
        let words: Vec<&str> = request.split_whitespace().take(5).collect();
        words.join(" ")
    }

    /// Convert AudienceType to AudienceLevel
    fn audience_type_to_audience_level(
        &self,
        audience: &AudienceType,
    ) -> super::templates::AudienceLevel {
        match audience {
            AudienceType::Beginner | AudienceType::Child => {
                super::templates::AudienceLevel::Beginner
            }
            AudienceType::Student | AudienceType::General => {
                super::templates::AudienceLevel::Intermediate
            }
            AudienceType::Expert | AudienceType::Technical | AudienceType::Academic => {
                super::templates::AudienceLevel::Advanced
            }
            _ => super::templates::AudienceLevel::Intermediate,
        }
    }

    /// Convert AudienceType to ComplexityLevel
    fn audience_type_to_complexity_level(&self, audience: &AudienceType) -> ComplexityLevel {
        match audience {
            AudienceType::Child | AudienceType::Beginner => ComplexityLevel::Elementary,
            AudienceType::General | AudienceType::Student => ComplexityLevel::Basic,
            AudienceType::Business | AudienceType::Executive => ComplexityLevel::Intermediate,
            AudienceType::Academic | AudienceType::Technical => ComplexityLevel::Advanced,
            AudienceType::Expert | AudienceType::Medical | AudienceType::Legal => {
                ComplexityLevel::Expert
            }
            _ => ComplexityLevel::Intermediate,
        }
    }
}

impl Default for OutputGenerationConfig {
    fn default() -> Self {
        Self {
            adaptation_config: AdaptationConfig::default(),
            template: OutputTemplate::Documentation {
                doc_type: super::templates::DocumentationType::UserGuide,
                structure: super::templates::DocumentStructure::Sequential,
            },
            output_format: OutputFormat::Markdown,
            quality_settings: QualitySettings::default(),
            output_filename: "generated_document".to_string(),
            preserve_source_metadata: true,
            enable_ai_adaptation: true,
            apply_template: true,
            include_generation_metadata: true,
        }
    }
}
