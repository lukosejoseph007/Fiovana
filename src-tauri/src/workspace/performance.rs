// src-tauri/src/workspace/performance.rs
//! Performance optimizations for workspace operations

use super::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Performance metrics for workspace operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorkspacePerformanceMetrics {
    pub operation_type: String,
    pub duration: Duration,
    pub memory_usage: Option<u64>,
    pub file_count: Option<usize>,
    pub directory_count: Option<usize>,
    pub total_size: Option<u64>,
    pub timestamp: Instant,
}

/// Cached workspace metadata to avoid repeated file system operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CachedWorkspaceMetadata {
    pub info: WorkspaceInfo,
    pub stats: Option<WorkspaceStats>,
    pub cached_at: Instant,
    pub ttl: Duration,
}

#[allow(dead_code)]
impl CachedWorkspaceMetadata {
    pub fn new(info: WorkspaceInfo, ttl: Duration) -> Self {
        Self {
            info,
            stats: None,
            cached_at: Instant::now(),
            ttl,
        }
    }

    pub fn with_stats(mut self, stats: WorkspaceStats) -> Self {
        self.stats = Some(stats);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Performance-optimized workspace cache
#[allow(dead_code)]
pub struct WorkspaceCache {
    metadata_cache: RwLock<HashMap<PathBuf, CachedWorkspaceMetadata>>,
    recent_workspaces_cache: RwLock<Option<(Vec<RecentWorkspace>, Instant)>>,
    stats_cache: RwLock<HashMap<PathBuf, (WorkspaceStats, Instant)>>,
    performance_metrics: RwLock<Vec<WorkspacePerformanceMetrics>>,
    #[allow(dead_code)]
    default_ttl: Duration,
    stats_ttl: Duration,
    recent_ttl: Duration,
    max_cache_size: usize,
}

#[allow(dead_code)]
impl WorkspaceCache {
    pub fn new() -> Self {
        Self {
            metadata_cache: RwLock::new(HashMap::new()),
            recent_workspaces_cache: RwLock::new(None),
            stats_cache: RwLock::new(HashMap::new()),
            performance_metrics: RwLock::new(Vec::new()),
            default_ttl: Duration::from_secs(300), // 5 minutes
            stats_ttl: Duration::from_secs(60),    // 1 minute
            recent_ttl: Duration::from_secs(30),   // 30 seconds
            max_cache_size: 100,
        }
    }

    /// Get cached workspace metadata if valid
    pub fn get_metadata(&self, path: &Path) -> Option<CachedWorkspaceMetadata> {
        let cache = self.metadata_cache.read().ok()?;
        let cached = cache.get(path)?;

        if cached.is_valid() {
            Some(cached.clone())
        } else {
            None
        }
    }

    /// Cache workspace metadata
    pub fn cache_metadata(&self, path: PathBuf, metadata: CachedWorkspaceMetadata) {
        if let Ok(mut cache) = self.metadata_cache.write() {
            // Implement LRU eviction if cache is full
            if cache.len() >= self.max_cache_size {
                // Remove oldest entries
                let mut entries: Vec<_> = cache
                    .iter()
                    .map(|(k, v)| (k.clone(), v.cached_at))
                    .collect();
                entries.sort_by_key(|(_, timestamp)| *timestamp);

                // Remove oldest 20% of entries
                let remove_count = self.max_cache_size / 5;
                for (path, _) in entries.iter().take(remove_count) {
                    cache.remove(path);
                }
            }

            cache.insert(path, metadata);
        }
    }

    /// Get cached workspace stats
    pub fn get_stats(&self, path: &Path) -> Option<WorkspaceStats> {
        let cache = self.stats_cache.read().ok()?;
        if let Some((stats, timestamp)) = cache.get(path) {
            if timestamp.elapsed() < self.stats_ttl {
                return Some(stats.clone());
            }
        }
        None
    }

    /// Cache workspace stats
    pub fn cache_stats(&self, path: PathBuf, stats: WorkspaceStats) {
        if let Ok(mut cache) = self.stats_cache.write() {
            cache.insert(path, (stats, Instant::now()));

            // Clean up expired entries
            cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.stats_ttl * 2);
        }
    }

    /// Get cached recent workspaces
    pub fn get_recent_workspaces(&self) -> Option<Vec<RecentWorkspace>> {
        let cache = self.recent_workspaces_cache.read().ok()?;
        if let Some((workspaces, timestamp)) = cache.as_ref() {
            if timestamp.elapsed() < self.recent_ttl {
                return Some(workspaces.clone());
            }
        }
        None
    }

    /// Cache recent workspaces
    pub fn cache_recent_workspaces(&self, workspaces: Vec<RecentWorkspace>) {
        if let Ok(mut cache) = self.recent_workspaces_cache.write() {
            *cache = Some((workspaces, Instant::now()));
        }
    }

    /// Invalidate cache for a specific workspace
    pub fn invalidate_workspace(&self, path: &Path) {
        if let Ok(mut metadata_cache) = self.metadata_cache.write() {
            metadata_cache.remove(path);
        }

        if let Ok(mut stats_cache) = self.stats_cache.write() {
            stats_cache.remove(path);
        }

        // Invalidate recent workspaces cache since it might contain this workspace
        if let Ok(mut recent_cache) = self.recent_workspaces_cache.write() {
            *recent_cache = None;
        }
    }

    /// Clear all cached data
    pub fn clear(&self) {
        if let Ok(mut metadata_cache) = self.metadata_cache.write() {
            metadata_cache.clear();
        }

        if let Ok(mut stats_cache) = self.stats_cache.write() {
            stats_cache.clear();
        }

        if let Ok(mut recent_cache) = self.recent_workspaces_cache.write() {
            *recent_cache = None;
        }
    }

    /// Record performance metric
    pub fn record_metric(&self, metric: WorkspacePerformanceMetrics) {
        if let Ok(mut metrics) = self.performance_metrics.write() {
            metrics.push(metric);

            // Keep only last 1000 metrics
            if metrics.len() > 1000 {
                let excess = metrics.len() - 1000;
                metrics.drain(..excess);
            }
        }
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> Vec<WorkspacePerformanceMetrics> {
        self.performance_metrics
            .read()
            .map(|metrics| metrics.clone())
            .unwrap_or_default()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let metadata_size = self
            .metadata_cache
            .read()
            .map(|cache| cache.len())
            .unwrap_or(0);

        let stats_size = self
            .stats_cache
            .read()
            .map(|cache| cache.len())
            .unwrap_or(0);

        let has_recent = self
            .recent_workspaces_cache
            .read()
            .map(|cache| cache.is_some())
            .unwrap_or(false);

        let metrics_count = self
            .performance_metrics
            .read()
            .map(|metrics| metrics.len())
            .unwrap_or(0);

        CacheStats {
            metadata_entries: metadata_size,
            stats_entries: stats_size,
            has_recent_cache: has_recent,
            metrics_count,
            max_cache_size: self.max_cache_size,
        }
    }
}

impl Default for WorkspaceCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheStats {
    pub metadata_entries: usize,
    pub stats_entries: usize,
    pub has_recent_cache: bool,
    pub metrics_count: usize,
    pub max_cache_size: usize,
}

/// Resource limiter for workspace operations
#[allow(dead_code)]
pub struct WorkspaceResourceLimiter {
    // Semaphore to limit concurrent file system operations
    fs_operations: Arc<Semaphore>,
    // Semaphore to limit concurrent metadata loading operations
    metadata_operations: Arc<Semaphore>,
    // Semaphore to limit concurrent stats calculations
    stats_operations: Arc<Semaphore>,
    // Memory usage tracking
    max_memory_mb: usize,
}

#[allow(dead_code)]
impl WorkspaceResourceLimiter {
    pub fn new() -> Self {
        Self {
            fs_operations: Arc::new(Semaphore::new(10)), // Allow 10 concurrent FS ops
            metadata_operations: Arc::new(Semaphore::new(5)), // Allow 5 concurrent metadata ops
            stats_operations: Arc::new(Semaphore::new(3)), // Allow 3 concurrent stats ops
            max_memory_mb: 500,                          // 500MB memory limit
        }
    }

    pub fn with_limits(
        fs_ops: usize,
        metadata_ops: usize,
        stats_ops: usize,
        memory_mb: usize,
    ) -> Self {
        Self {
            fs_operations: Arc::new(Semaphore::new(fs_ops)),
            metadata_operations: Arc::new(Semaphore::new(metadata_ops)),
            stats_operations: Arc::new(Semaphore::new(stats_ops)),
            max_memory_mb: memory_mb,
        }
    }

    /// Acquire permit for file system operation
    pub async fn acquire_fs_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.fs_operations
            .acquire()
            .await
            .expect("Semaphore should not be closed")
    }

    /// Acquire permit for metadata operation
    pub async fn acquire_metadata_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.metadata_operations
            .acquire()
            .await
            .expect("Semaphore should not be closed")
    }

