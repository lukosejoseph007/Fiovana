// src-tauri/src/ai/text_operations.rs
use serde::{Deserialize, Serialize};
use std::fmt;

/// Text operation types for AI-powered text manipulation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum TextOperation {
    /// Define terms/concepts
    Define,
    /// Explain in simpler terms
    Explain,
    /// Add more details/examples
    Expand,
    /// Make it simpler
    Simplify,
    /// Rewrite in different style
    Rewrite { style: Option<String> },
    /// Grammar/clarity improvements
    Improve,
    /// Create summary
    Summarize { length: Option<SummaryLength> },
    /// Translate to another language
    Translate { target_language: String },
    /// Find related content
    FindRelated,
    /// Custom prompt
    Custom(String),
}

impl fmt::Display for TextOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextOperation::Define => write!(f, "Define"),
            TextOperation::Explain => write!(f, "Explain"),
            TextOperation::Expand => write!(f, "Expand"),
            TextOperation::Simplify => write!(f, "Simplify"),
            TextOperation::Rewrite { .. } => write!(f, "Rewrite"),
            TextOperation::Improve => write!(f, "Improve"),
            TextOperation::Summarize { .. } => write!(f, "Summarize"),
            TextOperation::Translate { .. } => write!(f, "Translate"),
            TextOperation::FindRelated => write!(f, "FindRelated"),
            TextOperation::Custom(_) => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SummaryLength {
    Short,
    Medium,
    Long,
}

/// Document context for text operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContext {
    pub document_id: Option<String>,
    pub document_title: Option<String>,
    pub document_type: Option<String>,
    pub surrounding_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Result of a text operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOperationResult {
    pub original: String,
    pub result: String,
    pub operation: String,
    pub confidence: f32,
    pub reasoning: Option<String>,
    pub suggestions: Vec<String>,
    pub alternative_results: Vec<String>,
}

/// Text operation processor
pub struct TextOperationProcessor {
    // Future: Can hold AI model references, caches, etc.
}

impl TextOperationProcessor {
    pub fn new() -> Self {
        Self {}
    }

    /// Build prompt for the given operation
    pub fn build_prompt(
        &self,
        text: &str,
        operation: &TextOperation,
        context: Option<&DocumentContext>,
    ) -> String {
        let base_prompt = match operation {
            TextOperation::Define => {
                format!(
                    "Define the following term or concept clearly and concisely:\n\n\"{}\"",
                    text
                )
            }
            TextOperation::Explain => {
                format!(
                    "Explain the following concept in simpler terms that anyone can understand:\n\n\"{}\"",
                    text
                )
            }
            TextOperation::Expand => {
                format!(
                    "Expand on the following text with more details, examples, and context:\n\n\"{}\"",
                    text
                )
            }
            TextOperation::Simplify => {
                format!(
                    "Simplify the following text to make it easier to understand:\n\n\"{}\"",
                    text
                )
            }
            TextOperation::Rewrite { style } => {
                let style_instruction = style
                    .as_ref()
                    .map(|s| format!(" in a {} style", s))
                    .unwrap_or_default();
                format!(
                    "Rewrite the following text{}:\n\n\"{}\"",
                    style_instruction, text
                )
            }
            TextOperation::Improve => {
                format!(
                    "Improve the grammar, clarity, and readability of the following text:\n\n\"{}\"",
                    text
                )
            }
            TextOperation::Summarize { length } => {
                let length_instruction = match length {
                    Some(SummaryLength::Short) => " in 1-2 sentences",
                    Some(SummaryLength::Medium) => " in 3-5 sentences",
                    Some(SummaryLength::Long) => " in a paragraph",
                    None => "",
                };
                format!(
                    "Summarize the following text{}:\n\n\"{}\"",
                    length_instruction, text
                )
            }
            TextOperation::Translate { target_language } => {
                format!(
                    "Translate the following text to {}:\n\n\"{}\"",
                    target_language, text
                )
            }
            TextOperation::FindRelated => {
                format!(
                    "Based on the following text, suggest related topics, concepts, or areas to explore:\n\n\"{}\"",
                    text
                )
            }
            TextOperation::Custom(prompt) => {
                format!("{}:\n\n\"{}\"", prompt, text)
            }
        };

        // Add document context if available
        if let Some(ctx) = context {
            let mut prompt = base_prompt;

            if let Some(title) = &ctx.document_title {
                prompt.push_str(&format!("\n\nDocument: {}", title));
            }

            if let Some(doc_type) = &ctx.document_type {
                prompt.push_str(&format!("\nDocument Type: {}", doc_type));
            }

            if let Some(surrounding) = &ctx.surrounding_text {
                prompt.push_str(&format!("\n\nSurrounding Context:\n{}", surrounding));
            }

            prompt
        } else {
            base_prompt
        }
    }

