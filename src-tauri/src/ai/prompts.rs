// src-tauri/src/ai/prompts.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContext {
    pub document_content: Option<String>,
    pub document_metadata: Option<DocumentMetadata>,
    pub user_query: String,
    pub conversation_history: Vec<ConversationTurn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub document_type: Option<String>,
    pub sections: Vec<String>,
    pub key_concepts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

pub struct PromptTemplates;

impl PromptTemplates {
    /// Template for analyzing documents with context awareness
    pub const DOCUMENT_ANALYSIS: &'static str = r#"
You are an AI assistant specialized in analyzing and understanding documents. Your role is to help users understand, compare, and work with their document content.

DOCUMENT CONTEXT:
{document_content}

DOCUMENT METADATA:
Title: {document_title}
Type: {document_type}
Key Sections: {sections}
Key Concepts: {key_concepts}

USER QUESTION: {user_query}

Please provide a helpful, accurate response based on the document content above. When referencing specific information, cite the relevant section or context from the document. If the question cannot be answered from the provided document content, clearly state this and offer to help in other ways.

Response Guidelines:
- Be specific and reference document content when possible
- If information is missing, clearly state what's not available
- Offer follow-up suggestions when appropriate
- Maintain a helpful and professional tone
"#;

    /// Template for comparing two documents
    #[allow(dead_code)]
    pub const CONTENT_COMPARISON: &'static str = r#"
You are an AI assistant specialized in document comparison and analysis. You help users understand differences, similarities, and implications of changes between documents.

DOCUMENT A (BASELINE):
Title: {doc_a_title}
Content: {doc_a_content}

DOCUMENT B (UPDATED):
Title: {doc_b_title}
Content: {doc_b_content}

USER REQUEST: {user_query}

Please analyze the differences between these documents and provide:
1. Key changes identified
2. Additions, deletions, or modifications
3. Potential implications of the changes
4. Recommendations for next steps

Focus on meaningful changes that would impact users or processes. Highlight any critical differences that require attention.
"#;

    /// Template for question-answering with document context
    pub const QUESTION_ANSWERING: &'static str = r#"
You are a knowledgeable assistant helping users find information in their documents. Use the provided document content to answer questions accurately and comprehensively.

RELEVANT DOCUMENT CONTENT:
{document_content}

CONVERSATION HISTORY:
{conversation_history}

USER QUESTION: {user_query}

Based on the document content above, please provide a detailed answer to the user's question. If the document doesn't contain sufficient information to answer completely, explain what information is available and what might be missing.

Guidelines:
- Quote specific sections when relevant
- Explain concepts clearly if they appear complex
- Suggest related topics the user might find helpful
- If the answer isn't in the documents, say so clearly
"#;

    /// Template for content generation based on existing documents
    #[allow(dead_code)]
    pub const CONTENT_GENERATION: &'static str = r#"
You are an AI assistant specialized in creating new content based on existing documents. You help users generate training materials, summaries, guides, and other content while maintaining consistency with their existing documentation.

SOURCE DOCUMENT(S):
{document_content}

DOCUMENT STYLE PROFILE:
- Tone: {document_tone}
- Vocabulary Level: {vocabulary_level}
- Format Preferences: {format_preferences}

GENERATION REQUEST: {user_query}

Please create the requested content while:
1. Maintaining consistency with the source document's style and tone
2. Using appropriate vocabulary level for the intended audience
3. Following the established format patterns
4. Ensuring accuracy and relevance to the source material

If you need clarification about the request or additional context, please ask specific questions.
"#;

    /// Template for document structure analysis
    pub const STRUCTURE_ANALYSIS: &'static str = r#"
You are an AI assistant specialized in analyzing document structure and organization. You help users understand how their documents are organized and suggest improvements.

DOCUMENT CONTENT:
{document_content}

ANALYSIS REQUEST: {user_query}

Please analyze the document structure and provide insights on:
1. Overall organization and flow
2. Section hierarchy and relationships
3. Information gaps or redundancies
4. Suggestions for improvement
5. Accessibility and readability considerations

Focus on practical recommendations that would improve the document's effectiveness and usability.
"#;

    /// Template for procedural content analysis
    pub const PROCEDURE_ANALYSIS: &'static str = r#"
You are an AI assistant specialized in analyzing procedures, workflows, and step-by-step content. You help users understand, improve, and adapt procedural documentation.

PROCEDURAL CONTENT:
{document_content}

USER REQUEST: {user_query}

Please analyze the procedural content and provide:
1. Clear breakdown of the main process or workflow
2. Identification of decision points and alternatives
3. Prerequisites and dependencies
4. Potential issues or bottlenecks
5. Suggestions for clarity or completeness

If this is a comparison with another procedure, highlight the key differences and their implications.
"#;

