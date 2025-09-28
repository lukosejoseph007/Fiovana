// src-tauri/src/ai/response.rs

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::conversation_context::ConversationSession;
use super::intent::{Intent, IntentConfidence};
use super::ollama::OllamaClient;
use super::AIConfig;
use crate::document::StyleAnalyzer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseType {
    Action,
    Information,
    Clarification,
    Error,
    MultiStep,
    StyleGuidance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    VeryHigh, // 0.9+
    High,     // 0.8-0.89
    Medium,   // 0.6-0.79
    Low,      // 0.4-0.59
    VeryLow,  // <0.4
}

impl ConfidenceLevel {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => ConfidenceLevel::VeryHigh,
            s if s >= 0.8 => ConfidenceLevel::High,
            s if s >= 0.6 => ConfidenceLevel::Medium,
            s if s >= 0.4 => ConfidenceLevel::Low,
            _ => ConfidenceLevel::VeryLow,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleIssueType {
    Inconsistency, // Style is inconsistent with document/organizational standards
    Clarity,       // Style affects clarity and readability
    Tone,          // Inappropriate tone for audience/context
    Vocabulary,    // Vocabulary level issues (too complex/simple)
    Structure,     // Structural style issues (formatting, organization)
    Branding,      // Branding/voice inconsistencies
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleIssueSeverity {
    Critical, // Must be fixed
    High,     // Should be fixed
    Medium,   // Recommended to fix
    Low,      // Optional improvement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleIssue {
    pub issue_type: StyleIssueType,
    pub severity: StyleIssueSeverity,
    pub description: String,
    pub location: Option<String>,
    pub suggestion: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleGuidance {
    pub issues: Vec<StyleIssue>,
    pub overall_assessment: String,
    pub style_score: f32,
    pub recommendations: Vec<String>,
    pub positive_aspects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub response_type: ResponseType,
    pub content: String,
    pub intent: Intent,
    pub confidence: f32,
    pub confidence_level: ConfidenceLevel,
    pub suggested_actions: Vec<SuggestedAction>,
    pub follow_up_questions: Vec<FollowUpQuestion>,
    pub document_references: Vec<DocumentReferenceContext>,
    pub action_items: Vec<ActionItem>,
    pub metadata: ResponseMetadata,
    pub style_guidance: Option<StyleGuidance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: String,
    pub description: String,
    pub parameters: Option<serde_json::Value>,
    pub priority: ActionPriority,
    pub estimated_duration: Option<String>,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionPriority {
    Immediate,
    High,
    Medium,
    Low,
    Optional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpQuestion {
    pub question: String,
    pub purpose: String,
    pub options: Option<Vec<String>>,
    pub priority: QuestionPriority,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestionPriority {
    Critical,  // Must be answered to proceed
    Important, // Should be answered for best results
    Helpful,   // Nice to have for optimization
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReferenceContext {
    pub document_id: String,
    pub title: String,
    pub relevance_score: f32,
    pub specific_sections: Vec<String>,
    pub reference_reason: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub title: String,
    pub description: String,
    pub action_type: String,
    pub estimated_time: Option<String>,
    pub depends_on: Vec<String>,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub processing_time_ms: u64,
    pub model_used: String,
    pub tokens_used: Option<u32>,
    pub confidence_explanation: String,
    pub context_used: bool,
    pub documents_analyzed: usize,
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub reasoning_chain: Vec<String>,
}

#[allow(dead_code)]
pub struct ResponseGenerator {
    #[allow(dead_code)]
    system_prompts: std::collections::HashMap<Intent, String>,
    #[allow(dead_code)]
    style_analyzer: StyleAnalyzer,
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

        Self {
            system_prompts,
            style_analyzer: StyleAnalyzer::new(),
        }
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

        // Generate enhanced response components
        let confidence_level = ConfidenceLevel::from_score(intent_result.confidence);
        let suggested_actions = self.generate_suggested_actions(&intent_result.intent);
        let follow_up_questions = self.generate_follow_up_questions(
            &intent_result.intent,
            intent_result.confidence,
            None,
        );
        let document_references = self.extract_document_references(context, None);
        let documents_analyzed_count = document_references.len();
        let action_items = self.generate_action_items(&intent_result.intent, &suggested_actions);
        let reasoning_chain = self.build_reasoning_chain(intent_result, context.is_some(), false);

        // Generate style guidance if content analysis is involved
        let style_guidance = self.generate_style_guidance(&intent_result.intent, context);

        Ok(AIResponse {
            response_type,
            content: ai_response,
            intent: intent_result.intent.clone(),
            confidence: intent_result.confidence,
            confidence_level,
            suggested_actions,
            follow_up_questions,
            document_references,
            action_items,
            style_guidance,
            metadata: ResponseMetadata {
                processing_time_ms: processing_time,
                model_used: config.default_model.clone(),
                tokens_used: None, // Ollama doesn't provide token counts in simple API
                confidence_explanation: intent_result.reasoning.clone(),
                context_used: context.is_some(),
                documents_analyzed: documents_analyzed_count,
                session_id: None,
                turn_id: None, // Will be set by conversation manager
                reasoning_chain,
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
                    priority: ActionPriority::Medium,
                    estimated_duration: Some("1 minute".to_string()),
                    prerequisites: vec![],
                },
                SuggestedAction {
                    action_type: "run_comparison".to_string(),
                    description: "Run document comparison".to_string(),
                    parameters: None,
                    priority: ActionPriority::Medium,
                    estimated_duration: Some("2-5 minutes".to_string()),
                    prerequisites: vec!["select_documents".to_string()],
                },
            ],
            Intent::GenerateOutput => vec![
                SuggestedAction {
                    action_type: "select_format".to_string(),
                    description: "Choose output format (Word, PDF, HTML)".to_string(),
                    parameters: None,
                    priority: ActionPriority::Medium,
                    estimated_duration: Some("30 seconds".to_string()),
                    prerequisites: vec![],
                },
                SuggestedAction {
                    action_type: "configure_template".to_string(),
                    description: "Configure document template".to_string(),
                    parameters: None,
                    priority: ActionPriority::Medium,
                    estimated_duration: Some("2 minutes".to_string()),
                    prerequisites: vec![],
                },
            ],
            Intent::SearchDocuments => vec![
                SuggestedAction {
                    action_type: "open_search".to_string(),
                    description: "Open search interface".to_string(),
                    parameters: None,
                    priority: ActionPriority::Medium,
                    estimated_duration: Some("10 seconds".to_string()),
                    prerequisites: vec![],
                },
                SuggestedAction {
                    action_type: "filter_documents".to_string(),
                    description: "Apply document filters".to_string(),
                    parameters: None,
                    priority: ActionPriority::Medium,
                    estimated_duration: Some("30 seconds".to_string()),
                    prerequisites: vec![],
                },
            ],
            Intent::ManageFiles => vec![
                SuggestedAction {
                    action_type: "import_files".to_string(),
                    description: "Import new files".to_string(),
                    parameters: None,
                    priority: ActionPriority::Medium,
                    estimated_duration: Some("1-3 minutes".to_string()),
                    prerequisites: vec![],
                },
                SuggestedAction {
                    action_type: "organize_workspace".to_string(),
                    description: "Organize workspace files".to_string(),
                    parameters: None,
                    priority: ActionPriority::Low,
                    estimated_duration: Some("5-10 minutes".to_string()),
                    prerequisites: vec![],
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

    fn generate_follow_up_questions(
        &self,
        intent: &Intent,
        confidence: f32,
        session: Option<&ConversationSession>,
    ) -> Vec<FollowUpQuestion> {
        let mut questions = Vec::new();

        // Add confidence-based questions
        if confidence < 0.7 {
            questions.push(FollowUpQuestion {
                question: "Could you provide more details about what you're trying to accomplish?"
                    .to_string(),
                purpose: "Clarify intent and improve accuracy".to_string(),
                options: None,
                priority: QuestionPriority::Critical,
            });
        }

        // Add intent-specific questions
        match intent {
            Intent::CompareDocuments => {
                questions.extend(vec![
                    FollowUpQuestion {
                        question:
                            "What specific aspects would you like me to focus on in the comparison?"
                                .to_string(),
                        purpose: "Focus the comparison analysis".to_string(),
                        options: Some(vec![
                            "Content changes".to_string(),
                            "Structural changes".to_string(),
                            "Style differences".to_string(),
                            "All aspects".to_string(),
                        ]),
                        priority: QuestionPriority::Important,
                    },
                    FollowUpQuestion {
                        question: "Do you need the results in a specific format?".to_string(),
                        purpose: "Determine output format preferences".to_string(),
                        options: Some(vec![
                            "Summary report".to_string(),
                            "Detailed analysis".to_string(),
                            "Side-by-side view".to_string(),
                        ]),
                        priority: QuestionPriority::Helpful,
                    },
                ]);
            }
            Intent::GenerateOutput => {
                questions.extend(vec![
                    FollowUpQuestion {
                        question: "Who is the target audience for this output?".to_string(),
                        purpose: "Tailor content appropriately".to_string(),
                        options: Some(vec![
                            "Students/Beginners".to_string(),
                            "Professionals".to_string(),
                            "Experts".to_string(),
                            "General audience".to_string(),
                        ]),
                        priority: QuestionPriority::Important,
                    },
                    FollowUpQuestion {
                        question: "What's the primary purpose of this document?".to_string(),
                        purpose: "Guide content structure and style".to_string(),
                        options: Some(vec![
                            "Training material".to_string(),
                            "Reference guide".to_string(),
                            "Presentation".to_string(),
                            "Assessment".to_string(),
                        ]),
                        priority: QuestionPriority::Important,
                    },
                ]);
            }
            Intent::UpdateContent => {
                questions.push(FollowUpQuestion {
                    question: "Should I preserve the original writing style and tone?".to_string(),
                    purpose: "Maintain consistency in updates".to_string(),
                    options: Some(vec![
                        "Yes, preserve original style".to_string(),
                        "Update to match new standards".to_string(),
                        "Let me decide case by case".to_string(),
                    ]),
                    priority: QuestionPriority::Important,
                });
            }
            Intent::SearchDocuments => {
                if let Some(conv_session) = session {
                    if conv_session.document_references.is_empty() {
                        questions.push(FollowUpQuestion {
                            question:
                                "Would you like me to search across all documents or specific ones?"
                                    .to_string(),
                            purpose: "Scope the search appropriately".to_string(),
                            options: Some(vec![
                                "All documents".to_string(),
                                "Recent documents".to_string(),
                                "Specific document types".to_string(),
                            ]),
                            priority: QuestionPriority::Important,
                        });
                    }
                }
            }
            _ => {}
        }

        questions
    }

    fn extract_document_references(
        &self,
        context: Option<&str>,
        session: Option<&ConversationSession>,
    ) -> Vec<DocumentReferenceContext> {
        let mut references = Vec::new();

        // Extract from context if available
        if let Some(ctx) = context {
            // Simple pattern matching for document references
            let doc_patterns = [
                r#"(?i)document(?:s)?\s+(?:named|called|titled)\s+['"]([^'"]+)['"]?"#,
                r#"(?i)file(?:s)?\s+['"]([^'"]+)['"]?"#,
                r"(?i)([\w\s]+\.(?:docx?|pdf|txt|md))\b",
            ];

            for pattern in &doc_patterns {
                if let Ok(regex) = Regex::new(pattern) {
                    for cap in regex.captures_iter(ctx) {
                        if let Some(title) = cap.get(1) {
                            references.push(DocumentReferenceContext {
                                document_id: format!("doc_{}", references.len()),
                                title: title.as_str().to_string(),
                                relevance_score: 0.8,
                                specific_sections: vec![],
                                reference_reason: "Mentioned in context".to_string(),
                                path: None,
                            });
                        }
                    }
                }
            }
        }

        // Add from session context
        if let Some(conv_session) = session {
            for doc_ref in &conv_session.document_references {
                references.push(DocumentReferenceContext {
                    document_id: doc_ref.document_id.clone(),
                    title: doc_ref.title.clone(),
                    relevance_score: doc_ref.relevance_score,
                    specific_sections: vec![],
                    reference_reason: "Previously discussed in conversation".to_string(),
                    path: doc_ref.path.clone(),
                });
            }
        }

        references
    }

    fn generate_action_items(
        &self,
        intent: &Intent,
        _suggested_actions: &[SuggestedAction],
    ) -> Vec<ActionItem> {
        let mut action_items = Vec::new();

        match intent {
            Intent::CompareDocuments => {
                action_items.push(ActionItem {
                    title: "Document Comparison Analysis".to_string(),
                    description: "Complete comparison between selected documents".to_string(),
                    action_type: "analysis".to_string(),
                    estimated_time: Some("5-10 minutes".to_string()),
                    depends_on: vec![],
                    status: ActionStatus::Pending,
                });
            }
            Intent::GenerateOutput => {
                action_items.push(ActionItem {
                    title: "Document Generation".to_string(),
                    description: "Generate output in specified format".to_string(),
                    action_type: "generation".to_string(),
                    estimated_time: Some("3-8 minutes".to_string()),
                    depends_on: vec![],
                    status: ActionStatus::Pending,
                });
            }
            Intent::UpdateContent => {
                action_items.push(ActionItem {
                    title: "Content Update".to_string(),
                    description: "Apply changes while preserving style and structure".to_string(),
                    action_type: "update".to_string(),
                    estimated_time: Some("2-5 minutes".to_string()),
                    depends_on: vec![],
                    status: ActionStatus::Pending,
                });
            }
            _ => {}
        }

        action_items
    }

    fn build_reasoning_chain(
        &self,
        intent_result: &IntentConfidence,
        has_context: bool,
        has_session: bool,
    ) -> Vec<String> {
        let mut reasoning = vec![
            format!(
                "Analyzed user input and classified intent as {:?}",
                intent_result.intent
            ),
            format!(
                "Confidence score: {:.1}% based on {}",
                intent_result.confidence * 100.0,
                intent_result.reasoning
            ),
        ];

        if has_context {
            reasoning.push("Incorporated relevant context from document analysis".to_string());
        }

        if has_session {
            reasoning.push(
                "Referenced conversation history for better context understanding".to_string(),
            );
        }

        if intent_result.confidence < 0.7 {
            reasoning.push("Generated clarifying questions due to low confidence".to_string());
        }

        if !intent_result.alternative_intents.is_empty() {
            reasoning.push(format!(
                "Considered {} alternative interpretations",
                intent_result.alternative_intents.len()
            ));
        }

        reasoning.push("Generated structured response with actionable suggestions".to_string());

        reasoning
    }

    fn init_follow_up_templates(templates: &mut HashMap<Intent, Vec<String>>) {
        templates.insert(
            Intent::CompareDocuments,
            vec![
                "What specific changes are you most concerned about?".to_string(),
                "Do you need a summary or detailed analysis?".to_string(),
                "Should I focus on any particular sections?".to_string(),
            ],
        );

        templates.insert(
            Intent::GenerateOutput,
            vec![
                "What format would you prefer for the output?".to_string(),
                "Who is the target audience?".to_string(),
                "Do you have any style or branding requirements?".to_string(),
            ],
        );
    }

    fn init_action_templates(_templates: &mut HashMap<Intent, Vec<SuggestedAction>>) {
        // This method can be expanded with more sophisticated action templates
        // For now, we'll use the existing generate_suggested_actions logic
    }

    /// Generate style guidance for content analysis and improvement
    fn generate_style_guidance(
        &self,
        intent: &Intent,
        context: Option<&str>,
    ) -> Option<StyleGuidance> {
        // Only provide style guidance for content-related intents
        match intent {
            Intent::UpdateContent | Intent::CompareDocuments | Intent::GenerateOutput => {
                if let Some(content_text) = context {
                    self.analyze_style_issues(content_text)
                } else {
                    // Provide general style guidance when no specific content is available
                    Some(self.generate_general_style_guidance())
                }
            }
            _ => None,
        }
    }

    /// Analyze specific style issues in the provided content
    fn analyze_style_issues(&self, content: &str) -> Option<StyleGuidance> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut positive_aspects = Vec::new();
        let mut overall_score = 80.0; // Start with a baseline score

        // Analyze various style aspects

        // 1. Check for clarity issues
        if let Some(clarity_issues) = self.check_clarity_issues(content) {
            issues.extend(clarity_issues);
            overall_score -= 10.0;
        } else {
            positive_aspects.push("Content is clear and well-structured".to_string());
        }

        // 2. Check for tone consistency
        if let Some(tone_issues) = self.check_tone_issues(content) {
            issues.extend(tone_issues);
            overall_score -= 15.0;
        } else {
            positive_aspects.push("Tone is consistent throughout".to_string());
        }

        // 3. Check for vocabulary appropriateness
        if let Some(vocab_issues) = self.check_vocabulary_issues(content) {
            issues.extend(vocab_issues);
            overall_score -= 10.0;
        } else {
            positive_aspects.push("Vocabulary level is appropriate".to_string());
        }

        // 4. Check for structural issues
        if let Some(structure_issues) = self.check_structure_issues(content) {
            issues.extend(structure_issues);
            overall_score -= 10.0;
        } else {
            positive_aspects.push("Document structure is well-organized".to_string());
        }

        // Generate recommendations based on issues found
        if issues.is_empty() {
            recommendations.push("Your content maintains good style consistency. Consider reviewing for minor improvements in clarity or engagement.".to_string());
        } else {
            recommendations.push("Address the identified style issues to improve document consistency and readability.".to_string());
            if issues.iter().any(|i| i.issue_type == StyleIssueType::Tone) {
                recommendations.push(
                    "Consider establishing a style guide for consistent tone across documents."
                        .to_string(),
                );
            }
            if issues
                .iter()
                .any(|i| i.issue_type == StyleIssueType::Vocabulary)
            {
                recommendations.push(
                    "Ensure vocabulary matches your target audience's expertise level.".to_string(),
                );
            }
        }

        let overall_assessment = if overall_score >= 85.0 {
            "Excellent style consistency with minor areas for improvement.".to_string()
        } else if overall_score >= 70.0 {
            "Good style foundation with some inconsistencies to address.".to_string()
        } else if overall_score >= 50.0 {
            "Moderate style issues that should be addressed for better consistency.".to_string()
        } else {
            "Significant style inconsistencies require attention to improve document quality."
                .to_string()
        };

        Some(StyleGuidance {
            issues,
            overall_assessment,
            style_score: overall_score / 100.0,
            recommendations,
            positive_aspects,
        })
    }

    /// Generate general style guidance when no specific content is provided
    fn generate_general_style_guidance(&self) -> StyleGuidance {
        StyleGuidance {
            issues: vec![],
            overall_assessment:
                "I can provide style guidance when you share specific content for analysis."
                    .to_string(),
            style_score: 0.0,
            recommendations: vec![
                "Share a document or content snippet for detailed style analysis".to_string(),
                "Consider establishing a style guide for your organization".to_string(),
                "Regularly review content for tone and vocabulary consistency".to_string(),
            ],
            positive_aspects: vec!["Proactive approach to style consistency".to_string()],
        }
    }

    /// Check for clarity issues in content
    fn check_clarity_issues(&self, content: &str) -> Option<Vec<StyleIssue>> {
        let mut issues = Vec::new();

        // Check for overly long sentences (>30 words)
        let sentences: Vec<&str> = content.split(&['.', '!', '?'][..]).collect();
        for (i, sentence) in sentences.iter().enumerate() {
            let word_count = sentence.split_whitespace().count();
            if word_count > 30 {
                issues.push(StyleIssue {
                    issue_type: StyleIssueType::Clarity,
                    severity: StyleIssueSeverity::Medium,
                    description: format!("Sentence {} is very long ({} words)", i + 1, word_count),
                    location: Some(format!("Sentence {}", i + 1)),
                    suggestion: "Consider breaking this into shorter, more digestible sentences"
                        .to_string(),
                    explanation: "Long sentences can reduce readability and comprehension"
                        .to_string(),
                });
            }
        }

        // Check for passive voice overuse
        let passive_indicators = ["was", "were", "been", "being"];
        let passive_count = passive_indicators
            .iter()
            .map(|&indicator| content.matches(indicator).count())
            .sum::<usize>();

        if passive_count > content.split_whitespace().count() / 20 {
            issues.push(StyleIssue {
                issue_type: StyleIssueType::Clarity,
                severity: StyleIssueSeverity::Low,
                description: "Frequent use of passive voice detected".to_string(),
                location: None,
                suggestion: "Consider using more active voice constructions".to_string(),
                explanation: "Active voice generally makes content more direct and engaging"
                    .to_string(),
            });
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }

    /// Check for tone consistency issues
    fn check_tone_issues(&self, content: &str) -> Option<Vec<StyleIssue>> {
        let mut issues = Vec::new();

        // Check for mixed formality levels
        let formal_indicators = ["furthermore", "therefore", "consequently", "henceforth"];
        let informal_indicators = ["it's", "don't", "can't", "you're", "we're"];

        let formal_count = formal_indicators
            .iter()
            .map(|&indicator| content.to_lowercase().matches(indicator).count())
            .sum::<usize>();

        let informal_count = informal_indicators
            .iter()
            .map(|&indicator| content.to_lowercase().matches(indicator).count())
            .sum::<usize>();

        if formal_count > 0 && informal_count > 0 {
            let ratio = formal_count as f32 / (formal_count + informal_count) as f32;
            if ratio > 0.3 && ratio < 0.7 {
                issues.push(StyleIssue {
                    issue_type: StyleIssueType::Tone,
                    severity: StyleIssueSeverity::Medium,
                    description: "Mixed formal and informal language detected".to_string(),
                    location: None,
                    suggestion: "Choose either formal or informal tone and maintain consistency".to_string(),
                    explanation: "Inconsistent formality can confuse readers about the document's intended tone".to_string(),
                });
            }
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }

    /// Check for vocabulary appropriateness issues
    fn check_vocabulary_issues(&self, content: &str) -> Option<Vec<StyleIssue>> {
        let mut issues = Vec::new();

        // Check for overly complex words (>12 characters)
        let words: Vec<&str> = content.split_whitespace().collect();
        let complex_words: Vec<&str> = words
            .iter()
            .filter(|word| word.len() > 12 && word.chars().all(|c| c.is_alphabetic()))
            .cloned()
            .collect();

        if complex_words.len() > words.len() / 25 {
            issues.push(StyleIssue {
                issue_type: StyleIssueType::Vocabulary,
                severity: StyleIssueSeverity::Low,
                description: "High density of complex vocabulary detected".to_string(),
                location: None,
                suggestion: "Consider using simpler alternatives for some complex terms"
                    .to_string(),
                explanation: "Overly complex vocabulary can reduce accessibility for some readers"
                    .to_string(),
            });
        }

        // Check for jargon overuse (basic heuristic)
        let jargon_indicators = ["utilize", "facilitate", "endeavor", "leverage"];
        let jargon_count = jargon_indicators
            .iter()
            .map(|&indicator| content.to_lowercase().matches(indicator).count())
            .sum::<usize>();

        if jargon_count > 5 {
            issues.push(StyleIssue {
                issue_type: StyleIssueType::Vocabulary,
                severity: StyleIssueSeverity::Medium,
                description: "Potential overuse of business jargon".to_string(),
                location: None,
                suggestion: "Consider using more direct, plain language alternatives".to_string(),
                explanation:
                    "Excessive jargon can make content less accessible and harder to understand"
                        .to_string(),
            });
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }

    /// Check for structural style issues
    fn check_structure_issues(&self, content: &str) -> Option<Vec<StyleIssue>> {
        let mut issues = Vec::new();

        // Check for missing headings/structure in longer content
        if content.len() > 1000 && !content.contains('\n') {
            issues.push(StyleIssue {
                issue_type: StyleIssueType::Structure,
                severity: StyleIssueSeverity::Medium,
                description: "Long content appears to lack structural breaks".to_string(),
                location: None,
                suggestion: "Consider adding headings, bullet points, or paragraph breaks"
                    .to_string(),
                explanation: "Well-structured content is easier to read and navigate".to_string(),
            });
        }

        // Check for very short paragraphs (potential formatting issues)
        let paragraphs: Vec<&str> = content
            .split('\n')
            .filter(|p| !p.trim().is_empty())
            .collect();
        let short_paragraphs = paragraphs.iter().filter(|p| p.len() < 50).count();

        if short_paragraphs > paragraphs.len() / 2 && paragraphs.len() > 3 {
            issues.push(StyleIssue {
                issue_type: StyleIssueType::Structure,
                severity: StyleIssueSeverity::Low,
                description: "Many short paragraphs detected".to_string(),
                location: None,
                suggestion: "Consider combining related short paragraphs for better flow"
                    .to_string(),
                explanation: "Very short paragraphs can create choppy reading experience"
                    .to_string(),
            });
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
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

    #[test]
    fn test_enhanced_response_generation() {
        let generator = ResponseGenerator::new();

        // Test confidence level mapping
        assert_eq!(ConfidenceLevel::from_score(0.95), ConfidenceLevel::VeryHigh);
        assert_eq!(ConfidenceLevel::from_score(0.85), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.65), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.45), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.25), ConfidenceLevel::VeryLow);

        // Test follow-up question generation
        let questions =
            generator.generate_follow_up_questions(&Intent::CompareDocuments, 0.9, None);
        assert!(!questions.is_empty());

        // Test document reference extraction
        let context = "Please compare document_a.docx with the file 'project_spec.pdf'";
        let references = generator.extract_document_references(Some(context), None);
        assert!(!references.is_empty());

        // Test action item generation
        let actions = generator.generate_suggested_actions(&Intent::CompareDocuments);
        let action_items = generator.generate_action_items(&Intent::CompareDocuments, &actions);
        assert!(!action_items.is_empty());
    }

    #[test]
    fn test_follow_up_questions() {
        let generator = ResponseGenerator::new();

        // Test low confidence generates clarifying questions
        let questions =
            generator.generate_follow_up_questions(&Intent::CompareDocuments, 0.5, None);
        assert!(questions
            .iter()
            .any(|q| q.priority == QuestionPriority::Critical));

        // Test intent-specific questions
        let questions = generator.generate_follow_up_questions(&Intent::GenerateOutput, 0.9, None);
        assert!(questions.iter().any(|q| q.question.contains("audience")));
    }

    #[test]
    fn test_document_reference_extraction() {
        let generator = ResponseGenerator::new();

        // Test context extraction
        let context = "Compare user_guide.pdf with technical_manual.docx";
        let references = generator.extract_document_references(Some(context), None);
        assert_eq!(references.len(), 2);
        assert!(references.iter().any(|r| r.title.contains("user_guide")));
        assert!(references
            .iter()
            .any(|r| r.title.contains("technical_manual")));
    }

    #[test]
    fn test_reasoning_chain() {
        let generator = ResponseGenerator::new();
        let intent_result = IntentConfidence {
            intent: Intent::CompareDocuments,
            confidence: 0.8,
            reasoning: "High keyword match".to_string(),
            alternative_intents: vec![(Intent::SearchDocuments, 0.3)],
        };

        let reasoning = generator.build_reasoning_chain(&intent_result, true, false);
        assert!(reasoning.len() >= 3);
        assert!(reasoning.iter().any(|r| r.contains("CompareDocuments")));
        assert!(reasoning.iter().any(|r| r.contains("context")));
    }
}
