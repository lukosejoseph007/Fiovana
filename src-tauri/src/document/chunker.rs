// src-tauri/src/document/chunker.rs
//! Document chunking strategy for semantic understanding

use crate::document::DocumentIndexEntry;
use serde::{Deserialize, Serialize};
use std::cmp;

/// Configuration for document chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    /// Target chunk size in characters
    pub chunk_size: usize,
    /// Overlap size between chunks in characters
    pub overlap_size: usize,
    /// Minimum chunk size (chunks smaller than this are merged)
    pub min_chunk_size: usize,
    /// Maximum chunk size (hard limit)
    pub max_chunk_size: usize,
    /// Whether to respect paragraph boundaries
    pub respect_paragraphs: bool,
    /// Whether to respect sentence boundaries
    pub respect_sentences: bool,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,     // Target 1000 characters per chunk
            overlap_size: 200,    // 200 character overlap
            min_chunk_size: 100,  // Minimum 100 characters
            max_chunk_size: 2000, // Maximum 2000 characters
            respect_paragraphs: true,
            respect_sentences: true,
        }
    }
}

/// A document chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// Unique chunk ID
    pub id: String,
    /// Original document path
    pub document_path: String,
    /// Chunk index in the document
    pub chunk_index: usize,
    /// Chunk content
    pub content: String,
    /// Character start position in original document
    pub start_position: usize,
    /// Character end position in original document
    pub end_position: usize,
    /// Section this chunk belongs to (if any)
    pub section_title: Option<String>,
    /// Section level (if part of a section)
    pub section_level: Option<u32>,
    /// Keywords extracted from this chunk
    pub keywords: Vec<String>,
    /// Chunk metadata
    pub metadata: ChunkMetadata,
}

/// Metadata for a document chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// Word count in chunk
    pub word_count: usize,
    /// Character count in chunk
    pub char_count: usize,
    /// Whether chunk contains code
    pub has_code: bool,
    /// Whether chunk contains lists
    pub has_lists: bool,
    /// Whether chunk contains tables
    pub has_tables: bool,
    /// Estimated reading time in seconds
    pub reading_time_seconds: u32,
}

/// Document chunker for semantic understanding
pub struct DocumentChunker {
    config: ChunkConfig,
}

impl DocumentChunker {
    /// Create a new document chunker with default configuration
    pub fn new() -> Self {
        Self {
            config: ChunkConfig::default(),
        }
    }

    /// Create a new document chunker with custom configuration
    pub fn with_config(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// Chunk a document into semantic chunks
    pub fn chunk_document(&self, doc: &DocumentIndexEntry) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let content = &doc.content;

        // If document is small enough, return as single chunk
        if content.len() <= self.config.chunk_size {
            let chunk = self.create_chunk(doc, 0, content.clone(), 0, content.len(), None, None);
            return vec![chunk];
        }

        // Try section-based chunking first if sections are available
        if !doc.structure.sections.is_empty() {
            chunks = self.chunk_by_sections(doc);
        }

        // If section-based chunking didn't work well, fall back to content-based chunking
        if chunks.is_empty() || self.needs_further_chunking(&chunks) {
            chunks = self.chunk_by_content(doc);
        }

        // Post-process chunks to ensure quality
        self.post_process_chunks(chunks)
    }

    /// Chunk document by sections
    fn chunk_by_sections(&self, doc: &DocumentIndexEntry) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        for section in &doc.structure.sections {
            let section_content = &section.content;
            if section_content.len() <= self.config.chunk_size {
                // Section fits in one chunk
                let chunk = self.create_chunk(
                    doc,
                    chunk_index,
                    section_content.clone(),
                    section.position,
                    section.position + section_content.len(),
                    Some(section.title.clone()),
                    Some(section.level),
                );
                chunks.push(chunk);
                chunk_index += 1;
            } else {
                // Section needs to be split into multiple chunks
                let section_chunks = self.chunk_text_content(
                    doc,
                    section_content,
                    &mut chunk_index,
                    Some(section.title.clone()),
                    Some(section.level),
                );
                chunks.extend(section_chunks);
            }
        }

