// src-tauri/src/commands/conversational_intelligence_commands.rs
// Tauri commands for conversational intelligence - intent classification and action execution

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::ai::actions::ActionResult;
use crate::ai::intent::{Intent, IntentClassifier, IntentConfidence};
use crate::vector::EmbeddingEngine;

// State management for conversational intelligence
pub type ConversationalIntelligenceState = Arc<Mutex<Option<ConversationalIntelligenceSystem>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRequest {
    pub user_input: String,
    pub session_id: Option<String>,
    pub context: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub intent: Intent,
    pub confidence: f32,
    pub reasoning: String,
    pub action_executed: bool,
    pub action_result: Option<ActionResult>,
    pub suggested_actions: Vec<String>,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterExtractionRequest {
    pub user_input: String,
    pub intent: Intent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedParameters {
    pub parameters: HashMap<String, String>,
    pub missing_parameters: Vec<String>,
    pub confidence: f32,
}

pub struct ConversationalIntelligenceSystem {
    intent_classifier: IntentClassifier,
    action_executor: Arc<PlaceholderActionExecutor>,
    session_manager: SessionManager,
}

struct SessionManager {
    sessions: HashMap<String, ConversationSession>,
}

#[derive(Debug, Clone)]
struct ConversationSession {
    session_id: String,
    conversation_history: Vec<ConversationTurn>,
    context: HashMap<String, String>,
    created_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationTurn {
    user_input: String,
    intent: Intent,
    confidence: f32,
    action_executed: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
}

// Placeholder function for now - TODO: Implement proper integration
fn create_placeholder_system() -> ConversationalIntelligenceSystem {
    ConversationalIntelligenceSystem {
        intent_classifier: IntentClassifier::new(),
        action_executor: Arc::new(PlaceholderActionExecutor),
        session_manager: SessionManager::new(),
    }
}

// Placeholder ActionExecutor to avoid type conflicts
struct PlaceholderActionExecutor;

impl PlaceholderActionExecutor {
    pub async fn execute(
        &self,
        _action: crate::ai::actions::DocumentAction,
    ) -> anyhow::Result<crate::ai::actions::ActionResult> {
        Ok(crate::ai::actions::ActionResult {
            success: false,
            result_type: crate::ai::actions::ActionResultType::Error,
            data: serde_json::json!({"error": "Conversational intelligence not fully integrated yet"}),
            message: "Conversational intelligence is not fully integrated yet. Please use direct AI chat.".to_string(),
            execution_time_ms: 0,
            suggested_actions: vec![
                "Use direct AI chat for now".to_string(),
                "Wait for full integration".to_string(),
            ],
        })
    }
}

impl ConversationalIntelligenceSystem {
    pub async fn process_conversation(
        &mut self,
        request: ConversationRequest,
    ) -> Result<ConversationResponse> {
        // Generate or use provided session ID
        let session_id = request
            .session_id
            .unwrap_or_else(|| format!("session_{}", chrono::Utc::now().timestamp_millis()));

        // Ensure session exists
        self.session_manager.ensure_session(&session_id);

        // Classify intent
        let intent_result = self.intent_classifier.classify(&request.user_input).await?;

        // Extract parameters for action execution
        let parameters = self
            .extract_parameters(&request.user_input, &intent_result.intent)
            .await?;

        // Execute action if parameters are sufficient
        let (action_executed, action_result) = if !parameters.missing_parameters.is_empty() {
            // Parameters missing - return clarification request
            let clarification_message = format!(
                "I need more information to complete that action. Please provide: {}",
                parameters.missing_parameters.join(", ")
            );
            (
                false,
                Some(ActionResult {
                    success: false,
                    result_type: crate::ai::actions::ActionResultType::HelpContent,
                    data: serde_json::json!({
                        "clarification_needed": true,
                        "missing_parameters": parameters.missing_parameters,
                        "message": clarification_message
                    }),
                    message: clarification_message,
                    execution_time_ms: 0,
                    suggested_actions: vec![
                        "Provide the missing information".to_string(),
                        "Try a different approach".to_string(),
                    ],
                }),
            )
        } else {
            // Execute the action
            match self.intent_to_action(&intent_result.intent, &parameters.parameters) {
                Some(action) => {
                    match self.action_executor.execute(action).await {
                        Ok(result) => (true, Some(result)),
                        Err(e) => (false, Some(ActionResult {
                            success: false,
                            result_type: crate::ai::actions::ActionResultType::Error,
                            data: serde_json::json!({"error": e.to_string()}),
                            message: format!("Failed to execute action: {}", e),
                            execution_time_ms: 0,
                            suggested_actions: vec![
                                "Try a different approach".to_string(),
                                "Check your input".to_string(),
                            ],
                        }))
                    }
                },
                None => (false, Some(ActionResult {
                    success: false,
                    result_type: crate::ai::actions::ActionResultType::Error,
                    data: serde_json::json!({"error": "Unable to map intent to action"}),
                    message: "I understand what you want to do, but I'm not sure how to execute it yet.".to_string(),
                    execution_time_ms: 0,
                    suggested_actions: vec![
                        "Try rephrasing your request".to_string(),
                        "Be more specific".to_string(),
                    ],
                }))
            }
        };

        // Update session with conversation turn
        self.session_manager.add_conversation_turn(
            &session_id,
            ConversationTurn {
                user_input: request.user_input,
                intent: intent_result.intent.clone(),
                confidence: intent_result.confidence,
                action_executed,
                timestamp: chrono::Utc::now(),
            },
        );

        // Generate suggested actions
        let suggested_actions =
            self.generate_suggested_actions(&intent_result.intent, action_executed);

        Ok(ConversationResponse {
            intent: intent_result.intent,
            confidence: intent_result.confidence,
            reasoning: intent_result.reasoning,
            action_executed,
            action_result,
            suggested_actions,
            session_id,
        })
    }

    async fn extract_parameters(
        &self,
        user_input: &str,
        intent: &Intent,
    ) -> Result<ExtractedParameters> {
        let mut parameters = HashMap::new();
        let mut missing_parameters = Vec::new();

        // Extract parameters based on intent type
        match intent {
            Intent::CompareDocuments => {
                // Look for document names/IDs in the input
                let doc_patterns = self.extract_document_references(user_input);
                if doc_patterns.len() >= 2 {
                    parameters.insert("doc_a".to_string(), doc_patterns[0].clone());
                    parameters.insert("doc_b".to_string(), doc_patterns[1].clone());
                } else {
                    if doc_patterns.is_empty() {
                        missing_parameters.push("first_document".to_string());
                    }
                    if doc_patterns.len() < 2 {
                        missing_parameters.push("second_document".to_string());
                    }
                }

                // Extract comparison type if specified
                if user_input.contains("semantic") {
                    parameters.insert("comparison_type".to_string(), "semantic".to_string());
                } else if user_input.contains("structural") {
                    parameters.insert("comparison_type".to_string(), "structural".to_string());
                } else if user_input.contains("text") {
                    parameters.insert("comparison_type".to_string(), "text".to_string());
                }
            }
            Intent::SummarizeDocument => {
                let doc_patterns = self.extract_document_references(user_input);
                if !doc_patterns.is_empty() {
                    parameters.insert("document".to_string(), doc_patterns[0].clone());
                } else {
                    missing_parameters.push("document".to_string());
                }

                // Extract length if specified
                if user_input.contains("brief") {
                    parameters.insert("length".to_string(), "brief".to_string());
                } else if user_input.contains("short") {
                    parameters.insert("length".to_string(), "short".to_string());
                } else if user_input.contains("detailed") {
                    parameters.insert("length".to_string(), "detailed".to_string());
                }
            }
            Intent::AnalyzeDocument => {
                let doc_patterns = self.extract_document_references(user_input);
                if !doc_patterns.is_empty() {
                    parameters.insert("document".to_string(), doc_patterns[0].clone());
                } else {
                    missing_parameters.push("document".to_string());
                }

                // Extract analysis type
                if user_input.contains("structure") {
                    parameters.insert("analysis_type".to_string(), "structure".to_string());
                } else if user_input.contains("content") {
                    parameters.insert("analysis_type".to_string(), "content".to_string());
                } else if user_input.contains("style") {
                    parameters.insert("analysis_type".to_string(), "style".to_string());
                }
            }
            Intent::SearchDocuments => {
                // Extract search query - everything after "search for" or similar patterns
                let query = self.extract_search_query(user_input);
                if !query.is_empty() {
                    parameters.insert("query".to_string(), query);
                } else {
                    missing_parameters.push("search_query".to_string());
                }

                // Extract search type
                if user_input.contains("semantic") {
                    parameters.insert("search_type".to_string(), "semantic".to_string());
                } else if user_input.contains("keyword") {
                    parameters.insert("search_type".to_string(), "keyword".to_string());
                }
            }
            Intent::GetHelp => {
                // Extract help topic
                let topic = self.extract_help_topic(user_input);
                if !topic.is_empty() {
                    parameters.insert("topic".to_string(), topic);
                }
            }
            _ => {
                // For other intents, do basic extraction
                let doc_patterns = self.extract_document_references(user_input);
                if !doc_patterns.is_empty() {
                    parameters.insert("document".to_string(), doc_patterns[0].clone());
                }
            }
        }

        let confidence = if missing_parameters.is_empty() {
            0.9
        } else {
            0.5
        };

        Ok(ExtractedParameters {
            parameters,
            missing_parameters,
            confidence,
        })
    }

    fn extract_document_references(&self, input: &str) -> Vec<String> {
        let mut documents = Vec::new();

        // Look for quoted strings that might be document names
        let quote_pattern = regex::Regex::new(r#""([^"]+)""#).unwrap();
        for captures in quote_pattern.captures_iter(input) {
            if let Some(doc_name) = captures.get(1) {
                documents.push(doc_name.as_str().to_string());
            }
        }

        // Look for common file extensions
        let file_pattern = regex::Regex::new(r"\b([^\s]+\.(docx?|pdf|txt|md))\b").unwrap();
        for captures in file_pattern.captures_iter(input) {
            if let Some(filename) = captures.get(1) {
                documents.push(filename.as_str().to_string());
            }
        }

        // Look for patterns like "document A" and "document B"
        let doc_pattern =
            regex::Regex::new(r"\b(?:document|file|doc)\s+([A-Za-z0-9_-]+)\b").unwrap();
        for captures in doc_pattern.captures_iter(input) {
            if let Some(doc_id) = captures.get(1) {
                documents.push(format!("document_{}", doc_id.as_str()));
            }
        }

        documents
    }

    fn extract_search_query(&self, input: &str) -> String {
        // Look for patterns like "search for X", "find X", "look for X"
        let search_patterns = [
            r"search\s+for\s+(.+?)(?:\.|$)",
            r"find\s+(.+?)(?:\.|$)",
            r"look\s+for\s+(.+?)(?:\.|$)",
            r"show\s+me\s+(.+?)(?:\.|$)",
        ];

        for pattern in &search_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(input) {
                    if let Some(query) = captures.get(1) {
                        return query.as_str().trim().to_string();
                    }
                }
            }
        }

        // Fallback: use the whole input minus common command words
        let command_words = [
            "search",
            "find",
            "look",
            "show",
            "me",
            "for",
            "documents",
            "files",
        ];
        let words: Vec<&str> = input
            .split_whitespace()
            .filter(|word| !command_words.contains(&word.to_lowercase().as_str()))
            .collect();

        words.join(" ")
    }

    fn extract_help_topic(&self, input: &str) -> String {
        // Look for "help with X" or "how to X"
        let help_patterns = [
            r"help\s+(?:with\s+)?(.+?)(?:\?|$)",
            r"how\s+(?:do\s+I\s+|to\s+)?(.+?)(?:\?|$)",
        ];

        for pattern in &help_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(input) {
                    if let Some(topic) = captures.get(1) {
                        return topic.as_str().trim().to_string();
                    }
                }
            }
        }

        String::new()
    }

    fn intent_to_action(
        &self,
        _intent: &Intent,
        _parameters: &HashMap<String, String>,
    ) -> Option<crate::ai::actions::DocumentAction> {
        // For now, return None since we're using placeholder implementation
        None
    }

    fn generate_suggested_actions(&self, intent: &Intent, action_executed: bool) -> Vec<String> {
        if action_executed {
            match intent {
                Intent::CompareDocuments => vec![
                    "Review the comparison results".to_string(),
                    "Export the comparison report".to_string(),
                    "Compare with another document".to_string(),
                ],
                Intent::SummarizeDocument => vec![
                    "Generate a different length summary".to_string(),
                    "Focus on specific aspects".to_string(),
                    "Export the summary".to_string(),
                ],
                Intent::SearchDocuments => vec![
                    "Refine your search".to_string(),
                    "Try a different search method".to_string(),
                    "Filter the results".to_string(),
                ],
                _ => vec![
                    "Try another operation".to_string(),
                    "Ask for help".to_string(),
                ],
            }
        } else {
            vec![
                "Provide more specific information".to_string(),
                "Try rephrasing your request".to_string(),
                "Ask for help with this operation".to_string(),
            ]
        }
    }
}

