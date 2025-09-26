// src-tauri/src/ai/intent.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intent {
    // Document operations
    CompareDocuments,
    SummarizeDocument,
    AnalyzeDocument,
    UpdateContent,
    ReviewChanges,
    ExtractInformation,

    // Search operations
    SearchDocuments,
    FindDocuments,
    FilterDocuments,
    DiscoverContent,
    FindSimilarContent,
    ExtractConcepts,
    AnalyzeRelationships,

    // Generation operations
    CreateDocument,
    AdaptContent,
    TransformFormat,
    GenerateOutput,
    OptimizeContent,

    // Workspace operations
    OrganizeWorkspace,
    ReviewWorkspace,
    ExportDocuments,
    ConfigureWorkspace,
    ManageFiles,
    ManageTemplates,

    // Style and formatting
    AdaptStyle,
    AnalyzeStyle,
    ApplyStyle,

    // System operations
    CheckStatus,
    GetHelp,
    ListModels,
    ConfigureSystem,

    // Conversation management
    ClarifyQuestion,
    ProvideExample,
    ExplainProcess,

    // Unknown/unclear intent
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentConfidence {
    pub intent: Intent,
    pub confidence: f32,
    pub reasoning: String,
    pub alternative_intents: Vec<(Intent, f32)>,
}

pub struct IntentClassifier {
    patterns: HashMap<Intent, Vec<String>>,
}

impl IntentClassifier {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Document operations - Compare
        patterns.insert(
            Intent::CompareDocuments,
            vec![
                "compare".to_string(),
                "difference".to_string(),
                "diff".to_string(),
                "what changed".to_string(),
                "what's different".to_string(),
                "changes between".to_string(),
                "compare documents".to_string(),
                "analyze differences".to_string(),
                "side by side".to_string(),
                "versus".to_string(),
                "vs".to_string(),
            ],
        );

        // Document operations - Summarize
        patterns.insert(
            Intent::SummarizeDocument,
            vec![
                "summarize".to_string(),
                "summary".to_string(),
                "brief".to_string(),
                "overview".to_string(),
                "key points".to_string(),
                "main points".to_string(),
                "tldr".to_string(),
                "tl;dr".to_string(),
                "executive summary".to_string(),
                "condensed".to_string(),
                "highlight".to_string(),
            ],
        );

        // Document operations - Analyze
        patterns.insert(
            Intent::AnalyzeDocument,
            vec![
                "analyze".to_string(),
                "analysis".to_string(),
                "review".to_string(),
                "examine".to_string(),
                "evaluate".to_string(),
                "assess".to_string(),
                "study".to_string(),
                "investigate".to_string(),
                "breakdown".to_string(),
                "insights".to_string(),
                "understand".to_string(),
            ],
        );

        // Document operations - Extract Information
        patterns.insert(
            Intent::ExtractInformation,
            vec![
                "extract".to_string(),
                "get information".to_string(),
                "pull out".to_string(),
                "find details".to_string(),
                "what information".to_string(),
                "key data".to_string(),
                "important details".to_string(),
                "specific information".to_string(),
            ],
        );

        // Document operations - Update Content
        patterns.insert(
            Intent::UpdateContent,
            vec![
                "update".to_string(),
                "modify".to_string(),
                "change".to_string(),
                "edit".to_string(),
                "revise".to_string(),
                "apply changes".to_string(),
                "incorporate".to_string(),
                "merge".to_string(),
                "fix".to_string(),
                "correct".to_string(),
            ],
        );

        // Document operations - Review Changes
        patterns.insert(
            Intent::ReviewChanges,
            vec![
                "review changes".to_string(),
                "check changes".to_string(),
                "what was changed".to_string(),
                "review edits".to_string(),
                "validate changes".to_string(),
                "approve changes".to_string(),
                "inspect modifications".to_string(),
            ],
        );

        // Search operations - Search Documents
        patterns.insert(
            Intent::SearchDocuments,
            vec![
                "search".to_string(),
                "search documents".to_string(),
                "search for".to_string(),
                "look through".to_string(),
                "scan documents".to_string(),
                "query".to_string(),
                "full text search".to_string(),
                "search content".to_string(),
            ],
        );

