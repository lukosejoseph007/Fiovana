// src-tauri/src/ai/conversation_context.rs
// Enhanced conversation context management system for multi-turn conversations

use crate::ai::context::WorkspaceIntelligenceContext;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced conversation context manager with reference resolution
/// and multi-turn conversation memory
#[derive(Debug, Clone, Default)]
pub struct ConversationContextManager {
    /// Active conversation sessions
    sessions: HashMap<String, ConversationSession>,
}

/// A complete conversation session with enhanced context tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    /// Unique session identifier
    pub session_id: String,
    /// Complete conversation history
    pub conversation_history: Vec<ConversationTurn>,
    /// Document references mentioned in conversation
    pub document_references: Vec<DocumentReference>,
    /// Current conversation context and state
    pub conversation_state: ConversationState,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Workspace intelligence context for this session
    pub workspace_intelligence: Option<WorkspaceIntelligenceContext>,
}

/// Individual conversation turn with enhanced context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Turn identifier
    pub turn_id: String,
    /// Role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Intent extracted from this turn (if applicable)
    pub intent: Option<String>,
    /// Entities and references resolved in this turn
    pub resolved_references: Vec<ResolvedReference>,
    /// Actions performed in this turn
    pub actions_performed: Vec<String>,
    /// Context carried forward from this turn
    pub context_contributions: ContextContributions,
}

/// Current state of the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    /// Current topic being discussed
    pub current_topic: Option<String>,
    /// Active documents in context
    pub active_documents: Vec<DocumentReference>,
    /// Pending questions or clarifications
    pub pending_clarifications: Vec<String>,
    /// Current task or operation in progress
    pub current_task: Option<TaskContext>,
    /// Key entities being discussed
    pub active_entities: HashMap<String, EntityReference>,
    /// Conversation flow state
    pub flow_state: ConversationFlowState,
}

/// Document reference with conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReference {
    /// Document identifier
    pub document_id: String,
    /// Document title/name
    pub title: String,
    /// File path if available
    pub path: Option<String>,
    /// When this document was first mentioned
    pub first_mentioned: chrono::DateTime<chrono::Utc>,
    /// Last time referenced
    pub last_referenced: chrono::DateTime<chrono::Utc>,
    /// Reference count in conversation
    pub reference_count: usize,
    /// Context in which it was mentioned
    pub mention_contexts: Vec<String>,
    /// Relevance score to current conversation
    pub relevance_score: f32,
    /// Aliases used to refer to this document
    pub aliases: Vec<String>,
}

/// Resolved reference from conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedReference {
    /// Original text that was referenced
    pub original_text: String,
    /// Type of reference
    pub reference_type: ReferenceType,
    /// What it resolves to
    pub resolved_to: String,
    /// Confidence in resolution
    pub confidence: f32,
    /// Context that helped resolve it
    pub resolution_context: String,
}

/// Types of references that can be resolved
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Pronoun reference (it, that, this, etc.)
    Pronoun,
    /// Document reference (the file, that document, etc.)
    Document,
    /// Previous action (the comparison, last search, etc.)
    Action,
    /// Previous result (those results, the findings, etc.)
    Result,
    /// Entity reference (the user, that section, etc.)
    Entity,
    /// Temporal reference (before, earlier, previously, etc.)
    Temporal,
}

/// Context contributions from a conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextContributions {
    /// New entities introduced
    pub new_entities: Vec<EntityReference>,
    /// Document references added
    pub document_references: Vec<String>,
    /// Topics introduced or changed
    pub topics: Vec<String>,
    /// Actions that can be referenced later
    pub referenceable_actions: Vec<String>,
    /// Key information for future reference
    pub key_information: Vec<KeyInformation>,
}

/// Entity that can be referenced in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReference {
    /// Entity identifier
    pub entity_id: String,
    /// Entity type (document, person, concept, etc.)
    pub entity_type: String,
    /// Display name
    pub name: String,
    /// Description or context
    pub description: Option<String>,
    /// When first mentioned
    pub first_mentioned: chrono::DateTime<chrono::Utc>,
    /// Last referenced
    pub last_referenced: chrono::DateTime<chrono::Utc>,
    /// Alternative names/aliases
    pub aliases: Vec<String>,
    /// Associated metadata
    pub metadata: HashMap<String, String>,
}

/// Current task context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Task identifier
    pub task_id: String,
    /// Task type
    pub task_type: String,
    /// Task description
    pub description: String,
    /// Task status
    pub status: TaskStatus,
    /// Task parameters
    pub parameters: HashMap<String, String>,
    /// Started timestamp
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Expected completion time
    pub expected_completion: Option<chrono::DateTime<chrono::Utc>>,
    /// Progress updates
    pub progress_updates: Vec<String>,
}

/// Task execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is starting
    Starting,
    /// Task is in progress
    InProgress,
    /// Task is waiting for user input
    WaitingForInput,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Key information that should be remembered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInformation {
    /// Information identifier
    pub info_id: String,
    /// Information type
    pub info_type: String,
    /// The actual information
    pub content: String,
    /// When it was mentioned
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Importance score
    pub importance: f32,
    /// Context where it was mentioned
    pub context: String,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity time
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// User preferences for this session
    pub user_preferences: UserPreferences,
    /// Session tags or labels
    pub tags: Vec<String>,
    /// Session summary
    pub summary: Option<String>,
}

/// User preferences for conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Remember conversation history between sessions
    pub persist_history: bool,
    /// Maximum conversation history to maintain
    pub max_history_turns: usize,
    /// Enable automatic reference resolution
    pub auto_resolve_references: bool,
    /// Preferred level of context detail
    pub context_detail_level: ContextDetailLevel,
    /// Include document previews in context
    pub include_document_previews: bool,
}

/// Level of context detail to maintain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextDetailLevel {
    /// Minimal context - just basic references
    Minimal,
    /// Standard context - document refs and recent actions
    Standard,
    /// Detailed context - full conversation awareness
    Detailed,
    /// Complete context - everything remembered
    Complete,
}