impl SessionManager {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    fn ensure_session(&mut self, session_id: &str) {
        if !self.sessions.contains_key(session_id) {
            let session = ConversationSession {
                session_id: session_id.to_string(),
                conversation_history: Vec::new(),
                context: HashMap::new(),
                created_at: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
            };
            self.sessions.insert(session_id.to_string(), session);
        }
    }

    fn add_conversation_turn(&mut self, session_id: &str, turn: ConversationTurn) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.conversation_history.push(turn);
            session.last_activity = chrono::Utc::now();

            // Keep only last 10 turns to manage memory
            if session.conversation_history.len() > 10 {
                session.conversation_history.remove(0);
            }
        }
    }

    fn get_session(&self, session_id: &str) -> Option<&ConversationSession> {
        self.sessions.get(session_id)
    }

    #[allow(dead_code)]
    fn cleanup_old_sessions(&mut self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        self.sessions
            .retain(|_, session| session.last_activity > cutoff);
    }
}

// Tauri Commands

#[tauri::command]
pub async fn initialize_conversational_intelligence(
    state: State<'_, ConversationalIntelligenceState>,
    document_indexer_state: State<
        '_,
        crate::commands::document_indexing_commands::DocumentIndexerState,
    >,
    _vector_state: State<'_, crate::commands::vector_commands::VectorState>,
    embedding_engine_state: State<'_, Arc<tokio::sync::RwLock<Option<EmbeddingEngine>>>>,
) -> Result<String, String> {
    let mut conversational_state = state.lock().await;

    if conversational_state.is_some() {
        return Ok("Conversational intelligence already initialized".to_string());
    }

    // Check if all required dependencies are initialized
    let has_indexer = {
        let indexer_guard = document_indexer_state.lock().await;
        indexer_guard.is_some()
    };

    if !has_indexer {
        return Err(
            "Document indexer not initialized. Please initialize document indexer first."
                .to_string(),
        );
    }

    let has_embedding_engine = {
        let engine_guard = embedding_engine_state.read().await;
        engine_guard.is_some()
    };

    if !has_embedding_engine {
        return Err(
            "Embedding engine not initialized. Please initialize embedding system first."
                .to_string(),
        );
    }

    // For now, create a placeholder conversational system
    // TODO: Implement proper type-safe integration between Optional state and ActionExecutor requirements
    let conversational_system = create_placeholder_system();

    *conversational_state = Some(conversational_system);

    Ok("Conversational intelligence system initialized successfully".to_string())
}

