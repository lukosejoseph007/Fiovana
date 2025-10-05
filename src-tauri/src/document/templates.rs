// src-tauri/src/document/templates.rs
// Output Template System for Multi-Format Document Generation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Output template types for different document generation scenarios
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputTemplate {
    TrainingManual {
        audience: AudienceLevel,
    },
    QuickReference {
        format: ReferenceFormat,
    },
    Presentation {
        slides: usize,
        style: PresentationStyle,
    },
    Assessment {
        questions: usize,
        difficulty: Difficulty,
    },
    Report {
        report_type: ReportType,
        sections: Vec<String>,
    },
    Documentation {
        doc_type: DocumentationType,
        structure: DocumentStructure,
    },
    Summary {
        length: SummaryType,
        focus: SummaryFocus,
    },
    Checklist {
        category: ChecklistCategory,
        items: usize,
    },
}

/// Target audience levels for content adaptation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Mixed,
}

/// Reference format types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum ReferenceFormat {
    QuickCard,
    Handbook,
    CheatSheet,
    FlowChart,
    Glossary,
    FAQ,
}

/// Presentation style options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresentationStyle {
    Corporate,
    Educational,
    Technical,
    Creative,
    Minimal,
    Detailed,
}

/// Assessment difficulty levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Mixed,
}

/// Report types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportType {
    Executive,
    Technical,
    Progress,
    Analysis,
    Compliance,
    Research,
}

/// Documentation types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum DocumentationType {
    API,
    UserGuide,
    TechnicalSpec,
    Installation,
    Troubleshooting,
    PolicyProcedure,
}

/// Document structure patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentStructure {
    Hierarchical,
    Sequential,
    Modular,
    Reference,
    Tutorial,
    Cookbook,
}

/// Summary types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SummaryType {
    Executive,
    Technical,
    Overview,
    Highlights,
    KeyPoints,
}

/// Summary focus areas
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SummaryFocus {
    ActionItems,
    Decisions,
    Issues,
    Progress,
    Recommendations,
    General,
}

/// Checklist categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChecklistCategory {
    QualityAssurance,
    Safety,
    Compliance,
    Process,
    Review,
    Onboarding,
}

/// Template definition with metadata and content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template_type: OutputTemplate,
    pub output_format: OutputFormat,
    pub supported_formats: Vec<OutputFormat>,
    pub audience_level: AudienceLevel,
    pub content: String,
    pub sections: Vec<TemplateSection>,
    pub variables: HashMap<String, TemplateVariable>,
    pub metadata: TemplateMetadata,
}

/// Output format options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum OutputFormat {
    Markdown,
    HTML,
    LaTeX,
    Word,
    PDF,
    PowerPoint,
    JSON,
    XML,
}

/// Template section definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSection {
    pub id: String,
    pub title: String,
    pub content_template: String,
    pub is_required: bool,
    pub order: usize,
    pub subsections: Vec<TemplateSection>,
    pub content_type: ContentType,
    pub formatting: SectionFormatting,
}

/// Content types for sections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    List,
    Table,
    Image,
    Code,
    Quote,
    Formula,
    Chart,
    Diagram,
    Mixed,
}

/// Section formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionFormatting {
    pub font_size: Option<String>,
    pub font_weight: Option<String>,
    pub alignment: Option<String>,
    pub spacing: Option<String>,
    pub indentation: Option<usize>,
    pub numbering: Option<NumberingStyle>,
}

/// Numbering styles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingStyle {
    Numeric,
    Alphabetic,
    Roman,
    Bullet,
    None,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub variable_type: VariableType,
    pub default_value: Option<String>,
    pub is_required: bool,
    pub validation: Option<VariableValidation>,
}

/// Variable types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Number,
    Date,
    Boolean,
    List,
    Object,
}

/// Variable validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<String>>,
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub version: String,
    pub author: String,
    pub created_date: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub category: String,
    pub language: String,
    pub estimated_generation_time: Option<String>,
}

