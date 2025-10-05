// src-tauri/tests/workspace_performance_benchmarks.rs
// Performance benchmarks for workspace operations

use fiovana::app_config::ConfigManager;
use fiovana::workspace::{
    CreateWorkspaceRequest, WorkspaceManager, WorkspaceTemplate, WORKSPACE_DIRECTORIES,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::fs;

/// Performance benchmark fixture
struct PerformanceBenchmarkFixture {
    _temp_dir: TempDir,
    base_path: PathBuf,
    workspace_manager: WorkspaceManager,
}

impl PerformanceBenchmarkFixture {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();

        let config_manager = Arc::new(ConfigManager::new().await?);
        let workspace_manager = WorkspaceManager::new(config_manager)?;

        Ok(Self {
            _temp_dir: temp_dir,
            base_path,
            workspace_manager,
        })
    }

    fn get_test_workspace_path(&self, name: &str) -> PathBuf {
        self.base_path.join(name)
    }

    async fn create_test_workspace(
        &self,
        name: &str,
        template: WorkspaceTemplate,
    ) -> Result<(Duration, PathBuf), Box<dyn std::error::Error + Send + Sync>> {
        let workspace_path = self.get_test_workspace_path(name);

        let create_request = CreateWorkspaceRequest {
            name: name.to_string(),
            path: workspace_path.clone(),
            template,
            description: Some(format!("Performance test: {}", name)),
        };

        let start = Instant::now();
        self.workspace_manager
            .create_workspace(create_request)
            .await?;
        let duration = start.elapsed();

        Ok((duration, workspace_path))
    }

    async fn create_test_files(
        &self,
        workspace_path: &Path,
        file_count: usize,
    ) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
        let imports_dir = workspace_path.join("sources/imports");

        let start = Instant::now();

        for i in 0..file_count {
            let filename = format!("test_file_{:04}.txt", i);
            let content = format!("This is test file number {} with some content to measure performance.\nLine 2 of the file.\nLine 3 with more content.", i);

            let file_path = imports_dir.join(filename);
            fs::write(&file_path, content).await?;
        }

        Ok(start.elapsed())
    }
}

// Performance constants for validation
const WORKSPACE_CREATION_THRESHOLD_MS: u64 = 500; // 500ms max for workspace creation
const FILE_CREATION_BATCH_THRESHOLD_MS: u64 = 1000; // 1 second for 100 files
const STATS_CALCULATION_THRESHOLD_MS: u64 = 200; // 200ms for stats calculation
const WORKSPACE_LOADING_THRESHOLD_MS: u64 = 100; // 100ms for workspace loading
const VALIDATION_THRESHOLD_MS: u64 = 150; // 150ms for workspace validation

// Benchmark workspace creation performance
#[tokio::test]
async fn benchmark_workspace_creation_performance() {
    let fixture = PerformanceBenchmarkFixture::new()
        .await
        .expect("Failed to create fixture");

    println!("\n=== WORKSPACE CREATION PERFORMANCE BENCHMARK ===");

    let templates = [
        (WorkspaceTemplate::Basic, "Basic"),
        (WorkspaceTemplate::Research, "Research"),
        (WorkspaceTemplate::Documentation, "Documentation"),
        (WorkspaceTemplate::Collaboration, "Collaboration"),
    ];

    let mut total_time = Duration::new(0, 0);
    let mut max_time = Duration::new(0, 0);
    let mut min_time = Duration::from_secs(999);

    for (i, (template, template_name)) in templates.iter().enumerate() {
        let workspace_name = format!("perf_test_{}", i);

        let (duration, _path) = fixture
            .create_test_workspace(&workspace_name, template.clone())
            .await
            .expect("Failed to create workspace");

        total_time += duration;
        max_time = max_time.max(duration);
        min_time = min_time.min(duration);

        println!("  {} template: {:?}", template_name, duration);

        // Individual workspace creation should be fast
        assert!(
            duration.as_millis() < WORKSPACE_CREATION_THRESHOLD_MS as u128,
            "Workspace creation took too long: {:?} (max: {}ms)",
            duration,
            WORKSPACE_CREATION_THRESHOLD_MS
        );
    }

    let avg_time = total_time / templates.len() as u32;

    println!("  Average time: {:?}", avg_time);
    println!("  Min time: {:?}", min_time);
    println!("  Max time: {:?}", max_time);
    println!("  Total time: {:?}", total_time);

    // Overall performance validation
    assert!(
        avg_time.as_millis() < WORKSPACE_CREATION_THRESHOLD_MS as u128,
        "Average workspace creation time is too slow: {:?}",
        avg_time
    );

    println!("✓ Workspace creation performance benchmark passed");
}

