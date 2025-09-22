// src-tauri/src/ai/context.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ai::prompts::{ConversationTurn, DocumentMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRef {
    pub id: String,
    pub title: String,
    pub path: Option<String>,
    pub content_preview: String, // First 500 chars for quick reference
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContext {
    pub active_documents: Vec<DocumentRef>,
    pub conversation_history: Vec<ConversationTurn>,
    pub session_metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub workspace_path: Option<String>,
    pub current_task: Option<String>,
    pub user_preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_response_style: String, // "detailed", "concise", "technical"
    pub include_citations: bool,
    pub max_context_length: usize,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_response_style: "detailed".to_string(),
            include_citations: true,
            max_context_length: 4000,
        }
    }
}

pub struct DocumentContextManager {
    pub contexts: HashMap<String, DocumentContext>, // session_id -> context
    max_conversation_history: usize,
    #[allow(dead_code)]
    max_active_documents: usize,
}

impl Default for DocumentContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentContextManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            max_conversation_history: 20, // Keep last 20 turns
            max_active_documents: 10,     // Track up to 10 documents
        }
    }

    /// Get or create a document context for a session
    pub fn get_or_create_context(&mut self, session_id: &str) -> &mut DocumentContext {
        self.contexts
            .entry(session_id.to_string())
            .or_insert_with(|| DocumentContext {
                active_documents: Vec::new(),
                conversation_history: Vec::new(),
                session_metadata: SessionMetadata {
                    session_id: session_id.to_string(),
                    workspace_path: None,
                    current_task: None,
                    user_preferences: UserPreferences::default(),
                },
            })
    }

    /// Add relevant documents to the context
    #[allow(dead_code)]
    pub fn add_relevant_documents(
        &mut self,
        session_id: &str,
        documents: Vec<DocumentRef>,
    ) -> Result<()> {
        let max_docs = self.max_active_documents;
        let context = self.get_or_create_context(session_id);

        // Add new documents, removing oldest if we exceed the limit
        for doc in documents {
            // Check if document already exists (update relevance if so)
            if let Some(existing_doc) = context.active_documents.iter_mut().find(|d| d.id == doc.id)
            {
                existing_doc.relevance_score = doc.relevance_score;
                existing_doc.content_preview = doc.content_preview;
            } else {
                context.active_documents.push(doc);
            }
        }

        // Sort by relevance and keep only the most relevant documents
        context
            .active_documents
            .sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        context.active_documents.truncate(max_docs);

        Ok(())
    }

    /// Add a conversation turn to the history
    pub fn add_conversation_turn(
        &mut self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<()> {
        let max_history = self.max_conversation_history;
        let context = self.get_or_create_context(session_id);

        let turn = ConversationTurn {
            role: role.to_string(),
            content: content.to_string(),
        };

        context.conversation_history.push(turn);

        // Keep only the most recent conversation turns
        if context.conversation_history.len() > max_history {
            context
                .conversation_history
                .drain(0..context.conversation_history.len() - max_history);
        }

        Ok(())
    }

    /// Get relevant context for AI processing
    pub fn get_relevant_context(&self, session_id: &str, query: &str) -> String {
        if let Some(context) = self.contexts.get(session_id) {
            self.build_context_string(context, query)
        } else {
            "No document context available. Providing general response.".to_string()
        }
    }

    /// Build a comprehensive context string for AI processing
    fn build_context_string(&self, context: &DocumentContext, query: &str) -> String {
        let mut context_parts = Vec::new();

        // Add active documents context
        if !context.active_documents.is_empty() {
            context_parts.push("=== RELEVANT DOCUMENTS ===".to_string());

            for (i, doc) in context.active_documents.iter().enumerate() {
                let doc_section = format!(
                    "Document {}: {} (relevance: {:.2})\n{}\n",
                    i + 1,
                    doc.title,
                    doc.relevance_score,
                    doc.content_preview
                );
                context_parts.push(doc_section);
            }
        }

        // Add recent conversation history for context
        if !context.conversation_history.is_empty() {
            context_parts.push("=== RECENT CONVERSATION ===".to_string());

            // Only include the last few turns to avoid overwhelming the prompt
            let recent_turns = context
                .conversation_history
                .iter()
                .rev()
                .take(5)
                .rev()
                .collect::<Vec<_>>();

            for turn in recent_turns {
                context_parts.push(format!("{}: {}", turn.role.to_uppercase(), turn.content));
            }
        }

        // Add current query context
        context_parts.push("=== CURRENT QUERY ===".to_string());
        context_parts.push(format!("User Question: {}", query));

        // Add user preferences
        if context.session_metadata.user_preferences.include_citations {
            context_parts.push("\nNote: Please include citations to specific documents when referencing information.".to_string());
        }

        let response_style = &context
            .session_metadata
            .user_preferences
            .preferred_response_style;
        match response_style.as_str() {
            "concise" => {
                context_parts.push("Please provide a concise, focused response.".to_string())
            }
            "technical" => context_parts
                .push("Please provide a detailed technical response with specifics.".to_string()),
            _ => {
                context_parts.push("Please provide a comprehensive, helpful response.".to_string())
            }
        }

        let full_context = context_parts.join("\n\n");

        // Truncate if too long
        let max_length = context.session_metadata.user_preferences.max_context_length;
        if full_context.len() > max_length {
            format!(
                "{}...\n\n[Context truncated for length]",
                &full_context[..max_length]
            )
        } else {
            full_context
        }
    }

    /// Update user preferences
    #[allow(dead_code)]
    pub fn update_preferences(
        &mut self,
        session_id: &str,
        preferences: UserPreferences,
    ) -> Result<()> {
        let context = self.get_or_create_context(session_id);
        context.session_metadata.user_preferences = preferences;
        Ok(())
    }

    /// Set current task context
    #[allow(dead_code)]
    pub fn set_current_task(&mut self, session_id: &str, task: Option<String>) -> Result<()> {
        let context = self.get_or_create_context(session_id);
        context.session_metadata.current_task = task;
        Ok(())
    }

    /// Clear context for a session
    #[allow(dead_code)]
    pub fn clear_session(&mut self, session_id: &str) {
        self.contexts.remove(session_id);
    }

    /// Get session statistics
    #[allow(dead_code)]
    pub fn get_session_stats(&self, session_id: &str) -> Option<SessionStats> {
        self.contexts.get(session_id).map(|context| SessionStats {
            active_documents_count: context.active_documents.len(),
            conversation_turns: context.conversation_history.len(),
            average_document_relevance: context
                .active_documents
                .iter()
                .map(|doc| doc.relevance_score)
                .fold(0.0, |acc, score| acc + score)
                / context.active_documents.len().max(1) as f32,
        })
    }

    /// Extract document metadata from context
    pub fn extract_document_metadata(&self, session_id: &str) -> Option<DocumentMetadata> {
        if let Some(context) = self.contexts.get(session_id) {
            context
                .active_documents
                .first()
                .map(|primary_doc| DocumentMetadata {
                    title: Some(primary_doc.title.clone()),
                    document_type: None,  // Could be enhanced to detect type
                    sections: vec![],     // Could be enhanced to extract sections
                    key_concepts: vec![], // Could be enhanced with NLP
                })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub active_documents_count: usize,
    pub conversation_turns: usize,
    pub average_document_relevance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_creation() {
        let mut manager = DocumentContextManager::new();
        let context = manager.get_or_create_context("test_session");

        assert_eq!(context.session_metadata.session_id, "test_session");
        assert!(context.active_documents.is_empty());
        assert!(context.conversation_history.is_empty());
    }

    #[test]
    fn test_adding_documents() {
        let mut manager = DocumentContextManager::new();

        let docs = vec![DocumentRef {
            id: "doc1".to_string(),
            title: "Test Document".to_string(),
            path: None,
            content_preview: "This is a test document".to_string(),
            relevance_score: 0.8,
        }];

        manager
            .add_relevant_documents("test_session", docs)
            .unwrap();

        let context = manager.get_or_create_context("test_session");
        assert_eq!(context.active_documents.len(), 1);
        assert_eq!(context.active_documents[0].title, "Test Document");
    }

    #[test]
    fn test_conversation_history() {
        let mut manager = DocumentContextManager::new();

        manager
            .add_conversation_turn("test_session", "user", "Hello")
            .unwrap();
        manager
            .add_conversation_turn("test_session", "assistant", "Hi there!")
            .unwrap();

        let context = manager.get_or_create_context("test_session");
        assert_eq!(context.conversation_history.len(), 2);
        assert_eq!(context.conversation_history[0].role, "user");
        assert_eq!(context.conversation_history[1].role, "assistant");
    }

    #[test]
    fn test_context_building() {
        let mut manager = DocumentContextManager::new();

        let docs = vec![DocumentRef {
            id: "doc1".to_string(),
            title: "API Guide".to_string(),
            path: None,
            content_preview: "This guide covers API usage and authentication".to_string(),
            relevance_score: 0.9,
        }];

        manager
            .add_relevant_documents("test_session", docs)
            .unwrap();
        manager
            .add_conversation_turn("test_session", "user", "How do I authenticate?")
            .unwrap();

        let context_string =
            manager.get_relevant_context("test_session", "What's the API key format?");

        assert!(context_string.contains("API Guide"));
        assert!(context_string.contains("authenticate"));
        assert!(context_string.contains("API key format"));
    }
}
