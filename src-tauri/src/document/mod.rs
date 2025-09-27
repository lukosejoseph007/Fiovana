// src-tauri/src/document/mod.rs
// Document processing and analysis module

pub mod batch_processor;
pub mod chunker;
pub mod content_adapter;
pub mod content_classifier;
pub mod content_hasher;
pub mod deduplication;
pub mod document_comparison;
pub mod document_generator;
pub mod docx_parser;
pub mod file_processor;
pub mod format_converters;
pub mod import_errors;
pub mod indexer;
pub mod metadata_extractor;
pub mod output_generator;
pub mod pdf_parser;
pub mod progress_persistence;
pub mod progress_tracker;
pub mod relationship_analyzer;
pub mod structure_analyzer;
pub mod style_analyzer;
pub mod style_learner;
pub mod templates;

pub use batch_processor::*;
#[allow(unused_imports)]
pub use content_adapter::{
    AdaptationConfig, AdaptationPurpose, AdaptationResult, AudienceType, ComplexityLevel,
    ContentAdapter, ToneAdjustment,
};
#[allow(unused_imports)]
pub use content_classifier::{
    ContentCategory, ContentClassification, ContentClassifier, DocumentContentAnalysis,
};
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
pub use format_converters::{
    ConversionOptions, ConversionResult, DocumentFormat, FormatConverter, FormatInfo,
    QualitySettings,
};
#[allow(unused_imports)]
pub use indexer::{
    DocumentIndexEntry, DocumentIndexer, IndexDocumentSection, IndexStats, SearchFilter,
    SearchResult,
};
pub use metadata_extractor::*;
#[allow(unused_imports)]
pub use output_generator::{
    GenerationSummary, OutputGenerationConfig, OutputGenerationResult, OutputGenerator,
    SourceContent, SourceContentType,
};
pub use pdf_parser::*;
pub use progress_persistence::*;
pub use progress_tracker::*;
#[allow(unused_imports)]
pub use relationship_analyzer::{
    DocumentRelationship, EvidenceType, RelationshipAnalysisMetadata, RelationshipAnalysisResult,
    RelationshipAnalyzer, RelationshipConfig, RelationshipEvidence, RelationshipStats,
    RelationshipStrength, RelationshipType,
};
#[allow(unused_imports)]
pub use structure_analyzer::{
    AnalyzedSection, ContentPattern, ContentPatternMatch, DocumentFlow, DocumentOrganization,
    HeadingType, SectionType, StructuralStatistics,
};
pub use structure_analyzer::{DocumentStructureAnalysis, HeadingNode, StructureAnalyzer};
#[allow(unused_imports)]
pub use style_analyzer::{
    FormattingProfile, SentencePatterns, StructuralPatterns, StyleAnalyzer, StyleProfile,
    StyleSimilarity, ToneAnalysis, ToneType, VocabularyComplexity, VocabularyProfile,
};
#[allow(unused_imports)]
pub use style_learner::{
    OrganizationalStyle, StyleLearner, StyleLearnerError, StyleLearningResult, StylePattern,
    TermFrequency,
};
#[allow(unused_imports)]
pub use templates::{
    AudienceLevel, OutputFormat as TemplateOutputFormat, OutputTemplate, TemplateDefinition,
    TemplateManager, TemplateMetadata, TemplateSection, TemplateStatistics, TemplateVariable,
};

#[allow(dead_code)]
pub fn init() {
    println!("document module loaded");
}
