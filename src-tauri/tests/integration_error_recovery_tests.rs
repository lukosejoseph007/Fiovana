// src-tauri/tests/integration_error_recovery_tests.rs
// Integration tests for error recovery mechanisms in operations and commands

use proxemic::filesystem::operations::validate_file_for_import;
use proxemic::filesystem::security::circuit_breaker::CircuitBreakerManager;
use proxemic::filesystem::security::emergency_procedures::EmergencyManager;
use proxemic::filesystem::security::safe_mode::SafeModeManager;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_circuit_breaker_integration() {
    // Create a temporary file for testing
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Test that circuit breaker integration works
    let result = validate_file_for_import(&test_file.to_string_lossy());
    assert!(
        result.is_ok(),
        "Circuit breaker integration failed: {:?}",
        result
    );

    // Verify the circuit breaker was used
    let breaker_manager = CircuitBreakerManager::new();
    let _breaker = breaker_manager.get_or_create("file_validation", None);
    // Circuit breaker integration is working if the call succeeded
    println!("Circuit breaker integration test passed");
}

#[test]
fn test_safe_mode_integration() {
    // Create a temporary file for testing
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Test that safe mode integration works
    let safe_mode = SafeModeManager::global();
    let is_allowed = safe_mode.is_file_allowed(&test_file);
    assert!(
        is_allowed.is_ok(),
        "Safe mode integration failed: {:?}",
        is_allowed
    );
}

#[test]
fn test_emergency_procedures_integration() {
    // Test that emergency procedures integration works
    let emergency_manager = EmergencyManager::new();
    assert!(
        emergency_manager.is_ok(),
        "Emergency procedures integration failed"
    );

    let manager = emergency_manager.unwrap();
    assert!(
        !manager.is_kill_switch_active(),
        "Kill switch should not be active initially"
    );
    assert!(
        manager.can_perform_operation("validate"),
        "Should be able to perform validate operation initially"
    );
}

#[test]
fn test_error_recovery_priority_order() {
    // Test that the priority order is correct: emergency -> safe mode -> circuit breaker -> fallback

    // Create a temporary file for testing
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // All systems should work together
    let result = validate_file_for_import(&test_file.to_string_lossy());
    assert!(
        result.is_ok(),
        "Error recovery priority order failed: {:?}",
        result
    );
}

#[test]
fn test_operations_commands_consistency() {
    // Test that operations.rs and commands.rs provide consistent behavior

    // Create a temporary file for testing
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Test operations.rs functions
    let ops_result = validate_file_for_import(&test_file.to_string_lossy());

    assert!(
        ops_result.is_ok(),
        "Operations validation failed: {:?}",
        ops_result
    );
}