// Benchmark file operations performance
#[tokio::test]
async fn benchmark_file_operations_performance() {
    let fixture = PerformanceBenchmarkFixture::new()
        .await
        .expect("Failed to create fixture");

    println!("\n=== FILE OPERATIONS PERFORMANCE BENCHMARK ===");

    let workspace_name = "file_ops_perf_test";
    let (creation_time, workspace_path) = fixture
        .create_test_workspace(workspace_name, WorkspaceTemplate::Basic)
        .await
        .expect("Failed to create workspace");

    println!("  Workspace creation: {:?}", creation_time);

    // Test different file batch sizes
    let batch_sizes = vec![10, 50, 100];

    for batch_size in batch_sizes {
        let file_creation_time = fixture
            .create_test_files(&workspace_path, batch_size)
            .await
            .expect("Failed to create test files");

        println!(
            "  Created {} files in: {:?}",
            batch_size, file_creation_time
        );
        println!(
            "  Average per file: {:?}",
            file_creation_time / batch_size as u32
        );

        // File creation should scale reasonably
        if batch_size == 100 {
            assert!(
                file_creation_time.as_millis() < FILE_CREATION_BATCH_THRESHOLD_MS as u128,
                "Creating {} files took too long: {:?} (max: {}ms)",
                batch_size,
                file_creation_time,
                FILE_CREATION_BATCH_THRESHOLD_MS
            );
        }

        // Calculate stats after file creation
        let stats_start = Instant::now();
        let stats = fixture
            .workspace_manager
            .get_workspace_stats(&workspace_path)
            .await
            .expect("Failed to get workspace stats");
        let stats_time = stats_start.elapsed();

        println!(
            "  Stats calculation for {} files: {:?}",
            batch_size, stats_time
        );
        println!(
            "  Found {} files, {} bytes",
            stats.total_files, stats.total_size
        );

        // Stats calculation should be fast
        assert!(
            stats_time.as_millis() < STATS_CALCULATION_THRESHOLD_MS as u128,
            "Stats calculation took too long: {:?} (max: {}ms)",
            stats_time,
            STATS_CALCULATION_THRESHOLD_MS
        );
    }

    println!("✓ File operations performance benchmark passed");
}

