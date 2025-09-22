// src-tauri/src/ai/intent.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intent {
    // Core document operations
    CompareDocuments,
    UpdateContent,
    GenerateOutput,
    ReviewChanges,

    // Search and discovery
    SearchDocuments,
    FindSimilarContent,
    ExtractConcepts,
    AnalyzeRelationships,

    // Style and formatting
    AdaptStyle,
    TranslateFormat,
    OptimizeContent,

    // Workspace management
    ConfigureWorkspace,
    ManageFiles,
    ManageTemplates,

    // System operations
    CheckStatus,
    GetHelp,
    ListModels,

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

        // Document comparison patterns
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
            ],
        );

        // Content update patterns
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
            ],
        );

        // Output generation patterns
        patterns.insert(
            Intent::GenerateOutput,
            vec![
                "generate".to_string(),
                "create".to_string(),
                "produce".to_string(),
                "make".to_string(),
                "build".to_string(),
                "export".to_string(),
                "output".to_string(),
                "save as".to_string(),
            ],
        );

        // Search patterns
        patterns.insert(
            Intent::SearchDocuments,
            vec![
                "search".to_string(),
                "find".to_string(),
                "look for".to_string(),
                "locate".to_string(),
                "where is".to_string(),
                "show me".to_string(),
                "get".to_string(),
            ],
        );

        // Similar content patterns
        patterns.insert(
            Intent::FindSimilarContent,
            vec![
                "similar".to_string(),
                "like this".to_string(),
                "related".to_string(),
                "find similar".to_string(),
                "content like".to_string(),
                "same topic".to_string(),
            ],
        );

        // Style adaptation patterns
        patterns.insert(
            Intent::AdaptStyle,
            vec![
                "style".to_string(),
                "tone".to_string(),
                "format".to_string(),
                "adapt".to_string(),
                "rewrite".to_string(),
                "adjust tone".to_string(),
                "change style".to_string(),
            ],
        );

        // Help patterns
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
            ],
        );

        // Status patterns
        patterns.insert(
            Intent::CheckStatus,
            vec![
                "status".to_string(),
                "health".to_string(),
                "available".to_string(),
                "working".to_string(),
                "connection".to_string(),
            ],
        );

        // Model management patterns
        patterns.insert(
            Intent::ListModels,
            vec![
                "models".to_string(),
                "list models".to_string(),
                "available models".to_string(),
                "what models".to_string(),
            ],
        );

        // Workspace patterns
        patterns.insert(
            Intent::ConfigureWorkspace,
            vec![
                "workspace".to_string(),
                "project".to_string(),
                "configure".to_string(),
                "setup".to_string(),
                "settings".to_string(),
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
        // Document comparison heuristics
        if (input.contains("document")
            || input.contains("file")
            || input.contains("guide")
            || input.contains("version"))
            && (input.contains("compare") || input.contains("diff"))
        {
            *scores.entry(Intent::CompareDocuments).or_insert(0.0) += 1.0;
        }

        // Output generation heuristics
        if input.contains("generate")
            && (input.contains("word") || input.contains("pdf") || input.contains("document"))
        {
            *scores.entry(Intent::GenerateOutput).or_insert(0.0) += 1.0;
        }

        // Search heuristics
        if (input.contains("find") || input.contains("search"))
            && (input.contains("section") || input.contains("content"))
        {
            *scores.entry(Intent::SearchDocuments).or_insert(0.0) += 0.5;
        }

        // Question patterns (likely help) - but only if not containing action words
        if (input.starts_with("how") || input.starts_with("what") || input.starts_with("can"))
            && !input.contains("compare")
            && !input.contains("generate")
            && !input.contains("create")
        {
            *scores.entry(Intent::GetHelp).or_insert(0.0) += 0.5;
        }

        // Multi-file patterns
        if input.contains("all") && (input.contains("document") || input.contains("file")) {
            *scores.entry(Intent::SearchDocuments).or_insert(0.0) += 0.5;
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

        // Test document comparison
        let result = classifier
            .classify("Compare the old user guide with the new version")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::CompareDocuments);
        assert!(result.confidence > 0.5);

        // Test content generation
        let result = classifier
            .classify("Generate a Word document from this content")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::GenerateOutput);

        // Test help request
        let result = classifier
            .classify("How do I update a document?")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::GetHelp);

        // Test search
        let result = classifier
            .classify("Find all sections about authentication")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::SearchDocuments);
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