        chunks
    }

    /// Chunk document by content (fallback method)
    fn chunk_by_content(&self, doc: &DocumentIndexEntry) -> Vec<DocumentChunk> {
        let mut chunk_index = 0;
        self.chunk_text_content(doc, &doc.content, &mut chunk_index, None, None)
    }

    /// Chunk text content into smaller pieces
    fn chunk_text_content(
        &self,
        doc: &DocumentIndexEntry,
        content: &str,
        chunk_index: &mut usize,
        section_title: Option<String>,
        section_level: Option<u32>,
    ) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < content.len() {
            let end = self.find_chunk_end(content, start);
            let chunk_content = content[start..end].to_string();

            let chunk = self.create_chunk(
                doc,
                *chunk_index,
                chunk_content,
                start,
                end,
                section_title.clone(),
                section_level,
            );

            chunks.push(chunk);
            *chunk_index += 1;

            // Move start position with overlap
            start = if end >= content.len() {
                content.len()
            } else {
                cmp::max(start + 1, end.saturating_sub(self.config.overlap_size))
            };
        }

        chunks
    }

    /// Find the best end position for a chunk
    fn find_chunk_end(&self, content: &str, start: usize) -> usize {
        let target_end = cmp::min(start + self.config.chunk_size, content.len());
        let max_end = cmp::min(start + self.config.max_chunk_size, content.len());

        if target_end >= content.len() {
            return content.len();
        }

        // Try to find a good break point
        let mut best_end = target_end;

        // Look for paragraph breaks first
        if self.config.respect_paragraphs {
            if let Some(para_end) = self.find_paragraph_break(content, start, target_end, max_end) {
                best_end = para_end;
            }
        }

        // Look for sentence breaks if no good paragraph break found
        if self.config.respect_sentences && best_end == target_end {
            if let Some(sent_end) = self.find_sentence_break(content, start, target_end, max_end) {
                best_end = sent_end;
            }
        }

        // Ensure we don't go below minimum chunk size
        if best_end - start < self.config.min_chunk_size && best_end < content.len() {
            best_end = cmp::min(start + self.config.min_chunk_size, content.len());
        }

        best_end
    }

    /// Find a good paragraph break
    fn find_paragraph_break(
        &self,
        content: &str,
        start: usize,
        target: usize,
        max_end: usize,
    ) -> Option<usize> {
        // Look for double newlines (paragraph breaks)
        let search_start = cmp::max(start, target.saturating_sub(200));
        let search_end = cmp::min(max_end, target + 200);

        let search_content = &content[search_start..search_end];

        // Find paragraph breaks in the search area
        for (i, _) in search_content.match_indices("\n\n") {
            let pos = search_start + i + 2;
            if pos >= target.saturating_sub(100) && pos <= target + 200 {
                return Some(pos);
            }
        }

        None
    }

    /// Find a good sentence break
    fn find_sentence_break(
        &self,
        content: &str,
        start: usize,
        target: usize,
        max_end: usize,
    ) -> Option<usize> {
        let search_start = cmp::max(start, target.saturating_sub(100));
        let search_end = cmp::min(max_end, target + 100);

        let search_content = &content[search_start..search_end];

        // Look for sentence endings
        let sentence_endings = [". ", "! ", "? ", ".\n", "!\n", "?\n"];

        for ending in &sentence_endings {
            for (i, _) in search_content.match_indices(ending) {
                let pos = search_start + i + ending.len();
                if pos >= target.saturating_sub(50) && pos <= target + 50 {
                    return Some(pos);
                }
            }
        }

        None
    }

    /// Create a document chunk
    #[allow(clippy::too_many_arguments)]
    fn create_chunk(
        &self,
        doc: &DocumentIndexEntry,
        chunk_index: usize,
        content: String,
        start_pos: usize,
        end_pos: usize,
        section_title: Option<String>,
        section_level: Option<u32>,
    ) -> DocumentChunk {
        let chunk_id = format!("{}#{}", doc.path.display(), chunk_index);
        let metadata = self.analyze_chunk_metadata(&content);
        let keywords = self.extract_chunk_keywords(&content);

        DocumentChunk {
            id: chunk_id,
            document_path: doc.path.to_string_lossy().to_string(),
            chunk_index,
            content,
            start_position: start_pos,
            end_position: end_pos,
            section_title,
            section_level,
            keywords,
            metadata,
        }
    }

    /// Analyze chunk metadata
    fn analyze_chunk_metadata(&self, content: &str) -> ChunkMetadata {
        let word_count = content.split_whitespace().count();
        let char_count = content.chars().count();

        // Simple heuristics for content detection
        let has_code =
            content.contains("```") || content.contains("function ") || content.contains("def ");
        let has_lists = content.contains("- ") || content.contains("* ") || content.contains("1. ");
        let has_tables = content.contains("|") && content.matches("|").count() > 3;

        // Estimate reading time (average 200 words per minute)
        let reading_time_seconds = ((word_count as f32 / 200.0) * 60.0) as u32;

        ChunkMetadata {
            word_count,
            char_count,
            has_code,
            has_lists,
            has_tables,
            reading_time_seconds,
        }
    }

    /// Extract keywords from chunk content
    fn extract_chunk_keywords(&self, content: &str) -> Vec<String> {
        // Simple keyword extraction - can be enhanced later
        let words: Vec<&str> = content
            .split_whitespace()
            .filter(|word| word.len() > 3 && !self.is_stop_word(word))
            .collect();

        // Get unique words, limited to top keywords
        let mut keywords: Vec<String> = words
            .into_iter()
            .map(|word| {
                word.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect();

        keywords.sort();
        keywords.dedup();
        keywords.truncate(10); // Limit to 10 keywords per chunk

        keywords
    }

    /// Check if a word is a stop word
    fn is_stop_word(&self, word: &str) -> bool {
        let stop_words = [
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
            "one", "our", "has", "have", "this", "will", "from", "they", "know", "want", "been",
            "good", "much", "some", "time", "very", "when", "come", "here", "just", "like", "long",
            "make", "many", "over", "such", "take", "than", "them", "well", "were",
        ];
        stop_words.contains(&word.to_lowercase().as_str())
    }

    /// Check if chunks need further chunking
    fn needs_further_chunking(&self, chunks: &[DocumentChunk]) -> bool {
        chunks
            .iter()
            .any(|chunk| chunk.content.len() > self.config.max_chunk_size)
    }

    /// Post-process chunks to ensure quality
    fn post_process_chunks(&self, mut chunks: Vec<DocumentChunk>) -> Vec<DocumentChunk> {
        // Merge very small chunks with adjacent chunks
        let mut i = 0;
        while i < chunks.len() {
            if chunks[i].content.len() < self.config.min_chunk_size && chunks.len() > 1 {
                if i + 1 < chunks.len() {
                    // Merge with next chunk
                    let next_content = chunks.remove(i + 1).content;
                    chunks[i].content.push(' ');
                    chunks[i].content.push_str(&next_content);
                    chunks[i].end_position = chunks[i].start_position + chunks[i].content.len();
                } else if i > 0 {
                    // Merge with previous chunk
                    let current = chunks.remove(i);
                    chunks[i - 1].content.push(' ');
                    chunks[i - 1].content.push_str(&current.content);
                    chunks[i - 1].end_position =
                        chunks[i - 1].start_position + chunks[i - 1].content.len();
                    i -= 1;
                }
            }
            i += 1;
        }

        // Update chunk indices after merging
        for (index, chunk) in chunks.iter_mut().enumerate() {
            chunk.chunk_index = index;
            chunk.id = format!("{}#{}", chunk.document_path, index);
        }

        chunks
    }
}

impl Default for DocumentChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{
        indexer::DocumentStructure, indexer::IndexDocumentSection, EnhancedMetadata,
    };
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn create_test_document(
        content: String,
        sections: Vec<IndexDocumentSection>,
    ) -> DocumentIndexEntry {
        use crate::document::{
            BasicMetadata, ContentMetadata, ContentStats, SecurityMetadata, TechnicalMetadata,
        };

        DocumentIndexEntry {
            id: "test_doc_1".to_string(),
            path: PathBuf::from("test.md"),
            title: "Test Document".to_string(),
            summary: None,
            content,
            metadata: EnhancedMetadata {
                basic: BasicMetadata {
                    file_name: "test.md".to_string(),
                    file_extension: Some("md".to_string()),
                    file_size: 1024,
                    created: Some(SystemTime::now()),
                    modified: Some(SystemTime::now()),
                    accessed: Some(SystemTime::now()),
                    is_file: true,
                    is_dir: false,
                    is_symlink: false,
                },
                content: ContentMetadata {
                    detected_mime_type: Some("text/markdown".to_string()),
                    encoding: Some("utf-8".to_string()),
                    line_endings: Some("unix".to_string()),
                    preview: Some("Test content...".to_string()),
                    language: Some("en".to_string()),
                    stats: ContentStats {
                        char_count: Some(500),
                        word_count: Some(100),
                        line_count: Some(10),
                        paragraph_count: Some(2),
                        binary_ratio: 0.0,
                    },
                },
                security: SecurityMetadata {
                    permissions: None,
                    is_executable: false,
                    is_hidden: false,
                    has_extended_attributes: false,
                    security_flags: vec![],
                },
                technical: TechnicalMetadata {
                    entropy: 0.5,
                    compression_ratio: Some(0.8),
                    checksums: {
                        let mut map = std::collections::HashMap::new();
                        map.insert("md5".to_string(), "test_checksum".to_string());
                        map.insert("sha256".to_string(), "test_sha256".to_string());
                        map
                    },
                    structure: crate::document::FileStructure {
                        has_structure: true,
                        format_version: Some("1.0".to_string()),
                        embedded_resources: 0,
                        sections: vec![],
                    },
                },
                document: None,
            },
            structure: DocumentStructure {
                document_type: crate::document::indexer::DocumentType::Manual,
                sections,
                toc: None,
                page_count: None,
                has_images: false,
                has_tables: false,
                has_code: false,
            },
            keywords: vec![],
            content_hash: "test_hash".to_string(),
            indexed_at: SystemTime::now(),
            index_version: 1,
        }
    }

    #[test]
    fn test_small_document_single_chunk() {
        let chunker = DocumentChunker::new();
        let doc = create_test_document("Short document content.".to_string(), vec![]);

        let chunks = chunker.chunk_document(&doc);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Short document content.");
        assert_eq!(chunks[0].chunk_index, 0);
    }

    #[test]
    fn test_large_document_multiple_chunks() {
        let chunker = DocumentChunker::with_config(ChunkConfig {
            chunk_size: 50,
            overlap_size: 10,
            min_chunk_size: 20,
            max_chunk_size: 100,
            respect_paragraphs: false,
            respect_sentences: true,
        });

        let long_content = "This is a long document. ".repeat(10);
        let doc = create_test_document(long_content, vec![]);

        let chunks = chunker.chunk_document(&doc);

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.content.len() <= 100));
        assert!(chunks.iter().all(|chunk| chunk.content.len() >= 20));
    }

    #[test]
    fn test_chunk_metadata() {
        let chunker = DocumentChunker::new();
        let content = "This is a test document with code:\n```rust\nfn test() {}\n```\nAnd a list:\n- Item 1\n- Item 2";
        let metadata = chunker.analyze_chunk_metadata(content);

        assert!(metadata.word_count > 0);
        assert!(metadata.char_count > 0);
        assert!(metadata.has_code);
        assert!(metadata.has_lists);
        assert!(!metadata.has_tables);
    }

    #[test]
    fn test_keyword_extraction() {
        let chunker = DocumentChunker::new();
        let content = "This document discusses machine learning algorithms and neural networks for artificial intelligence applications.";
        let keywords = chunker.extract_chunk_keywords(content);

        assert!(keywords.contains(&"machine".to_string()));
        assert!(keywords.contains(&"learning".to_string()));
        assert!(keywords.contains(&"algorithms".to_string()));
        assert!(!keywords.contains(&"this".to_string())); // stop word
    }
}