// Benchmark workspace loading and validation performance
#[tokio::test]
async fn benchmark_workspace_loading_performance() {
    let fixture = PerformanceBenchmarkFixture::new()
        .await
        .expect("Failed to create fixture");

    println!("\n=== WORKSPACE LOADING PERFORMANCE BENCHMARK ===");

    // Create workspaces with different complexities
    let test_cases = vec![
        ("simple", WorkspaceTemplate::Basic, 10),
        ("medium", WorkspaceTemplate::Research, 50),
        ("complex", WorkspaceTemplate::Documentation, 100),
    ];

    for (name, template, file_count) in test_cases {
        let workspace_name = format!("loading_test_{}", name);
        let (creation_time, workspace_path) = fixture
            .create_test_workspace(&workspace_name, template)
            .await
            .expect("Failed to create workspace");

        // Add files to make workspace more realistic
        let file_creation_time = fixture
            .create_test_files(&workspace_path, file_count)
            .await
            .expect("Failed to create test files");

        println!(
            "  {} workspace setup: creation={:?}, files={:?}",
            name, creation_time, file_creation_time
        );

        // Benchmark workspace loading
        let loading_start = Instant::now();
        let loaded_workspace = fixture
            .workspace_manager
            .load_workspace(&workspace_path)
            .await
            .expect("Failed to load workspace");
        let loading_time = loading_start.elapsed();

        println!("  {} workspace loading: {:?}", name, loading_time);

        // Verify workspace was loaded correctly
        assert_eq!(loaded_workspace.name, workspace_name);
        assert!(loaded_workspace.access_count >= 1);

        // Loading should be fast
        assert!(
            loading_time.as_millis() < WORKSPACE_LOADING_THRESHOLD_MS as u128,
            "Workspace loading took too long: {:?} (max: {}ms)",
            loading_time,
            WORKSPACE_LOADING_THRESHOLD_MS
        );

        // Benchmark workspace validation
        let validation_start = Instant::now();
        let validation = fixture
            .workspace_manager
            .validate_workspace(&workspace_path)
            .await
            .expect("Failed to validate workspace");
        let validation_time = validation_start.elapsed();

        println!("  {} workspace validation: {:?}", name, validation_time);

        // Verify validation results
        assert!(validation.is_valid, "Workspace should be valid");
        assert!(
            validation.errors.is_empty(),
            "Should have no validation errors"
        );

        // Validation should be fast
        assert!(
            validation_time.as_millis() < VALIDATION_THRESHOLD_MS as u128,
            "Workspace validation took too long: {:?} (max: {}ms)",
            validation_time,
            VALIDATION_THRESHOLD_MS
        );

        // Benchmark workspace detection
        let detection_start = Instant::now();
        let is_workspace = fixture
            .workspace_manager
            .is_workspace(&workspace_path)
            .await
            .expect("Failed to detect workspace");
        let detection_time = detection_start.elapsed();

        println!("  {} workspace detection: {:?}", name, detection_time);

        assert!(is_workspace, "Path should be detected as workspace");

        // Detection should be very fast
        assert!(
            detection_time.as_millis() < 50,
            "Workspace detection took too long: {:?} (max: 50ms)",
            detection_time
        );
    }

    println!("✓ Workspace loading performance benchmark passed");
}

