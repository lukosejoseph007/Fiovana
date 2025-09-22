// src-tauri/src/document/document_generator.rs
// Document generation capabilities for creating various output formats

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub title: String,
    pub sections: Vec<DocumentSection>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub title: String,
    pub content: String,
    pub level: u8, // 1 for main headings, 2 for subheadings, etc.
    pub section_type: SectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SectionType {
    Heading,
    Paragraph,
    List,
    CodeBlock,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Docx,
    Pdf,
    Html,
    Markdown,
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub format: OutputFormat,
    pub template: Option<String>,
    pub style_options: HashMap<String, String>,
    pub include_metadata: bool,
}

pub struct DocumentGenerator {
    output_dir: PathBuf,
}

impl DocumentGenerator {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    pub async fn generate_document(
        &self,
        content: &DocumentContent,
        options: &GenerationOptions,
        output_filename: &str,
    ) -> Result<PathBuf> {
        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir).context("Failed to create output directory")?;

        let output_path = self.output_dir.join(output_filename);

        match options.format {
            OutputFormat::Markdown => self.generate_markdown(content, &output_path).await,
            OutputFormat::Html => self.generate_html(content, options, &output_path).await,
            OutputFormat::PlainText => self.generate_plain_text(content, &output_path).await,
            OutputFormat::Docx => {
                // For now, fall back to plain text
                // In the future, this would use a proper DOCX generation library
                let text_path = output_path.with_extension("txt");
                self.generate_plain_text(content, &text_path).await?;
                Ok(text_path)
            }
            OutputFormat::Pdf => {
                // For now, fall back to HTML then generate info
                let html_path = output_path.with_extension("html");
                self.generate_html(content, options, &html_path).await?;
                Ok(html_path)
            }
        }
    }

    async fn generate_markdown(
        &self,
        content: &DocumentContent,
        output_path: &Path,
    ) -> Result<PathBuf> {
        let mut markdown = String::new();

        // Title
        markdown.push_str(&format!("# {}\n\n", content.title));

        // Metadata
        if !content.metadata.is_empty() {
            markdown.push_str("---\n");
            for (key, value) in &content.metadata {
                markdown.push_str(&format!("{}: {}\n", key, value));
            }
            markdown.push_str("---\n\n");
        }

        // Sections
        for section in &content.sections {
            match section.section_type {
                SectionType::Heading => {
                    let heading_prefix = "#".repeat(section.level as usize + 1);
                    markdown.push_str(&format!("{} {}\n\n", heading_prefix, section.title));
                }
                SectionType::Paragraph => {
                    if !section.title.is_empty() {
                        let heading_prefix = "#".repeat(section.level as usize + 1);
                        markdown.push_str(&format!("{} {}\n\n", heading_prefix, section.title));
                    }
                    markdown.push_str(&format!("{}\n\n", section.content));
                }
                SectionType::List => {
                    if !section.title.is_empty() {
                        let heading_prefix = "#".repeat(section.level as usize + 1);
                        markdown.push_str(&format!("{} {}\n\n", heading_prefix, section.title));
                    }
                    for line in section.content.lines() {
                        if !line.trim().is_empty() {
                            markdown.push_str(&format!("- {}\n", line.trim()));
                        }
                    }
                    markdown.push('\n');
                }
                SectionType::CodeBlock => {
                    if !section.title.is_empty() {
                        let heading_prefix = "#".repeat(section.level as usize + 1);
                        markdown.push_str(&format!("{} {}\n\n", heading_prefix, section.title));
                    }
                    markdown.push_str("```\n");
                    markdown.push_str(&section.content);
                    markdown.push_str("\n```\n\n");
                }
                SectionType::Table => {
                    if !section.title.is_empty() {
                        let heading_prefix = "#".repeat(section.level as usize + 1);
                        markdown.push_str(&format!("{} {}\n\n", heading_prefix, section.title));
                    }
                    // Simple table formatting - would need more sophisticated parsing for real tables
                    markdown.push_str(&format!("{}\n\n", section.content));
                }
            }
        }

        fs::write(output_path, markdown).context("Failed to write markdown file")?;

        Ok(output_path.to_path_buf())
    }

    async fn generate_html(
        &self,
        content: &DocumentContent,
        options: &GenerationOptions,
        output_path: &Path,
    ) -> Result<PathBuf> {
        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!("    <title>{}</title>\n", content.title));
        html.push_str("    <style>\n");
        html.push_str(include_str!("../assets/default_styles.css"));
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Title
        html.push_str(&format!("    <h1>{}</h1>\n", html_escape(&content.title)));

        // Metadata
        if options.include_metadata && !content.metadata.is_empty() {
            html.push_str("    <div class=\"metadata\">\n");
            html.push_str("        <h2>Document Information</h2>\n");
            html.push_str("        <dl>\n");
            for (key, value) in &content.metadata {
                html.push_str(&format!(
                    "            <dt>{}</dt><dd>{}</dd>\n",
                    html_escape(key),
                    html_escape(value)
                ));
            }
            html.push_str("        </dl>\n");
            html.push_str("    </div>\n");
        }

        // Sections
        for section in &content.sections {
            match section.section_type {
                SectionType::Heading => {
                    let heading_level = std::cmp::min(section.level + 1, 6);
                    html.push_str(&format!(
                        "    <h{}>{}</h{}>\n",
                        heading_level,
                        html_escape(&section.title),
                        heading_level
                    ));
                }
                SectionType::Paragraph => {
                    if !section.title.is_empty() {
                        let heading_level = std::cmp::min(section.level + 1, 6);
                        html.push_str(&format!(
                            "    <h{}>{}</h{}>\n",
                            heading_level,
                            html_escape(&section.title),
                            heading_level
                        ));
                    }
                    html.push_str(&format!("    <p>{}</p>\n", html_escape(&section.content)));
                }
                SectionType::List => {
                    if !section.title.is_empty() {
                        let heading_level = std::cmp::min(section.level + 1, 6);
                        html.push_str(&format!(
                            "    <h{}>{}</h{}>\n",
                            heading_level,
                            html_escape(&section.title),
                            heading_level
                        ));
                    }
                    html.push_str("    <ul>\n");
                    for line in section.content.lines() {
                        if !line.trim().is_empty() {
                            html.push_str(&format!(
                                "        <li>{}</li>\n",
                                html_escape(line.trim())
                            ));
                        }
                    }
                    html.push_str("    </ul>\n");
                }
                SectionType::CodeBlock => {
                    if !section.title.is_empty() {
                        let heading_level = std::cmp::min(section.level + 1, 6);
                        html.push_str(&format!(
                            "    <h{}>{}</h{}>\n",
                            heading_level,
                            html_escape(&section.title),
                            heading_level
                        ));
                    }
                    html.push_str("    <pre><code>");
                    html.push_str(&html_escape(&section.content));
                    html.push_str("</code></pre>\n");
                }
                SectionType::Table => {
                    if !section.title.is_empty() {
                        let heading_level = std::cmp::min(section.level + 1, 6);
                        html.push_str(&format!(
                            "    <h{}>{}</h{}>\n",
                            heading_level,
                            html_escape(&section.title),
                            heading_level
                        ));
                    }
                    // Simple table - would need more sophisticated parsing for real tables
                    html.push_str(&format!(
                        "    <div class=\"table-content\">{}</div>\n",
                        html_escape(&section.content)
                    ));
                }
            }
        }

        // HTML footer
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        fs::write(output_path, html).context("Failed to write HTML file")?;

        Ok(output_path.to_path_buf())
    }

    async fn generate_plain_text(
        &self,
        content: &DocumentContent,
        output_path: &Path,
    ) -> Result<PathBuf> {
        let mut text = String::new();

        // Title
        text.push_str(&format!("{}\n", content.title));
        text.push_str(&"=".repeat(content.title.len()));
        text.push_str("\n\n");

        // Metadata
        if !content.metadata.is_empty() {
            text.push_str("Document Information:\n");
            text.push_str("--------------------\n");
            for (key, value) in &content.metadata {
                text.push_str(&format!("{}: {}\n", key, value));
            }
            text.push('\n');
        }

        // Sections
        for section in &content.sections {
            match section.section_type {
                SectionType::Heading => {
                    text.push_str(&format!("{}\n", section.title));
                    let underline = match section.level {
                        1 => "-",
                        2 => "~",
                        _ => "^",
                    };
                    text.push_str(&underline.repeat(section.title.len()));
                    text.push_str("\n\n");
                }
                SectionType::Paragraph => {
                    if !section.title.is_empty() {
                        text.push_str(&format!("{}\n", section.title));
                        let underline = match section.level {
                            1 => "-",
                            2 => "~",
                            _ => "^",
                        };
                        text.push_str(&underline.repeat(section.title.len()));
                        text.push_str("\n\n");
                    }
                    text.push_str(&format!("{}\n\n", section.content));
                }
                SectionType::List => {
                    if !section.title.is_empty() {
                        text.push_str(&format!("{}\n", section.title));
                        let underline = match section.level {
                            1 => "-",
                            2 => "~",
                            _ => "^",
                        };
                        text.push_str(&underline.repeat(section.title.len()));
                        text.push_str("\n\n");
                    }
                    for line in section.content.lines() {
                        if !line.trim().is_empty() {
                            text.push_str(&format!("• {}\n", line.trim()));
                        }
                    }
                    text.push('\n');
                }
                SectionType::CodeBlock => {
                    if !section.title.is_empty() {
                        text.push_str(&format!("{}\n", section.title));
                        let underline = match section.level {
                            1 => "-",
                            2 => "~",
                            _ => "^",
                        };
                        text.push_str(&underline.repeat(section.title.len()));
                        text.push_str("\n\n");
                    }
                    text.push_str("```\n");
                    text.push_str(&section.content);
                    text.push_str("\n```\n\n");
                }
                SectionType::Table => {
                    if !section.title.is_empty() {
                        text.push_str(&format!("{}\n", section.title));
                        let underline = match section.level {
                            1 => "-",
                            2 => "~",
                            _ => "^",
                        };
                        text.push_str(&underline.repeat(section.title.len()));
                        text.push_str("\n\n");
                    }
                    text.push_str(&format!("{}\n\n", section.content));
                }
            }
        }

        fs::write(output_path, text).context("Failed to write text file")?;

        Ok(output_path.to_path_buf())
    }

    pub fn get_supported_formats() -> Vec<OutputFormat> {
        vec![
            OutputFormat::Markdown,
            OutputFormat::Html,
            OutputFormat::PlainText,
            OutputFormat::Docx,
            OutputFormat::Pdf,
        ]
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// Utility function to convert parsed document content to our generation format
pub fn convert_parsed_content_to_document(
    title: String,
    content: &str,
    metadata: HashMap<String, String>,
) -> DocumentContent {
    let mut sections = Vec::new();
    let mut current_section = String::new();
    let mut current_title = String::new();
    let mut in_list = false;
    let mut in_code = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if !current_section.is_empty() {
                sections.push(DocumentSection {
                    title: current_title.clone(),
                    content: current_section.trim().to_string(),
                    level: 2,
                    section_type: if in_list {
                        SectionType::List
                    } else if in_code {
                        SectionType::CodeBlock
                    } else {
                        SectionType::Paragraph
                    },
                });
                current_section.clear();
                current_title.clear();
                in_list = false;
                in_code = false;
            }
            continue;
        }

        // Detect headers (simple heuristic)
        if trimmed.len() < 100
            && !trimmed.contains('.')
            && !trimmed.starts_with('-')
            && !trimmed.starts_with('•')
        {
            // Possible header
            if !current_section.is_empty() {
                sections.push(DocumentSection {
                    title: current_title.clone(),
                    content: current_section.trim().to_string(),
                    level: 2,
                    section_type: if in_list {
                        SectionType::List
                    } else if in_code {
                        SectionType::CodeBlock
                    } else {
                        SectionType::Paragraph
                    },
                });
                current_section.clear();
                in_list = false;
                in_code = false;
            }
            current_title = trimmed.to_string();
        } else if trimmed.starts_with('-') || trimmed.starts_with('•') || trimmed.starts_with('*')
        {
            // List item
            in_list = true;
            in_code = false;
            current_section.push_str(line);
            current_section.push('\n');
        } else {
            // Regular content
            if in_list
                && !trimmed.starts_with('-')
                && !trimmed.starts_with('•')
                && !trimmed.starts_with('*')
            {
                // End of list
                if !current_section.is_empty() {
                    sections.push(DocumentSection {
                        title: current_title.clone(),
                        content: current_section.trim().to_string(),
                        level: 2,
                        section_type: SectionType::List,
                    });
                    current_section.clear();
                    current_title.clear();
                    in_list = false;
                }
            }
            in_code = false;
            current_section.push_str(line);
            current_section.push('\n');
        }
    }

    // Add final section if any
    if !current_section.is_empty() {
        sections.push(DocumentSection {
            title: current_title,
            content: current_section.trim().to_string(),
            level: 2,
            section_type: if in_list {
                SectionType::List
            } else if in_code {
                SectionType::CodeBlock
            } else {
                SectionType::Paragraph
            },
        });
    }

    DocumentContent {
        title,
        sections,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_markdown_generation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = DocumentGenerator::new(temp_dir.path().to_path_buf());

        let content = DocumentContent {
            title: "Test Document".to_string(),
            sections: vec![
                DocumentSection {
                    title: "Introduction".to_string(),
                    content: "This is a test document.".to_string(),
                    level: 1,
                    section_type: SectionType::Heading,
                },
                DocumentSection {
                    title: "Content".to_string(),
                    content: "This is the main content.".to_string(),
                    level: 2,
                    section_type: SectionType::Paragraph,
                },
            ],
            metadata: HashMap::new(),
        };

        let options = GenerationOptions {
            format: OutputFormat::Markdown,
            template: None,
            style_options: HashMap::new(),
            include_metadata: false,
        };

        let result = generator
            .generate_document(&content, &options, "test.md")
            .await;

        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(output_path.exists());

        let generated_content = fs::read_to_string(output_path).unwrap();
        assert!(generated_content.contains("# Test Document"));
        assert!(generated_content.contains("## Introduction"));
    }

    #[test]
    fn test_content_conversion() {
        let content =
            "This is a title\n\nThis is some content.\n\n- Item 1\n- Item 2\n\nMore content.";
        let document =
            convert_parsed_content_to_document("Test".to_string(), content, HashMap::new());

        assert_eq!(document.title, "Test");
        assert!(document.sections.len() > 0);

        // Check that we have different section types
        let has_paragraph = document
            .sections
            .iter()
            .any(|s| matches!(s.section_type, SectionType::Paragraph));
        let has_list = document
            .sections
            .iter()
            .any(|s| matches!(s.section_type, SectionType::List));

        assert!(has_paragraph || has_list);
    }
}
