// src/commands/mod.rs
// Module for organizing Tauri commands

pub mod deduplication_commands;
pub mod health_commands;
pub mod main_commands;
pub mod progress_commands;
pub mod workspace_commands;
pub mod workspace_performance_commands;

// Re-export all commands for easy access
// Note: deduplication_commands are available but not auto-imported to avoid unused warnings
pub use health_commands::*;
pub use main_commands::*;
pub use progress_commands::*;
pub use workspace_commands::*;
