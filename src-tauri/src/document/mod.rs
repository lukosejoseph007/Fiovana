// src-tauri/src/document/mod.rs
// Document processing and analysis module

pub mod batch_processor;
pub mod content_hasher;
pub mod deduplication;
pub mod docx_parser;
pub mod file_processor;
pub mod import_errors;
pub mod metadata_extractor;
pub mod pdf_parser;
pub mod progress_persistence;
pub mod progress_tracker;

pub use batch_processor::*;
pub use content_hasher::*;
// Note: deduplication module is available but not auto-imported to avoid unused warnings
pub use docx_parser::*;
pub use file_processor::*;
pub use import_errors::*;
pub use metadata_extractor::*;
pub use pdf_parser::*;
pub use progress_persistence::*;
pub use progress_tracker::*;

#[allow(dead_code)]
pub fn init() {
    println!("document module loaded");
}
