// src-tauri/src/document/indexer.rs
// Document indexing system for fast document retrieval

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::document::{EnhancedMetadata, MetadataExtractor};

/// Document index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndexEntry {
    /// Unique document identifier
    pub id: String,
    /// File path
    pub path: PathBuf,
    /// Document title
    pub title: String,
    /// Content summary
    pub summary: Option<String>,
    /// Full-text content for searching
    pub content: String,
    /// Enhanced metadata
    pub metadata: EnhancedMetadata,
    /// Document structure information
    pub structure: DocumentStructure,
    /// Search keywords extracted from content
    pub keywords: Vec<String>,
    /// Content hash for change detection
    pub content_hash: String,
    /// Last indexed timestamp
    pub indexed_at: SystemTime,
    /// Index version for compatibility
    pub index_version: u32,
}

/// Document structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructure {
    /// Detected document type
    pub document_type: DocumentType,
    /// Section hierarchy
    pub sections: Vec<IndexDocumentSection>,
    /// Table of contents if available
    pub toc: Option<Vec<TocEntry>>,
    /// Total page count
    pub page_count: Option<u32>,
    /// Has images
    pub has_images: bool,
    /// Has tables
    pub has_tables: bool,
    /// Has code blocks
    pub has_code: bool,
}

/// Document type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Manual,
    Guide,
    Procedure,
    Reference,
    Training,
    Policy,
    Template,
    Other(String),
}

/// Document section for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentSection {
    /// Section ID
    pub id: String,
    /// Section title
    pub title: String,
    /// Section level (0 = top level)
    pub level: u32,
    /// Section content
    pub content: String,
    /// Section keywords
    pub keywords: Vec<String>,
    /// Character position in document
    pub position: usize,
    /// Section length
    pub length: usize,
}

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Entry title
    pub title: String,
    /// Nesting level
    pub level: u32,
    /// Page number if available
    pub page: Option<u32>,
    /// Position in document
    pub position: Option<usize>,
}

/// Search filter criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SearchFilter {
    /// Document type filter
    pub document_type: Option<DocumentType>,
    /// File extension filter
    pub extensions: Option<Vec<String>>,
    /// Date range filter
    pub modified_after: Option<SystemTime>,
    pub modified_before: Option<SystemTime>,
    /// Size range filter
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    /// Content type filter
    pub has_images: Option<bool>,
    pub has_tables: Option<bool>,
    pub has_code: Option<bool>,
    /// Keyword filter
    pub must_contain_keywords: Option<Vec<String>>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document entry
    pub document: DocumentIndexEntry,
    /// Relevance score (0.0 to 1.0)
    pub score: f64,
    /// Matching sections
    pub matching_sections: Vec<IndexDocumentSection>,
    /// Highlighted snippets
    pub snippets: Vec<String>,
}

/// Document indexer
#[allow(dead_code)]
pub struct DocumentIndexer {
    /// Index storage directory
    index_dir: PathBuf,
    /// In-memory index
    index: HashMap<String, DocumentIndexEntry>,
    /// Keyword to document mapping
    keyword_index: HashMap<String, HashSet<String>>,
    /// Metadata extractor
    #[allow(dead_code)]
    metadata_extractor: MetadataExtractor,
    /// Current index version
    index_version: u32,
}

#[allow(dead_code)]
impl DocumentIndexer {
    /// Create new document indexer
    pub fn new(index_dir: PathBuf) -> Result<Self> {
        // Ensure index directory exists
        fs::create_dir_all(&index_dir).context("Failed to create index directory")?;

        let mut indexer = Self {
            index_dir,
            index: HashMap::new(),
            keyword_index: HashMap::new(),
            metadata_extractor: MetadataExtractor,
            index_version: 1,
        };

        // Load existing index
        indexer.load_index()?;

        Ok(indexer)
    }

    /// Index a document
    pub async fn index_document(&mut self, file_path: &Path) -> Result<DocumentIndexEntry> {
        // Generate document ID from path
        let id = self.generate_document_id(file_path);

        // Extract metadata
        let metadata =
            MetadataExtractor::extract(file_path).context("Failed to extract metadata")?;

        // Read file content
        let content = self
            .read_file_content(file_path)
            .context("Failed to read file content")?;

        // Extract structure
        let structure = self.analyze_document_structure(&content, file_path)?;

        // Generate title from filename or first heading
        let title = self.extract_title(&content, file_path);

        // Extract keywords
        let keywords = self.extract_keywords(&content);

        // Generate content hash
        let content_hash = self.calculate_content_hash(&content);

        // Create index entry
        let entry = DocumentIndexEntry {
            id: id.clone(),
            path: file_path.to_path_buf(),
            title,
            summary: self.generate_summary(&content),
            content,
            metadata,
            structure,
            keywords: keywords.clone(),
            content_hash,
            indexed_at: SystemTime::now(),
            index_version: self.index_version,
        };

        // Update keyword index
        self.update_keyword_index(&id, &keywords);

        // Store in index
        self.index.insert(id.clone(), entry.clone());

        // Persist index
        self.save_index()?;

        Ok(entry)
    }

