// src-tauri/src/ai/response.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::intent::{Intent, IntentConfidence};
use super::ollama::OllamaClient;
use super::AIConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseType {
    Action,
    Information,
    Clarification,
    Error,
    MultiStep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub response_type: ResponseType,
    pub content: String,
    pub intent: Intent,
    pub confidence: f32,
    pub suggested_actions: Vec<SuggestedAction>,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: String,
    pub description: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub processing_time_ms: u64,
    pub model_used: String,
    pub tokens_used: Option<u32>,
    pub confidence_explanation: String,
}

#[allow(dead_code)]
pub struct ResponseGenerator {
    #[allow(dead_code)]
    system_prompts: std::collections::HashMap<Intent, String>,
}

#[allow(dead_code)]
impl ResponseGenerator {
    pub fn new() -> Self {
        let mut system_prompts = std::collections::HashMap::new();

        // Document comparison prompt
        system_prompts.insert(
            Intent::CompareDocuments,
            "You are an expert document analyst. When users ask you to compare documents, provide a clear, structured analysis focusing on:
            1. Key changes and differences
            2. Impact of changes
            3. Recommendations for action
            Be specific and actionable in your responses.".to_string()
        );

        // Content update prompt
        system_prompts.insert(
            Intent::UpdateContent,
            "You are a content editor. Help users update and modify their documents by:
            1. Understanding what changes they want to make
            2. Suggesting how to implement changes while preserving style
            3. Providing clear next steps
            Always ask for clarification if the request is ambiguous."
                .to_string(),
        );

        // Output generation prompt
        system_prompts.insert(
            Intent::GenerateOutput,
            "You are a document generation assistant. Help users create professional outputs by:
            1. Understanding the target format and audience
            2. Suggesting appropriate structure and content
            3. Providing clear instructions for generation
            Focus on practical, actionable guidance."
                .to_string(),
        );

        // Search prompt
        system_prompts.insert(
            Intent::SearchDocuments,
            "You are a search assistant. Help users find information in their documents by:
            1. Understanding what they're looking for
            2. Suggesting search strategies
            3. Explaining how to refine searches
            Be helpful and guide them to relevant content."
                .to_string(),
        );

        // Help prompt
        system_prompts.insert(
            Intent::GetHelp,
            "You are a helpful assistant for Proxemic, an AI-powered document processing system. Provide clear, concise help by:
            1. Explaining features and capabilities
            2. Giving step-by-step instructions
            3. Suggesting best practices
            Keep responses practical and easy to follow.".to_string()
        );

        // Default prompt for unknown intents
        system_prompts.insert(
            Intent::Unknown,
            "You are an AI assistant for Proxemic document processing. The user's intent is unclear. Please:
            1. Ask clarifying questions
            2. Suggest what they might be trying to do
            3. Offer to help with common tasks
            Be helpful and guide them toward a clear request.".to_string()
        );

        Self { system_prompts }
    }

