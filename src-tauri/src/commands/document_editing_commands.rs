use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

// Re-use the DocumentIndexerState from document_indexing_commands
use crate::commands::document_indexing_commands::DocumentIndexerState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DocumentFormat {
    Markdown,
    PlainText,
    Html,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentRequest {
    pub document_id: String,
    pub content: String,
    pub format: DocumentFormat,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentResponse {
    pub success: bool,
    pub document_id: String,
    pub path: String,
    pub size: u64,
    pub modified_at: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub version_id: String,
    pub document_id: String,
    pub created_at: String,
    pub size: u64,
    pub hash: String,
    pub description: String,
}

/// Save document content to file system
#[tauri::command]
pub async fn save_document(
    request: SaveDocumentRequest,
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<SaveDocumentResponse, String> {
    tracing::info!(
        "Saving document: {} (format: {:?})",
        request.document_id,
        request.format
    );

    // Get the indexer
    let indexer_guard = indexer_state.lock().await;
    let indexer = indexer_guard
        .as_ref()
        .ok_or_else(|| "Document indexer not initialized".to_string())?;

    // Get document details from indexer to find the file path
    let document = indexer
        .get_document(&request.document_id)
        .ok_or_else(|| format!("Document not found: {}", request.document_id))?;

    let file_path = PathBuf::from(&document.path);

    // Validate file path
    if !file_path.exists() {
        return Err(format!("Document file does not exist: {:?}", file_path));
    }

    // Create backup before saving
    create_backup(&file_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    // Write new content to file
    fs::write(&file_path, &request.content)
        .map_err(|e| format!("Failed to write document: {}", e))?;

    // Get updated file metadata
    let metadata =
        fs::metadata(&file_path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let modified_at = metadata
        .modified()
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Utc> = time.into();
            datetime.to_rfc3339()
        })
        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

    // Re-index the document to update metadata
    drop(indexer_guard); // Release the lock before re-indexing
    let mut indexer_guard = indexer_state.lock().await;
    if let Some(indexer) = indexer_guard.as_mut() {
        indexer
            .index_document(&file_path)
            .await
            .map_err(|e| format!("Failed to re-index document: {}", e))?;
    }

    tracing::info!("Document saved successfully: {}", request.document_id);

    Ok(SaveDocumentResponse {
        success: true,
        document_id: request.document_id,
        path: file_path.to_string_lossy().to_string(),
        size: metadata.len(),
        modified_at,
        message: "Document saved successfully".to_string(),
    })
}

/// Create a version snapshot of the document
#[tauri::command]
pub async fn create_document_version(
    document_id: String,
    content: String,
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<VersionInfo, String> {
    tracing::info!("Creating version for document: {}", document_id);

    // Get the indexer
    let indexer_guard = indexer_state.lock().await;
    let indexer = indexer_guard
        .as_ref()
        .ok_or_else(|| "Document indexer not initialized".to_string())?;

    // Get document details
    let document = indexer
        .get_document(&document_id)
        .ok_or_else(|| format!("Document not found: {}", document_id))?;

    let file_path = PathBuf::from(&document.path);

    // Create versions directory
    let versions_dir = get_versions_directory(&file_path)?;
    fs::create_dir_all(&versions_dir)
        .map_err(|e| format!("Failed to create versions directory: {}", e))?;

    // Generate version ID (timestamp-based)
    let version_id = format!(
        "v_{}_{}",
        chrono::Utc::now().timestamp(),
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    // Calculate content hash
    let hash = calculate_hash(&content);

    // Save version file
    let version_path = versions_dir.join(format!("{}.md", version_id));
    fs::write(&version_path, &content)
        .map_err(|e| format!("Failed to write version file: {}", e))?;

    // Get version metadata
    let metadata = fs::metadata(&version_path)
        .map_err(|e| format!("Failed to get version metadata: {}", e))?;

    let created_at = metadata
        .created()
        .or_else(|_| metadata.modified())
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Utc> = time.into();
            datetime.to_rfc3339()
        })
        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

    tracing::info!(
        "Version created: {} for document: {}",
        version_id,
        document_id
    );

    Ok(VersionInfo {
        version_id: version_id.clone(),
        document_id,
        created_at,
        size: metadata.len(),
        hash,
        description: format!(
            "Version snapshot created at {}",
            chrono::Utc::now().to_rfc3339()
        ),
    })
}

/// Get all versions of a document
#[tauri::command]
pub async fn get_document_versions(
    document_id: String,
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<Vec<VersionInfo>, String> {
    tracing::info!("Getting versions for document: {}", document_id);

    // Get the indexer
    let indexer_guard = indexer_state.lock().await;
    let indexer = indexer_guard
        .as_ref()
        .ok_or_else(|| "Document indexer not initialized".to_string())?;

    // Get document details
    let document = indexer
        .get_document(&document_id)
        .ok_or_else(|| format!("Document not found: {}", document_id))?;

    let file_path = PathBuf::from(&document.path);
    let versions_dir = get_versions_directory(&file_path)?;

    // Check if versions directory exists
    if !versions_dir.exists() {
        return Ok(Vec::new());
    }

    // Read all version files
    let entries = fs::read_dir(&versions_dir)
        .map_err(|e| format!("Failed to read versions directory: {}", e))?;

    let mut versions = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read version file: {}", e))?;

                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Failed to get version metadata: {}", e))?;

                let created_at = metadata
                    .created()
                    .or_else(|_| metadata.modified())
                    .map(|time| {
                        let datetime: chrono::DateTime<chrono::Utc> = time.into();
                        datetime.to_rfc3339()
                    })
                    .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

                versions.push(VersionInfo {
                    version_id: file_stem.to_string(),
                    document_id: document_id.clone(),
                    created_at,
                    size: metadata.len(),
                    hash: calculate_hash(&content),
                    description: "Version snapshot".to_string(),
                });
            }
        }
    }

    // Sort by creation time (newest first)
    versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    tracing::info!(
        "Found {} versions for document: {}",
        versions.len(),
        document_id
    );

    Ok(versions)
}

/// Restore a document to a previous version
#[tauri::command]
pub async fn restore_document_version(
    document_id: String,
    version_id: String,
    indexer_state: State<'_, DocumentIndexerState>,
) -> Result<String, String> {
    tracing::info!(
        "Restoring document: {} to version: {}",
        document_id,
        version_id
    );

    // Get the indexer
    let indexer_guard = indexer_state.lock().await;
    let indexer = indexer_guard
        .as_ref()
        .ok_or_else(|| "Document indexer not initialized".to_string())?;

    // Get document details
    let document = indexer
        .get_document(&document_id)
        .ok_or_else(|| format!("Document not found: {}", document_id))?;

    let file_path = PathBuf::from(&document.path);
    let versions_dir = get_versions_directory(&file_path)?;

    // Read version file
    let version_path = versions_dir.join(format!("{}.md", version_id));

    if !version_path.exists() {
        return Err(format!("Version not found: {}", version_id));
    }

    let version_content = fs::read_to_string(&version_path)
        .map_err(|e| format!("Failed to read version file: {}", e))?;

    // Create backup of current version before restoring
    create_backup(&file_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    // Write version content to main file
    fs::write(&file_path, &version_content)
        .map_err(|e| format!("Failed to restore version: {}", e))?;

    // Re-index the document
    drop(indexer_guard);
    let mut indexer_guard = indexer_state.lock().await;
    if let Some(indexer) = indexer_guard.as_mut() {
        indexer
            .index_document(&file_path)
            .await
            .map_err(|e| format!("Failed to re-index document: {}", e))?;
    }

    tracing::info!(
        "Document restored successfully: {} to version: {}",
        document_id,
        version_id
    );

    Ok(version_content)
}

// Helper functions

/// Create a backup of the file
fn create_backup(file_path: &Path) -> Result<PathBuf, std::io::Error> {
    let backup_dir = file_path
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No parent directory"))?
        .join(".proxemic_backups");

    fs::create_dir_all(&backup_dir)?;

    let file_name = file_path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No file name"))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("{}_{}", timestamp, file_name.to_string_lossy()));

    fs::copy(file_path, &backup_path)?;

    Ok(backup_path)
}

/// Get the versions directory for a document
fn get_versions_directory(file_path: &Path) -> Result<PathBuf, String> {
    let parent = file_path
        .parent()
        .ok_or_else(|| "File has no parent directory".to_string())?;

    let file_stem = file_path
        .file_stem()
        .ok_or_else(|| "File has no name".to_string())?;

    Ok(parent.join(".proxemic_versions").join(file_stem))
}

/// Calculate SHA256 hash of content
fn calculate_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
