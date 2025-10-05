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

### Configuration Structure
```rust
pub struct SecurityConfig {
    pub allowed_extensions: HashSet<String>,  // Allowed file extensions
    pub allowed_mime_types: HashSet<String>,  // Permitted MIME types
    pub max_path_length: usize,               // Max path length (chars)
    pub max_file_size: u64,                   // Max file size in bytes
    pub prohibited_filename_chars: HashSet<char>, // Dangerous characters
    pub enable_magic_number_validation: bool, // File signature checking
    pub security_level: SecurityLevel,        // Security level enum
    pub enforce_workspace_boundaries: bool,   // Workspace boundary enforcement
    pub max_concurrent_operations: u32,       // Max concurrent operations
    pub audit_logging_enabled: bool,         // Audit logging status
}
```

### Environment Variables Configuration

Security settings can be configured via environment variables for deployment flexibility:

#### Core Security Settings
- `FIOVANA_SECURITY_LEVEL`: Security level (`development`, `production`, `high_security`)
- `FIOVANA_MAX_FILE_SIZE`: Maximum file size in bytes (default: 100MB)
- `FIOVANA_MAX_PATH_LENGTH`: Maximum path length in characters (default: 260)
- `FIOVANA_MAX_CONCURRENT_OPERATIONS`: Maximum concurrent operations (default: 10)

#### Critical Security Features
- `FIOVANA_ENABLE_MAGIC_VALIDATION`: Enable magic number validation (`true`/`false`)
- `FIOVANA_ENFORCE_WORKSPACE_BOUNDARIES`: Enforce workspace boundaries (`true`/`false`)
- `FIOVANA_AUDIT_LOGGING_ENABLED`: Enable audit logging (`true`/`false`)

#### Advanced Security Options
- `FIOVANA_ENABLE_CONTENT_SCANNING`: Enable content scanning (`true`/`false`)
- `FIOVANA_SUSPICIOUS_FILE_AGE_THRESHOLD`: Suspicious file age threshold in seconds
- `FIOVANA_RATE_LIMIT_PER_MINUTE`: Rate limiting threshold
- `FIOVANA_CONFIG_VALIDATION`: Configuration validation strictness

### JSON Schema Validation

The security configuration is validated using JSON Schema for structural integrity:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Security Configuration Schema",
  "type": "object",
  "properties": {
    "max_file_size": {
      "type": "integer",
      "minimum": 1024,
      "maximum": 2147483648,
      "description": "Maximum file size in bytes"
    },
    "security_level": {
      "type": "string",
      "enum": ["development", "production", "high_security"],
      "description": "Security level"
    }
  },
  "required": ["max_file_size", "security_level", "allowed_extensions"],
  "additionalProperties": false
}
```

### Configuration Validation Levels

The system supports multiple validation levels:

1. **JSON Schema Validation**: Structural validation using Draft 7 JSON Schema
2. **Programmatic Validation**: Business logic validation with custom rules
3. **Environment Validation**: Environment variable parsing and validation
4. **Security Level Constraints**: Production/high-security specific requirements

### Production Security Hardening

For production deployments, the following constraints are automatically enforced:

- Magic number validation must be enabled
- Workspace boundaries must be enforced
- Audit logging must be enabled
- File size limits are stricter (max 100MB)
- Concurrent operations are limited (max 10)
- Path traversal protection is maximized

## Validation Workflow

1. Path Length Check
2. Traversal Attempt Detection
3. File Size Validation
4. MIME Type Verification
5. Magic Number Check
6. Filename Character Validation
7. File Extension Whitelist Check
8. Workspace Boundary Enforcement

## Incident Response Procedures

### Security Violation Protocol
1. **Immediate Containment**
   - Quarantine affected files/systems
   - Preserve audit logs and evidence
2. **Impact Assessment**
   - Determine scope using audit trail
   - Classify incident severity (Low/Med/High)
3. **Remediation**
   - Apply security patches/updates
   - Rotate credentials if compromised
4. **Post-Mortem**
   - Document root cause analysis
   - Update security controls accordingly

## Security Update Procedures

```rust
// Example security update workflow
async fn apply_security_update(update: SecurityPatch) -> Result<(), SecurityError> {
    verify_patch_signature(&update)?;
    create_system_snapshot()?;
    apply_patch_files(&update)?;
    restart_security_services()?;
    log_update(update)
}
```

## Developer Security Guidelines

### Secure Coding Practices
- Validate ALL user-provided paths
- Use type-safe handles for file operations
- Limit file descriptors per operation
- Implement resource quotas
- Never log sensitive file contents

### Code Review Checklist
- [ ] Path validation present
- [ ] Error handling for IO operations
- [ ] Memory limits enforced
- [ ] Async cancellation points
- [ ] Tests for security edge cases

## Audit Logging Implementation

```rust
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub file_path: PathBuf,
    pub user: UserContext,
    pub metadata: serde_json::Value,
}

impl AuditLogger {
    pub fn log_event(&self, event: SecurityEvent) {
        // Uses encrypted write-ahead logging
        self.log_queue.send(event).await?;
    }
}
```
