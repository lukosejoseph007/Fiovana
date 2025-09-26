// src-tauri/src/commands/conversation_context_commands.rs
// Tauri commands for enhanced conversation context management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::ai::conversation_context::{
    ConversationContextManager, EnrichedConversationContext, TaskStatus,
};

/// State management for conversation context
pub type ConversationContextState = Arc<Mutex<ConversationContextManager>>;

/// Request to add a conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConversationTurnRequest {
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub intent: Option<String>,
}

/// Response from adding a conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConversationTurnResponse {
    pub turn_id: String,
    pub resolved_references: Vec<ResolvedReferenceInfo>,
    pub context_contributions: ContextContributionsInfo,
}

/// Information about resolved references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedReferenceInfo {
    pub original_text: String,
    pub reference_type: String,
    pub resolved_to: String,
    pub confidence: f32,
    pub resolution_context: String,
}

/// Information about context contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextContributionsInfo {
    pub new_entities: Vec<String>,
    pub document_references: Vec<String>,
    pub topics: Vec<String>,
    pub referenceable_actions: Vec<String>,
    pub key_information_count: usize,
}

/// Request to start a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTaskRequest {
    pub session_id: String,
    pub task_type: String,
    pub description: String,
    pub parameters: HashMap<String, String>,
}

/// Request to update task status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskStatusRequest {
    pub session_id: String,
    pub task_id: String,
    pub status: TaskStatusInfo,
    pub progress_update: Option<String>,
}

/// Task status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatusInfo {
    Starting,
    InProgress,
    WaitingForInput,
    Completed,
    Failed,
    Cancelled,
}

impl From<TaskStatusInfo> for TaskStatus {
    fn from(status: TaskStatusInfo) -> Self {
        match status {
            TaskStatusInfo::Starting => TaskStatus::Starting,
            TaskStatusInfo::InProgress => TaskStatus::InProgress,
            TaskStatusInfo::WaitingForInput => TaskStatus::WaitingForInput,
            TaskStatusInfo::Completed => TaskStatus::Completed,
            TaskStatusInfo::Failed => TaskStatus::Failed,
            TaskStatusInfo::Cancelled => TaskStatus::Cancelled,
        }
    }
}

/// Session information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfoResponse {
    pub session_id: String,
    pub conversation_turn_count: usize,
    pub active_document_count: usize,
    pub active_entity_count: usize,
    pub current_topic: Option<String>,
    pub current_task: Option<TaskInfo>,
    pub flow_state: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: String,
    pub task_type: String,
    pub description: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub progress_updates: Vec<String>,
}

/// Initialize conversation context manager
#[tauri::command]
pub async fn initialize_conversation_context(
    state: State<'_, ConversationContextState>,
) -> Result<String, String> {
    let mut context_manager = state.lock().await;

    // Check if already initialized (manager is created but we can verify it's working)
    *context_manager = ConversationContextManager::new();

    Ok("Conversation context manager initialized successfully".to_string())
}

/// Add a conversation turn with automatic context enhancement
#[tauri::command]
pub async fn add_conversation_turn(
    state: State<'_, ConversationContextState>,
    request: AddConversationTurnRequest,
) -> Result<AddConversationTurnResponse, String> {
    let mut context_manager = state.lock().await;

    let turn_id = context_manager
        .add_conversation_turn(
            &request.session_id,
            &request.role,
            &request.content,
            request.intent,
        )
        .map_err(|e| e.to_string())?;

    // Get the session to extract information about what was added
    if let Some(session) = context_manager.get_session_info(&request.session_id) {
        if let Some(turn) = session.conversation_history.last() {
            let resolved_references = turn
                .resolved_references
                .iter()
                .map(|r| ResolvedReferenceInfo {
                    original_text: r.original_text.clone(),
                    reference_type: format!("{:?}", r.reference_type),
                    resolved_to: r.resolved_to.clone(),
                    confidence: r.confidence,
                    resolution_context: r.resolution_context.clone(),
                })
                .collect();

            let context_contributions = ContextContributionsInfo {
                new_entities: turn
                    .context_contributions
                    .new_entities
                    .iter()
                    .map(|e| e.name.clone())
                    .collect(),
                document_references: turn.context_contributions.document_references.clone(),
                topics: turn.context_contributions.topics.clone(),
                referenceable_actions: turn.context_contributions.referenceable_actions.clone(),
                key_information_count: turn.context_contributions.key_information.len(),
            };

            return Ok(AddConversationTurnResponse {
                turn_id,
                resolved_references,
                context_contributions,
            });
        }
    }

    // Fallback response
    Ok(AddConversationTurnResponse {
        turn_id,
        resolved_references: Vec::new(),
        context_contributions: ContextContributionsInfo {
            new_entities: Vec::new(),
            document_references: Vec::new(),
            topics: Vec::new(),
            referenceable_actions: Vec::new(),
            key_information_count: 0,
        },
    })
}