    /// Template for style and tone analysis
    pub const STYLE_ANALYSIS: &'static str = r#"
You are an AI assistant specialized in analyzing writing style, tone, and communication patterns in documents. You help users understand and maintain consistent style across their documentation.

DOCUMENT CONTENT:
{document_content}

STYLE ANALYSIS REQUEST: {user_query}

Please analyze the document's style and provide insights on:
1. Overall tone and voice (formal, casual, technical, friendly, etc.)
2. Vocabulary complexity and audience level
3. Sentence structure and readability
4. Consistency patterns
5. Recommendations for style improvements or adaptations

If comparing multiple documents, highlight style differences and suggest harmonization approaches.
"#;

    /// Template for training content adaptation
    #[allow(dead_code)]
    pub const TRAINING_ADAPTATION: &'static str = r#"
You are an AI assistant specialized in adapting content for training and educational purposes. You help transform technical documentation into effective learning materials.

SOURCE CONTENT:
{document_content}

TARGET AUDIENCE: {target_audience}
LEARNING OBJECTIVES: {learning_objectives}
FORMAT REQUEST: {user_query}

Please adapt the content for training purposes by:
1. Organizing information for optimal learning progression
2. Adding explanations appropriate for the target audience
3. Identifying key concepts that need emphasis
4. Suggesting interactive elements or assessments
5. Maintaining accuracy while improving accessibility

Consider adult learning principles and practical application needs.
"#;
}

impl PromptTemplates {
    /// Build a document analysis prompt with context
    pub fn build_document_analysis_prompt(context: &PromptContext) -> String {
        let mut prompt = Self::DOCUMENT_ANALYSIS.to_string();

        // Replace placeholders with actual content
        prompt = prompt.replace(
            "{document_content}",
            context
                .document_content
                .as_deref()
                .unwrap_or("No document content available"),
        );

        if let Some(metadata) = &context.document_metadata {
            prompt = prompt.replace(
                "{document_title}",
                metadata.title.as_deref().unwrap_or("Untitled"),
            );
            prompt = prompt.replace(
                "{document_type}",
                metadata.document_type.as_deref().unwrap_or("Unknown"),
            );
            prompt = prompt.replace("{sections}", &metadata.sections.join(", "));
            prompt = prompt.replace("{key_concepts}", &metadata.key_concepts.join(", "));
        } else {
            prompt = prompt.replace("{document_title}", "Untitled");
            prompt = prompt.replace("{document_type}", "Unknown");
            prompt = prompt.replace("{sections}", "No sections identified");
            prompt = prompt.replace("{key_concepts}", "No key concepts identified");
        }

        prompt = prompt.replace("{user_query}", &context.user_query);

        prompt
    }

    /// Build a content comparison prompt
    #[allow(dead_code)]
    pub fn build_comparison_prompt(
        doc_a_title: &str,
        doc_a_content: &str,
        doc_b_title: &str,
        doc_b_content: &str,
        user_query: &str,
    ) -> String {
        Self::CONTENT_COMPARISON
            .replace("{doc_a_title}", doc_a_title)
            .replace("{doc_a_content}", doc_a_content)
            .replace("{doc_b_title}", doc_b_title)
            .replace("{doc_b_content}", doc_b_content)
            .replace("{user_query}", user_query)
    }

    /// Build a question-answering prompt with conversation history
    pub fn build_qa_prompt(context: &PromptContext) -> String {
        let mut prompt = Self::QUESTION_ANSWERING.to_string();

        prompt = prompt.replace(
            "{document_content}",
            context
                .document_content
                .as_deref()
                .unwrap_or("No document content available"),
        );

        // Format conversation history
        let history = if context.conversation_history.is_empty() {
            "No previous conversation".to_string()
        } else {
            context
                .conversation_history
                .iter()
                .map(|turn| format!("{}: {}", turn.role.to_uppercase(), turn.content))
                .collect::<Vec<_>>()
                .join("\n")
        };
        prompt = prompt.replace("{conversation_history}", &history);
        prompt = prompt.replace("{user_query}", &context.user_query);

        prompt
    }

    /// Build a content generation prompt
    #[allow(dead_code)]
    pub fn build_generation_prompt(
        document_content: &str,
        document_tone: &str,
        vocabulary_level: &str,
        format_preferences: &str,
        user_query: &str,
    ) -> String {
        Self::CONTENT_GENERATION
            .replace("{document_content}", document_content)
            .replace("{document_tone}", document_tone)
            .replace("{vocabulary_level}", vocabulary_level)
            .replace("{format_preferences}", format_preferences)
            .replace("{user_query}", user_query)
    }

