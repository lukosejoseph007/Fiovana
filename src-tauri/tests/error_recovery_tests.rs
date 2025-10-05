use fiovana::filesystem::security::{
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    safe_mode::{SafeModeLevel, SafeModeManager},
};

#[test]
fn test_circuit_breaker_recovery() -> anyhow::Result<()> {
    let breaker = CircuitBreaker::new(
        "test_breaker".to_string(),
        CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: std::time::Duration::from_secs(1),
            success_threshold: 1,
        },
    );

    // First failure
    let result = tokio::runtime::Runtime::new()?.block_on(async {
        breaker.call(|| Err::<(), anyhow::Error>(anyhow::anyhow!("test failure")))
    });
    assert!(result.is_err());

    // Second failure should trigger circuit breaker
    let result = tokio::runtime::Runtime::new()?.block_on(async {
        breaker.call(|| Err::<(), anyhow::Error>(anyhow::anyhow!("test failure")))
    });
    assert!(result.is_err());

    // Wait for recovery timeout
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Should be in half-open state and allow one more call
    let result = tokio::runtime::Runtime::new()?
        .block_on(async { breaker.call(|| Ok::<(), anyhow::Error>(())) });
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_safe_mode_levels() -> anyhow::Result<()> {
    let manager = SafeModeManager::new();

    // Test default level
    assert!(matches!(
        manager.get_config().level,
        SafeModeLevel::Disabled
    ));

    // Test setting levels
    manager.set_level(SafeModeLevel::Restricted)?;
    assert!(matches!(
        manager.get_config().level,
        SafeModeLevel::Restricted
    ));

    manager.set_level(SafeModeLevel::Paranoid)?;
    assert!(matches!(
        manager.get_config().level,
        SafeModeLevel::Paranoid
    ));

    manager.set_level(SafeModeLevel::Emergency)?;
    assert!(matches!(
        manager.get_config().level,
        SafeModeLevel::Emergency
    ));

    // Reset to Restricted level for file type testing (Emergency blocks all files)
    manager.set_level(SafeModeLevel::Restricted)?;

    // Test file type restrictions
    let temp_dir = tempfile::tempdir()?;
    let text_file = temp_dir.path().join("text.txt");
    let script_file = temp_dir.path().join("script.exe");

    std::fs::write(&text_file, "test content")?;
    std::fs::write(&script_file, "binary content")?;

    // Debug output
    println!("Text file path: {:?}", text_file);
    println!("Script file path: {:?}", script_file);

    let text_result = manager.is_file_allowed(&text_file);
    let script_result = manager.is_file_allowed(&script_file);

    println!("Text file result: {:?}", text_result);
    println!("Script file result: {:?}", script_result);

    assert!(text_result?);
    assert!(!script_result?);

    Ok(())
}