/// Conversation flow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationFlowState {
    /// Starting a new conversation
    Starting,
    /// Normal conversation flow
    Normal,
    /// Asking for clarification
    Clarifying,
    /// Executing a multi-step task
    ExecutingTask,
    /// Waiting for user confirmation
    WaitingConfirmation,
    /// Conversation ending/wrapping up
    Ending,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            persist_history: true,
            max_history_turns: 50,
            auto_resolve_references: true,
            context_detail_level: ContextDetailLevel::Standard,
            include_document_previews: true,
        }
    }
}

impl ConversationContextManager {
    /// Create a new conversation context manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Get or create a conversation session
    pub fn get_or_create_session(&mut self, session_id: &str) -> &mut ConversationSession {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| ConversationSession {
                session_id: session_id.to_string(),
                conversation_history: Vec::new(),
                document_references: Vec::new(),
                conversation_state: ConversationState {
                    current_topic: None,
                    active_documents: Vec::new(),
                    pending_clarifications: Vec::new(),
                    current_task: None,
                    active_entities: HashMap::new(),
                    flow_state: ConversationFlowState::Starting,
                },
                metadata: SessionMetadata {
                    created_at: chrono::Utc::now(),
                    last_activity: chrono::Utc::now(),
                    user_preferences: UserPreferences::default(),
                    tags: Vec::new(),
                    summary: None,
                },
                workspace_intelligence: None,
            })
    }

    /// Add a conversation turn with automatic context enhancement
    pub fn add_conversation_turn(
        &mut self,
        session_id: &str,
        role: &str,
        content: &str,
        intent: Option<String>,
    ) -> Result<String> {
        let turn_id = format!(
            "turn_{}_{}",
            session_id,
            chrono::Utc::now().timestamp_millis()
        );

        // Create the basic turn first
        let mut turn = ConversationTurn {
            turn_id: turn_id.clone(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            intent,
            resolved_references: Vec::new(),
            actions_performed: Vec::new(),
            context_contributions: ContextContributions {
                new_entities: Vec::new(),
                document_references: Vec::new(),
                topics: Vec::new(),
                referenceable_actions: Vec::new(),
                key_information: Vec::new(),
            },
        };

        // First, process references and context before getting mutable session access
        let temp_session_data = self.sessions.get(session_id).cloned();

        // Process references and context
        if let Some(temp_session) = temp_session_data {
            turn.resolved_references = self.resolve_references_immutable(content, &temp_session);
            turn.context_contributions =
                self.extract_context_contributions_immutable(content, &turn.resolved_references);
        } else {
            // For new sessions, just extract context contributions without references
            turn.context_contributions =
                self.extract_context_contributions_immutable(content, &Vec::new());
        }

        // Now get mutable access to the session
        let session = self.get_or_create_session(session_id);

        // Update document references inline
        for doc_ref in &turn.context_contributions.document_references {
            if let Some(existing) = session
                .conversation_state
                .active_documents
                .iter_mut()
                .find(|d| d.title == *doc_ref || d.aliases.contains(doc_ref))
            {
                // Update existing reference
                existing.last_referenced = chrono::Utc::now();
                existing.reference_count += 1;
                if !existing.aliases.contains(doc_ref) {
                    existing.aliases.push(doc_ref.clone());
                }
            } else {
                // Add new document reference
                let doc_reference = DocumentReference {
                    document_id: format!(
                        "doc_{}_{}",
                        doc_ref.replace(" ", "_"),
                        chrono::Utc::now().timestamp_millis()
                    ),
                    title: doc_ref.clone(),
                    path: None,
                    first_mentioned: chrono::Utc::now(),
                    last_referenced: chrono::Utc::now(),
                    reference_count: 1,
                    mention_contexts: Vec::new(),
                    relevance_score: 1.0,
                    aliases: vec![doc_ref.clone()],
                };
                session
                    .conversation_state
                    .active_documents
                    .push(doc_reference);
            }
        }

        // Update topics inline
        if !turn.context_contributions.topics.is_empty() {
            session.conversation_state.current_topic =
                turn.context_contributions.topics.first().cloned();
        }

        // Update entities from resolved references inline
        for reference in &turn.resolved_references {
            if matches!(
                reference.reference_type,
                ReferenceType::Entity | ReferenceType::Document
            ) {
                let entity_name = &reference.resolved_to;
                let entity_type = match reference.reference_type {
                    ReferenceType::Document => "document",
                    ReferenceType::Entity => "entity",
                    _ => "unknown",
                };

                if let Some(existing) = session
                    .conversation_state
                    .active_entities
                    .get_mut(entity_name)
                {
                    existing.last_referenced = chrono::Utc::now();
                } else {
                    let entity = EntityReference {
                        entity_id: format!(
                            "entity_{}_{}",
                            entity_name.replace(" ", "_"),
                            chrono::Utc::now().timestamp_millis()
                        ),
                        entity_type: entity_type.to_string(),
                        name: entity_name.clone(),
                        description: None,
                        first_mentioned: chrono::Utc::now(),
                        last_referenced: chrono::Utc::now(),
                        aliases: vec![entity_name.clone()],
                        metadata: HashMap::new(),
                    };
                    session
                        .conversation_state
                        .active_entities
                        .insert(entity_name.clone(), entity);
                }
            }
        }

        // Update flow state based on content inline
        let content_lower = turn.content.to_lowercase();
        session.conversation_state.flow_state = if content_lower.contains("what do you mean")
            || content_lower.contains("can you clarify")
            || content_lower.contains("i don't understand")
            || content_lower.contains("could you explain")
        {
            ConversationFlowState::Clarifying
        } else if content_lower.contains("is that correct")
            || content_lower.contains("should i proceed")
            || content_lower.contains("do you want me to")
            || content_lower.contains("shall i")
        {
            ConversationFlowState::WaitingConfirmation
        } else if content_lower.contains("let me")
            || content_lower.contains("i'll")
            || content_lower.contains("starting")
            || content_lower.contains("processing")
        {
            ConversationFlowState::ExecutingTask
        } else if content_lower.contains("goodbye")
            || content_lower.contains("thank you")
            || content_lower.contains("that's all")
            || content_lower.contains("bye")
        {
            ConversationFlowState::Ending
        } else {
            // Default to normal flow
            match session.conversation_state.flow_state {
                ConversationFlowState::Starting => ConversationFlowState::Normal,
                ref state => state.clone(),
            }
        };

        // Add turn to history
        session.conversation_history.push(turn);

        // Trim history if needed - inline implementation
        let max_turns = session.metadata.user_preferences.max_history_turns;
        if session.conversation_history.len() > max_turns {
            let excess = session.conversation_history.len() - max_turns;
            session.conversation_history.drain(0..excess);
        }

        // Update last activity
        session.metadata.last_activity = chrono::Utc::now();

        Ok(turn_id)
    }

    /// Resolve references without mutable borrow
    fn resolve_references_immutable(
        &self,
        content: &str,
        session: &ConversationSession,
    ) -> Vec<ResolvedReference> {
        let mut resolved = Vec::new();

        // Common pronoun patterns
        let pronoun_patterns = [
            ("it", ReferenceType::Pronoun),
            ("that", ReferenceType::Pronoun),
            ("this", ReferenceType::Pronoun),
            ("them", ReferenceType::Pronoun),
            ("those", ReferenceType::Pronoun),
            ("these", ReferenceType::Pronoun),
        ];

        // Document reference patterns
        let doc_patterns = [
            ("the document", ReferenceType::Document),
            ("that file", ReferenceType::Document),
            ("this file", ReferenceType::Document),
            ("the file", ReferenceType::Document),
            ("that document", ReferenceType::Document),
            ("this document", ReferenceType::Document),
        ];

        // Action reference patterns
        let action_patterns = [
            ("the comparison", ReferenceType::Action),
            ("that analysis", ReferenceType::Action),
            ("the search", ReferenceType::Action),
            ("that search", ReferenceType::Action),
            ("the results", ReferenceType::Result),
            ("those results", ReferenceType::Result),
            ("the findings", ReferenceType::Result),
        ];

        // Temporal reference patterns
        let temporal_patterns = [
            ("earlier", ReferenceType::Temporal),
            ("before", ReferenceType::Temporal),
            ("previously", ReferenceType::Temporal),
            ("last time", ReferenceType::Temporal),
            ("the last", ReferenceType::Temporal),
        ];

        let all_patterns = [
            &pronoun_patterns[..],
            &doc_patterns[..],
            &action_patterns[..],
            &temporal_patterns[..],
        ]
        .concat();

        for (pattern, ref_type) in all_patterns {
            if content.to_lowercase().contains(pattern) {
                if let Some(resolution) =
                    self.resolve_pattern_immutable(pattern, &ref_type, session)
                {
                    resolved.push(ResolvedReference {
                        original_text: pattern.to_string(),
                        reference_type: ref_type,
                        resolved_to: resolution.0,
                        confidence: resolution.1,
                        resolution_context: resolution.2,
                    });
                }
            }
        }

        resolved
    }

    /// Extract context contributions immutably
    fn extract_context_contributions_immutable(
        &self,
        content: &str,
        resolved_references: &[ResolvedReference],
    ) -> ContextContributions {
        let mut contributions = ContextContributions {
            new_entities: Vec::new(),
            document_references: Vec::new(),
            topics: Vec::new(),
            referenceable_actions: Vec::new(),
            key_information: Vec::new(),
        };

        // Extract document references (file names, quoted documents, etc.)
        if let Ok(file_regex) = Regex::new(r"([a-zA-Z0-9_.-]+\.(docx?|pdf|txt|md|xlsx?|pptx?))") {
            for captures in file_regex.captures_iter(content) {
                if let Some(filename) = captures.get(1) {
                    contributions
                        .document_references
                        .push(filename.as_str().to_string());
                }
            }
        } else {
            // Fallback: simple pattern matching for common file extensions
            for word in content.split_whitespace() {
                if word.ends_with(".pdf")
                    || word.ends_with(".docx")
                    || word.ends_with(".doc")
                    || word.ends_with(".txt")
                    || word.ends_with(".md")
                {
                    contributions.document_references.push(word.to_string());
                }
            }
        }

        // Extract quoted document names
        if let Ok(quote_regex) = Regex::new(r#""([^"]+)""#) {
            for captures in quote_regex.captures_iter(content) {
                if let Some(quoted) = captures.get(1) {
                    // If it looks like a document name, add it
                    let quoted_str = quoted.as_str();
                    if quoted_str.len() > 3
                        && (quoted_str.contains("guide")
                            || quoted_str.contains("manual")
                            || quoted_str.contains("document")
                            || quoted_str.contains("report"))
                    {
                        contributions
                            .document_references
                            .push(quoted_str.to_string());
                    }
                }
            }
        }

        // Extract action references
        let action_keywords = [
            "compare", "analyze", "search", "find", "generate", "create", "update", "review",
        ];
        for keyword in &action_keywords {
            if content.to_lowercase().contains(keyword) {
                contributions
                    .referenceable_actions
                    .push(format!("{} action", keyword));
            }
        }

        // Extract topics (simple keyword-based for now)
        let topic_keywords = [
            "authentication",
            "security",
            "API",
            "documentation",
            "training",
            "procedure",
            "workflow",
        ];
        for keyword in &topic_keywords {
            if content.to_lowercase().contains(&keyword.to_lowercase()) {
                contributions.topics.push(keyword.to_string());
            }
        }

        // Create key information from resolved references
        for reference in resolved_references {
            contributions.key_information.push(KeyInformation {
                info_id: format!(
                    "ref_{}_{}",
                    reference.reference_type as u8,
                    chrono::Utc::now().timestamp_millis()
                ),
                info_type: "resolved_reference".to_string(),
                content: format!("{} -> {}", reference.original_text, reference.resolved_to),
                timestamp: chrono::Utc::now(),
                importance: reference.confidence,
                context: reference.resolution_context.clone(),
            });
        }

        contributions
    }

    /// Resolve a pattern immutably
    #[allow(clippy::manual_map)]
    fn resolve_pattern_immutable(
        &self,
        pattern: &str,
        ref_type: &ReferenceType,
        session: &ConversationSession,
    ) -> Option<(String, f32, String)> {
        match ref_type {
            ReferenceType::Document => {
                // Find the most recently referenced document
                if let Some(doc) = session
                    .conversation_state
                    .active_documents
                    .iter()
                    .max_by_key(|d| d.last_referenced)
                {
                    Some((
                        doc.title.clone(),
                        0.8,
                        format!("Most recent document: {}", doc.title),
                    ))
                } else {
                    None
                }
            }
            ReferenceType::Action => {
                // Find the most recent action mentioned
                if let Some(turn) = session
                    .conversation_history
                    .iter()
                    .rev()
                    .find(|t| !t.actions_performed.is_empty())
                {
                    if let Some(action) = turn.actions_performed.first() {
                        Some((
                            action.clone(),
                            0.7,
                            format!("Most recent action: {}", action),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ReferenceType::Result => {
                // Find the most recent result or output
                if let Some(turn) = session.conversation_history.iter().rev().find(|t| {
                    t.role == "assistant"
                        && (t.content.contains("result")
                            || t.content.contains("found")
                            || t.content.contains("analysis"))
                }) {
                    Some((
                        format!(
                            "Result from: {}",
                            turn.content.chars().take(50).collect::<String>()
                        ),
                        0.6,
                        "Most recent assistant result".to_string(),
                    ))
                } else {
                    None
                }
            }
            ReferenceType::Temporal => {
                // Find previous relevant turn
                if session.conversation_history.len() > 1 {
                    let prev_turn =
                        &session.conversation_history[session.conversation_history.len() - 2];
                    Some((
                        format!(
                            "Previous turn: {}",
                            prev_turn.content.chars().take(50).collect::<String>()
                        ),
                        0.5,
                        "Previous conversation turn".to_string(),
                    ))
                } else {
                    None
                }
            }
            ReferenceType::Pronoun => {
                // Try to resolve pronoun to most recent entity
                if let Some((_, entity)) = session
                    .conversation_state
                    .active_entities
                    .iter()
                    .max_by_key(|(_, e)| e.last_referenced)
                {
                    Some((
                        entity.name.clone(),
                        0.6,
                        format!("Most recent entity: {}", entity.name),
                    ))
                } else {
                    // Fallback to most recent document
                    if let Some(doc) = session
                        .conversation_state
                        .active_documents
                        .iter()
                        .max_by_key(|d| d.last_referenced)
                    {
                        Some((
                            doc.title.clone(),
                            0.5,
                            format!("Most recent document: {}", doc.title),
                        ))
                    } else {
                        None
                    }
                }
            }
            ReferenceType::Entity => {
                // Try to resolve to a known entity
                if let Some((_, entity)) = session
                    .conversation_state
                    .active_entities
                    .iter()
                    .find(|(_, e)| e.aliases.iter().any(|alias| pattern.contains(alias)))
                {
                    Some((
                        entity.name.clone(),
                        0.8,
                        format!("Known entity: {}", entity.name),
                    ))
                } else {
                    None
                }
            }
        }
    }

    /// Update session state from a completed turn
    #[allow(dead_code)]
    fn update_session_state_from_turn(
        &mut self,
        session: &mut ConversationSession,
        turn: &ConversationTurn,
    ) -> Result<()> {
        // Update document references
        for doc_ref in &turn.context_contributions.document_references {
            self.add_or_update_document_reference_in_session(session, doc_ref);
        }

        // Update topics
        if !turn.context_contributions.topics.is_empty() {
            session.conversation_state.current_topic =
                turn.context_contributions.topics.first().cloned();
        }

        // Update entities from resolved references
        for reference in &turn.resolved_references {
            if matches!(
                reference.reference_type,
                ReferenceType::Entity | ReferenceType::Document
            ) {
                self.add_or_update_entity_in_session(
                    session,
                    &reference.resolved_to,
                    &reference.reference_type,
                );
            }
        }

        // Update flow state based on content
        session.conversation_state.flow_state =
            self.determine_flow_state(&turn.content, &session.conversation_state.flow_state);

        Ok(())
    }

    /// Add or update document reference in specific session
    #[allow(dead_code)]
    fn add_or_update_document_reference_in_session(
        &self,
        session: &mut ConversationSession,
        doc_name: &str,
    ) {
        if let Some(existing) = session
            .conversation_state
            .active_documents
            .iter_mut()
            .find(|d| d.title == doc_name || d.aliases.contains(&doc_name.to_string()))
        {
            // Update existing reference
            existing.last_referenced = chrono::Utc::now();
            existing.reference_count += 1;
            if !existing.aliases.contains(&doc_name.to_string()) {
                existing.aliases.push(doc_name.to_string());
            }
        } else {
            // Add new document reference
            let doc_ref = DocumentReference {
                document_id: format!(
                    "doc_{}_{}",
                    doc_name.replace(" ", "_"),
                    chrono::Utc::now().timestamp_millis()
                ),
                title: doc_name.to_string(),
                path: None,
                first_mentioned: chrono::Utc::now(),
                last_referenced: chrono::Utc::now(),
                reference_count: 1,
                mention_contexts: Vec::new(),
                relevance_score: 1.0,
                aliases: vec![doc_name.to_string()],
            };
            session.conversation_state.active_documents.push(doc_ref);
        }
    }

    /// Add or update entity reference in specific session
    #[allow(dead_code)]
    fn add_or_update_entity_in_session(
        &self,
        session: &mut ConversationSession,
        entity_name: &str,
        ref_type: &ReferenceType,
    ) {
        let entity_type = match ref_type {
            ReferenceType::Document => "document",
            ReferenceType::Entity => "entity",
            _ => "unknown",
        };

        if let Some(existing) = session
            .conversation_state
            .active_entities
            .get_mut(entity_name)
        {
            existing.last_referenced = chrono::Utc::now();
        } else {
            let entity = EntityReference {
                entity_id: format!(
                    "entity_{}_{}",
                    entity_name.replace(" ", "_"),
                    chrono::Utc::now().timestamp_millis()
                ),
                entity_type: entity_type.to_string(),
                name: entity_name.to_string(),
                description: None,
                first_mentioned: chrono::Utc::now(),
                last_referenced: chrono::Utc::now(),
                aliases: vec![entity_name.to_string()],
                metadata: HashMap::new(),
            };
            session
                .conversation_state
                .active_entities
                .insert(entity_name.to_string(), entity);
        }
    }

    /// Resolve references in text using conversation context
    #[allow(dead_code)]
    fn resolve_references(
        &mut self,
        content: &str,
        session: &ConversationSession,
    ) -> Result<Vec<ResolvedReference>> {
        let mut resolved = Vec::new();

        // Common pronoun patterns
        let pronoun_patterns = [
            ("it", ReferenceType::Pronoun),
            ("that", ReferenceType::Pronoun),
            ("this", ReferenceType::Pronoun),
            ("them", ReferenceType::Pronoun),
            ("those", ReferenceType::Pronoun),
            ("these", ReferenceType::Pronoun),
        ];

        // Document reference patterns
        let doc_patterns = [
            ("the document", ReferenceType::Document),
            ("that file", ReferenceType::Document),
            ("this file", ReferenceType::Document),
            ("the file", ReferenceType::Document),
            ("that document", ReferenceType::Document),
            ("this document", ReferenceType::Document),
        ];

        // Action reference patterns
        let action_patterns = [
            ("the comparison", ReferenceType::Action),
            ("that analysis", ReferenceType::Action),
            ("the search", ReferenceType::Action),
            ("that search", ReferenceType::Action),
            ("the results", ReferenceType::Result),
            ("those results", ReferenceType::Result),
            ("the findings", ReferenceType::Result),
        ];

        // Temporal reference patterns
        let temporal_patterns = [
            ("earlier", ReferenceType::Temporal),
            ("before", ReferenceType::Temporal),
            ("previously", ReferenceType::Temporal),
            ("last time", ReferenceType::Temporal),
            ("the last", ReferenceType::Temporal),
        ];

        let all_patterns = [
            &pronoun_patterns[..],
            &doc_patterns[..],
            &action_patterns[..],
            &temporal_patterns[..],
        ]
        .concat();

        for (pattern, ref_type) in all_patterns {
            if content.to_lowercase().contains(pattern) {
                if let Some(resolution) = self.resolve_pattern(pattern, &ref_type, session) {
                    resolved.push(ResolvedReference {
                        original_text: pattern.to_string(),
                        reference_type: ref_type,
                        resolved_to: resolution.0,
                        confidence: resolution.1,
                        resolution_context: resolution.2,
                    });
                }
            }
        }

        Ok(resolved)
    }

    /// Resolve a specific pattern to its referent
    #[allow(dead_code, clippy::manual_map)]
    fn resolve_pattern(
        &self,
        pattern: &str,
        ref_type: &ReferenceType,
        session: &ConversationSession,
    ) -> Option<(String, f32, String)> {
        match ref_type {
            ReferenceType::Document => {
                // Find the most recently referenced document
                if let Some(doc) = session
                    .conversation_state
                    .active_documents
                    .iter()
                    .max_by_key(|d| d.last_referenced)
                {
                    Some((
                        doc.title.clone(),
                        0.8,
                        format!("Most recent document: {}", doc.title),
                    ))
                } else {
                    None
                }
            }
            ReferenceType::Action => {
                // Find the most recent action mentioned
                if let Some(turn) = session
                    .conversation_history
                    .iter()
                    .rev()
                    .find(|t| !t.actions_performed.is_empty())
                {
                    if let Some(action) = turn.actions_performed.first() {
                        Some((
                            action.clone(),
                            0.7,
                            format!("Most recent action: {}", action),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ReferenceType::Result => {
                // Find the most recent result or output
                if let Some(turn) = session.conversation_history.iter().rev().find(|t| {
                    t.role == "assistant"
                        && (t.content.contains("result")
                            || t.content.contains("found")
                            || t.content.contains("analysis"))
                }) {
                    Some((
                        format!(
                            "Result from: {}",
                            turn.content.chars().take(50).collect::<String>()
                        ),
                        0.6,
                        "Most recent assistant result".to_string(),
                    ))
                } else {
                    None
                }
            }
            ReferenceType::Temporal => {
                // Find previous relevant turn
                if session.conversation_history.len() > 1 {
                    let prev_turn =
                        &session.conversation_history[session.conversation_history.len() - 2];
                    Some((
                        format!(
                            "Previous turn: {}",
                            prev_turn.content.chars().take(50).collect::<String>()
                        ),
                        0.5,
                        "Previous conversation turn".to_string(),
                    ))
                } else {
                    None
                }
            }
            ReferenceType::Pronoun => {
                // Try to resolve pronoun to most recent entity
                if let Some((_, entity)) = session
                    .conversation_state
                    .active_entities
                    .iter()
                    .max_by_key(|(_, e)| e.last_referenced)
                {
                    Some((
                        entity.name.clone(),
                        0.6,
                        format!("Most recent entity: {}", entity.name),
                    ))
                } else {
                    // Fallback to most recent document
                    if let Some(doc) = session
                        .conversation_state
                        .active_documents
                        .iter()
                        .max_by_key(|d| d.last_referenced)
                    {
                        Some((
                            doc.title.clone(),
                            0.5,
                            format!("Most recent document: {}", doc.title),
                        ))
                    } else {
                        None
                    }
                }
            }
            ReferenceType::Entity => {
                // Try to resolve to a known entity
                if let Some((_, entity)) = session
                    .conversation_state
                    .active_entities
                    .iter()
                    .find(|(_, e)| e.aliases.iter().any(|alias| pattern.contains(alias)))
                {
                    Some((
                        entity.name.clone(),
                        0.8,
                        format!("Known entity: {}", entity.name),
                    ))
                } else {
                    None
                }
            }
        }
    }

    /// Extract context contributions from a conversation turn
    #[allow(dead_code)]
    fn extract_context_contributions(
        &self,
        content: &str,
        resolved_references: &[ResolvedReference],
        _session: &ConversationSession,
    ) -> Result<ContextContributions> {
        let mut contributions = ContextContributions {
            new_entities: Vec::new(),
            document_references: Vec::new(),
            topics: Vec::new(),
            referenceable_actions: Vec::new(),
            key_information: Vec::new(),
        };

        // Extract document references (file names, quoted documents, etc.)
        if let Ok(file_regex) = Regex::new(r"([a-zA-Z0-9_.-]+\.(docx?|pdf|txt|md|xlsx?|pptx?))") {
            for captures in file_regex.captures_iter(content) {
                if let Some(filename) = captures.get(1) {
                    contributions
                        .document_references
                        .push(filename.as_str().to_string());
                }
            }
        }

        // Extract quoted document names
        if let Ok(quote_regex) = Regex::new(r#""([^"]+)""#) {
            for captures in quote_regex.captures_iter(content) {
                if let Some(quoted) = captures.get(1) {
                    // If it looks like a document name, add it
                    let quoted_str = quoted.as_str();
                    if quoted_str.len() > 3
                        && (quoted_str.contains("guide")
                            || quoted_str.contains("manual")
                            || quoted_str.contains("document")
                            || quoted_str.contains("report"))
                    {
                        contributions
                            .document_references
                            .push(quoted_str.to_string());
                    }
                }
            }
        }

        // Extract action references
        let action_keywords = [
            "compare", "analyze", "search", "find", "generate", "create", "update", "review",
        ];
        for keyword in &action_keywords {
            if content.to_lowercase().contains(keyword) {
                contributions
                    .referenceable_actions
                    .push(format!("{} action", keyword));
            }
        }

        // Extract topics (simple keyword-based for now)
        let topic_keywords = [
            "authentication",
            "security",
            "API",
            "documentation",
            "training",
            "procedure",
            "workflow",
        ];
        for keyword in &topic_keywords {
            if content.to_lowercase().contains(&keyword.to_lowercase()) {
                contributions.topics.push(keyword.to_string());
            }
        }

        // Create key information from resolved references
        for reference in resolved_references {
            contributions.key_information.push(KeyInformation {
                info_id: format!(
                    "ref_{}_{}",
                    reference.reference_type as u8,
                    chrono::Utc::now().timestamp_millis()
                ),
                info_type: "resolved_reference".to_string(),
                content: format!("{} -> {}", reference.original_text, reference.resolved_to),
                timestamp: chrono::Utc::now(),
                importance: reference.confidence,
                context: reference.resolution_context.clone(),
            });
        }

        Ok(contributions)
    }

    /// Update session state based on a conversation turn
    #[allow(dead_code)]
    fn update_session_state(
        &mut self,
        session: &mut ConversationSession,
        turn: &ConversationTurn,
    ) -> Result<()> {
        // Update document references
        for doc_ref in &turn.context_contributions.document_references {
            self.add_or_update_document_reference(session, doc_ref);
        }

        // Update topics
        if !turn.context_contributions.topics.is_empty() {
            session.conversation_state.current_topic =
                turn.context_contributions.topics.first().cloned();
        }

        // Update entities from resolved references
        for reference in &turn.resolved_references {
            if matches!(
                reference.reference_type,
                ReferenceType::Entity | ReferenceType::Document
            ) {
                self.add_or_update_entity(
                    session,
                    &reference.resolved_to,
                    &reference.reference_type,
                );
            }
        }

        // Update flow state based on content
        session.conversation_state.flow_state =
            self.determine_flow_state(&turn.content, &session.conversation_state.flow_state);

        Ok(())
    }

    /// Add or update a document reference in the session
    #[allow(dead_code)]
    fn add_or_update_document_reference(&self, session: &mut ConversationSession, doc_name: &str) {
        if let Some(existing) = session
            .conversation_state
            .active_documents
            .iter_mut()
            .find(|d| d.title == doc_name || d.aliases.contains(&doc_name.to_string()))
        {
            // Update existing reference
            existing.last_referenced = chrono::Utc::now();
            existing.reference_count += 1;
            if !existing.aliases.contains(&doc_name.to_string()) {
                existing.aliases.push(doc_name.to_string());
            }
        } else {
            // Add new document reference
            let doc_ref = DocumentReference {
                document_id: format!(
                    "doc_{}_{}",
                    doc_name.replace(" ", "_"),
                    chrono::Utc::now().timestamp_millis()
                ),
                title: doc_name.to_string(),
                path: None,
                first_mentioned: chrono::Utc::now(),
                last_referenced: chrono::Utc::now(),
                reference_count: 1,
                mention_contexts: Vec::new(),
                relevance_score: 1.0,
                aliases: vec![doc_name.to_string()],
            };
            session.conversation_state.active_documents.push(doc_ref);
        }
    }

    /// Add or update an entity reference
    #[allow(dead_code)]
    fn add_or_update_entity(
        &self,
        session: &mut ConversationSession,
        entity_name: &str,
        ref_type: &ReferenceType,
    ) {
        let entity_type = match ref_type {
            ReferenceType::Document => "document",
            ReferenceType::Entity => "entity",
            _ => "unknown",
        };

        if let Some(existing) = session
            .conversation_state
            .active_entities
            .get_mut(entity_name)
        {
            existing.last_referenced = chrono::Utc::now();
        } else {
            let entity = EntityReference {
                entity_id: format!(
                    "entity_{}_{}",
                    entity_name.replace(" ", "_"),
                    chrono::Utc::now().timestamp_millis()
                ),
                entity_type: entity_type.to_string(),
                name: entity_name.to_string(),
                description: None,
                first_mentioned: chrono::Utc::now(),
                last_referenced: chrono::Utc::now(),
                aliases: vec![entity_name.to_string()],
                metadata: HashMap::new(),
            };
            session
                .conversation_state
                .active_entities
                .insert(entity_name.to_string(), entity);
        }
    }

    /// Determine conversation flow state based on content
    #[allow(dead_code)]
    fn determine_flow_state(
        &self,
        content: &str,
        current_state: &ConversationFlowState,
    ) -> ConversationFlowState {
        let content_lower = content.to_lowercase();

        // Check for clarifying questions
        if content_lower.contains("what do you mean")
            || content_lower.contains("can you clarify")
            || content_lower.contains("i don't understand")
            || content_lower.contains("could you explain")
        {
            return ConversationFlowState::Clarifying;
        }

        // Check for confirmation requests
        if content_lower.contains("is that correct")
            || content_lower.contains("should i proceed")
            || content_lower.contains("do you want me to")
            || content_lower.contains("shall i")
        {
            return ConversationFlowState::WaitingConfirmation;
        }

        // Check for task execution indicators
        if content_lower.contains("let me")
            || content_lower.contains("i'll")
            || content_lower.contains("starting")
            || content_lower.contains("processing")
        {
            return ConversationFlowState::ExecutingTask;
        }

        // Check for ending indicators
        if content_lower.contains("goodbye")
            || content_lower.contains("thank you")
            || content_lower.contains("that's all")
            || content_lower.contains("bye")
        {
            return ConversationFlowState::Ending;
        }

        // Default to normal flow
        match current_state {
            ConversationFlowState::Starting => ConversationFlowState::Normal,
            _ => current_state.clone(),
        }
    }

    /// Trim conversation history to stay within limits
    #[allow(dead_code)]
    fn trim_conversation_history(&self, session: &mut ConversationSession) {
        let max_turns = session.metadata.user_preferences.max_history_turns;
        if session.conversation_history.len() > max_turns {
            let excess = session.conversation_history.len() - max_turns;
            session.conversation_history.drain(0..excess);
        }
    }

    /// Get enriched context for AI processing
    pub fn get_enriched_context(
        &self,
        session_id: &str,
        current_query: &str,
    ) -> Option<EnrichedConversationContext> {
        self.sessions
            .get(session_id)
            .map(|session| self.build_enriched_context(session, current_query))
    }

    /// Build enriched context for AI
    fn build_enriched_context(
        &self,
        session: &ConversationSession,
        current_query: &str,
    ) -> EnrichedConversationContext {
        // Get recent conversation history
        let recent_turns = session
            .conversation_history
            .iter()
            .rev()
            .take(10)
            .rev()
            .cloned()
            .collect();

        // Get relevant document references
        let relevant_docs = session
            .conversation_state
            .active_documents
            .iter()
            .filter(|d| d.relevance_score > 0.3)
            .cloned()
            .collect();

        // Get active entities
        let entities = session
            .conversation_state
            .active_entities
            .values()
            .cloned()
            .collect();

        // Build reference resolution map
        let mut reference_map = HashMap::new();
        for turn in &session.conversation_history {
            for reference in &turn.resolved_references {
                reference_map.insert(
                    reference.original_text.clone(),
                    reference.resolved_to.clone(),
                );
            }
        }

        EnrichedConversationContext {
            session_id: session.session_id.clone(),
            current_query: current_query.to_string(),
            recent_conversation: recent_turns,
            active_documents: relevant_docs,
            active_entities: entities,
            current_topic: session.conversation_state.current_topic.clone(),
            current_task: session.conversation_state.current_task.clone(),
            flow_state: session.conversation_state.flow_state.clone(),
            reference_resolution_map: reference_map,
            context_summary: self.generate_context_summary(session),
        }
    }

    /// Generate a summary of the current conversation context
    fn generate_context_summary(&self, session: &ConversationSession) -> String {
        let mut summary_parts = Vec::new();

        // Add document context
        if !session.conversation_state.active_documents.is_empty() {
            let doc_names: Vec<_> = session
                .conversation_state
                .active_documents
                .iter()
                .map(|d| d.title.as_str())
                .collect();
            summary_parts.push(format!("Documents in context: {}", doc_names.join(", ")));
        }

        // Add current topic
        if let Some(topic) = &session.conversation_state.current_topic {
            summary_parts.push(format!("Current topic: {}", topic));
        }

        // Add current task
        if let Some(task) = &session.conversation_state.current_task {
            summary_parts.push(format!(
                "Current task: {} ({})",
                task.description, task.status as u8
            ));
        }

        // Add conversation length
        summary_parts.push(format!(
            "Conversation turns: {}",
            session.conversation_history.len()
        ));

        if summary_parts.is_empty() {
            "New conversation - no prior context".to_string()
        } else {
            summary_parts.join("; ")
        }
    }

    /// Update task status in conversation context
    pub fn update_task_status(
        &mut self,
        session_id: &str,
        task_id: &str,
        status: TaskStatus,
        progress_update: Option<String>,
    ) -> Result<()> {
        let session = self.get_or_create_session(session_id);

        if let Some(task) = &mut session.conversation_state.current_task {
            if task.task_id == task_id {
                task.status = status;
                if let Some(update) = progress_update {
                    task.progress_updates.push(update);
                }
            }
        }

        Ok(())
    }

    /// Start a new task in the conversation context
    pub fn start_task(
        &mut self,
        session_id: &str,
        task_type: &str,
        description: &str,
        parameters: HashMap<String, String>,
    ) -> Result<String> {
        let task_id = format!(
            "task_{}_{}",
            session_id,
            chrono::Utc::now().timestamp_millis()
        );
        let session = self.get_or_create_session(session_id);

        let task = TaskContext {
            task_id: task_id.clone(),
            task_type: task_type.to_string(),
            description: description.to_string(),
            status: TaskStatus::Starting,
            parameters,
            started_at: chrono::Utc::now(),
            expected_completion: None,
            progress_updates: Vec::new(),
        };

        session.conversation_state.current_task = Some(task);
        session.conversation_state.flow_state = ConversationFlowState::ExecutingTask;

        Ok(task_id)
    }

    /// Update workspace intelligence context for a session
    pub fn update_workspace_intelligence(
        &mut self,
        session_id: &str,
        workspace_intelligence: WorkspaceIntelligenceContext,
    ) -> Result<()> {
        let session = self.get_or_create_session(session_id);
        session.workspace_intelligence = Some(workspace_intelligence);
        Ok(())
    }

    /// Get workspace intelligence for a session
    pub fn get_workspace_intelligence(
        &self,
        session_id: &str,
    ) -> Option<&WorkspaceIntelligenceContext> {
        self.sessions
            .get(session_id)
            .and_then(|session| session.workspace_intelligence.as_ref())
    }

    /// Check if session has workspace intelligence
    pub fn has_workspace_intelligence(&self, session_id: &str) -> bool {
        self.get_workspace_intelligence(session_id).is_some()
    }

    /// Get session information
    pub fn get_session_info(&self, session_id: &str) -> Option<&ConversationSession> {
        self.sessions.get(session_id)
    }

    /// Clear old sessions to manage memory
    pub fn cleanup_old_sessions(&mut self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        self.sessions
            .retain(|_, session| session.metadata.last_activity > cutoff);
    }

    /// Export session for persistence
    pub fn export_session(&self, session_id: &str) -> Option<String> {
        if let Some(session) = self.sessions.get(session_id) {
            serde_json::to_string_pretty(session).ok()
        } else {
            None
        }
    }

    /// Import session from persistence
    pub fn import_session(&mut self, session_data: &str) -> Result<String> {
        let session: ConversationSession = serde_json::from_str(session_data)?;
        let session_id = session.session_id.clone();
        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }
}

/// Enriched conversation context for AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedConversationContext {
    /// Session identifier
    pub session_id: String,
    /// Current user query
    pub current_query: String,
    /// Recent conversation turns
    pub recent_conversation: Vec<ConversationTurn>,
    /// Currently active documents
    pub active_documents: Vec<DocumentReference>,
    /// Active entities being discussed
    pub active_entities: Vec<EntityReference>,
    /// Current conversation topic
    pub current_topic: Option<String>,
    /// Current task being executed
    pub current_task: Option<TaskContext>,
    /// Conversation flow state
    pub flow_state: ConversationFlowState,
    /// Reference resolution mappings
    pub reference_resolution_map: HashMap<String, String>,
    /// Summary of conversation context
    pub context_summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_context_creation() {
        let mut manager = ConversationContextManager::new();
        let session = manager.get_or_create_session("test_session");

        assert_eq!(session.session_id, "test_session");
        assert!(session.conversation_history.is_empty());
        assert_eq!(
            session.conversation_state.flow_state.clone() as u8,
            ConversationFlowState::Starting as u8
        );
    }

    #[test]
    fn test_adding_conversation_turns() {
        let mut manager = ConversationContextManager::new();

        let turn_id = manager
            .add_conversation_turn(
                "test_session",
                "user",
                "Hello, can you help me compare document A with document B?",
                Some("compare_documents".to_string()),
            )
            .unwrap();

        assert!(!turn_id.is_empty());

        let session = manager.get_session_info("test_session").unwrap();
        assert_eq!(session.conversation_history.len(), 1);
        assert_eq!(session.conversation_history[0].role, "user");
    }

    #[test]
    fn test_document_reference_extraction() {
        let mut manager = ConversationContextManager::new();

        manager
            .add_conversation_turn(
                "test_session",
                "user",
                "Please compare user_guide.pdf with the technical_manual.docx",
                Some("compare_documents".to_string()),
            )
            .unwrap();

        let session = manager.get_session_info("test_session").unwrap();

        // The test should pass if we found any documents
        assert!(!session.conversation_state.active_documents.is_empty(),
               "Expected to find document references in: 'Please compare user_guide.pdf with the technical_manual.docx'");
    }

    #[test]
    fn test_reference_resolution() {
        let mut manager = ConversationContextManager::new();

        // First mention a document
        manager
            .add_conversation_turn(
                "test_session",
                "user",
                "Please analyze the user_guide.pdf",
                Some("analyze_document".to_string()),
            )
            .unwrap();

        // Then refer to "it"
        manager
            .add_conversation_turn(
                "test_session",
                "user",
                "Can you also check if it has any security issues?",
                Some("analyze_document".to_string()),
            )
            .unwrap();

        let session = manager.get_session_info("test_session").unwrap();
        let last_turn = session.conversation_history.last().unwrap();

        // Debug output
        println!("Resolved references: {:?}", last_turn.resolved_references);
        println!("All conversation turns:");
        for (i, turn) in session.conversation_history.iter().enumerate() {
            println!("  Turn {}: {} - '{}'", i, turn.role, turn.content);
            println!(
                "    Document refs: {:?}",
                turn.context_contributions.document_references
            );
        }

        // Should have resolved "it" to something
        assert!(!last_turn.resolved_references.is_empty(),
               "Expected to resolve 'it' reference in: 'Can you also check if it has any security issues?'");
    }

    #[test]
    fn test_enriched_context_generation() {
        let mut manager = ConversationContextManager::new();

        manager
            .add_conversation_turn(
                "test_session",
                "user",
                "Analyze user_guide.pdf",
                Some("analyze_document".to_string()),
            )
            .unwrap();

        let context = manager.get_enriched_context("test_session", "What did you find?");
        assert!(context.is_some());

        let context = context.unwrap();
        assert_eq!(context.session_id, "test_session");
        assert_eq!(context.current_query, "What did you find?");
        assert!(!context.recent_conversation.is_empty());
    }

    #[test]
    fn test_task_management() {
        let mut manager = ConversationContextManager::new();

        let task_id = manager
            .start_task(
                "test_session",
                "document_comparison",
                "Compare two documents",
                HashMap::new(),
            )
            .unwrap();

        assert!(!task_id.is_empty());

        manager
            .update_task_status(
                "test_session",
                &task_id,
                TaskStatus::InProgress,
                Some("Starting comparison...".to_string()),
            )
            .unwrap();

        let session = manager.get_session_info("test_session").unwrap();
        assert!(session.conversation_state.current_task.is_some());

        let task = session.conversation_state.current_task.as_ref().unwrap();
        assert_eq!(task.task_id, task_id);
        assert_eq!(task.status as u8, TaskStatus::InProgress as u8);
        assert!(!task.progress_updates.is_empty());
    }
}
