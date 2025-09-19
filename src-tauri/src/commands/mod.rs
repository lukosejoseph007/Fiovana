// src/commands/mod.rs
// Module for organizing Tauri commands

pub mod health_commands;
pub mod main_commands;
pub mod workspace_commands;

// Re-export all commands for easy access
pub use health_commands::*;
pub use main_commands::*;
pub use workspace_commands::*;
