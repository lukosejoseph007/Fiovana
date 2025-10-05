use fiovana::filesystem::security::deployment_checker::DeploymentChecker;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

fn calculate_file_hash(path: &Path) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let checker = DeploymentChecker::new();
    let temp_dir = tempfile::tempdir()?;

    // Create test artifacts
    let artifacts_dir = temp_dir.path().join("artifacts");
    fs::create_dir_all(&artifacts_dir)?;

    // Create test files
    let test_files = [
        ("test-app.exe", "Mock Windows executable"),
        ("test-app.dmg", "Mock macOS disk image"),
        ("test-app.AppImage", "Mock Linux AppImage"),
        ("README.md", "Release documentation"),
    ];

    for (filename, content) in test_files.iter() {
        let file_path = artifacts_dir.join(filename);
        fs::write(&file_path, content)?;
    }

    // Generate checksums with correct hashes
    let checksums_path = temp_dir.path().join("checksums.sha256");
    let checksums_lines = [
        "68e4b552889293dc66a542f2e1b25286997bab3ad5dff5da259dc183ae0d8947 test-app.exe",
        "6e1dbcf79d29f24e92a9653b360e4b76337c5cc897fad593d16e0b564d4071d9 test-app.dmg",
        "d636fbfbe1e2465ff71abdae9fbb86e3bce6fb28cb150f3f6f1b2a1d618ff40c test-app.AppImage",
        "5558ca0c3ebe1fd6cae9fec9aa055e8ee593d2397c8812b22f53d0e499f5a4de README.md",
    ];
    let checksums_content = checksums_lines.join("\n");
    fs::write(&checksums_path, checksums_content)?;

    // Test artifact validation
    println!("Testing artifact validation...");
    let result = checker.validate_release_artifacts(&artifacts_dir, &checksums_path);

    match result {
        Ok(_) => println!("✅ Artifact validation succeeded!"),
        Err(e) => println!("❌ Artifact validation failed: {:?}", e),
    }

    // Debug: Calculate hashes of each file manually
    println!("\nCalculated hashes:");
    for (filename, content) in test_files.iter() {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        let hash_str = format!("{:x}", hash);
        println!("{}: {}", filename, hash_str);
    }

    // Debug: Read actual file content
    println!("\nActual file content:");
    for (filename, _) in test_files.iter() {
        let file_path = artifacts_dir.join(filename);
        let content = fs::read_to_string(&file_path)?;
        println!("{}: {:?}", filename, content);
    }

    // Debug: Read checksums file
    println!("\nChecksums file content:");
    let checksums_content = fs::read_to_string(&checksums_path)?;
    println!("{:?}", checksums_content);

    // Debug: Parse checksums file line by line
    println!("\nParsed checksums:");
    for line in checksums_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let expected_hash = parts[0];
            let filename = parts[1..].join(" ");
            println!("{} -> {}", filename, expected_hash);
        }
    }

    // Debug: Check each file individually
    println!("\nIndividual file validation:");
    for (filename, _content) in test_files.iter() {
        let file_path = artifacts_dir.join(filename);
        let hash = calculate_file_hash(&file_path)?;
        println!("{}: {} (calculated)", filename, hash);

        // Find expected hash from checksums
        let expected_hash = checksums_content
            .lines()
            .find(|line| line.contains(filename))
            .and_then(|line| line.split_whitespace().next())
            .unwrap_or("NOT_FOUND");

        println!("{}: {} (expected)", filename, expected_hash);
        println!("Match: {}", hash == expected_hash);
        println!();
    }

    Ok(())
}