        // Search operations - Find Documents
        patterns.insert(
            Intent::FindDocuments,
            vec![
                "find".to_string(),
                "find documents".to_string(),
                "locate".to_string(),
                "where is".to_string(),
                "show me".to_string(),
                "get documents".to_string(),
                "retrieve".to_string(),
                "look for".to_string(),
            ],
        );

        // Search operations - Filter Documents
        patterns.insert(
            Intent::FilterDocuments,
            vec![
                "filter".to_string(),
                "filter documents".to_string(),
                "narrow down".to_string(),
                "refine search".to_string(),
                "show only".to_string(),
                "documents with".to_string(),
                "documents containing".to_string(),
                "subset".to_string(),
            ],
        );

        // Search operations - Discover Content
        patterns.insert(
            Intent::DiscoverContent,
            vec![
                "discover".to_string(),
                "explore".to_string(),
                "what's available".to_string(),
                "browse".to_string(),
                "what content".to_string(),
                "what documents".to_string(),
                "content overview".to_string(),
                "inventory".to_string(),
            ],
        );

        // Search operations - Find Similar Content
        patterns.insert(
            Intent::FindSimilarContent,
            vec![
                "similar".to_string(),
                "like this".to_string(),
                "related".to_string(),
                "find similar".to_string(),
                "content like".to_string(),
                "same topic".to_string(),
                "related content".to_string(),
                "comparable".to_string(),
                "resembling".to_string(),
            ],
        );

        // Generation operations - Create Document
        patterns.insert(
            Intent::CreateDocument,
            vec![
                "create document".to_string(),
                "new document".to_string(),
                "write document".to_string(),
                "draft".to_string(),
                "compose".to_string(),
                "author".to_string(),
                "create new".to_string(),
                "start document".to_string(),
            ],
        );

        // Generation operations - Adapt Content
        patterns.insert(
            Intent::AdaptContent,
            vec![
                "adapt".to_string(),
                "adapt content".to_string(),
                "modify for".to_string(),
                "adjust for".to_string(),
                "tailor".to_string(),
                "customize".to_string(),
                "personalize".to_string(),
                "rewrite for".to_string(),
            ],
        );

        // Generation operations - Transform Format
        patterns.insert(
            Intent::TransformFormat,
            vec![
                "transform".to_string(),
                "convert".to_string(),
                "transform format".to_string(),
                "convert to".to_string(),
                "change format".to_string(),
                "reformat".to_string(),
                "restructure".to_string(),
                "format as".to_string(),
            ],
        );

        // Generation operations - Generate Output
        patterns.insert(
            Intent::GenerateOutput,
            vec![
                "generate".to_string(),
                "produce".to_string(),
                "make".to_string(),
                "build".to_string(),
                "export".to_string(),
                "output".to_string(),
                "save as".to_string(),
                "create output".to_string(),
                "generate file".to_string(),
            ],
        );

        // Generation operations - Optimize Content
        patterns.insert(
            Intent::OptimizeContent,
            vec![
                "optimize".to_string(),
                "improve".to_string(),
                "enhance".to_string(),
                "refine".to_string(),
                "polish".to_string(),
                "optimize content".to_string(),
                "make better".to_string(),
                "improve readability".to_string(),
            ],
        );

        // Workspace operations - Organize
        patterns.insert(
            Intent::OrganizeWorkspace,
            vec![
                "organize".to_string(),
                "organize workspace".to_string(),
                "structure".to_string(),
                "arrange".to_string(),
                "categorize".to_string(),
                "sort documents".to_string(),
                "organize files".to_string(),
                "tidy up".to_string(),
            ],
        );

        // Workspace operations - Review
        patterns.insert(
            Intent::ReviewWorkspace,
            vec![
                "review workspace".to_string(),
                "workspace overview".to_string(),
                "workspace status".to_string(),
                "check workspace".to_string(),
                "workspace health".to_string(),
                "workspace summary".to_string(),
                "assess workspace".to_string(),
            ],
        );