#[tauri::command]
pub async fn process_conversation(
    state: State<'_, ConversationalIntelligenceState>,
    request: ConversationRequest,
) -> Result<ConversationResponse, String> {
    let mut conversational_state = state.lock().await;

    if let Some(ref mut system) = *conversational_state {
        system
            .process_conversation(request)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Conversational intelligence not initialized. Please initialize first.".to_string())
    }
}

#[tauri::command]
pub async fn classify_user_intent(
    state: State<'_, ConversationalIntelligenceState>,
    user_input: String,
) -> Result<IntentConfidence, String> {
    let conversational_state = state.lock().await;

    if let Some(ref system) = *conversational_state {
        system
            .intent_classifier
            .classify(&user_input)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Conversational intelligence not initialized. Please initialize first.".to_string())
    }
}

#[tauri::command]
pub async fn extract_parameters(
    state: State<'_, ConversationalIntelligenceState>,
    request: ParameterExtractionRequest,
) -> Result<ExtractedParameters, String> {
    let conversational_state = state.lock().await;

    if let Some(ref system) = *conversational_state {
        system
            .extract_parameters(&request.user_input, &request.intent)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Conversational intelligence not initialized. Please initialize first.".to_string())
    }
}

#[tauri::command]
pub async fn get_conversation_session(
    state: State<'_, ConversationalIntelligenceState>,
    session_id: String,
) -> Result<Option<serde_json::Value>, String> {
    let conversational_state = state.lock().await;

    if let Some(ref system) = *conversational_state {
        if let Some(session) = system.session_manager.get_session(&session_id) {
            Ok(Some(serde_json::json!({
                "session_id": session.session_id,
                "created_at": session.created_at,
                "last_activity": session.last_activity,
                "conversation_history": session.conversation_history,
                "context": session.context
            })))
        } else {
            Ok(None)
        }
    } else {
        Err("Conversational intelligence not initialized. Please initialize first.".to_string())
    }
}

