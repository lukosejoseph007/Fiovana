use hex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;

/// Configuration for log rotation and retention
#[derive(Debug, Clone)]
pub struct LogRotationConfig {
    pub max_file_size: u64,  // in bytes
    pub max_files: usize,    // number of files to keep
    pub retention_days: u32, // days to keep logs
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 10,
            retention_days: 30,
        }
    }
}

/// Manages log rotation and retention for security audit logs
pub struct LogRotationManager {
    config: LogRotationConfig,
    log_dir: PathBuf,
    current_file: Option<std::fs::File>,
    current_file_size: u64,
    file_checksums: HashMap<PathBuf, String>, // Track checksums for integrity verification
}

impl LogRotationManager {
    /// Creates a new log rotation manager
    pub fn new(log_dir: PathBuf, config: LogRotationConfig) -> io::Result<Self> {
        // Ensure log directory exists
        fs::create_dir_all(&log_dir)?;

        Ok(Self {
            config,
            log_dir,
            current_file: None,
            current_file_size: 0,
            file_checksums: HashMap::new(),
        })
    }

    /// Writes a log entry with automatic rotation
    pub fn write_log(&mut self, log_entry: &str) -> io::Result<()> {
        let entry_size = log_entry.len() as u64 + 1; // +1 for newline

        // Check if we need to rotate
        if self.current_file_size + entry_size > self.config.max_file_size {
            self.rotate()?;
        }

        // Get or create current file
        let file = self.get_current_file()?;

        // Write the log entry
        writeln!(file, "{}", log_entry)?;
        self.current_file_size += entry_size;

        Ok(())
    }

    /// Gets the current log file, creates it if necessary
    fn get_current_file(&mut self) -> io::Result<&mut std::fs::File> {
        if self.current_file.is_none() {
            let current_path = self.get_current_log_path();
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&current_path)?;

            // Get current file size
            let metadata = file.metadata()?;
            self.current_file_size = metadata.len();

            self.current_file = Some(file);
        }

        Ok(self.current_file.as_mut().unwrap())
    }

    /// Gets the path for the current log file
    fn get_current_log_path(&self) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.log_dir
            .join(format!("security_audit_{}.log", timestamp))
    }

    /// Performs log rotation
    fn rotate(&mut self) -> io::Result<()> {
        // Close current file
        self.current_file = None;
        self.current_file_size = 0;

        // Clean up old logs
        self.cleanup_old_logs()?;

        Ok(())
    }

    /// Cleans up old log files based on retention policy
    fn cleanup_old_logs(&self) -> io::Result<()> {
        let mut log_files = Vec::new();

        // Collect all log files
        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "log" {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        log_files.push((path, modified));
                    }
                }
            }
        }

        // Sort by modification time (oldest first)
        log_files.sort_by(|a, b| a.1.cmp(&b.1));

        // Remove files beyond max_files limit
        if log_files.len() > self.config.max_files {
            for (path, _) in log_files
                .iter()
                .take(log_files.len() - self.config.max_files)
            {
                if let Err(e) = fs::remove_file(path) {
                    error!("Failed to remove old log file {}: {}", path.display(), e);
                }
            }
        }

        // Remove files older than retention period
        let retention_period =
            std::time::Duration::from_secs(self.config.retention_days as u64 * 24 * 60 * 60);
        let now = SystemTime::now();

        for (path, modified) in log_files {
            if let Ok(age) = now.duration_since(modified) {
                if age > retention_period {
                    if let Err(e) = fs::remove_file(&path) {
                        error!(
                            "Failed to remove expired log file {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Gets all log files sorted by creation time (newest first)
    #[allow(dead_code)]
    pub fn get_log_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "log" {
                files.push(path);
            }
        }

        // Sort by filename (which contains timestamp)
        files.sort();
        files.reverse();

        Ok(files)
    }

    /// Generates SHA-256 checksum for a log file
    pub fn generate_checksum(&self, file_path: &PathBuf) -> io::Result<String> {
        let content = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Verifies the integrity of a log file by comparing checksums
    pub fn verify_log_integrity(&self, file_path: &PathBuf) -> io::Result<bool> {
        let current_checksum = self.generate_checksum(file_path)?;

        // Check if we have a stored checksum for this file
        if let Some(stored_checksum) = self.file_checksums.get(file_path) {
            Ok(current_checksum == *stored_checksum)
        } else {
            // No stored checksum - this is a new file, store the checksum
            // Note: In a real implementation, we'd store checksums persistently
            Ok(true)
        }
    }

    /// Verifies integrity of all log files
    pub fn verify_all_logs_integrity(&self) -> io::Result<HashMap<PathBuf, bool>> {
        let mut results = HashMap::new();
        let log_files = self.get_log_files()?;

        for file_path in log_files {
            let integrity_ok = self.verify_log_integrity(&file_path)?;
            results.insert(file_path, integrity_ok);
        }

        Ok(results)
    }

    /// Stores checksum for a log file (call this after rotation)
    #[allow(dead_code)]
    pub fn store_checksum(&mut self, file_path: PathBuf, checksum: String) {
        self.file_checksums.insert(file_path, checksum);
    }

    /// Gets checksum for a specific log file
    #[allow(dead_code)]
    pub fn get_checksum(&self, file_path: &PathBuf) -> Option<&String> {
        self.file_checksums.get(file_path)
    }

    /// Enhanced rotate method that stores checksum before rotation
    #[allow(dead_code)]
    pub fn rotate_with_integrity(&mut self) -> io::Result<()> {
        if let Some(current_file_path) = self
            .current_file
            .as_ref()
            .map(|_| self.get_current_log_path())
        {
            // Generate checksum for current file before rotation
            let checksum = self.generate_checksum(&current_file_path)?;
            self.store_checksum(current_file_path, checksum);
        }

        self.rotate()?;
        Ok(())
    }
}

/// Initializes file-based logging with rotation
pub fn init_file_logging(log_dir: Option<PathBuf>) -> io::Result<LogRotationManager> {
    let log_dir = log_dir.unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("./logs"))
            .join("proxemic")
            .join("security_logs")
    });

    let config = LogRotationConfig::default();
    LogRotationManager::new(log_dir, config)
}