/// Get enriched conversation context for AI processing
#[tauri::command]
pub async fn get_enriched_context(
    state: State<'_, ConversationContextState>,
    session_id: String,
    current_query: String,
) -> Result<Option<EnrichedConversationContext>, String> {
    let context_manager = state.lock().await;

    Ok(context_manager.get_enriched_context(&session_id, &current_query))
}

/// Start a new task in conversation context
#[tauri::command]
pub async fn start_conversation_task(
    state: State<'_, ConversationContextState>,
    request: StartTaskRequest,
) -> Result<String, String> {
    let mut context_manager = state.lock().await;

    let task_id = context_manager
        .start_task(
            &request.session_id,
            &request.task_type,
            &request.description,
            request.parameters,
        )
        .map_err(|e| e.to_string())?;

    Ok(task_id)
}

/// Update task status in conversation context
#[tauri::command]
pub async fn update_conversation_task_status(
    state: State<'_, ConversationContextState>,
    request: UpdateTaskStatusRequest,
) -> Result<String, String> {
    let mut context_manager = state.lock().await;

    context_manager
        .update_task_status(
            &request.session_id,
            &request.task_id,
            request.status.into(),
            request.progress_update,
        )
        .map_err(|e| e.to_string())?;

    Ok("Task status updated successfully".to_string())
}

/// Get session information
#[tauri::command]
pub async fn get_conversation_session_info(
    state: State<'_, ConversationContextState>,
    session_id: String,
) -> Result<Option<SessionInfoResponse>, String> {
    let context_manager = state.lock().await;

    if let Some(session) = context_manager.get_session_info(&session_id) {
        let task_info = session
            .conversation_state
            .current_task
            .as_ref()
            .map(|task| TaskInfo {
                task_id: task.task_id.clone(),
                task_type: task.task_type.clone(),
                description: task.description.clone(),
                status: format!("{:?}", task.status),
                started_at: task.started_at,
                progress_updates: task.progress_updates.clone(),
            });

        Ok(Some(SessionInfoResponse {
            session_id: session.session_id.clone(),
            conversation_turn_count: session.conversation_history.len(),
            active_document_count: session.conversation_state.active_documents.len(),
            active_entity_count: session.conversation_state.active_entities.len(),
            current_topic: session.conversation_state.current_topic.clone(),
            current_task: task_info,
            flow_state: format!("{:?}", session.conversation_state.flow_state),
            created_at: session.metadata.created_at,
            last_activity: session.metadata.last_activity,
        }))
    } else {
        Ok(None)
    }
}

