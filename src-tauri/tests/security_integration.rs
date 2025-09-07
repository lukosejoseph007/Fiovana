// src-tauri/tests/security_integration.rs

use proxemic::commands::import_file;

#[tokio::test]
async fn validate_import_rejects_exe() {
    // Call the actual Tauri command (simulating a front-end request)
    let result = import_file("C:/malicious.exe".into()).await;

    assert!(
        result.is_err(),
        "EXE should not be accepted by Tauri command layer"
    );
}

#[tokio::test]
async fn validate_import_accepts_pdf() {
    let result = import_file("C:/safe.pdf".into()).await;

    assert!(
        result.is_ok(),
        "PDF should be accepted by Tauri command layer"
    );
}