    /// Search documents by query
    pub fn search(&self, query: &str, filter: Option<SearchFilter>) -> Result<Vec<SearchResult>> {
        let query_terms = self.tokenize_query(query);
        let mut results = Vec::new();

        // Find matching documents
        for entry in self.index.values() {
            // Apply filters
            if let Some(ref filter) = filter {
                if !self.matches_filter(entry, filter) {
                    continue;
                }
            }

            // Calculate relevance score
            let score = self.calculate_relevance_score(entry, &query_terms);

            if score > 0.0 {
                // Find matching sections
                let matching_sections = self.find_matching_sections(entry, &query_terms);

                // Generate snippets
                let snippets = self.generate_snippets(&entry.content, &query_terms);

                results.push(SearchResult {
                    document: entry.clone(),
                    score,
                    matching_sections,
                    snippets,
                });
            }
        }

        // Sort by relevance score
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Get document by ID
    pub fn get_document(&self, id: &str) -> Option<&DocumentIndexEntry> {
        self.index.get(id)
    }

    /// Remove document from index
    pub fn remove_document(&mut self, id: &str) -> Result<bool> {
        if let Some(entry) = self.index.remove(id) {
            // Remove from keyword index
            for keyword in &entry.keywords {
                if let Some(doc_set) = self.keyword_index.get_mut(keyword) {
                    doc_set.remove(id);
                    if doc_set.is_empty() {
                        self.keyword_index.remove(keyword);
                    }
                }
            }

            // Persist changes
            self.save_index()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all indexed documents
    pub fn get_all_documents(&self) -> Vec<&DocumentIndexEntry> {
        self.index.values().collect()
    }

    /// Get documents by file type
    pub fn get_documents_by_type(&self, doc_type: &DocumentType) -> Vec<&DocumentIndexEntry> {
        self.index
            .values()
            .filter(|entry| {
                std::mem::discriminant(&entry.structure.document_type)
                    == std::mem::discriminant(doc_type)
            })
            .collect()
    }

    /// Get index statistics
    pub fn get_stats(&self) -> IndexStats {
        let total_documents = self.index.len();
        let total_keywords = self.keyword_index.len();
        let total_content_size: usize = self.index.values().map(|entry| entry.content.len()).sum();

        IndexStats {
            total_documents,
            total_keywords,
            total_content_size,
            index_version: self.index_version,
        }
    }

    // Private helper methods

    fn generate_document_id(&self, file_path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        format!("doc_{:x}", hasher.finish())
    }

    fn read_file_content(&self, file_path: &Path) -> Result<String> {
        // Handle different file types
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" | "md" | "markdown" => {
                fs::read_to_string(file_path).context("Failed to read text file")
            }
            "docx" => {
                // Use existing DOCX parser
                use crate::document::DocxParser;
                let content = DocxParser::parse(file_path).context("Failed to parse DOCX file")?;
                Ok(content.text)
            }
            "pdf" => {
                // Use existing PDF parser
                use crate::document::PdfParser;
                let content = PdfParser::parse(file_path).context("Failed to parse PDF file")?;
                Ok(content.text)
            }
            _ => {
                // Try to read as text, fallback to empty string
                Ok(fs::read_to_string(file_path).unwrap_or_else(|_| String::new()))
            }
        }
    }

    fn analyze_document_structure(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<DocumentStructure> {
        let sections = self.extract_sections(content);
        let toc = self.extract_toc(content);
        let document_type = self.classify_document_type(content, file_path);

        Ok(DocumentStructure {
            document_type,
            sections,
            toc,
            page_count: None, // TODO: Implement page counting
            has_images: content.contains("![") || content.contains("<img"),
            has_tables: content.contains("|") || content.contains("<table"),
            has_code: content.contains("```") || content.contains("<code"),
        })
    }

    fn extract_sections(&self, content: &str) -> Vec<IndexDocumentSection> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_section: Option<IndexDocumentSection> = None;
        let mut position = 0;

        for line in lines.iter() {
            let line_position = position;
            position += line.len() + 1; // +1 for newline

            // Detect headings (Markdown-style or simple detection)
            if let Some((level, title)) = self.detect_heading(line) {
                // Save previous section
                if let Some(mut section) = current_section.take() {
                    section.length = line_position - section.position;
                    sections.push(section);
                }

                // Start new section
                let section_id = format!("section_{}", sections.len());
                current_section = Some(IndexDocumentSection {
                    id: section_id,
                    title: title.to_string(),
                    level,
                    content: String::new(),
                    keywords: Vec::new(),
                    position: line_position,
                    length: 0,
                });
            } else if let Some(ref mut section) = current_section {
                // Add line to current section
                if !section.content.is_empty() {
                    section.content.push('\n');
                }
                section.content.push_str(line);
            }
        }

        // Save last section
        if let Some(mut section) = current_section {
            section.length = content.len() - section.position;
            section.keywords = self.extract_keywords(&section.content);
            sections.push(section);
        }

        // If no sections found, create one big section
        if sections.is_empty() {
            sections.push(IndexDocumentSection {
                id: "main".to_string(),
                title: "Main Content".to_string(),
                level: 0,
                content: content.to_string(),
                keywords: self.extract_keywords(content),
                position: 0,
                length: content.len(),
            });
        }

        sections
    }

    fn detect_heading<'a>(&self, line: &'a str) -> Option<(u32, &'a str)> {
        let trimmed = line.trim();

        // Markdown-style headings
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count() as u32;
            let title = trimmed.trim_start_matches('#').trim();
            if !title.is_empty() {
                return Some((level - 1, title)); // 0-based level
            }
        }