    /// Execute text operation with AI
    pub async fn execute(
        &self,
        text: String,
        operation: TextOperation,
        context: Option<DocumentContext>,
        ai_orchestrator: &crate::ai::AIOrchestrator,
    ) -> Result<TextOperationResult, String> {
        // Build the prompt
        let prompt = self.build_prompt(&text, &operation, context.as_ref());

        // Build context string
        let context_str = context
            .as_ref()
            .and_then(|ctx| ctx.document_title.as_ref())
            .map(|title| format!("Document: {}", title));

        // Execute AI request using process_conversation
        let ai_response = ai_orchestrator
            .process_conversation(&prompt, context_str.as_deref())
            .await
            .map_err(|e| format!("AI processing failed: {}", e))?;

        // Extract response content
        let response_text = ai_response.content;

        // Parse response and create result
        let result = TextOperationResult {
            original: text.clone(),
            result: response_text.clone(),
            operation: operation.to_string(),
            confidence: 0.85, // Default confidence, can be calculated based on AI response
            reasoning: Some(format!("Applied {} operation", operation)),
            suggestions: vec![],
            alternative_results: vec![],
        };

        Ok(result)
    }

    /// Get available operations list
    pub fn get_available_operations() -> Vec<String> {
        vec![
            "Define".to_string(),
            "Explain".to_string(),
            "Expand".to_string(),
            "Simplify".to_string(),
            "Rewrite".to_string(),
            "Improve".to_string(),
            "Summarize".to_string(),
            "Translate".to_string(),
            "FindRelated".to_string(),
            "Custom".to_string(),
        ]
    }

    /// Get operation description
    pub fn get_operation_description(operation_name: &str) -> Option<String> {
        match operation_name {
            "Define" => Some("Define terms and concepts clearly".to_string()),
            "Explain" => Some("Explain concepts in simpler terms".to_string()),
            "Expand" => Some("Add more details and examples".to_string()),
            "Simplify" => Some("Make text easier to understand".to_string()),
            "Rewrite" => Some("Rewrite in a different style or tone".to_string()),
            "Improve" => Some("Improve grammar, clarity, and readability".to_string()),
            "Summarize" => Some("Create a brief summary".to_string()),
            "Translate" => Some("Translate to another language".to_string()),
            "FindRelated" => Some("Find related topics and concepts".to_string()),
            "Custom" => Some("Custom AI operation with your prompt".to_string()),
            _ => None,
        }
    }
}

impl Default for TextOperationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_operation_display() {
        assert_eq!(TextOperation::Define.to_string(), "Define");
        assert_eq!(TextOperation::Explain.to_string(), "Explain");
        assert_eq!(
            TextOperation::Rewrite { style: None }.to_string(),
            "Rewrite"
        );
    }

    #[test]
    fn test_build_prompt_define() {
        let processor = TextOperationProcessor::new();
        let prompt = processor.build_prompt("API", &TextOperation::Define, None);
        assert!(prompt.contains("Define the following term"));
        assert!(prompt.contains("API"));
    }

    #[test]
    fn test_build_prompt_with_context() {
        let processor = TextOperationProcessor::new();
        let context = DocumentContext {
            document_id: Some("doc1".to_string()),
            document_title: Some("API Guide".to_string()),
            document_type: Some("Technical".to_string()),
            surrounding_text: Some("REST API concepts".to_string()),
            metadata: None,
        };

        let prompt = processor.build_prompt("endpoint", &TextOperation::Define, Some(&context));
        assert!(prompt.contains("endpoint"));
        assert!(prompt.contains("API Guide"));
        assert!(prompt.contains("REST API concepts"));
    }

    #[test]
    fn test_get_available_operations() {
        let operations = TextOperationProcessor::get_available_operations();
        assert!(operations.contains(&"Define".to_string()));
        assert!(operations.contains(&"Explain".to_string()));
        assert_eq!(operations.len(), 10);
    }

    #[test]
    fn test_get_operation_description() {
        let desc = TextOperationProcessor::get_operation_description("Define");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("Define terms"));

        let none_desc = TextOperationProcessor::get_operation_description("Invalid");
        assert!(none_desc.is_none());
    }
}
