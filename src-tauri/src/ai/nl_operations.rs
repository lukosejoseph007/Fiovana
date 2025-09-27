// src-tauri/src/ai/nl_operations.rs
// Natural Language Document Operations Parser
// Converts natural language requests into structured document commands

use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::document_commands::{
    AnalysisType, DocumentCommand, DocumentCommandProcessor, SearchType, SummaryLength,
};
use crate::document::document_comparison::ComparisonType;

/// Natural language operation parser and executor
pub struct NaturalLanguageOperations {
    command_processor: DocumentCommandProcessor,
    patterns: OperationPatterns,
}

/// Compiled regex patterns for natural language parsing
struct OperationPatterns {
    // Document references
    document_ref: Regex,
    #[allow(dead_code)]
    document_list: Regex,

    // Operation patterns
    summarize: Regex,
    compare: Regex,
    find_similar: Regex,
    search: Regex,
    analyze: Regex,
    extract_points: Regex,

    // Parameter patterns
    summary_length: Regex,
    #[allow(dead_code)]
    comparison_type: Regex,
    #[allow(dead_code)]
    analysis_type: Regex,
    max_results: Regex,
}

/// Parsed natural language operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOperation {
    pub operation_type: OperationType,
    pub documents: Vec<String>,
    pub parameters: HashMap<String, String>,
    pub confidence: f32,
    pub query: String,
}

/// Types of operations that can be parsed from natural language
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    Summarize,
    Compare,
    FindSimilar,
    Search,
    Analyze,
    ExtractKeyPoints,
    Unknown,
}

/// Result of executing a natural language operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLOperationResult {
    pub success: bool,
    pub operation: ParsedOperation,
    pub result: String,
    pub execution_time_ms: u64,
    pub suggestions: Vec<String>,
}

impl NaturalLanguageOperations {
    pub fn new(command_processor: DocumentCommandProcessor) -> Result<Self> {
        let patterns = OperationPatterns::new()?;

        Ok(Self {
            command_processor,
            patterns,
        })
    }

    /// Parse natural language input into a structured operation
    pub fn parse_operation(&self, input: &str) -> Result<ParsedOperation> {
        let input = input.trim().to_lowercase();

        // Extract document references
        let documents = self.extract_document_references(&input);

        // Determine operation type and confidence
        let (operation_type, confidence) = self.classify_operation(&input);

        // Extract parameters based on operation type
        let parameters = self.extract_parameters(&input, &operation_type);

        Ok(ParsedOperation {
            operation_type,
            documents,
            parameters,
            confidence,
            query: input,
        })
    }

    /// Execute a natural language operation
    pub async fn execute_operation(&self, input: &str) -> Result<NLOperationResult> {
        let start_time = std::time::Instant::now();

        // Parse the natural language input
        let parsed_operation = self.parse_operation(input)?;

        if parsed_operation.confidence < 0.3 {
            return Ok(NLOperationResult {
                success: false,
                operation: parsed_operation.clone(),
                result: "I'm not sure what you want me to do. Could you please be more specific?"
                    .to_string(),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                suggestions: self.generate_suggestions(&parsed_operation),
            });
        }

        // Convert to document command
        let command = self.convert_to_command(&parsed_operation)?;

        // Execute the command
        let command_result = self.command_processor.execute_command(command).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(NLOperationResult {
            success: command_result.success,
            operation: parsed_operation.clone(),
            result: if command_result.success {
                self.format_result(&command_result.result, &command_result.command)
            } else {
                command_result.result
            },
            execution_time_ms: execution_time,
            suggestions: if command_result.success {
                self.generate_follow_up_suggestions(&command_result.command)
            } else {
                self.generate_error_suggestions(&parsed_operation)
            },
        })
    }