    /// Acquire permit for stats operation
    pub async fn acquire_stats_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.stats_operations
            .acquire()
            .await
            .expect("Semaphore should not be closed")
    }

    /// Check if operation should be throttled based on memory usage
    pub fn should_throttle(&self) -> bool {
        // Simple memory check - in a real implementation you could use system monitoring
        // For now, just return false
        false
    }

    /// Get resource usage statistics
    pub fn get_resource_stats(&self) -> ResourceStats {
        ResourceStats {
            available_fs_permits: self.fs_operations.available_permits(),
            available_metadata_permits: self.metadata_operations.available_permits(),
            available_stats_permits: self.stats_operations.available_permits(),
            max_memory_mb: self.max_memory_mb,
        }
    }
}

impl Default for WorkspaceResourceLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource usage statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResourceStats {
    pub available_fs_permits: usize,
    pub available_metadata_permits: usize,
    pub available_stats_permits: usize,
    pub max_memory_mb: usize,
}

/// Performance-optimized workspace manager wrapper
#[allow(dead_code)]
#[derive(Clone)]
pub struct OptimizedWorkspaceManager {
    inner: Arc<WorkspaceManager>,
    cache: Arc<WorkspaceCache>,
    resource_limiter: Arc<WorkspaceResourceLimiter>,
}

