use chrono::{DateTime, Utc};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
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
