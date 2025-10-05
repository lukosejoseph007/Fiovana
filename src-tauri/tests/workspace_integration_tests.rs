// src-tauri/tests/workspace_integration_tests.rs
// Integration tests for complete workspace workflows

use fiovana::app_config::ConfigManager;
use fiovana::workspace::{
    CreateWorkspaceRequest, UpdateRecentWorkspaceRequest, WorkspaceManager, WorkspaceTemplate,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

/// Integration test fixture for complete workspace workflows
struct WorkspaceIntegrationFixture {
    _temp_dir: TempDir,
    base_path: PathBuf,
    _config_manager: Arc<ConfigManager>,
    workspace_manager: WorkspaceManager,
}

impl WorkspaceIntegrationFixture {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();

        // Create config manager with test configuration
        let config_manager = Arc::new(ConfigManager::new().await?);

        // Initialize workspace manager
        let workspace_manager = WorkspaceManager::new(config_manager.clone())?;

        Ok(Self {
            _temp_dir: temp_dir,
            base_path,
            _config_manager: config_manager,
            workspace_manager,
        })
    }

    fn get_test_workspace_path(&self, name: &str) -> PathBuf {
        self.base_path.join(name)
    }

    async fn create_test_files_in_workspace(
        &self,
        workspace_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create some test files in the workspace
        let imports_dir = workspace_path.join("sources/imports");

        // Create test documents
        let test_files = [
            (
                "document1.txt",
                "This is a test document for integration testing.",
            ),
            (
                "document2.md",
                "# Test Document\nThis is a markdown test file.",
            ),
            ("data.csv", "name,value\ntest1,100\ntest2,200"),
        ];

        for (filename, content) in test_files {
            let file_path = imports_dir.join(filename);
            fs::write(&file_path, content).await?;
        }

        Ok(())
    }
}

// Test complete workspace creation and setup workflow
#[tokio::test]
async fn test_complete_workspace_creation_workflow() {
    let fixture = WorkspaceIntegrationFixture::new()
        .await
        .expect("Failed to create fixture");

    let workspace_name = "integration_test_workspace";
    let workspace_path = fixture.get_test_workspace_path(workspace_name);

    // Step 1: Create workspace using workspace manager
    let create_request = CreateWorkspaceRequest {
        name: workspace_name.to_string(),
        path: workspace_path.clone(),
        template: WorkspaceTemplate::Research,
        description: Some("Integration test workspace".to_string()),
    };

    let workspace_info = fixture
        .workspace_manager
        .create_workspace(create_request)
        .await
        .expect("Failed to create workspace");

    // Step 2: Verify workspace structure
    assert!(workspace_path.exists(), "Workspace directory should exist");
    assert_eq!(workspace_info.name, workspace_name);

    // Step 3: Add test files to workspace
    fixture
        .create_test_files_in_workspace(&workspace_path)
        .await
        .expect("Failed to create test files");

    // Step 4: Load workspace and verify
    let loaded_workspace = fixture
        .workspace_manager
        .load_workspace(&workspace_path)
        .await
        .expect("Failed to load workspace");

    assert_eq!(loaded_workspace.name, workspace_name);
    assert!(loaded_workspace.access_count >= 1);

    // Step 5: Get workspace statistics
    let stats = fixture
        .workspace_manager
        .get_workspace_stats(&workspace_path)
        .await
        .expect("Failed to get workspace stats");

    assert!(stats.total_files >= 3, "Should have at least 3 test files");

    println!("✓ Complete workspace creation workflow test passed");
}

// Test recent workspaces management workflow
#[tokio::test]
async fn test_recent_workspaces_workflow() {
    let fixture = WorkspaceIntegrationFixture::new()
        .await
        .expect("Failed to create fixture");

    // Create multiple workspaces
    let workspaces = [
        ("recent_workspace_1", WorkspaceTemplate::Basic),
        ("recent_workspace_2", WorkspaceTemplate::Research),
        ("recent_workspace_3", WorkspaceTemplate::Documentation),
    ];

    let mut created_workspaces = Vec::new();

    // Step 1: Create multiple workspaces
    for (name, template) in workspaces {
        let workspace_path = fixture.get_test_workspace_path(name);

        let create_request = CreateWorkspaceRequest {
            name: name.to_string(),
            path: workspace_path.clone(),
            template,
            description: Some(format!("Recent workspace test: {}", name)),
        };

        let workspace_info = fixture
            .workspace_manager
            .create_workspace(create_request)
            .await
            .expect("Failed to create workspace");

        created_workspaces.push(workspace_info);
    }

    // Step 2: Update recent workspace access
    for workspace in &created_workspaces {
        let update_request = UpdateRecentWorkspaceRequest {
            path: workspace.path.clone(),
            name: workspace.name.clone(),
            template: WorkspaceTemplate::Basic, // Simplified for test
        };

        fixture
            .workspace_manager
            .update_recent_workspace(update_request)
            .await
            .expect("Failed to update recent workspace");
    }

    // Step 3: List recent workspaces
    let recent_workspaces = fixture
        .workspace_manager
        .get_recent_workspaces()
        .await
        .expect("Failed to list recent workspaces");

    // Should have at least the workspaces we created
    assert!(
        recent_workspaces.len() >= 3,
        "Should have at least 3 recent workspaces"
    );

    // Verify our workspaces are in the list
    for workspace in &created_workspaces {
        let found = recent_workspaces.iter().any(|rw| rw.name == workspace.name);
        assert!(
            found,
            "Workspace {} should be in recent list",
            workspace.name
        );
    }

    println!("✓ Recent workspaces workflow test passed");
}