#[allow(dead_code)]
impl OptimizedWorkspaceManager {
    pub fn new(manager: WorkspaceManager) -> Self {
        Self {
            inner: Arc::new(manager),
            cache: Arc::new(WorkspaceCache::new()),
            resource_limiter: Arc::new(WorkspaceResourceLimiter::new()),
        }
    }

    /// Load workspace with caching
    pub async fn load_workspace_cached(&self, path: &Path) -> WorkspaceResult<WorkspaceInfo> {
        let start_time = Instant::now();

        // Check cache first
        if let Some(cached) = self.cache.get_metadata(path) {
            self.cache.record_metric(WorkspacePerformanceMetrics {
                operation_type: "load_workspace_cached_hit".to_string(),
                duration: start_time.elapsed(),
                memory_usage: None,
                file_count: None,
                directory_count: None,
                total_size: None,
                timestamp: start_time,
            });

            return Ok(cached.info);
        }

        // Acquire resource permit
        let _permit = self.resource_limiter.acquire_metadata_permit().await;

        // Load from disk
        let info = self.inner.load_workspace(path).await?;

        // Cache the result
        let cached_metadata = CachedWorkspaceMetadata::new(info.clone(), Duration::from_secs(300));
        self.cache
            .cache_metadata(path.to_path_buf(), cached_metadata);

        let duration = start_time.elapsed();
        self.cache.record_metric(WorkspacePerformanceMetrics {
            operation_type: "load_workspace_cached_miss".to_string(),
            duration,
            memory_usage: None,
            file_count: None,
            directory_count: None,
            total_size: None,
            timestamp: start_time,
        });

        Ok(info)
    }

    /// Get workspace stats with caching
    pub async fn get_workspace_stats_cached(&self, path: &Path) -> WorkspaceResult<WorkspaceStats> {
        let start_time = Instant::now();

        // Check cache first
        if let Some(cached_stats) = self.cache.get_stats(path) {
            self.cache.record_metric(WorkspacePerformanceMetrics {
                operation_type: "get_stats_cached_hit".to_string(),
                duration: start_time.elapsed(),
                memory_usage: None,
                file_count: Some(cached_stats.total_files as usize),
                directory_count: None,
                total_size: Some(cached_stats.total_size),
                timestamp: start_time,
            });

            return Ok(cached_stats);
        }

        // Acquire resource permit
        let _permit = self.resource_limiter.acquire_stats_permit().await;

        // Calculate stats
        let stats = self.inner.get_workspace_stats(path).await?;

        // Cache the result
        self.cache.cache_stats(path.to_path_buf(), stats.clone());

        let duration = start_time.elapsed();
        self.cache.record_metric(WorkspacePerformanceMetrics {
            operation_type: "get_stats_cached_miss".to_string(),
            duration,
            memory_usage: None,
            file_count: Some(stats.total_files as usize),
            directory_count: None,
            total_size: Some(stats.total_size),
            timestamp: start_time,
        });

        Ok(stats)
    }

