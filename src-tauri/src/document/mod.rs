// src-tauri/src/document/mod.rs
// Document processing and analysis module

pub mod batch_processor;
pub mod chunker;
pub mod content_hasher;
pub mod deduplication;
pub mod document_comparison;
pub mod document_generator;
pub mod docx_parser;
pub mod file_processor;
pub mod import_errors;
pub mod indexer;
pub mod metadata_extractor;
pub mod pdf_parser;
pub mod progress_persistence;
pub mod progress_tracker;
pub mod structure_analyzer;

pub use batch_processor::*;
pub use content_hasher::*;
// Note: deduplication module is available but not auto-imported to avoid unused warnings
pub use document_comparison::*;
pub use document_generator::{
    convert_parsed_content_to_document, DocumentContent, DocumentGenerator, GenerationOptions,
    OutputFormat,
};
pub use docx_parser::*;
#[allow(unused_imports)]
pub use file_processor::{
    CorruptionCheckResult, DocumentContent as ProcessorDocumentContent,
    DocumentHeading as ProcessorDocumentHeading, DocumentImage, DocumentList,
    DocumentSection as ProcessorDocumentSection, DocumentStructure as ProcessorDocumentStructure,
    DocumentTable, DocumentType, FileProcessor, FileValidationResult, ImageType, ListType,
    MagicNumbers, ProcessedDocumentResult, ProcessingStatus, ValidationStatus,
};
pub use import_errors::*;
// Document indexer - now enabled for document intelligence
#[allow(unused_imports)]
pub use indexer::{
    DocumentIndexEntry, DocumentIndexer, IndexDocumentSection, IndexStats, SearchFilter,
    SearchResult,
};
pub use metadata_extractor::*;
pub use pdf_parser::*;
pub use progress_persistence::*;
pub use progress_tracker::*;
#[allow(unused_imports)]
pub use structure_analyzer::{
    AnalyzedSection, ContentPattern, ContentPatternMatch, DocumentFlow, DocumentOrganization,
    HeadingType, SectionType, StructuralStatistics,
};
pub use structure_analyzer::{DocumentStructureAnalysis, HeadingNode, StructureAnalyzer};

#[allow(dead_code)]
pub fn init() {
    println!("document module loaded");
}