#[tauri::command]
pub async fn get_supported_intents(
    state: State<'_, ConversationalIntelligenceState>,
) -> Result<Vec<Intent>, String> {
    let conversational_state = state.lock().await;

    if let Some(ref system) = *conversational_state {
        Ok(system.intent_classifier.get_supported_intents())
    } else {
        Err("Conversational intelligence not initialized. Please initialize first.".to_string())
    }
}

#[tauri::command]
#[allow(dead_code)]
pub async fn cleanup_conversation_sessions(
    state: State<'_, ConversationalIntelligenceState>,
) -> Result<String, String> {
    let mut conversational_state = state.lock().await;

    if let Some(ref mut system) = *conversational_state {
        system.session_manager.cleanup_old_sessions();
        Ok("Old conversation sessions cleaned up".to_string())
    } else {
        Err("Conversational intelligence not initialized. Please initialize first.".to_string())
    }
}

#[tauri::command]
pub async fn get_conversational_intelligence_status(
    state: State<'_, ConversationalIntelligenceState>,
) -> Result<serde_json::Value, String> {
    let conversational_state = state.lock().await;

    let is_initialized = conversational_state.is_some();
    let session_count = if let Some(ref system) = *conversational_state {
        system.session_manager.sessions.len()
    } else {
        0
    };

    Ok(serde_json::json!({
        "initialized": is_initialized,
        "active_sessions": session_count,
        "capabilities": [
            "intent_classification",
            "parameter_extraction",
            "action_execution",
            "session_management"
        ]
    }))
}
