// src-tauri/src/document/mod.rs
// Document processing and analysis module

pub mod batch_processor;
pub mod content_hasher;
pub mod file_processor;
pub mod import_errors;
pub mod metadata_extractor;
pub mod progress_persistence;
pub mod progress_tracker;

pub use batch_processor::*;
pub use content_hasher::*;
pub use file_processor::*;
pub use import_errors::*;
pub use metadata_extractor::*;
pub use progress_persistence::*;
pub use progress_tracker::*;

#[allow(dead_code)]
pub fn init() {
    println!("document module loaded");
}
