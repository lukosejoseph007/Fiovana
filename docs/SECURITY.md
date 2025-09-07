# Permissions & Security Rationale

## Filesystem (fs:read)
- **Why**: Required for importing user documents (PDF, DOCX, TXT, MD).
- **Risks**: Arbitrary file access → mitigated via PathValidator (length, traversal, chars, extension).

## Magic Number Validation
- **Why**: Prevents spoofed file uploads (e.g., .exe renamed to .pdf).
- **Risks**: Malware injection → mitigated via MagicNumberValidator signatures.

## Path Restrictions
- **Why**: Ensures user cannot escape into OS/system directories.
- **Risks**: Access to sensitive files → mitigated via SecurityConfig `max_path_length`, prohibited chars, and traversal checks.

## Future Permission Escalation
- **Plan**: Temporary privilege escalation only for approved operations.
- **Process**:
  1. Add scope to `SecurityConfig`
  2. Document in `docs/permissions.md`
  3. Require explicit code review/approval before enabling


## Permission Escalation (Future Plan)

Escalation is **disabled by default**.
When enabled:
- Must log the escalation event
- Must include an approval step (e.g., admin consent or explicit UI confirmation)
- Must be time-bound (temporary elevation only)
- Must be auditable