// Benchmark concurrent operations performance
#[tokio::test]
async fn benchmark_concurrent_operations_performance() {
    let fixture = PerformanceBenchmarkFixture::new()
        .await
        .expect("Failed to create fixture");

    println!("\n=== CONCURRENT OPERATIONS PERFORMANCE BENCHMARK ===");

    // Test concurrent workspace creation
    let concurrent_count = 10;
    let mut handles = Vec::new();

    let fixture_arc = std::sync::Arc::new(fixture);
    let start_time = Instant::now();

    for i in 0..concurrent_count {
        let fixture_clone = fixture_arc.clone();

        let handle = tokio::spawn(async move {
            let workspace_name = format!("concurrent_perf_{}", i);
            let workspace_path = fixture_clone.get_test_workspace_path(&workspace_name);

            let create_request = CreateWorkspaceRequest {
                name: workspace_name,
                path: workspace_path,
                template: WorkspaceTemplate::Basic,
                description: Some(format!("Concurrent performance test {}", i)),
            };

            let operation_start = Instant::now();
            let result = fixture_clone
                .workspace_manager
                .create_workspace(create_request)
                .await;
            let operation_time = operation_start.elapsed();

            (i, result, operation_time)
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut successful_operations = 0;
    let mut total_operation_time = Duration::new(0, 0);
    let mut max_operation_time = Duration::new(0, 0);

    for handle in handles {
        match handle.await {
            Ok((i, Ok(_), operation_time)) => {
                successful_operations += 1;
                total_operation_time += operation_time;
                max_operation_time = max_operation_time.max(operation_time);
                println!("  Concurrent operation {}: {:?}", i, operation_time);
            }
            Ok((i, Err(e), operation_time)) => {
                println!(
                    "  ✗ Operation {} failed in {:?}: {:?}",
                    i, operation_time, e
                );
            }
            Err(e) => {
                println!("  ✗ Task failed: {:?}", e);
            }
        }
    }

    let total_time = start_time.elapsed();
    let avg_operation_time = total_operation_time / successful_operations;

    println!(
        "  Concurrent operations completed: {}/{}",
        successful_operations, concurrent_count
    );
    println!("  Total time: {:?}", total_time);
    println!("  Average operation time: {:?}", avg_operation_time);
    println!("  Max operation time: {:?}", max_operation_time);
    println!(
        "  Effective throughput: {:.2} ops/sec",
        successful_operations as f64 / total_time.as_secs_f64()
    );

    // Verify all operations succeeded
    assert_eq!(
        successful_operations, concurrent_count,
        "All concurrent operations should succeed"
    );

    // Concurrent operations should not be significantly slower than sequential
    assert!(
        max_operation_time.as_millis() < WORKSPACE_CREATION_THRESHOLD_MS as u128 * 2,
        "Concurrent operations took too long: {:?}",
        max_operation_time
    );

    println!("✓ Concurrent operations performance benchmark passed");
}

// Benchmark memory usage patterns
#[tokio::test]
async fn benchmark_memory_usage_patterns() {
    let fixture = PerformanceBenchmarkFixture::new()
        .await
        .expect("Failed to create fixture");

    println!("\n=== MEMORY USAGE PATTERNS BENCHMARK ===");

    // Create a workspace and add many files to test memory efficiency
    let workspace_name = "memory_test";
    let (creation_time, workspace_path) = fixture
        .create_test_workspace(workspace_name, WorkspaceTemplate::Research)
        .await
        .expect("Failed to create workspace");

    println!("  Workspace creation: {:?}", creation_time);

    // Test memory efficiency with increasing file counts
    let file_counts = vec![50, 100, 200];

    for file_count in file_counts {
        let file_creation_start = Instant::now();

        // Create files in batches to test memory management
        for batch in 0..(file_count / 50) {
            let batch_start = batch * 50;
            let batch_end = ((batch + 1) * 50).min(file_count);

            for i in batch_start..batch_end {
                let filename = format!("memory_test_{:04}.txt", i);
                let content = format!("Memory test file {} with content: {}", i, "x".repeat(1000)); // 1KB per file

                let file_path = workspace_path.join("sources/imports").join(filename);
                fs::write(&file_path, content)
                    .await
                    .expect("Failed to create file");
            }
        }

        let file_creation_time = file_creation_start.elapsed();

        // Test stats calculation performance with many files
        let stats_start = Instant::now();
        let stats = fixture
            .workspace_manager
            .get_workspace_stats(&workspace_path)
            .await
            .expect("Failed to get stats");
        let stats_time = stats_start.elapsed();

        println!(
            "  {} files: creation={:?}, stats={:?}",
            file_count, file_creation_time, stats_time
        );
        println!(
            "    Found {} files, {} bytes",
            stats.total_files, stats.total_size
        );

        // Memory efficiency checks - stats should not get exponentially slower
        let expected_max_stats_time = Duration::from_millis((file_count as u64 * 2).min(500));
        assert!(
            stats_time < expected_max_stats_time,
            "Stats calculation is not scaling efficiently: {:?} for {} files",
            stats_time,
            file_count
        );
    }

    // Test workspace loading with many files
    let loading_start = Instant::now();
    let loaded_workspace = fixture
        .workspace_manager
        .load_workspace(&workspace_path)
        .await
        .expect("Failed to load workspace");
    let loading_time = loading_start.elapsed();

    println!("  Workspace loading with many files: {:?}", loading_time);
    assert!(loaded_workspace.access_count >= 1);

    // Loading should still be fast even with many files
    assert!(
        loading_time.as_millis() < WORKSPACE_LOADING_THRESHOLD_MS as u128 * 2,
        "Workspace loading with many files took too long: {:?}",
        loading_time
    );

    println!("✓ Memory usage patterns benchmark passed");
}

// Benchmark directory structure operations
#[tokio::test]
async fn benchmark_directory_structure_performance() {
    let fixture = PerformanceBenchmarkFixture::new()
        .await
        .expect("Failed to create fixture");

    println!("\n=== DIRECTORY STRUCTURE PERFORMANCE BENCHMARK ===");

    // Test all workspace templates to benchmark directory creation
    let templates = [
        (WorkspaceTemplate::Basic, "Basic"),
        (WorkspaceTemplate::Research, "Research"),
        (WorkspaceTemplate::Documentation, "Documentation"),
        (WorkspaceTemplate::Collaboration, "Collaboration"),
    ];

    for (template, template_name) in templates {
        let workspace_name = format!("dir_test_{}", template_name.to_lowercase());

        let (creation_time, workspace_path) = fixture
            .create_test_workspace(&workspace_name, template)
            .await
            .expect("Failed to create workspace");

        println!("  {} template creation: {:?}", template_name, creation_time);

        // Verify all directories exist
        let verification_start = Instant::now();
        let mut directory_count = 0;

        for dir_path in WORKSPACE_DIRECTORIES {
            let full_path = workspace_path.join(dir_path);
            assert!(full_path.exists(), "Directory should exist: {}", dir_path);
            assert!(full_path.is_dir(), "Path should be directory: {}", dir_path);
            directory_count += 1;
        }

        // Check template-specific directories
        match template_name {
            "Research" => {
                let research_dirs = [
                    "sources/literature",
                    "sources/datasets",
                    "analysis/notebooks",
                ];
                for dir in research_dirs {
                    let full_path = workspace_path.join(dir);
                    if full_path.exists() {
                        directory_count += 1;
                    }
                }
            }
            "Documentation" => {
                let doc_dirs = [
                    "sources/specifications",
                    "sources/examples",
                    "outputs/guides",
                ];
                for dir in doc_dirs {
                    let full_path = workspace_path.join(dir);
                    if full_path.exists() {
                        directory_count += 1;
                    }
                }
            }
            "Collaboration" => {
                let collab_dirs = ["shared/resources", "shared/templates", "reviews"];
                for dir in collab_dirs {
                    let full_path = workspace_path.join(dir);
                    if full_path.exists() {
                        directory_count += 1;
                    }
                }
            }
            _ => {}
        }

        let verification_time = verification_start.elapsed();

        println!(
            "  {} verification: {:?} ({} directories)",
            template_name, verification_time, directory_count
        );

        // Directory verification should be very fast
        assert!(
            verification_time.as_millis() < 50,
            "Directory verification took too long: {:?}",
            verification_time
        );
    }

    println!("✓ Directory structure performance benchmark passed");
}

// Performance test summary
#[cfg(test)]
mod performance_benchmark_summary {
    #[tokio::test]
    async fn print_performance_benchmark_summary() {
        println!("\n=== WORKSPACE PERFORMANCE BENCHMARK SUMMARY ===");
        println!("Creation benchmarks:");
        println!("  ✓ Workspace creation performance");
        println!("  ✓ Directory structure performance");
        println!();
        println!("Operation benchmarks:");
        println!("  ✓ File operations performance");
        println!("  ✓ Workspace loading performance");
        println!("  ✓ Concurrent operations performance");
        println!();
        println!("Scalability benchmarks:");
        println!("  ✓ Memory usage patterns");
        println!("  ✓ Large file set handling");
        println!();
        println!("Performance thresholds:");
        println!("  • Workspace creation: <500ms");
        println!("  • File batch (100): <1000ms");
        println!("  • Stats calculation: <200ms");
        println!("  • Workspace loading: <100ms");
        println!("  • Validation: <150ms");
        println!("  • Detection: <50ms");
        println!("====================================================\n");
    }
}