    /// Extract document references from natural language input
    fn extract_document_references(&self, input: &str) -> Vec<String> {
        let mut documents = std::collections::HashSet::new();

        // Extract quoted document names
        for cap in self.patterns.document_ref.captures_iter(input) {
            if let Some(doc_match) = cap.get(1) {
                documents.insert(doc_match.as_str().trim().to_string());
            }
        }

        // Extract file extensions
        let file_pattern = Regex::new(r"\b(\w+\.(pdf|docx?|txt|md))\b").unwrap();
        for cap in file_pattern.captures_iter(input) {
            if let Some(file_match) = cap.get(1) {
                documents.insert(file_match.as_str().to_string());
            }
        }

        documents.into_iter().collect()
    }

    /// Classify the type of operation from natural language
    fn classify_operation(&self, input: &str) -> (OperationType, f32) {
        let mut scores = HashMap::new();

        // Check for summarize patterns
        if self.patterns.summarize.is_match(input) {
            scores.insert(OperationType::Summarize, 0.9);
        }

        // Check for compare patterns
        if self.patterns.compare.is_match(input) {
            scores.insert(OperationType::Compare, 0.9);
        }

        // Check for find similar patterns
        if self.patterns.find_similar.is_match(input) {
            scores.insert(OperationType::FindSimilar, 0.9);
        }

        // Check for search patterns
        if self.patterns.search.is_match(input) {
            scores.insert(OperationType::Search, 0.8);
        }

        // Check for analyze patterns
        if self.patterns.analyze.is_match(input) {
            scores.insert(OperationType::Analyze, 0.8);
        }

        // Check for extract points patterns
        if self.patterns.extract_points.is_match(input) {
            scores.insert(OperationType::ExtractKeyPoints, 0.8);
        }

        // Return the highest scoring operation
        if let Some((op_type, score)) = scores.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
            (op_type.clone(), *score)
        } else {
            (OperationType::Unknown, 0.1)
        }
    }

    /// Extract parameters from natural language based on operation type
    fn extract_parameters(
        &self,
        input: &str,
        operation_type: &OperationType,
    ) -> HashMap<String, String> {
        let mut parameters = HashMap::new();

        match operation_type {
            OperationType::Summarize => {
                // Extract summary length
                if let Some(cap) = self.patterns.summary_length.captures(input) {
                    if let Some(length_match) = cap.get(1) {
                        let length = match length_match.as_str() {
                            "brief" | "short" | "quickly" => "brief",
                            "detailed" | "comprehensive" | "thorough" => "detailed",
                            "medium" | "moderate" => "medium",
                            _ => "short",
                        };
                        parameters.insert("length".to_string(), length.to_string());
                    }
                } else {
                    parameters.insert("length".to_string(), "short".to_string());
                }
            }
            OperationType::Compare => {
                // Extract comparison type
                if input.contains("content") || input.contains("text") {
                    parameters.insert("type".to_string(), "text".to_string());
                } else if input.contains("structure") || input.contains("format") {
                    parameters.insert("type".to_string(), "structural".to_string());
                } else if input.contains("semantic") || input.contains("meaning") {
                    parameters.insert("type".to_string(), "semantic".to_string());
                } else {
                    parameters.insert("type".to_string(), "comprehensive".to_string());
                }
            }
            OperationType::FindSimilar => {
                // Extract max results
                if let Some(cap) = self.patterns.max_results.captures(input) {
                    if let Some(num_match) = cap.get(1) {
                        parameters
                            .insert("max_results".to_string(), num_match.as_str().to_string());
                    }
                } else {
                    parameters.insert("max_results".to_string(), "5".to_string());
                }
            }
            OperationType::Search => {
                // Extract search query
                let query_patterns = [
                    r"(?:about|for|regarding)\s+(.+)",
                    r"(?:find|search)\s+(.+)",
                    r"documents\s+(.+)",
                ];

                for pattern in &query_patterns {
                    if let Ok(regex) = Regex::new(pattern) {
                        if let Some(cap) = regex.captures(input) {
                            if let Some(query_match) = cap.get(1) {
                                parameters.insert(
                                    "query".to_string(),
                                    query_match.as_str().trim().to_string(),
                                );
                                break;
                            }
                        }
                    }
                }

                // Determine search type
                if input.contains("semantic") || input.contains("meaning") {
                    parameters.insert("search_type".to_string(), "semantic".to_string());
                } else if input.contains("keyword") || input.contains("exact") {
                    parameters.insert("search_type".to_string(), "keyword".to_string());
                } else {
                    parameters.insert("search_type".to_string(), "hybrid".to_string());
                }
            }
            OperationType::Analyze => {
                // Extract analysis type
                if input.contains("structure") || input.contains("organization") {
                    parameters.insert("type".to_string(), "structure".to_string());
                } else if input.contains("content") || input.contains("topics") {
                    parameters.insert("type".to_string(), "content".to_string());
                } else if input.contains("style") || input.contains("tone") {
                    parameters.insert("type".to_string(), "style".to_string());
                } else if input.contains("complete") || input.contains("gaps") {
                    parameters.insert("type".to_string(), "completeness".to_string());
                } else {
                    parameters.insert("type".to_string(), "content".to_string());
                }
            }
            OperationType::ExtractKeyPoints => {
                // Extract max points
                if let Some(cap) = self.patterns.max_results.captures(input) {
                    if let Some(num_match) = cap.get(1) {
                        parameters.insert("max_points".to_string(), num_match.as_str().to_string());
                    }
                } else {
                    parameters.insert("max_points".to_string(), "5".to_string());
                }
            }
            OperationType::Unknown => {}
        }

        parameters
    }

    /// Convert parsed operation to document command
    fn convert_to_command(&self, operation: &ParsedOperation) -> Result<DocumentCommand> {
        match operation.operation_type {
            OperationType::Summarize => {
                let document_id = operation
                    .documents
                    .first()
                    .ok_or_else(|| anyhow!("No document specified for summarization"))?;
                let length = operation
                    .parameters
                    .get("length")
                    .map(|l| match l.as_str() {
                        "brief" => SummaryLength::Brief,
                        "medium" => SummaryLength::Medium,
                        "detailed" => SummaryLength::Detailed,
                        _ => SummaryLength::Short,
                    })
                    .unwrap_or(SummaryLength::Short);

                Ok(DocumentCommand::Summarize {
                    document_id: document_id.clone(),
                    length,
                })
            }
            OperationType::Compare => {
                if operation.documents.len() < 2 {
                    return Err(anyhow!("Need at least two documents for comparison"));
                }

                let comparison_type = operation
                    .parameters
                    .get("type")
                    .map(|t| match t.as_str() {
                        "text" => ComparisonType::TextDiff,
                        "structural" => ComparisonType::StructuralDiff,
                        "semantic" => ComparisonType::SemanticSimilarity,
                        _ => ComparisonType::Comprehensive,
                    })
                    .unwrap_or(ComparisonType::Comprehensive);

                Ok(DocumentCommand::Compare {
                    doc_a: operation.documents[0].clone(),
                    doc_b: operation.documents[1].clone(),
                    comparison_type,
                })
            }
            OperationType::FindSimilar => {
                let document_id = operation
                    .documents
                    .first()
                    .ok_or_else(|| anyhow!("No reference document specified"))?;
                let max_results = operation
                    .parameters
                    .get("max_results")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5);

                Ok(DocumentCommand::FindSimilar {
                    reference_document: document_id.clone(),
                    max_results,
                })
            }
            OperationType::Search => {
                let query = operation
                    .parameters
                    .get("query")
                    .ok_or_else(|| anyhow!("No search query specified"))?;
                let search_type = operation
                    .parameters
                    .get("search_type")
                    .map(|t| match t.as_str() {
                        "keyword" => SearchType::Keyword,
                        "semantic" => SearchType::Semantic,
                        _ => SearchType::Hybrid,
                    })
                    .unwrap_or(SearchType::Hybrid);

                Ok(DocumentCommand::SearchDocuments {
                    query: query.clone(),
                    max_results: 10,
                    search_type,
                })
            }
            OperationType::Analyze => {
                let document_id = operation
                    .documents
                    .first()
                    .ok_or_else(|| anyhow!("No document specified for analysis"))?;
                let analysis_type = operation
                    .parameters
                    .get("type")
                    .map(|t| match t.as_str() {
                        "structure" => AnalysisType::Structure,
                        "content" => AnalysisType::Content,
                        "style" => AnalysisType::Style,
                        "completeness" => AnalysisType::Completeness,
                        _ => AnalysisType::Content,
                    })
                    .unwrap_or(AnalysisType::Content);

                Ok(DocumentCommand::AnalyzeDocument {
                    document_id: document_id.clone(),
                    analysis_type,
                })
            }
            OperationType::ExtractKeyPoints => {
                let document_id = operation
                    .documents
                    .first()
                    .ok_or_else(|| anyhow!("No document specified for key point extraction"))?;
                let max_points = operation
                    .parameters
                    .get("max_points")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5);

                Ok(DocumentCommand::ExtractKeyPoints {
                    document_id: document_id.clone(),
                    max_points,
                })
            }
            OperationType::Unknown => Err(anyhow!("Unknown operation type")),
        }
    }

    /// Format command result for natural language response
    fn format_result(&self, result: &str, command: &DocumentCommand) -> String {
        match command {
            DocumentCommand::Summarize { .. } => {
                format!("Here's a summary of the document:\n\n{}", result)
            }
            DocumentCommand::Compare { .. } => {
                format!("Document comparison results:\n\n{}", result)
            }
            DocumentCommand::FindSimilar { .. } => {
                format!("I found these similar documents:\n\n{}", result)
            }
            DocumentCommand::SearchDocuments { .. } => {
                format!("Search results:\n\n{}", result)
            }
            DocumentCommand::AnalyzeDocument { .. } => {
                format!("Document analysis:\n\n{}", result)
            }
            DocumentCommand::ExtractKeyPoints { .. } => {
                format!("Key points from the document:\n\n{}", result)
            }
        }
    }

    /// Generate suggestions for unclear requests
    fn generate_suggestions(&self, operation: &ParsedOperation) -> Vec<String> {
        match operation.operation_type {
            OperationType::Unknown => vec![
                "Try: 'Summarize document.pdf'".to_string(),
                "Try: 'Compare doc1.pdf with doc2.pdf'".to_string(),
                "Try: 'Find documents similar to report.docx'".to_string(),
                "Try: 'Search for documents about machine learning'".to_string(),
            ],
            _ => vec![
                "Be more specific about which documents you want to work with".to_string(),
                "Include document names or file paths".to_string(),
            ],
        }
    }

    /// Generate follow-up suggestions after successful operations
    fn generate_follow_up_suggestions(&self, command: &DocumentCommand) -> Vec<String> {
        match command {
            DocumentCommand::Summarize { .. } => vec![
                "Want a more detailed summary?".to_string(),
                "Would you like to analyze the document structure?".to_string(),
                "Want to find similar documents?".to_string(),
            ],
            DocumentCommand::Compare { .. } => vec![
                "Want to see a different type of comparison?".to_string(),
                "Would you like to analyze the style differences?".to_string(),
                "Want to compare with additional documents?".to_string(),
            ],
            DocumentCommand::FindSimilar { .. } => vec![
                "Want to compare any of these similar documents?".to_string(),
                "Would you like to analyze common themes?".to_string(),
                "Want to search for more specific criteria?".to_string(),
            ],
            DocumentCommand::SearchDocuments { .. } => vec![
                "Want to refine your search?".to_string(),
                "Would you like to search semantically instead?".to_string(),
                "Want to analyze any of these documents?".to_string(),
            ],
            DocumentCommand::AnalyzeDocument { .. } => vec![
                "Want to analyze a different aspect?".to_string(),
                "Would you like to summarize this document?".to_string(),
                "Want to find similar documents?".to_string(),
            ],
            DocumentCommand::ExtractKeyPoints { .. } => vec![
                "Want more or fewer key points?".to_string(),
                "Would you like a summary of the document?".to_string(),
                "Want to analyze the document structure?".to_string(),
            ],
        }
    }

    /// Generate suggestions for error recovery
    fn generate_error_suggestions(&self, operation: &ParsedOperation) -> Vec<String> {
        let mut suggestions = vec![
            "Check that the document names are correct".to_string(),
            "Make sure the documents are imported in your workspace".to_string(),
        ];

        match operation.operation_type {
            OperationType::Compare => {
                suggestions.push("Specify exactly two documents to compare".to_string());
            }
            OperationType::Search => {
                suggestions.push("Try a different search term".to_string());
                suggestions.push("Use keywords that might appear in the documents".to_string());
            }
            _ => {
                suggestions.push("Try rephrasing your request".to_string());
            }
        }

        suggestions
    }
}