/// Template manager for loading, storing, and managing templates
pub struct TemplateManager {
    templates_dir: PathBuf,
    loaded_templates: HashMap<String, TemplateDefinition>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new(templates_dir: PathBuf) -> Result<Self> {
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)?;
        }

        Ok(Self {
            templates_dir,
            loaded_templates: HashMap::new(),
        })
    }

    /// Load all templates from the templates directory
    #[allow(dead_code)]
    pub fn load_templates(&mut self) -> Result<()> {
        self.loaded_templates.clear();

        for entry in fs::read_dir(&self.templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                match self.load_template_from_file(&path) {
                    Ok(template) => {
                        self.loaded_templates.insert(template.id.clone(), template);
                    }
                    Err(e) => {
                        eprintln!("Failed to load template from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a template from a YAML file
    #[allow(dead_code)]
    fn load_template_from_file(&self, path: &Path) -> Result<TemplateDefinition> {
        let content = fs::read_to_string(path)?;
        let template: TemplateDefinition = serde_yaml::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse template YAML: {}", e))?;
        Ok(template)
    }

    /// Save a template to a YAML file
    pub fn save_template(&self, template: &TemplateDefinition) -> Result<()> {
        let filename = format!("{}.yaml", template.id);
        let path = self.templates_dir.join(filename);

        let yaml_content = serde_yaml::to_string(template)
            .map_err(|e| anyhow!("Failed to serialize template to YAML: {}", e))?;

        fs::write(path, yaml_content)?;
        Ok(())
    }

    /// Get a template by ID
    pub fn get_template(&self, id: &str) -> Option<&TemplateDefinition> {
        self.loaded_templates.get(id)
    }

    /// Get all loaded templates
    #[allow(dead_code)]
    pub fn get_all_templates(&self) -> &HashMap<String, TemplateDefinition> {
        &self.loaded_templates
    }

    /// Get templates by type
    #[allow(dead_code)]
    pub fn get_templates_by_type(
        &self,
        template_type: &OutputTemplate,
    ) -> Vec<&TemplateDefinition> {
        self.loaded_templates
            .values()
            .filter(|template| {
                std::mem::discriminant(&template.template_type)
                    == std::mem::discriminant(template_type)
            })
            .collect()
    }

    /// Get templates by output format
    #[allow(dead_code)]
    pub fn get_templates_by_format(&self, format: &OutputFormat) -> Vec<&TemplateDefinition> {
        self.loaded_templates
            .values()
            .filter(|template| template.output_format == *format)
            .collect()
    }

    /// Create default templates
    pub fn create_default_templates(&mut self) -> Result<()> {
        // Training Manual Template
        let training_manual = self.create_training_manual_template()?;
        self.loaded_templates
            .insert(training_manual.id.clone(), training_manual.clone());
        self.save_template(&training_manual)?;

        // Quick Reference Template
        let quick_reference = self.create_quick_reference_template()?;
        self.loaded_templates
            .insert(quick_reference.id.clone(), quick_reference.clone());
        self.save_template(&quick_reference)?;

        // Presentation Template
        let presentation = self.create_presentation_template()?;
        self.loaded_templates
            .insert(presentation.id.clone(), presentation.clone());
        self.save_template(&presentation)?;

        // Assessment Template
        let assessment = self.create_assessment_template()?;
        self.loaded_templates
            .insert(assessment.id.clone(), assessment.clone());
        self.save_template(&assessment)?;

        Ok(())
    }

    /// Create a training manual template
    fn create_training_manual_template(&self) -> Result<TemplateDefinition> {
        Ok(TemplateDefinition {
            id: "training_manual_basic".to_string(),
            name: "Basic Training Manual".to_string(),
            description: "Comprehensive training manual template for employee onboarding and skill development".to_string(),
            template_type: OutputTemplate::TrainingManual {
                audience: AudienceLevel::Intermediate
            },
            output_format: OutputFormat::Markdown,
            supported_formats: vec![OutputFormat::Markdown, OutputFormat::HTML, OutputFormat::PDF],
            audience_level: AudienceLevel::Intermediate,
            content: "# {{title}}\n\n## Overview\n{{overview}}\n\n## Content\n{{content}}".to_string(),
            sections: vec![
                TemplateSection {
                    id: "cover".to_string(),
                    title: "Cover Page".to_string(),
                    content_template: "# {{title}}\n\n**Training Manual**\n\nVersion: {{version}}\nDate: {{date}}\nAuthor: {{author}}".to_string(),
                    is_required: true,
                    order: 1,
                    subsections: vec![],
                    content_type: ContentType::Text,
                    formatting: SectionFormatting {
                        font_size: Some("large".to_string()),
                        font_weight: Some("bold".to_string()),
                        alignment: Some("center".to_string()),
                        spacing: Some("double".to_string()),
                        indentation: None,
                        numbering: Some(NumberingStyle::None),
                    },
                },
                TemplateSection {
                    id: "toc".to_string(),
                    title: "Table of Contents".to_string(),
                    content_template: "## Table of Contents\n\n{{auto_generated_toc}}".to_string(),
                    is_required: true,
                    order: 2,
                    subsections: vec![],
                    content_type: ContentType::List,
                    formatting: SectionFormatting {
                        font_size: None,
                        font_weight: None,
                        alignment: None,
                        spacing: None,
                        indentation: Some(0),
                        numbering: Some(NumberingStyle::Numeric),
                    },
                },
                TemplateSection {
                    id: "introduction".to_string(),
                    title: "Introduction".to_string(),
                    content_template: "## Introduction\n\n{{introduction_content}}\n\n### Objectives\n\n{{learning_objectives}}".to_string(),
                    is_required: true,
                    order: 3,
                    subsections: vec![],
                    content_type: ContentType::Text,
                    formatting: SectionFormatting {
                        font_size: None,
                        font_weight: None,
                        alignment: None,
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::Numeric),
                    },
                },
                TemplateSection {
                    id: "main_content".to_string(),
                    title: "Main Content".to_string(),
                    content_template: "{{main_content_sections}}".to_string(),
                    is_required: true,
                    order: 4,
                    subsections: vec![],
                    content_type: ContentType::Mixed,
                    formatting: SectionFormatting {
                        font_size: None,
                        font_weight: None,
                        alignment: None,
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::Numeric),
                    },
                },
            ],
            variables: HashMap::from([
                ("title".to_string(), TemplateVariable {
                    name: "title".to_string(),
                    description: "Training manual title".to_string(),
                    variable_type: VariableType::String,
                    default_value: Some("Training Manual".to_string()),
                    is_required: true,
                    validation: Some(VariableValidation {
                        min_length: Some(5),
                        max_length: Some(100),
                        pattern: None,
                        allowed_values: None,
                    }),
                }),
                ("version".to_string(), TemplateVariable {
                    name: "version".to_string(),
                    description: "Document version".to_string(),
                    variable_type: VariableType::String,
                    default_value: Some("1.0".to_string()),
                    is_required: true,
                    validation: None,
                }),
            ]),
            metadata: TemplateMetadata {
                version: "1.0".to_string(),
                author: "Fiovana System".to_string(),
                created_date: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
                tags: vec!["training".to_string(), "manual".to_string(), "education".to_string()],
                category: "Training".to_string(),
                language: "en".to_string(),
                estimated_generation_time: Some("5-10 minutes".to_string()),
            },
        })
    }

    /// Create a quick reference template
    fn create_quick_reference_template(&self) -> Result<TemplateDefinition> {
        Ok(TemplateDefinition {
            id: "quick_reference_card".to_string(),
            name: "Quick Reference Card".to_string(),
            description: "Concise reference card for quick lookup of key information".to_string(),
            template_type: OutputTemplate::QuickReference {
                format: ReferenceFormat::QuickCard,
            },
            output_format: OutputFormat::HTML,
            supported_formats: vec![
                OutputFormat::HTML,
                OutputFormat::Markdown,
                OutputFormat::PDF,
            ],
            audience_level: AudienceLevel::Mixed,
            content: "<h1>{{title}}</h1>\n<p>{{description}}</p>\n<div>{{content}}</div>"
                .to_string(),
            sections: vec![
                TemplateSection {
                    id: "header".to_string(),
                    title: "Header".to_string(),
                    content_template: "<h1>{{title}}</h1>\n<p class=\"subtitle\">{{subtitle}}</p>"
                        .to_string(),
                    is_required: true,
                    order: 1,
                    subsections: vec![],
                    content_type: ContentType::Text,
                    formatting: SectionFormatting {
                        font_size: Some("large".to_string()),
                        font_weight: Some("bold".to_string()),
                        alignment: Some("center".to_string()),
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::None),
                    },
                },
                TemplateSection {
                    id: "key_points".to_string(),
                    title: "Key Points".to_string(),
                    content_template: "<div class=\"key-points\">\n{{key_points_list}}\n</div>"
                        .to_string(),
                    is_required: true,
                    order: 2,
                    subsections: vec![],
                    content_type: ContentType::List,
                    formatting: SectionFormatting {
                        font_size: None,
                        font_weight: None,
                        alignment: None,
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::Bullet),
                    },
                },
            ],
            variables: HashMap::from([(
                "title".to_string(),
                TemplateVariable {
                    name: "title".to_string(),
                    description: "Quick reference title".to_string(),
                    variable_type: VariableType::String,
                    default_value: Some("Quick Reference".to_string()),
                    is_required: true,
                    validation: None,
                },
            )]),
            metadata: TemplateMetadata {
                version: "1.0".to_string(),
                author: "Fiovana System".to_string(),
                created_date: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
                tags: vec![
                    "reference".to_string(),
                    "quick".to_string(),
                    "lookup".to_string(),
                ],
                category: "Reference".to_string(),
                language: "en".to_string(),
                estimated_generation_time: Some("2-5 minutes".to_string()),
            },
        })
    }

    /// Create a presentation template
    fn create_presentation_template(&self) -> Result<TemplateDefinition> {
        Ok(TemplateDefinition {
            id: "presentation_corporate".to_string(),
            name: "Corporate Presentation".to_string(),
            description: "Professional presentation template for corporate environments".to_string(),
            template_type: OutputTemplate::Presentation {
                slides: 10,
                style: PresentationStyle::Corporate
            },
            output_format: OutputFormat::Markdown,
            supported_formats: vec![OutputFormat::Markdown, OutputFormat::PowerPoint, OutputFormat::HTML],
            audience_level: AudienceLevel::Advanced,
            content: "# {{title}}\n\n## {{slide_content}}\n\n---".to_string(),
            sections: vec![
                TemplateSection {
                    id: "title_slide".to_string(),
                    title: "Title Slide".to_string(),
                    content_template: "---\n# {{title}}\n\n## {{subtitle}}\n\n**{{presenter_name}}**\n\n{{date}}\n\n---".to_string(),
                    is_required: true,
                    order: 1,
                    subsections: vec![],
                    content_type: ContentType::Text,
                    formatting: SectionFormatting {
                        font_size: Some("large".to_string()),
                        font_weight: Some("bold".to_string()),
                        alignment: Some("center".to_string()),
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::None),
                    },
                },
                TemplateSection {
                    id: "agenda".to_string(),
                    title: "Agenda".to_string(),
                    content_template: "---\n# Agenda\n\n{{agenda_items}}\n\n---".to_string(),
                    is_required: true,
                    order: 2,
                    subsections: vec![],
                    content_type: ContentType::List,
                    formatting: SectionFormatting {
                        font_size: None,
                        font_weight: None,
                        alignment: None,
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::Numeric),
                    },
                },
            ],
            variables: HashMap::from([
                ("title".to_string(), TemplateVariable {
                    name: "title".to_string(),
                    description: "Presentation title".to_string(),
                    variable_type: VariableType::String,
                    default_value: Some("Presentation Title".to_string()),
                    is_required: true,
                    validation: None,
                }),
                ("slides".to_string(), TemplateVariable {
                    name: "slides".to_string(),
                    description: "Number of slides".to_string(),
                    variable_type: VariableType::Number,
                    default_value: Some("10".to_string()),
                    is_required: false,
                    validation: Some(VariableValidation {
                        min_length: None,
                        max_length: None,
                        pattern: Some(r"^\d+$".to_string()),
                        allowed_values: None,
                    }),
                }),
            ]),
            metadata: TemplateMetadata {
                version: "1.0".to_string(),
                author: "Fiovana System".to_string(),
                created_date: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
                tags: vec!["presentation".to_string(), "corporate".to_string(), "slides".to_string()],
                category: "Presentation".to_string(),
                language: "en".to_string(),
                estimated_generation_time: Some("10-15 minutes".to_string()),
            },
        })
    }

    /// Create an assessment template
    fn create_assessment_template(&self) -> Result<TemplateDefinition> {
        Ok(TemplateDefinition {
            id: "assessment_quiz".to_string(),
            name: "Assessment Quiz".to_string(),
            description: "Interactive quiz template for knowledge assessment".to_string(),
            template_type: OutputTemplate::Assessment {
                questions: 10,
                difficulty: Difficulty::Medium
            },
            output_format: OutputFormat::JSON,
            supported_formats: vec![OutputFormat::JSON, OutputFormat::HTML, OutputFormat::Markdown],
            audience_level: AudienceLevel::Intermediate,
            content: "{{questions_json}}".to_string(),
            sections: vec![
                TemplateSection {
                    id: "instructions".to_string(),
                    title: "Instructions".to_string(),
                    content_template: "{\n  \"instructions\": \"{{instructions_text}}\",\n  \"time_limit\": \"{{time_limit}}\",\n  \"passing_score\": {{passing_score}}\n}".to_string(),
                    is_required: true,
                    order: 1,
                    subsections: vec![],
                    content_type: ContentType::Text,
                    formatting: SectionFormatting {
                        font_size: None,
                        font_weight: None,
                        alignment: None,
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::None),
                    },
                },
                TemplateSection {
                    id: "questions".to_string(),
                    title: "Questions".to_string(),
                    content_template: "\"questions\": [\n{{question_list}}\n]".to_string(),
                    is_required: true,
                    order: 2,
                    subsections: vec![],
                    content_type: ContentType::List,
                    formatting: SectionFormatting {
                        font_size: None,
                        font_weight: None,
                        alignment: None,
                        spacing: None,
                        indentation: None,
                        numbering: Some(NumberingStyle::Numeric),
                    },
                },
            ],
            variables: HashMap::from([
                ("questions".to_string(), TemplateVariable {
                    name: "questions".to_string(),
                    description: "Number of questions".to_string(),
                    variable_type: VariableType::Number,
                    default_value: Some("10".to_string()),
                    is_required: true,
                    validation: Some(VariableValidation {
                        min_length: None,
                        max_length: None,
                        pattern: Some(r"^\d+$".to_string()),
                        allowed_values: None,
                    }),
                }),
                ("difficulty".to_string(), TemplateVariable {
                    name: "difficulty".to_string(),
                    description: "Assessment difficulty level".to_string(),
                    variable_type: VariableType::String,
                    default_value: Some("Medium".to_string()),
                    is_required: false,
                    validation: Some(VariableValidation {
                        min_length: None,
                        max_length: None,
                        pattern: None,
                        allowed_values: Some(vec!["Easy".to_string(), "Medium".to_string(), "Hard".to_string()]),
                    }),
                }),
            ]),
            metadata: TemplateMetadata {
                version: "1.0".to_string(),
                author: "Fiovana System".to_string(),
                created_date: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
                tags: vec!["assessment".to_string(), "quiz".to_string(), "evaluation".to_string()],
                category: "Assessment".to_string(),
                language: "en".to_string(),
                estimated_generation_time: Some("15-20 minutes".to_string()),
            },
        })
    }

    /// Validate a template definition
    pub fn validate_template(&self, template: &TemplateDefinition) -> Result<()> {
        // Validate basic fields
        if template.id.is_empty() {
            return Err(anyhow!("Template ID cannot be empty"));
        }

        if template.name.is_empty() {
            return Err(anyhow!("Template name cannot be empty"));
        }

        // Validate sections
        for section in &template.sections {
            if section.id.is_empty() {
                return Err(anyhow!("Section ID cannot be empty"));
            }

            if section.title.is_empty() {
                return Err(anyhow!("Section title cannot be empty"));
            }
        }

        // Validate variables
        for (key, variable) in &template.variables {
            if key != &variable.name {
                return Err(anyhow!(
                    "Variable key '{}' does not match variable name '{}'",
                    key,
                    variable.name
                ));
            }

            if variable.is_required && variable.default_value.is_none() {
                return Err(anyhow!(
                    "Required variable '{}' must have a default value",
                    variable.name
                ));
            }
        }

        Ok(())
    }

    /// Get template statistics
    pub fn get_statistics(&self) -> TemplateStatistics {
        let total_templates = self.loaded_templates.len();
        let mut by_type = HashMap::new();
        let mut by_format = HashMap::new();

        for template in self.loaded_templates.values() {
            let _type_key = std::mem::discriminant(&template.template_type);
            *by_type
                .entry(format!("{:?}", template.template_type))
                .or_insert(0) += 1;
            *by_format.entry(template.output_format.clone()).or_insert(0) += 1;
        }

        TemplateStatistics {
            total_templates,
            templates_by_type: by_type,
            templates_by_format: by_format,
        }
    }

    /// Initialize default templates
    pub fn initialize_default_templates(&mut self) -> Result<()> {
        self.create_default_templates()
    }

    /// Create a new template
    pub fn create_template(&mut self, template: TemplateDefinition) -> Result<String> {
        let template_id = template.id.clone();
        self.loaded_templates
            .insert(template_id.clone(), template.clone());
        self.save_template(&template)?;
        Ok(template_id)
    }

    /// Update an existing template
    pub fn update_template(
        &mut self,
        template_id: &str,
        template: TemplateDefinition,
    ) -> Result<()> {
        if !self.loaded_templates.contains_key(template_id) {
            return Err(anyhow::anyhow!("Template not found: {}", template_id));
        }
        self.loaded_templates
            .insert(template_id.to_string(), template.clone());
        self.save_template(&template)?;
        Ok(())
    }

    /// Delete a template
    pub fn delete_template(&mut self, template_id: &str) -> Result<()> {
        if !self.loaded_templates.contains_key(template_id) {
            return Err(anyhow::anyhow!("Template not found: {}", template_id));
        }

        self.loaded_templates.remove(template_id);

        // Delete the template file
        let template_path = self.templates_dir.join(format!("{}.yaml", template_id));
        if template_path.exists() {
            fs::remove_file(template_path)?;
        }

        Ok(())
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<&TemplateDefinition> {
        self.loaded_templates.values().collect()
    }

    /// Generate content from template
    pub fn generate_from_template(
        &self,
        template_id: &str,
        variables: &HashMap<String, String>,
        output_format: &OutputFormat,
    ) -> Result<String> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_id))?;

        if !template.supported_formats.contains(output_format) {
            return Err(anyhow::anyhow!(
                "Template {} does not support format {:?}",
                template_id,
                output_format
            ));
        }

        // Simple template variable substitution
        let mut content = template.content.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            content = content.replace(&placeholder, value);
        }

        Ok(content)
    }

    /// Create a new default TemplateManager for main.rs
    pub fn new_default() -> Self {
        let templates_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./data"))
            .join("fiovana")
            .join("templates");

        TemplateManager::new(templates_dir).unwrap_or_else(|_| {
            // Fallback to current directory if data directory fails
            TemplateManager::new(PathBuf::from("./templates")).unwrap()
        })
    }
}