        // Simple heuristic: lines that are short, title-case, and followed by empty line
        if trimmed.len() < 100
            && trimmed.chars().next().is_some_and(|c| c.is_uppercase())
            && !trimmed.ends_with('.')
        {
            return Some((0, trimmed));
        }

        None
    }

    fn extract_toc(&self, content: &str) -> Option<Vec<TocEntry>> {
        // Simple TOC extraction - look for numbered lists that might be TOCs
        let lines: Vec<&str> = content.lines().collect();
        let mut toc_entries = Vec::new();
        let mut in_toc = false;

        for line in lines {
            let trimmed = line.trim();

            // Simple heuristic: look for numbered or bulleted lists
            if trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.chars().next().is_some_and(|c| c.is_numeric())
            {
                let level = (line.len() - line.trim_start().len()) / 2; // Indent level
                let title = trimmed
                    .trim_start_matches(|c: char| {
                        c.is_numeric() || c == '.' || c == '-' || c == '*'
                    })
                    .trim()
                    .to_string();

                if !title.is_empty() {
                    toc_entries.push(TocEntry {
                        title,
                        level: level as u32,
                        page: None,
                        position: None,
                    });
                    in_toc = true;
                }
            } else if in_toc && !trimmed.is_empty() {
                // Break TOC if we hit non-list content
                break;
            }
        }

        if toc_entries.is_empty() {
            None
        } else {
            Some(toc_entries)
        }
    }

    fn classify_document_type(&self, content: &str, file_path: &Path) -> DocumentType {
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();

        let content_lower = content.to_lowercase();

        // Simple classification based on filename and content
        if filename.contains("manual") || content_lower.contains("user manual") {
            DocumentType::Manual
        } else if filename.contains("guide") || content_lower.contains("guide") {
            DocumentType::Guide
        } else if filename.contains("procedure") || content_lower.contains("procedure") {
            DocumentType::Procedure
        } else if filename.contains("reference") || content_lower.contains("reference") {
            DocumentType::Reference
        } else if filename.contains("training") || content_lower.contains("training") {
            DocumentType::Training
        } else if filename.contains("policy") || content_lower.contains("policy") {
            DocumentType::Policy
        } else if filename.contains("template") || content_lower.contains("template") {
            DocumentType::Template
        } else {
            DocumentType::Other(filename)
        }
    }

    fn extract_title(&self, content: &str, file_path: &Path) -> String {
        // Try to extract title from first heading
        for line in content.lines().take(10) {
            if let Some((_, title)) = self.detect_heading(line) {
                return title.to_string();
            }
        }

        // Fallback to filename
        file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Untitled Document")
            .to_string()
    }

    fn extract_keywords(&self, content: &str) -> Vec<String> {
        let mut keywords = HashSet::new();

        // Simple keyword extraction
        let content_lower = content.to_lowercase();
        let words: Vec<&str> = content_lower
            .split_whitespace()
            .filter(|word| word.len() > 3) // Filter short words
            .filter(|word| !self.is_stop_word(word))
            .collect();

        // Count word frequency
        let mut word_counts = HashMap::new();
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }

        // Take most frequent words
        let mut sorted_words: Vec<_> = word_counts.into_iter().collect();
        sorted_words.sort_by(|a, b| b.1.cmp(&a.1));

        for (word, _count) in sorted_words.into_iter().take(20) {
            keywords.insert(word.to_string());
        }

        keywords.into_iter().collect()
    }

    fn is_stop_word(&self, word: &str) -> bool {
        // Simple stop word list
        const STOP_WORDS: &[&str] = &[
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
            "one", "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new",
            "now", "old", "see", "two", "who", "boy", "did", "did", "she", "use", "her", "now",
            "air", "day", "men", "get", "old", "see", "him", "two", "how", "its", "who", "oil",
            "sit", "set", "but", "end", "why", "let", "great", "same", "big", "group", "take",
            "seem", "work", "three", "small", "part", "year", "turn", "want", "show", "every",
            "good", "much", "where", "come", "back", "little", "only", "round", "man", "work",
            "take", "way", "come", "could", "show", "also", "after", "back", "other", "many",
            "than", "then", "them", "these", "some", "what", "make", "like", "into", "time",
            "very", "when", "much", "know", "take", "people", "into", "just", "first", "well",
            "water", "been", "call", "who", "its", "now", "find", "long", "down", "day", "did",
            "get", "come", "made", "may", "part",
        ];

        STOP_WORDS.contains(&word)
    }

    fn calculate_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn generate_summary(&self, content: &str) -> Option<String> {
        // Extract first paragraph as summary
        let paragraphs: Vec<&str> = content.split("\n\n").collect();

        for paragraph in paragraphs {
            let trimmed = paragraph.trim();
            if trimmed.len() > 50 && trimmed.len() < 500 {
                return Some(trimmed.to_string());
            }
        }

        // Fallback: first 200 characters
        if content.len() > 200 {
            Some(format!("{}...", &content[..200]))
        } else if !content.is_empty() {
            Some(content.to_string())
        } else {
            None
        }
    }

    fn update_keyword_index(&mut self, doc_id: &str, keywords: &[String]) {
        for keyword in keywords {
            self.keyword_index
                .entry(keyword.clone())
                .or_default()
                .insert(doc_id.to_string());
        }
    }

    fn tokenize_query(&self, query: &str) -> Vec<String> {
        query
            .to_lowercase()
            .split_whitespace()
            .map(|term| term.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|term| !term.is_empty() && !self.is_stop_word(term))
            .map(|term| term.to_string())
            .collect()
    }

    fn matches_filter(&self, entry: &DocumentIndexEntry, filter: &SearchFilter) -> bool {
        // Document type filter
        if let Some(ref doc_type) = filter.document_type {
            if std::mem::discriminant(&entry.structure.document_type)
                != std::mem::discriminant(doc_type)
            {
                return false;
            }
        }

        // Extension filter
        if let Some(ref extensions) = filter.extensions {
            if let Some(ext) = entry.path.extension().and_then(|e| e.to_str()) {
                if !extensions.contains(&ext.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Date filters
        if let Some(modified_after) = filter.modified_after {
            if let Some(modified) = entry.metadata.basic.modified {
                if modified < modified_after {
                    return false;
                }
            }
        }

        if let Some(modified_before) = filter.modified_before {
            if let Some(modified) = entry.metadata.basic.modified {
                if modified > modified_before {
                    return false;
                }
            }
        }

        // Size filters
        if let Some(min_size) = filter.min_size {
            if entry.metadata.basic.file_size < min_size {
                return false;
            }
        }

        if let Some(max_size) = filter.max_size {
            if entry.metadata.basic.file_size > max_size {
                return false;
            }
        }

        // Content filters
        if let Some(has_images) = filter.has_images {
            if entry.structure.has_images != has_images {
                return false;
            }
        }

        if let Some(has_tables) = filter.has_tables {
            if entry.structure.has_tables != has_tables {
                return false;
            }
        }

        if let Some(has_code) = filter.has_code {
            if entry.structure.has_code != has_code {
                return false;
            }
        }

        // Keyword filter
        if let Some(ref required_keywords) = filter.must_contain_keywords {
            for required in required_keywords {
                if !entry
                    .keywords
                    .iter()
                    .any(|k| k.contains(&required.to_lowercase()))
                {
                    return false;
                }
            }
        }

        true
    }

    fn calculate_relevance_score(&self, entry: &DocumentIndexEntry, query_terms: &[String]) -> f64 {
        if query_terms.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;
        let total_terms = query_terms.len() as f64;

        for term in query_terms {
            let mut term_score = 0.0;

            // Title match (highest weight)
            if entry.title.to_lowercase().contains(term) {
                term_score += 3.0;
            }

            // Keyword match (high weight)
            if entry.keywords.iter().any(|k| k.contains(term)) {
                term_score += 2.0;
            }

            // Content match (base weight)
            let content_matches = entry.content.to_lowercase().matches(term).count() as f64;
            term_score += content_matches * 0.1;

            // Section title match (medium weight)
            for section in &entry.structure.sections {
                if section.title.to_lowercase().contains(term) {
                    term_score += 1.0;
                }
            }

            score += term_score / total_terms;
        }

        // Normalize score to 0-1 range
        (score / 10.0).min(1.0)
    }

    fn find_matching_sections(
        &self,
        entry: &DocumentIndexEntry,
        query_terms: &[String],
    ) -> Vec<IndexDocumentSection> {
        let mut matching_sections = Vec::new();

        for section in &entry.structure.sections {
            let section_text = format!("{} {}", section.title, section.content).to_lowercase();

            let mut matches = 0;
            for term in query_terms {
                if section_text.contains(term) {
                    matches += 1;
                }
            }

            if matches > 0 {
                matching_sections.push(section.clone());
            }
        }

        matching_sections
    }

    fn generate_snippets(&self, content: &str, query_terms: &[String]) -> Vec<String> {
        let mut snippets = Vec::new();
        let content_lower = content.to_lowercase();

        for term in query_terms {
            if let Some(pos) = content_lower.find(term) {
                let start = pos.saturating_sub(50);
                let end = (pos + term.len() + 50).min(content.len());

                let snippet = &content[start..end];
                let highlighted = snippet.replace(term, &format!("**{}**", term));

                snippets.push(format!("...{}...", highlighted));
            }
        }

        snippets.into_iter().take(3).collect() // Limit to 3 snippets
    }

    fn load_index(&mut self) -> Result<()> {
        let index_file = self.index_dir.join("index.json");

        if index_file.exists() {
            let data = fs::read_to_string(&index_file).context("Failed to read index file")?;

            let entries: Vec<DocumentIndexEntry> =
                serde_json::from_str(&data).context("Failed to parse index file")?;

            // Rebuild index
            self.index.clear();
            self.keyword_index.clear();

            for entry in entries {
                self.update_keyword_index(&entry.id, &entry.keywords);
                self.index.insert(entry.id.clone(), entry);
            }
        }

        Ok(())
    }

    pub fn save_index(&self) -> Result<()> {
        let index_file = self.index_dir.join("index.json");
        let entries: Vec<&DocumentIndexEntry> = self.index.values().collect();

        let data = serde_json::to_string_pretty(&entries).context("Failed to serialize index")?;

        fs::write(&index_file, data).context("Failed to write index file")?;

        Ok(())
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: usize,
    pub total_keywords: usize,
    pub total_content_size: usize,
    pub index_version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_document_indexer() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join("index");

        let indexer = DocumentIndexer::new(index_dir).unwrap();

        // Test with empty index
        let results = indexer.search("test", None).unwrap();
        assert_eq!(results.len(), 0);

        let stats = indexer.get_stats();
        assert_eq!(stats.total_documents, 0);
    }

    #[test]
    fn test_keyword_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join("index");
        let indexer = DocumentIndexer::new(index_dir).unwrap();

        let content = "This is a test document about machine learning and artificial intelligence.";
        let keywords = indexer.extract_keywords(content);

        assert!(keywords.len() > 0);
        assert!(
            keywords.contains(&"machine".to_string()) || keywords.contains(&"learning".to_string())
        );
    }

    #[test]
    fn test_document_classification() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join("index");
        let indexer = DocumentIndexer::new(index_dir).unwrap();

        let path = Path::new("user_manual.txt");
        let content = "This is a user manual for the application.";
        let doc_type = indexer.classify_document_type(content, path);

        match doc_type {
            DocumentType::Manual => {}
            _ => panic!("Expected Manual document type"),
        }
    }
}
