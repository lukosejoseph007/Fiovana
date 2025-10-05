use fiovana::filesystem::security::safe_mode::{SafeModeLevel, SafeModeManager};
use std::fs;
use tempfile::tempdir;

fn main() -> anyhow::Result<()> {
    let manager = SafeModeManager::new();

    println!("Initial level: {:?}", manager.get_config().level);

    // Test setting levels
    manager.set_level(SafeModeLevel::Restricted)?;
    println!("After setting Restricted: {:?}", manager.get_config().level);

    // Create test files
    let temp_dir = tempdir()?;
    let text_file = temp_dir.path().join("text.txt");
    let script_file = temp_dir.path().join("script.exe");

    fs::write(&text_file, "test content")?;
    fs::write(&script_file, "binary content")?;

    println!("Text file path: {:?}", text_file);
    println!("Script file path: {:?}", script_file);

    // Test file type restrictions
    let text_result = manager.is_file_allowed(&text_file);
    let script_result = manager.is_file_allowed(&script_file);

    println!("Text file allowed: {:?}", text_result);
    println!("Script file allowed: {:?}", script_result);

    Ok(())
}