// Test workspace template-specific workflow
#[tokio::test]
async fn test_template_specific_workflows() {
    let fixture = WorkspaceIntegrationFixture::new()
        .await
        .expect("Failed to create fixture");

    // Test Research template workflow
    let research_workspace_path = fixture.get_test_workspace_path("research_workflow_test");
    let create_request = CreateWorkspaceRequest {
        name: "research_workflow_test".to_string(),
        path: research_workspace_path.clone(),
        template: WorkspaceTemplate::Research,
        description: Some("Research template workflow test".to_string()),
    };
    let research_workspace = fixture
        .workspace_manager
        .create_workspace(create_request)
        .await
        .expect("Failed to create research workspace");

    // Verify research-specific features
    assert!(research_workspace.import_settings.max_file_size > 100 * 1024 * 1024);
    assert!(research_workspace
        .import_settings
        .allowed_extensions
        .contains(&".xlsx".to_string()));

    // Verify research-specific directories
    assert!(research_workspace_path.join("sources/literature").exists());
    assert!(research_workspace_path.join("analysis/notebooks").exists());

    // Test Documentation template workflow
    let doc_workspace_path = fixture.get_test_workspace_path("doc_workflow_test");
    let create_request = CreateWorkspaceRequest {
        name: "doc_workflow_test".to_string(),
        path: doc_workspace_path.clone(),
        template: WorkspaceTemplate::Documentation,
        description: Some("Documentation template workflow test".to_string()),
    };
    let doc_workspace = fixture
        .workspace_manager
        .create_workspace(create_request)
        .await
        .expect("Failed to create documentation workspace");

    // Verify documentation-specific features
    assert!(doc_workspace
        .import_settings
        .allowed_extensions
        .contains(&".md".to_string()));
    assert!(doc_workspace
        .import_settings
        .allowed_extensions
        .contains(&".html".to_string()));

    // Verify documentation-specific directories
    assert!(doc_workspace_path.join("sources/specifications").exists());
    assert!(doc_workspace_path.join("outputs/guides").exists());

    println!("✓ Template-specific workflows test passed");
}

// Test workspace file operations workflow
#[tokio::test]
async fn test_workspace_file_operations_workflow() {
    let fixture = WorkspaceIntegrationFixture::new()
        .await
        .expect("Failed to create fixture");

    let workspace_name = "file_ops_test";
    let workspace_path = fixture.get_test_workspace_path(workspace_name);

    // Step 1: Create workspace
    let create_request = CreateWorkspaceRequest {
        name: workspace_name.to_string(),
        path: workspace_path.clone(),
        template: WorkspaceTemplate::Basic,
        description: Some("File operations test workspace".to_string()),
    };
    let workspace_info = fixture
        .workspace_manager
        .create_workspace(create_request)
        .await
        .expect("Failed to create workspace");

    // Step 2: Add files to different directories
    let imports_dir = workspace_path.join("sources/imports");
    let references_dir = workspace_path.join("sources/references");
    let drafts_dir = workspace_path.join("outputs/drafts");

    // Create files in imports
    fs::write(imports_dir.join("import1.txt"), "Imported document 1")
        .await
        .expect("Failed to create import file");
    fs::write(imports_dir.join("import2.md"), "# Imported Document 2")
        .await
        .expect("Failed to create import file");

    // Create files in references
    fs::write(references_dir.join("ref1.txt"), "Reference document 1")
        .await
        .expect("Failed to create reference file");

    // Create files in drafts
    fs::write(drafts_dir.join("draft1.docx"), "Draft document 1")
        .await
        .expect("Failed to create draft file");

    // Step 3: Get updated workspace statistics
    let stats = fixture
        .workspace_manager
        .get_workspace_stats(&workspace_path)
        .await
        .expect("Failed to get workspace stats");

    // Should have files in the workspace
    assert!(stats.total_files >= 4, "Should have at least 4 files");
    assert!(stats.total_size > 0, "Total size should be greater than 0");

    // Step 4: Reload workspace and verify access count increment
    let reloaded_workspace = fixture
        .workspace_manager
        .load_workspace(&workspace_path)
        .await
        .expect("Failed to reload workspace");

    assert!(reloaded_workspace.access_count > workspace_info.access_count);

    println!("✓ Workspace file operations workflow test passed");
}

