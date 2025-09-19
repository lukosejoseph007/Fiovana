// src-tauri/src/document/mod.rs
// Document processing and analysis module

pub mod file_processor;
pub mod metadata_extractor;
pub mod content_hasher;
pub mod progress_tracker;
pub mod import_errors;
pub mod progress_persistence;

pub use file_processor::*;
pub use metadata_extractor::*;
pub use content_hasher::*;
pub use progress_tracker::*;
pub use import_errors::*;
pub use progress_persistence::*;

#[allow(dead_code)]
pub fn init() {
    println!("document module loaded");
}