/// Template system statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateStatistics {
    pub total_templates: usize,
    pub templates_by_type: HashMap<String, usize>,
    pub templates_by_format: HashMap<OutputFormat, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TemplateManager::new(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_default_template_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = TemplateManager::new(temp_dir.path().to_path_buf()).unwrap();

        let result = manager.create_default_templates();
        assert!(result.is_ok());
        assert_eq!(manager.loaded_templates.len(), 4);
    }

    #[test]
    fn test_template_validation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TemplateManager::new(temp_dir.path().to_path_buf()).unwrap();

        let valid_template = manager.create_training_manual_template().unwrap();
        assert!(manager.validate_template(&valid_template).is_ok());

        let mut invalid_template = valid_template.clone();
        invalid_template.id = String::new();
        assert!(manager.validate_template(&invalid_template).is_err());
    }

    #[test]
    fn test_template_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = TemplateManager::new(temp_dir.path().to_path_buf()).unwrap();
        manager.create_default_templates().unwrap();

        let training_templates = manager.get_templates_by_type(&OutputTemplate::TrainingManual {
            audience: AudienceLevel::Beginner,
        });
        assert!(!training_templates.is_empty());

        let html_templates = manager.get_templates_by_format(&OutputFormat::HTML);
        assert!(!html_templates.is_empty());
    }

    #[test]
    fn test_template_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = TemplateManager::new(temp_dir.path().to_path_buf()).unwrap();
        manager.create_default_templates().unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.total_templates, 4);
        assert!(!stats.templates_by_type.is_empty());
        assert!(!stats.templates_by_format.is_empty());
    }
}
