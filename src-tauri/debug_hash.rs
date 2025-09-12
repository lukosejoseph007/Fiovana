use sha2::{Digest, Sha256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test the content that should be in the files
    let test_contents = [
        ("test-app.exe", "Mock Windows executable"),
        ("test-app.dmg", "Mock macOS disk image"),
        ("test-app.AppImage", "Mock Linux AppImage"),
        ("README.md", "Release documentation"),
    ];

    println!("Expected SHA256 hashes:");
    for (filename, content) in test_contents.iter() {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        let hash_str = format!("{:X}", hash);
        println!("{}: {}", filename, hash_str);
    }

    Ok(())
}