impl OperationPatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            // Document reference patterns
            document_ref: Regex::new(r#"["']([^"']+)["']"#)?,
            document_list: Regex::new(r"\b(\w+\.(pdf|docx?|txt|md))\b")?,

            // Operation patterns
            summarize: Regex::new(
                r"(?i)\b(summarize|summary|sum up|key points from|overview of|brief)\b",
            )?,
            compare: Regex::new(
                r"(?i)\b(compare|comparison|differences|diff|contrast|versus|vs)\b",
            )?,
            find_similar: Regex::new(r"(?i)\b(similar|like|related|comparable|resembling)\b")?,
            search: Regex::new(r"(?i)\b(search|find|look for|documents about|show me)\b")?,
            analyze: Regex::new(r"(?i)\b(analyze|analysis|examine|study|review|assess)\b")?,
            extract_points: Regex::new(
                r"(?i)\b(key points|main points|important points|extract|highlights)\b",
            )?,

            // Parameter patterns
            summary_length: Regex::new(
                r"(?i)\b(brief|short|quick|detailed|comprehensive|thorough|medium|moderate)\b",
            )?,
            comparison_type: Regex::new(
                r"(?i)\b(content|text|structure|format|semantic|meaning)\b",
            )?,
            analysis_type: Regex::new(
                r"(?i)\b(structure|organization|content|topics|style|tone|complete|gaps)\b",
            )?,
            max_results: Regex::new(r"\b(\d+)\b")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let patterns = OperationPatterns::new();
        assert!(patterns.is_ok());
    }

    #[test]
    fn test_document_reference_extraction() {
        let patterns = OperationPatterns::new().unwrap();
        let nl_ops = NaturalLanguageOperations {
            command_processor: DocumentCommandProcessor::new(std::sync::Arc::new(
                crate::document::indexer::DocumentIndexer::new("test".to_string().into()).unwrap(),
            )),
            patterns,
        };

        let input = "Compare 'document1.pdf' with document2.docx";
        let docs = nl_ops.extract_document_references(input);
        assert_eq!(docs.len(), 2);
        assert!(docs.contains(&"document1.pdf".to_string()));
        assert!(docs.contains(&"document2.docx".to_string()));
    }

    #[test]
    fn test_operation_classification() {
        let patterns = OperationPatterns::new().unwrap();
        let nl_ops = NaturalLanguageOperations {
            command_processor: DocumentCommandProcessor::new(std::sync::Arc::new(
                crate::document::indexer::DocumentIndexer::new("test".to_string().into()).unwrap(),
            )),
            patterns,
        };

        let (op_type, confidence) = nl_ops.classify_operation("summarize the document");
        assert!(matches!(op_type, OperationType::Summarize));
        assert!(confidence > 0.8);

        let (op_type, confidence) = nl_ops.classify_operation("compare document A with document B");
        assert!(matches!(op_type, OperationType::Compare));
        assert!(confidence > 0.8);
    }

    #[test]
    fn test_parameter_extraction() {
        let patterns = OperationPatterns::new().unwrap();
        let nl_ops = NaturalLanguageOperations {
            command_processor: DocumentCommandProcessor::new(std::sync::Arc::new(
                crate::document::indexer::DocumentIndexer::new("test".to_string().into()).unwrap(),
            )),
            patterns,
        };

        let params =
            nl_ops.extract_parameters("give me a brief summary", &OperationType::Summarize);
        assert_eq!(params.get("length"), Some(&"brief".to_string()));

        let params = nl_ops.extract_parameters(
            "compare the content of these documents",
            &OperationType::Compare,
        );
        assert_eq!(params.get("type"), Some(&"text".to_string()));
    }
}