    pub async fn generate(
        &self,
        ollama_client: &OllamaClient,
        user_input: &str,
        intent_result: &IntentConfidence,
        context: Option<&str>,
        config: &AIConfig,
    ) -> Result<AIResponse> {
        let start_time = std::time::Instant::now();

        // Select appropriate system prompt
        let system_prompt = self
            .system_prompts
            .get(&intent_result.intent)
            .or_else(|| self.system_prompts.get(&Intent::Unknown))
            .unwrap()
            .clone();

        // Build context-aware prompt
        let full_prompt = self.build_prompt(user_input, context, intent_result);

        // Generate response using Ollama
        let ai_response = if ollama_client.is_available().await {
            ollama_client
                .system_chat(
                    &config.default_model,
                    &system_prompt,
                    &full_prompt,
                    Some(config.temperature),
                )
                .await?
        } else {
            self.generate_fallback_response(intent_result)
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Determine response type
        let response_type =
            self.determine_response_type(&intent_result.intent, intent_result.confidence);

        // Generate suggested actions
        let suggested_actions = self.generate_suggested_actions(&intent_result.intent);

        Ok(AIResponse {
            response_type,
            content: ai_response,
            intent: intent_result.intent.clone(),
            confidence: intent_result.confidence,
            suggested_actions,
            metadata: ResponseMetadata {
                processing_time_ms: processing_time,
                model_used: config.default_model.clone(),
                tokens_used: None, // Ollama doesn't provide token counts in simple API
                confidence_explanation: intent_result.reasoning.clone(),
            },
        })
    }

    fn build_prompt(
        &self,
        user_input: &str,
        context: Option<&str>,
        intent_result: &IntentConfidence,
    ) -> String {
        let mut prompt = format!("User request: {}\n", user_input);

        if let Some(ctx) = context {
            prompt.push_str(&format!("Context: {}\n", ctx));
        }

        prompt.push_str(&format!(
            "Detected intent: {:?} (confidence: {:.1}%)\n",
            intent_result.intent,
            intent_result.confidence * 100.0
        ));

        if intent_result.confidence < 0.7 {
            prompt.push_str(
                "Note: Intent confidence is low. Please ask for clarification if needed.\n",
            );
        }

        if !intent_result.alternative_intents.is_empty() {
            prompt.push_str("Alternative interpretations: ");
            for (alt_intent, alt_score) in &intent_result.alternative_intents {
                prompt.push_str(&format!("{:?} ({:.1}%), ", alt_intent, alt_score * 100.0));
            }
            prompt.push('\n');
        }

        prompt.push_str("\nProvide a helpful response based on the detected intent. If the intent is unclear, ask clarifying questions.");

        prompt
    }

    fn determine_response_type(&self, intent: &Intent, confidence: f32) -> ResponseType {
        if confidence < 0.5 {
            return ResponseType::Clarification;
        }

        match intent {
            Intent::CompareDocuments => ResponseType::Action,
            Intent::UpdateContent => ResponseType::Action,
            Intent::GenerateOutput => ResponseType::Action,
            Intent::SearchDocuments => ResponseType::Action,
            Intent::FindSimilarContent => ResponseType::Action,
            Intent::GetHelp => ResponseType::Information,
            Intent::CheckStatus => ResponseType::Information,
            Intent::ListModels => ResponseType::Information,
            Intent::ConfigureWorkspace => ResponseType::MultiStep,
            Intent::ManageFiles => ResponseType::Action,
            Intent::Unknown => ResponseType::Clarification,
            _ => ResponseType::Information,
        }
    }

    fn generate_suggested_actions(&self, intent: &Intent) -> Vec<SuggestedAction> {
        match intent {
            Intent::CompareDocuments => vec![
                SuggestedAction {
                    action_type: "select_documents".to_string(),
                    description: "Select documents to compare".to_string(),
                    parameters: None,
                },
                SuggestedAction {
                    action_type: "run_comparison".to_string(),
                    description: "Run document comparison".to_string(),
                    parameters: None,
                },
            ],
            Intent::GenerateOutput => vec![
                SuggestedAction {
                    action_type: "select_format".to_string(),
                    description: "Choose output format (Word, PDF, HTML)".to_string(),
                    parameters: None,
                },
                SuggestedAction {
                    action_type: "configure_template".to_string(),
                    description: "Configure document template".to_string(),
                    parameters: None,
                },
            ],
            Intent::SearchDocuments => vec![
                SuggestedAction {
                    action_type: "open_search".to_string(),
                    description: "Open search interface".to_string(),
                    parameters: None,
                },
                SuggestedAction {
                    action_type: "filter_documents".to_string(),
                    description: "Apply document filters".to_string(),
                    parameters: None,
                },
            ],
            Intent::ManageFiles => vec![
                SuggestedAction {
                    action_type: "import_files".to_string(),
                    description: "Import new files".to_string(),
                    parameters: None,
                },
                SuggestedAction {
                    action_type: "organize_workspace".to_string(),
                    description: "Organize workspace files".to_string(),
                    parameters: None,
                },
            ],
            _ => vec![],
        }
    }

    fn generate_fallback_response(&self, intent_result: &IntentConfidence) -> String {
        match intent_result.intent {
            Intent::CompareDocuments => {
                "I can help you compare documents. Please select the documents you'd like to compare, and I'll analyze the differences for you. Note: AI model is currently unavailable, but I can still perform document comparison using built-in analysis.".to_string()
            },
            Intent::GenerateOutput => {
                "I can help you generate documents in various formats (Word, PDF, HTML). Please specify what type of output you need and I'll guide you through the process. Note: AI model is currently unavailable, but template-based generation is still available.".to_string()
            },
            Intent::SearchDocuments => {
                "I can help you search through your documents. You can search by keywords, document type, or content similarity. Note: AI model is currently unavailable, but keyword search is still functional.".to_string()
            },
            Intent::GetHelp => {
                "Welcome to Proxemic! I can help you with:\n• Comparing documents\n• Generating outputs in multiple formats\n• Searching and organizing content\n• Managing your workspace\n\nWhat would you like to do first?".to_string()
            },
            Intent::CheckStatus => {
                "System Status:\n• Document processing: Available\n• File management: Available\n• AI models: Currently unavailable\n• Vector search: Available (basic mode)\n\nAll core features are functional. AI enhancement requires Ollama to be running.".to_string()
            },
            _ => {
                "I'm here to help with document processing tasks. Could you please clarify what you'd like to do? I can help with comparing documents, generating outputs, searching content, and managing your workspace.".to_string()
            }
        }
    }
}

impl Default for ResponseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_type_determination() {
        let generator = ResponseGenerator::new();

        let response_type = generator.determine_response_type(&Intent::CompareDocuments, 0.8);
        assert_eq!(response_type, ResponseType::Action);

        let response_type = generator.determine_response_type(&Intent::GetHelp, 0.9);
        assert_eq!(response_type, ResponseType::Information);

        let response_type = generator.determine_response_type(&Intent::Unknown, 0.3);
        assert_eq!(response_type, ResponseType::Clarification);
    }

    #[test]
    fn test_suggested_actions() {
        let generator = ResponseGenerator::new();

        let actions = generator.generate_suggested_actions(&Intent::CompareDocuments);
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.action_type == "select_documents"));

        let actions = generator.generate_suggested_actions(&Intent::GenerateOutput);
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.action_type == "select_format"));
    }

    #[test]
    fn test_fallback_response() {
        let generator = ResponseGenerator::new();
        let intent_result = IntentConfidence {
            intent: Intent::GetHelp,
            confidence: 0.8,
            reasoning: "Test".to_string(),
            alternative_intents: vec![],
        };

        let response = generator.generate_fallback_response(&intent_result);
        assert!(response.contains("Proxemic"));
        assert!(response.contains("help"));
    }
}