    /// Get recent workspaces with caching
    pub async fn get_recent_workspaces_cached(&self) -> WorkspaceResult<Vec<RecentWorkspace>> {
        let start_time = Instant::now();

        // Check cache first
        if let Some(cached_recent) = self.cache.get_recent_workspaces() {
            self.cache.record_metric(WorkspacePerformanceMetrics {
                operation_type: "get_recent_cached_hit".to_string(),
                duration: start_time.elapsed(),
                memory_usage: None,
                file_count: None,
                directory_count: None,
                total_size: None,
                timestamp: start_time,
            });

            return Ok(cached_recent);
        }

        // Load from disk
        let recent = self.inner.get_recent_workspaces().await?;

        // Cache the result
        self.cache.cache_recent_workspaces(recent.clone());

        let duration = start_time.elapsed();
        self.cache.record_metric(WorkspacePerformanceMetrics {
            operation_type: "get_recent_cached_miss".to_string(),
            duration,
            memory_usage: None,
            file_count: None,
            directory_count: None,
            total_size: None,
            timestamp: start_time,
        });

        Ok(recent)
    }

    /// Batch workspace validation with resource limits
    pub async fn validate_workspaces_batch(
        &self,
        paths: Vec<PathBuf>,
    ) -> Vec<(PathBuf, WorkspaceResult<WorkspaceValidation>)> {
        let mut results = Vec::new();
        let batch_size = 5; // Process in batches of 5

        for chunk in paths.chunks(batch_size) {
            let mut batch_futures = Vec::new();

            for path in chunk {
                let inner = self.inner.clone();
                let path_clone = path.clone();
                let permit_future = self.resource_limiter.acquire_fs_permit();

                let future = async move {
                    let _permit = permit_future.await;
                    let result = inner.validate_workspace(&path_clone).await;
                    (path_clone, result)
                };

                batch_futures.push(future);
            }

            // Wait for batch to complete
            let batch_results = futures::future::join_all(batch_futures).await;
            results.extend(batch_results);
        }

        results
    }

    /// Clear cache for a workspace (call when workspace is modified)
    pub fn invalidate_workspace_cache(&self, path: &Path) {
        self.cache.invalidate_workspace(path);
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> Vec<WorkspacePerformanceMetrics> {
        self.cache.get_metrics()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_cache_stats()
    }

    /// Get resource statistics
    pub fn get_resource_stats(&self) -> ResourceStats {
        self.resource_limiter.get_resource_stats()
    }

    /// Delegate other operations to the inner manager
    pub async fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> WorkspaceResult<WorkspaceInfo> {
        let result = self.inner.create_workspace(request).await;

        // Invalidate caches since we created a new workspace
        if let Ok(ref info) = result {
            self.cache.invalidate_workspace(&info.path);
        }

        result
    }

    pub async fn is_workspace(&self, path: &Path) -> WorkspaceResult<bool> {
        self.inner.is_workspace(path).await
    }

    pub async fn validate_workspace(&self, path: &Path) -> WorkspaceResult<WorkspaceValidation> {
        let _permit = self.resource_limiter.acquire_fs_permit().await;
        self.inner.validate_workspace(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_workspace_cache() {
        let cache = WorkspaceCache::new();

        // Test metadata caching
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Should return None initially
        assert!(cache.get_metadata(&path).is_none());

        // Cache some metadata
        let info = WorkspaceInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            path: path.clone(),
            created: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            import_settings: ImportSettings::default(),
            ai_settings: WorkspaceAISettings::default(),
            is_favorite: false,
            access_count: 1,
        };

        let cached_metadata = CachedWorkspaceMetadata::new(info.clone(), Duration::from_secs(10));
        cache.cache_metadata(path.clone(), cached_metadata);

        // Should return cached metadata
        let retrieved = cache.get_metadata(&path);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().info.name, "test");
    }

    #[test]
    fn test_resource_limiter() {
        let limiter = WorkspaceResourceLimiter::new();
        let stats = limiter.get_resource_stats();

        // Should have some available permits
        assert!(stats.available_fs_permits > 0);
        assert!(stats.available_metadata_permits > 0);
        assert!(stats.available_stats_permits > 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = WorkspaceCache::new();
        let stats = cache.get_cache_stats();

        // Initially should be empty
        assert_eq!(stats.metadata_entries, 0);
        assert_eq!(stats.stats_entries, 0);
        assert!(!stats.has_recent_cache);
        assert_eq!(stats.metrics_count, 0);
    }
}
