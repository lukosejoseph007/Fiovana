# Security Architecture

## Defense-in-Depth Security Measures

### File Validation
- **Magic Number Validation**: Verifies file signatures match known patterns for allowed types
- **MIME Type Detection**: Uses both file signature and extension to validate content type
- **File Size Limits**: Enforces maximum file size (default: 100MB)

### Path Security
- **Path Length Limits**: Prevents path overflow attacks (260 char max)
- **Traversal Prevention**: Blocks paths containing `..` sequences
- **Character Whitelisting**: Prohibits dangerous characters in filenames `<>:"|?*`

### Access Control
- **Workspace Boundaries**: Restricts operations to user directories (Desktop/Documents/Downloads)
- **Scope Validation**: Ensures files remain within allowed directories

## Threat Model Mitigations

| Threat Type              | Mitigation                                                                 |
|--------------------------|----------------------------------------------------------------------------|
| Path Traversal           | Canonicalization + workspace boundary checks                              |
| Malicious File Upload    | Magic number + MIME type validation                                        |
| Resource Exhaustion      | Strict file size limits (configurable)                                     |
| File Spoofing            | Extension/MIME type correlation enforcement                                |
| Directory Climbing       | Multiple .. detection layers                                               |

## Security Configuration

```rust
pub struct SecurityConfig {
    pub allowed_extensions: HashSet<String>,  // Allowed file extensions
    pub allowed_mime_types: HashSet<String>,  // Permitted MIME types
    pub max_path_length: usize,               // Max path length (chars)
    pub max_file_size: u64,                   // Max file size in bytes
    pub prohibited_filename_chars: HashSet<char>, // Dangerous characters
    pub enable_magic_number_validation: bool, // File signature checking
}
```

## Validation Workflow

1. Path Length Check
2. Traversal Attempt Detection
3. File Size Validation
4. MIME Type Verification
5. Magic Number Check
6. Filename Character Validation
7. File Extension Whitelist Check
8. Workspace Boundary Enforcement

## Audit Logging
All security violations are logged with:
- Timestamp
- File path
- Validation failure type
- User context
- Stack trace