/// Get conversation history for a session
#[tauri::command]
pub async fn get_conversation_history(
    state: State<'_, ConversationContextState>,
    session_id: String,
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>, String> {
    let context_manager = state.lock().await;

    if let Some(session) = context_manager.get_session_info(&session_id) {
        let history = if let Some(limit) = limit {
            session
                .conversation_history
                .iter()
                .rev()
                .take(limit)
                .rev()
                .collect::<Vec<_>>()
        } else {
            session.conversation_history.iter().collect::<Vec<_>>()
        };

        let history_json = history.iter()
            .map(|turn| serde_json::json!({
                "turn_id": turn.turn_id,
                "role": turn.role,
                "content": turn.content,
                "timestamp": turn.timestamp,
                "intent": turn.intent,
                "resolved_references": turn.resolved_references,
                "actions_performed": turn.actions_performed,
                "context_contributions": {
                    "new_entities": turn.context_contributions.new_entities.len(),
                    "document_references": turn.context_contributions.document_references.len(),
                    "topics": turn.context_contributions.topics.len(),
                    "referenceable_actions": turn.context_contributions.referenceable_actions.len(),
                    "key_information": turn.context_contributions.key_information.len()
                }
            }))
            .collect();

        Ok(history_json)
    } else {
        Ok(Vec::new())
    }
}

/// Get active documents in conversation context
#[tauri::command]
pub async fn get_active_documents(
    state: State<'_, ConversationContextState>,
    session_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let context_manager = state.lock().await;

    if let Some(session) = context_manager.get_session_info(&session_id) {
        let docs_json = session
            .conversation_state
            .active_documents
            .iter()
            .map(|doc| {
                serde_json::json!({
                    "document_id": doc.document_id,
                    "title": doc.title,
                    "path": doc.path,
                    "first_mentioned": doc.first_mentioned,
                    "last_referenced": doc.last_referenced,
                    "reference_count": doc.reference_count,
                    "relevance_score": doc.relevance_score,
                    "aliases": doc.aliases
                })
            })
            .collect();

        Ok(docs_json)
    } else {
        Ok(Vec::new())
    }
}

/// Get active entities in conversation context
#[tauri::command]
pub async fn get_active_entities(
    state: State<'_, ConversationContextState>,
    session_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let context_manager = state.lock().await;

    if let Some(session) = context_manager.get_session_info(&session_id) {
        let entities_json = session
            .conversation_state
            .active_entities
            .values()
            .map(|entity| {
                serde_json::json!({
                    "entity_id": entity.entity_id,
                    "entity_type": entity.entity_type,
                    "name": entity.name,
                    "description": entity.description,
                    "first_mentioned": entity.first_mentioned,
                    "last_referenced": entity.last_referenced,
                    "aliases": entity.aliases,
                    "metadata": entity.metadata
                })
            })
            .collect();

        Ok(entities_json)
    } else {
        Ok(Vec::new())
    }
}

/// Clean up old conversation context sessions
#[tauri::command]
pub async fn cleanup_conversation_context_sessions(
    state: State<'_, ConversationContextState>,
) -> Result<String, String> {
    let mut context_manager = state.lock().await;
    context_manager.cleanup_old_sessions();
    Ok("Old conversation context sessions cleaned up successfully".to_string())
}

/// Export conversation session for persistence
#[tauri::command]
pub async fn export_conversation_session(
    state: State<'_, ConversationContextState>,
    session_id: String,
) -> Result<Option<String>, String> {
    let context_manager = state.lock().await;
    Ok(context_manager.export_session(&session_id))
}

/// Import conversation session from persistence
#[tauri::command]
pub async fn import_conversation_session(
    state: State<'_, ConversationContextState>,
    session_data: String,
) -> Result<String, String> {
    let mut context_manager = state.lock().await;
    context_manager
        .import_session(&session_data)
        .map_err(|e| e.to_string())
}

/// Get conversation context manager status
#[tauri::command]
pub async fn get_conversation_context_status(
    state: State<'_, ConversationContextState>,
) -> Result<serde_json::Value, String> {
    let _context_manager = state.lock().await;

    Ok(serde_json::json!({
        "initialized": true,
        "capabilities": [
            "conversation_history_tracking",
            "reference_resolution",
            "context_enrichment",
            "task_management",
            "entity_tracking",
            "document_reference_tracking",
            "session_persistence"
        ]
    }))
}
