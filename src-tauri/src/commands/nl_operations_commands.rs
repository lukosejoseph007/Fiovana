// src-tauri/src/commands/nl_operations_commands.rs
// Natural Language Operations Tauri Commands

use crate::ai::document_commands::DocumentCommandProcessor;
use crate::ai::nl_operations::{NLOperationResult, NaturalLanguageOperations, ParsedOperation};
use crate::document::indexer::DocumentIndexer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct NLOperationResponse {
    pub success: bool,
    pub result: Option<NLOperationResult>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseResponse {
    pub success: bool,
    pub operation: Option<ParsedOperation>,
    pub error: Option<String>,
}

/// Execute a natural language document operation
#[tauri::command]
pub async fn execute_natural_language_operation(
    input: String,
    indexer: State<'_, Arc<DocumentIndexer>>,
) -> Result<NLOperationResponse, String> {
    // Create command processor
    let command_processor = DocumentCommandProcessor::new(indexer.inner().clone());

    // Create natural language operations handler
    let nl_operations = NaturalLanguageOperations::new(command_processor)
        .map_err(|e| format!("Failed to initialize NL operations: {}", e))?;

    // Execute the operation
    match nl_operations.execute_operation(&input).await {
        Ok(result) => Ok(NLOperationResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Ok(NLOperationResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Parse natural language input without executing
#[tauri::command]
pub async fn parse_natural_language_operation(
    input: String,
    indexer: State<'_, Arc<DocumentIndexer>>,
) -> Result<ParseResponse, String> {
    // Create command processor
    let command_processor = DocumentCommandProcessor::new(indexer.inner().clone());

    // Create natural language operations handler
    let nl_operations = NaturalLanguageOperations::new(command_processor)
        .map_err(|e| format!("Failed to initialize NL operations: {}", e))?;

    // Parse the operation
    match nl_operations.parse_operation(&input) {
        Ok(operation) => Ok(ParseResponse {
            success: true,
            operation: Some(operation),
            error: None,
        }),
        Err(e) => Ok(ParseResponse {
            success: false,
            operation: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get examples of natural language operations
#[tauri::command]
pub async fn get_nl_operation_examples() -> Result<Vec<String>, String> {
    Ok(vec![
        "Summarize user_guide.pdf".to_string(),
        "Give me a brief summary of the technical manual".to_string(),
        "Compare report_v1.docx with report_v2.docx".to_string(),
        "Show me the differences between these two documents".to_string(),
        "Find documents similar to project_spec.pdf".to_string(),
        "What documents are related to this one?".to_string(),
        "Search for documents about machine learning".to_string(),
        "Show me all documents regarding API documentation".to_string(),
        "Analyze the structure of requirements.docx".to_string(),
        "Examine the content themes in this document".to_string(),
        "Extract key points from meeting_notes.txt".to_string(),
        "What are the main points in this document?".to_string(),
    ])
}

/// Get suggestions for improving natural language queries
#[tauri::command]
pub async fn get_nl_operation_suggestions(operation_type: String) -> Result<Vec<String>, String> {
    let suggestions = match operation_type.as_str() {
        "summarize" => vec![
            "Be specific about summary length: 'brief', 'detailed', etc.".to_string(),
            "Include the document name or path".to_string(),
            "Example: 'Give me a brief summary of report.pdf'".to_string(),
        ],
        "compare" => vec![
            "Specify exactly two documents to compare".to_string(),
            "Mention what to focus on: 'content', 'structure', 'style'".to_string(),
            "Example: 'Compare content of doc1.pdf with doc2.pdf'".to_string(),
        ],
        "search" => vec![
            "Use specific keywords that might appear in documents".to_string(),
            "Be clear about what you're looking for".to_string(),
            "Example: 'Find documents about user authentication'".to_string(),
        ],
        "analyze" => vec![
            "Specify what to analyze: 'structure', 'content', 'style', 'completeness'".to_string(),
            "Include the document name".to_string(),
            "Example: 'Analyze the structure of technical_spec.docx'".to_string(),
        ],
        "similar" => vec![
            "Specify the reference document clearly".to_string(),
            "Mention how many results you want".to_string(),
            "Example: 'Find 5 documents similar to user_guide.pdf'".to_string(),
        ],
        "extract" => vec![
            "Specify how many key points you want".to_string(),
            "Include the document name".to_string(),
            "Example: 'Extract 5 key points from meeting_notes.txt'".to_string(),
        ],
        _ => vec![
            "Be specific about the operation you want to perform".to_string(),
            "Include document names or paths".to_string(),
            "Use clear action words like 'summarize', 'compare', 'find', 'search'".to_string(),
        ],
    };

    Ok(suggestions)
}

/// Test natural language operations with sample data
#[tauri::command]
pub async fn test_nl_operations(
    indexer: State<'_, Arc<DocumentIndexer>>,
) -> Result<Vec<NLOperationResponse>, String> {
    let test_queries = vec![
        "summarize test document",
        "compare doc1 with doc2",
        "find similar documents",
        "search for documentation",
        "analyze document structure",
        "extract key points",
    ];

    let mut results = Vec::new();

    for query in test_queries {
        // Create command processor
        let command_processor = DocumentCommandProcessor::new(indexer.inner().clone());

        // Create natural language operations handler
        let nl_operations = NaturalLanguageOperations::new(command_processor)
            .map_err(|e| format!("Failed to initialize NL operations: {}", e))?;

        // Test parsing (not executing to avoid errors with missing documents)
        match nl_operations.parse_operation(query) {
            Ok(operation) => {
                results.push(NLOperationResponse {
                    success: true,
                    result: Some(NLOperationResult {
                        success: true,
                        operation,
                        result: format!("Successfully parsed: {}", query),
                        execution_time_ms: 0,
                        suggestions: vec![],
                    }),
                    error: None,
                });
            }
            Err(e) => {
                results.push(NLOperationResponse {
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}