        // Workspace operations - Export
        patterns.insert(
            Intent::ExportDocuments,
            vec![
                "export".to_string(),
                "export documents".to_string(),
                "download".to_string(),
                "backup".to_string(),
                "save all".to_string(),
                "package".to_string(),
                "archive".to_string(),
                "export workspace".to_string(),
            ],
        );

        // Style operations - Adapt Style
        patterns.insert(
            Intent::AdaptStyle,
            vec![
                "style".to_string(),
                "adapt style".to_string(),
                "change tone".to_string(),
                "adjust tone".to_string(),
                "rewrite style".to_string(),
                "match style".to_string(),
                "writing style".to_string(),
                "tone adjustment".to_string(),
            ],
        );

        // Style operations - Analyze Style
        patterns.insert(
            Intent::AnalyzeStyle,
            vec![
                "analyze style".to_string(),
                "style analysis".to_string(),
                "writing analysis".to_string(),
                "tone analysis".to_string(),
                "style check".to_string(),
                "style review".to_string(),
                "style consistency".to_string(),
            ],
        );

        // Style operations - Apply Style
        patterns.insert(
            Intent::ApplyStyle,
            vec![
                "apply style".to_string(),
                "format".to_string(),
                "apply formatting".to_string(),
                "style document".to_string(),
                "format document".to_string(),
                "apply template".to_string(),
            ],
        );

        // System operations - Help
        patterns.insert(
            Intent::GetHelp,
            vec![
                "help".to_string(),
                "how to".to_string(),
                "what can".to_string(),
                "explain".to_string(),
                "guide".to_string(),
                "instructions".to_string(),
                "usage".to_string(),
                "tutorial".to_string(),
                "assistance".to_string(),
            ],
        );

        // System operations - Status
        patterns.insert(
            Intent::CheckStatus,
            vec![
                "status".to_string(),
                "health".to_string(),
                "available".to_string(),
                "working".to_string(),
                "connection".to_string(),
                "system status".to_string(),
                "check status".to_string(),
            ],
        );

        // System operations - List Models
        patterns.insert(
            Intent::ListModels,
            vec![
                "models".to_string(),
                "list models".to_string(),
                "available models".to_string(),
                "what models".to_string(),
                "ai models".to_string(),
                "show models".to_string(),
            ],
        );

        // System operations - Configure System
        patterns.insert(
            Intent::ConfigureSystem,
            vec![
                "configure".to_string(),
                "setup".to_string(),
                "settings".to_string(),
                "configuration".to_string(),
                "preferences".to_string(),
                "system settings".to_string(),
            ],
        );

        // Workspace patterns
        patterns.insert(
            Intent::ConfigureWorkspace,
            vec![
                "workspace".to_string(),
                "project".to_string(),
                "workspace settings".to_string(),
                "workspace config".to_string(),
                "project settings".to_string(),
            ],
        );

        // File management patterns
        patterns.insert(
            Intent::ManageFiles,
            vec![
                "files".to_string(),
                "documents".to_string(),
                "import".to_string(),
                "upload".to_string(),
                "add files".to_string(),
                "manage files".to_string(),
                "file management".to_string(),
            ],
        );

        // Template management
        patterns.insert(
            Intent::ManageTemplates,
            vec![
                "templates".to_string(),
                "template".to_string(),
                "manage templates".to_string(),
                "template management".to_string(),
                "create template".to_string(),
            ],
        );

        // Conversation management - Clarify
        patterns.insert(
            Intent::ClarifyQuestion,
            vec![
                "clarify".to_string(),
                "what do you mean".to_string(),
                "can you explain".to_string(),
                "more details".to_string(),
                "be more specific".to_string(),
                "unclear".to_string(),
                "confused".to_string(),
            ],
        );

        // Conversation management - Example
        patterns.insert(
            Intent::ProvideExample,
            vec![
                "example".to_string(),
                "show example".to_string(),
                "for example".to_string(),
                "sample".to_string(),
                "demonstrate".to_string(),
                "illustration".to_string(),
                "instance".to_string(),
            ],
        );