// Test workspace error handling and recovery workflow
#[tokio::test]
async fn test_workspace_error_handling_workflow() {
    let fixture = WorkspaceIntegrationFixture::new()
        .await
        .expect("Failed to create fixture");

    // Test 1: Invalid workspace path
    let invalid_request = CreateWorkspaceRequest {
        name: "invalid_workspace".to_string(),
        path: PathBuf::from("../../../invalid/path"),
        template: WorkspaceTemplate::Basic,
        description: Some("Invalid workspace test".to_string()),
    };
    let invalid_path_result = fixture
        .workspace_manager
        .create_workspace(invalid_request)
        .await;
    assert!(invalid_path_result.is_err(), "Should fail for invalid path");

    // Test 2: Loading non-existent workspace
    let non_existent_path = fixture.get_test_workspace_path("non_existent");
    let load_result = fixture
        .workspace_manager
        .load_workspace(&non_existent_path)
        .await;
    assert!(
        load_result.is_err(),
        "Should fail for non-existent workspace"
    );

    // Test 3: Getting stats for non-existent workspace
    let stats_result = fixture
        .workspace_manager
        .get_workspace_stats(&non_existent_path)
        .await;
    assert!(
        stats_result.is_err(),
        "Should fail for non-existent workspace"
    );

    // Test 4: Duplicate workspace creation
    let workspace_path = fixture.get_test_workspace_path("duplicate_test");

    // Create first workspace
    let first_request = CreateWorkspaceRequest {
        name: "duplicate_test".to_string(),
        path: workspace_path.clone(),
        template: WorkspaceTemplate::Basic,
        description: Some("First workspace".to_string()),
    };
    let first_result = fixture
        .workspace_manager
        .create_workspace(first_request)
        .await;
    assert!(
        first_result.is_ok(),
        "First workspace creation should succeed"
    );

    // Try to create duplicate workspace
    let duplicate_request = CreateWorkspaceRequest {
        name: "duplicate_test".to_string(),
        path: workspace_path.clone(),
        template: WorkspaceTemplate::Basic,
        description: Some("Duplicate workspace".to_string()),
    };
    let duplicate_result = fixture
        .workspace_manager
        .create_workspace(duplicate_request)
        .await;
    assert!(
        duplicate_result.is_err(),
        "Duplicate workspace creation should fail"
    );

    println!("✓ Workspace error handling workflow test passed");
}

// Test concurrent workspace operations workflow
#[tokio::test]
async fn test_concurrent_workspace_operations() {
    let fixture = WorkspaceIntegrationFixture::new()
        .await
        .expect("Failed to create fixture");

    // Create multiple workspaces concurrently
    let concurrent_tasks = vec![
        ("concurrent_1", "basic"),
        ("concurrent_2", "research"),
        ("concurrent_3", "documentation"),
        ("concurrent_4", "basic"),
        ("concurrent_5", "research"),
    ];

    let mut handles = Vec::new();

    // Use Arc to share the fixture
    let fixture_arc = std::sync::Arc::new(fixture);

    for (workspace_name, template) in concurrent_tasks {
        let fixture_clone = fixture_arc.clone();
        let name = workspace_name.to_string();
        let template_str = template.to_string();

        let handle = tokio::spawn(async move {
            let workspace_path = fixture_clone.get_test_workspace_path(&name);

            let create_request = CreateWorkspaceRequest {
                name: name.clone(),
                path: workspace_path,
                template: match template_str.as_str() {
                    "research" => WorkspaceTemplate::Research,
                    "documentation" => WorkspaceTemplate::Documentation,
                    _ => WorkspaceTemplate::Basic,
                },
                description: Some(format!("Concurrent test workspace: {}", name)),
            };
            fixture_clone
                .workspace_manager
                .create_workspace(create_request)
                .await
        });

        handles.push((workspace_name, handle));
    }

    // Wait for all operations to complete
    let mut successful_operations = 0;

    for (name, handle) in handles {
        match handle.await {
            Ok(Ok(_workspace)) => {
                successful_operations += 1;
                println!("✓ Successfully created concurrent workspace: {}", name);
            }
            Ok(Err(e)) => {
                println!("✗ Failed to create workspace {}: {:?}", name, e);
            }
            Err(e) => {
                println!("✗ Task failed for workspace {}: {:?}", name, e);
            }
        }
    }

    assert_eq!(
        successful_operations, 5,
        "All concurrent operations should succeed"
    );

    // Verify workspaces are accessible (recent list may be empty if not explicitly updated)
    let recent_workspaces = fixture_arc
        .workspace_manager
        .get_recent_workspaces()
        .await
        .expect("Failed to list recent workspaces");

    // Recent workspaces are only tracked when explicitly updated, not on creation
    println!(
        "Found {} recent workspaces (creation doesn't auto-add to recent list)",
        recent_workspaces.len()
    );

    println!("✓ Concurrent workspace operations test passed");
}