    /// Build a structure analysis prompt
    pub fn build_structure_analysis_prompt(document_content: &str, user_query: &str) -> String {
        Self::STRUCTURE_ANALYSIS
            .replace("{document_content}", document_content)
            .replace("{user_query}", user_query)
    }

    /// Build a procedure analysis prompt
    pub fn build_procedure_analysis_prompt(document_content: &str, user_query: &str) -> String {
        Self::PROCEDURE_ANALYSIS
            .replace("{document_content}", document_content)
            .replace("{user_query}", user_query)
    }

    /// Build a style analysis prompt
    pub fn build_style_analysis_prompt(document_content: &str, user_query: &str) -> String {
        Self::STYLE_ANALYSIS
            .replace("{document_content}", document_content)
            .replace("{user_query}", user_query)
    }

    /// Build a training adaptation prompt
    #[allow(dead_code)]
    pub fn build_training_adaptation_prompt(
        document_content: &str,
        target_audience: &str,
        learning_objectives: &str,
        user_query: &str,
    ) -> String {
        Self::TRAINING_ADAPTATION
            .replace("{document_content}", document_content)
            .replace("{target_audience}", target_audience)
            .replace("{learning_objectives}", learning_objectives)
            .replace("{user_query}", user_query)
    }

    /// Detect the appropriate prompt template based on user query
    pub fn detect_prompt_type(user_query: &str) -> PromptType {
        let query_lower = user_query.to_lowercase();

        // Check for more specific patterns first
        if query_lower.contains("training")
            || query_lower.contains("teach")
            || query_lower.contains("learn")
        {
            PromptType::TrainingAdaptation
        } else if query_lower.contains("compare")
            || query_lower.contains("difference")
            || query_lower.contains("versus")
        {
            PromptType::Comparison
        } else if query_lower.contains("structure")
            || query_lower.contains("organize")
            || query_lower.contains("layout")
        {
            PromptType::StructureAnalysis
        } else if query_lower.contains("procedure")
            || query_lower.contains("step")
            || query_lower.contains("workflow")
        {
            PromptType::ProcedureAnalysis
        } else if query_lower.contains("style")
            || query_lower.contains("tone")
            || query_lower.contains("writing")
        {
            PromptType::StyleAnalysis
        } else if query_lower.contains("generate")
            || query_lower.contains("create")
            || query_lower.contains("write")
        {
            PromptType::Generation
        } else {
            PromptType::QuestionAnswering
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PromptType {
    #[allow(dead_code)]
    DocumentAnalysis,
    Comparison,
    QuestionAnswering,
    Generation,
    StructureAnalysis,
    ProcedureAnalysis,
    StyleAnalysis,
    TrainingAdaptation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_type_detection() {
        assert_eq!(
            PromptTemplates::detect_prompt_type("Compare these two documents"),
            PromptType::Comparison
        );
        assert_eq!(
            PromptTemplates::detect_prompt_type("Generate a summary"),
            PromptType::Generation
        );
        assert_eq!(
            PromptTemplates::detect_prompt_type("What is the main point?"),
            PromptType::QuestionAnswering
        );
        assert_eq!(
            PromptTemplates::detect_prompt_type("Analyze the structure"),
            PromptType::StructureAnalysis
        );
        assert_eq!(
            PromptTemplates::detect_prompt_type("Explain the procedure"),
            PromptType::ProcedureAnalysis
        );
        assert_eq!(
            PromptTemplates::detect_prompt_type("What's the writing style?"),
            PromptType::StyleAnalysis
        );
        assert_eq!(
            PromptTemplates::detect_prompt_type("Create training materials"),
            PromptType::TrainingAdaptation
        );
    }

    #[test]
    fn test_prompt_building() {
        let context = PromptContext {
            document_content: Some("Test document content".to_string()),
            document_metadata: Some(DocumentMetadata {
                title: Some("Test Document".to_string()),
                document_type: Some("Technical Guide".to_string()),
                sections: vec!["Introduction".to_string(), "Main Content".to_string()],
                key_concepts: vec!["Testing".to_string(), "Documentation".to_string()],
            }),
            user_query: "What is this document about?".to_string(),
            conversation_history: vec![],
        };

        let prompt = PromptTemplates::build_document_analysis_prompt(&context);
        assert!(prompt.contains("Test document content"));
        assert!(prompt.contains("Test Document"));
        assert!(prompt.contains("What is this document about?"));
    }

    #[test]
    fn test_comparison_prompt_building() {
        let prompt = PromptTemplates::build_comparison_prompt(
            "Document A",
            "Content A",
            "Document B",
            "Content B",
            "What changed?",
        );

        assert!(prompt.contains("Document A"));
        assert!(prompt.contains("Content A"));
        assert!(prompt.contains("Document B"));
        assert!(prompt.contains("Content B"));
        assert!(prompt.contains("What changed?"));
    }
}