        // Conversation management - Explain Process
        patterns.insert(
            Intent::ExplainProcess,
            vec![
                "explain process".to_string(),
                "how does this work".to_string(),
                "walk me through".to_string(),
                "step by step".to_string(),
                "process explanation".to_string(),
                "workflow".to_string(),
                "procedure".to_string(),
            ],
        );

        Self { patterns }
    }

    pub async fn classify(&self, input: &str) -> Result<IntentConfidence> {
        let input_lower = input.to_lowercase();
        let mut scores: HashMap<Intent, f32> = HashMap::new();

        // Score each intent based on pattern matching
        for (intent, patterns) in &self.patterns {
            let mut score = 0.0;
            let mut matched_patterns = 0;

            for pattern in patterns {
                if input_lower.contains(pattern) {
                    // Weight longer patterns more heavily
                    let pattern_weight = pattern.len() as f32 / 10.0;
                    score += 1.0 + pattern_weight;
                    matched_patterns += 1;
                }
            }

            // Normalize by number of patterns and add bonus for multiple matches
            if matched_patterns > 0 {
                score /= patterns.len() as f32;
                if matched_patterns > 1 {
                    score *= 1.5; // Bonus for multiple pattern matches
                }
                scores.insert(intent.clone(), score);
            }
        }

        // Special handling for complex queries
        self.apply_heuristics(&input_lower, &mut scores);

        // Find the best match
        let (best_intent, best_score) = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(intent, score)| (intent.clone(), *score))
            .unwrap_or((Intent::Unknown, 0.0));

        // Create alternative intents (sorted by score)
        let mut alternatives: Vec<(Intent, f32)> = scores
            .into_iter()
            .filter(|(intent, _)| intent != &best_intent)
            .collect();
        alternatives.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        alternatives.truncate(3); // Keep top 3 alternatives

        // Determine confidence and reasoning
        let (confidence, reasoning) = if best_score >= 1.5 {
            (0.9, format!("Strong pattern match for {:?}", best_intent))
        } else if best_score >= 1.0 {
            (0.7, format!("Good pattern match for {:?}", best_intent))
        } else if best_score >= 0.5 {
            (0.5, format!("Weak pattern match for {:?}", best_intent))
        } else {
            (0.2, "No clear pattern match found".to_string())
        };

        Ok(IntentConfidence {
            intent: best_intent,
            confidence,
            reasoning,
            alternative_intents: alternatives,
        })
    }

    fn apply_heuristics(&self, input: &str, scores: &mut HashMap<Intent, f32>) {
        // Document operation context boosting
        let document_context = input.contains("document")
            || input.contains("file")
            || input.contains("guide")
            || input.contains("version")
            || input.contains("content")
            || input.contains("text");

        // Document comparison heuristics
        if document_context
            && (input.contains("compare") || input.contains("diff") || input.contains("versus"))
        {
            *scores.entry(Intent::CompareDocuments).or_insert(0.0) += 1.2;
        }

        // Document summarization heuristics
        if document_context
            && (input.contains("summary")
                || input.contains("summarize")
                || input.contains("key points")
                || input.contains("overview"))
        {
            *scores.entry(Intent::SummarizeDocument).or_insert(0.0) += 1.2;
        }

        // Document analysis heuristics
        if document_context
            && (input.contains("analyze")
                || input.contains("analysis")
                || input.contains("review")
                || input.contains("examine"))
        {
            *scores.entry(Intent::AnalyzeDocument).or_insert(0.0) += 1.0;
        }

        // Information extraction heuristics
        if input.contains("extract")
            || input.contains("get information")
            || input.contains("find details")
            || input.contains("what information")
        {
            *scores.entry(Intent::ExtractInformation).or_insert(0.0) += 1.0;
        }

        // Search operation context boosting
        let search_context = input.contains("find")
            || input.contains("search")
            || input.contains("look for")
            || input.contains("locate");

        if search_context {
            if input.contains("similar") || input.contains("related") || input.contains("like") {
                *scores.entry(Intent::FindSimilarContent).or_insert(0.0) += 1.0;
            } else if input.contains("filter")
                || input.contains("narrow")
                || input.contains("refine")
            {
                *scores.entry(Intent::FilterDocuments).or_insert(0.0) += 1.0;
            } else if input.contains("discover")
                || input.contains("explore")
                || input.contains("what's available")
            {
                *scores.entry(Intent::DiscoverContent).or_insert(0.0) += 1.0;
            } else if document_context {
                *scores.entry(Intent::FindDocuments).or_insert(0.0) += 0.8;
            } else {
                *scores.entry(Intent::SearchDocuments).or_insert(0.0) += 0.8;
            }
        }

        // Generation operation context boosting
        let generation_context = input.contains("generate")
            || input.contains("create")
            || input.contains("produce")
            || input.contains("make");

        if generation_context {
            if input.contains("new") || input.contains("document") {
                *scores.entry(Intent::CreateDocument).or_insert(0.0) += 1.0;
            } else if input.contains("adapt")
                || input.contains("modify for")
                || input.contains("tailor")
            {
                *scores.entry(Intent::AdaptContent).or_insert(0.0) += 1.0;
            } else if input.contains("convert")
                || input.contains("transform")
                || input.contains("format")
            {
                *scores.entry(Intent::TransformFormat).or_insert(0.0) += 1.0;
            } else if input.contains("word") || input.contains("pdf") || input.contains("export") {
                *scores.entry(Intent::GenerateOutput).or_insert(0.0) += 1.0;
            }
        }

        // Workspace operation context boosting
        let workspace_context = input.contains("workspace")
            || input.contains("project")
            || input.contains("files")
            || input.contains("documents");

        if workspace_context {
            if input.contains("organize")
                || input.contains("structure")
                || input.contains("arrange")
            {
                *scores.entry(Intent::OrganizeWorkspace).or_insert(0.0) += 1.0;
            } else if input.contains("review")
                || input.contains("overview")
                || input.contains("status")
            {
                *scores.entry(Intent::ReviewWorkspace).or_insert(0.0) += 1.0;
            } else if input.contains("export")
                || input.contains("backup")
                || input.contains("download")
            {
                *scores.entry(Intent::ExportDocuments).or_insert(0.0) += 1.0;
            }
        }

        // Style operation context boosting
        if input.contains("style") || input.contains("tone") || input.contains("writing") {
            if input.contains("analyze") || input.contains("analysis") || input.contains("check") {
                *scores.entry(Intent::AnalyzeStyle).or_insert(0.0) += 1.0;
            } else if input.contains("adapt")
                || input.contains("change")
                || input.contains("adjust")
            {
                *scores.entry(Intent::AdaptStyle).or_insert(0.0) += 1.0;
            } else if input.contains("apply") || input.contains("format") {
                *scores.entry(Intent::ApplyStyle).or_insert(0.0) += 1.0;
            }
        }

        // Question patterns analysis (more nuanced help detection)
        if input.starts_with("how") || input.starts_with("what") || input.starts_with("can") {
            if input.contains("do I") || input.contains("can I") || input.contains("should I") {
                *scores.entry(Intent::GetHelp).or_insert(0.0) += 0.8;
            } else if input.contains("does this work") || input.contains("is the process") {
                *scores.entry(Intent::ExplainProcess).or_insert(0.0) += 0.8;
            }
            // Reduce boost for action-oriented questions
            if input.contains("compare") || input.contains("generate") || input.contains("create") {
                if let Some(score) = scores.get_mut(&Intent::GetHelp) {
                    *score *= 0.5;
                }
            }
        }

        // Multi-item patterns
        if input.contains("all") && (input.contains("document") || input.contains("file")) {
            if input.contains("export") || input.contains("download") {
                *scores.entry(Intent::ExportDocuments).or_insert(0.0) += 0.8;
            } else if input.contains("organize") || input.contains("structure") {
                *scores.entry(Intent::OrganizeWorkspace).or_insert(0.0) += 0.8;
            } else {
                *scores.entry(Intent::DiscoverContent).or_insert(0.0) += 0.6;
            }
        }

        // Conversational context patterns
        if input.contains("unclear")
            || input.contains("confused")
            || input.contains("what do you mean")
        {
            *scores.entry(Intent::ClarifyQuestion).or_insert(0.0) += 1.0;
        }

        if input.contains("example") || input.contains("show me") || input.contains("demonstrate") {
            *scores.entry(Intent::ProvideExample).or_insert(0.0) += 0.8;
        }

        // System operation patterns
        if input.contains("status") || input.contains("health") || input.contains("working") {
            *scores.entry(Intent::CheckStatus).or_insert(0.0) += 1.0;
        }

        if input.contains("models") || input.contains("ai models") {
            *scores.entry(Intent::ListModels).or_insert(0.0) += 1.0;
        }

        // Boost confidence for compound commands
        let command_count = scores.len();
        if command_count > 3 {
            // Multiple intents detected - boost the highest scoring ones
            let mut sorted_scores: Vec<_> = scores.iter_mut().collect();
            sorted_scores
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Boost top scoring intent
            if let Some((_, top_score)) = sorted_scores.first_mut() {
                **top_score *= 1.3;
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_supported_intents(&self) -> Vec<Intent> {
        self.patterns.keys().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn get_patterns_for_intent(&self, intent: &Intent) -> Option<&Vec<String>> {
        self.patterns.get(intent)
    }
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_intent_classification() {
        let classifier = IntentClassifier::new();

        // Test document operations
        let result = classifier
            .classify("Compare the old user guide with the new version")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::CompareDocuments);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Summarize this document for me")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::SummarizeDocument);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Analyze the content structure of this document")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::AnalyzeDocument);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Extract key information from this file")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::ExtractInformation);
        assert!(result.confidence > 0.5);

        // Test search operations
        let result = classifier
            .classify("Search for authentication information")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::SearchDocuments);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Find documents about user management")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::FindDocuments);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Filter documents to show only PDF files")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::FilterDocuments);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Find similar content to this document")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::FindSimilarContent);
        assert!(result.confidence > 0.5);

        // Test generation operations
        let result = classifier
            .classify("Create a new document from this template")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::CreateDocument);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Generate a Word document from this content")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::GenerateOutput);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Transform this content to PDF format")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::TransformFormat);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Adapt this content for a technical audience")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::AdaptContent);
        assert!(result.confidence > 0.5);

        // Test workspace operations
        let result = classifier
            .classify("Organize my workspace files")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::OrganizeWorkspace);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Export all documents to a backup")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::ExportDocuments);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Review workspace status and health")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::ReviewWorkspace);
        assert!(result.confidence > 0.5);

        // Test style operations
        let result = classifier
            .classify("Analyze the writing style of this document")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::AnalyzeStyle);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Adapt the style to match the corporate tone")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::AdaptStyle);
        assert!(result.confidence > 0.5);

        // Test system operations
        let result = classifier
            .classify("How do I update a document?")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::GetHelp);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Check system status and health")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::CheckStatus);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("List all available AI models")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::ListModels);
        assert!(result.confidence > 0.5);

        // Test conversation management
        let result = classifier
            .classify("I'm confused, what do you mean by that?")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::ClarifyQuestion);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Can you show me an example of document comparison?")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::ProvideExample);
        assert!(result.confidence > 0.5);

        let result = classifier
            .classify("Explain the process of document analysis step by step")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::ExplainProcess);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_intent_serialization() {
        let intent = Intent::CompareDocuments;
        let json = serde_json::to_string(&intent).unwrap();
        let deserialized: Intent = serde_json::from_str(&json).unwrap();
        assert_eq!(intent, deserialized);
    }

    #[test]
    fn test_supported_intents() {
        let classifier = IntentClassifier::new();
        let intents = classifier.get_supported_intents();
        assert!(!intents.is_empty());
        assert!(intents.contains(&Intent::CompareDocuments));
    }
}