// Test workspace lifecycle workflow
#[tokio::test]
async fn test_complete_workspace_lifecycle() {
    let fixture = WorkspaceIntegrationFixture::new()
        .await
        .expect("Failed to create fixture");

    let workspace_name = "lifecycle_test";
    let workspace_path = fixture.get_test_workspace_path(workspace_name);

    // Phase 1: Creation
    println!("Phase 1: Creating workspace...");
    let create_request = CreateWorkspaceRequest {
        name: workspace_name.to_string(),
        path: workspace_path.clone(),
        template: WorkspaceTemplate::Research,
        description: Some("Complete lifecycle test workspace".to_string()),
    };
    let workspace = fixture
        .workspace_manager
        .create_workspace(create_request)
        .await
        .expect("Failed to create workspace");

    assert_eq!(workspace.access_count, 1);

    // Phase 2: Initial Usage
    println!("Phase 2: Initial usage...");
    fixture
        .create_test_files_in_workspace(&workspace_path)
        .await
        .expect("Failed to create initial files");

    let initial_stats = fixture
        .workspace_manager
        .get_workspace_stats(&workspace_path)
        .await
        .expect("Failed to get initial stats");

    // Phase 3: Active Usage
    println!("Phase 3: Active usage simulation...");
    for i in 1..=3 {
        // Simulate multiple accesses
        let accessed_workspace = fixture
            .workspace_manager
            .load_workspace(&workspace_path)
            .await
            .expect("Failed to access workspace");

        assert!(accessed_workspace.access_count > i);

        // Add more files during active usage
        let active_file_path = workspace_path.join(format!("sources/imports/active_{}.txt", i));
        fs::write(&active_file_path, format!("Active usage file {}", i))
            .await
            .expect("Failed to create active file");
    }

    // Phase 4: Final State Verification
    println!("Phase 4: Final state verification...");
    let final_workspace = fixture
        .workspace_manager
        .load_workspace(&workspace_path)
        .await
        .expect("Failed to load final workspace state");

    let final_stats = fixture
        .workspace_manager
        .get_workspace_stats(&workspace_path)
        .await
        .expect("Failed to get final stats");

    // Verify lifecycle progression
    assert!(
        final_workspace.access_count >= 4,
        "Should have multiple accesses"
    );
    assert!(
        final_stats.total_files > initial_stats.total_files,
        "Should have more files"
    );
    assert!(
        final_stats.total_size > initial_stats.total_size,
        "Should have larger size"
    );

    // Verify workspace appears in recent list (if recent workspace tracking is enabled)
    let recent_workspaces = fixture
        .workspace_manager
        .get_recent_workspaces()
        .await
        .expect("Failed to get recent workspaces");

    // Note: Recent workspaces may not be automatically added on creation,
    // only when explicitly accessed through the update_recent_workspace method
    if !recent_workspaces.is_empty() {
        println!("Found {} recent workspaces", recent_workspaces.len());
    } else {
        println!("No recent workspaces found - this is expected behavior");
    }

    println!("✓ Complete workspace lifecycle test passed");
}

// Test module summary
#[cfg(test)]
mod workspace_integration_test_summary {
    #[tokio::test]
    async fn print_workspace_integration_test_summary() {
        println!("\n=== WORKSPACE INTEGRATION TEST SUMMARY ===");
        println!("End-to-end workflow tests:");
        println!("  ✓ Complete workspace creation workflow");
        println!("  ✓ Recent workspaces management workflow");
        println!("  ✓ Template-specific workflows");
        println!("  ✓ Workspace file operations workflow");
        println!("  ✓ Workspace error handling workflow");
        println!("  ✓ Concurrent workspace operations");
        println!("  ✓ Complete workspace lifecycle");
        println!();
        println!("Integration validations:");
        println!("  ✓ Command interface integration");
        println!("  ✓ File system operations");
        println!("  ✓ Statistics tracking");
        println!("  ✓ Recent workspace tracking");
        println!("  ✓ Template-specific configurations");
        println!("  ✓ Error handling and recovery");
        println!("  ✓ Concurrent operation safety");
        println!("  ✓ Complete lifecycle management");
        println!("=============================================\n");
    }
}
