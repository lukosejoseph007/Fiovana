# Backup and Recovery Procedures

## Overview
This document outlines the backup and recovery procedures for the Fiovana application. The system includes automated configuration backups with integrity checking and version management.

## Backup Types

### 1. Security Configuration Backups
- **Frequency**: Hourly (configurable)
- **Retention**: 30 days
- **Files**: `security_config.json`, `magic_number_rules.json`, `safe_mode_config.json`, `circuit_breaker_config.json`
- **Priority**: Critical

### 2. Application Configuration Backups
- **Frequency**: Every 6 hours
- **Retention**: 14 days
- **Files**: `app_config.json`, `user_preferences.json`, `workspace_config.json`
- **Priority**: High

### 3. Audit Log Backups
- **Frequency**: Daily
- **Retention**: 7 days
- **Files**: `security_audit.log`, `operation_audit.log`, `error.log`
- **Priority**: Medium

## Backup Configuration

The backup system is configured via the `BackupConfig` structure:

```rust
pub struct BackupConfig {
    pub enabled: bool,
    pub max_backups: usize,
    pub retention_days: u32,
    pub backup_interval_hours: u32,
    pub priority_levels: HashMap<BackupPriority, BackupPriorityConfig>,
    pub backup_directory: PathBuf,
    pub encryption_enabled: bool,
    pub integrity_checks: bool,
    pub auto_cleanup: bool,
}
```

## Manual Backup Operations

### Creating a Manual Backup
```bash
# Backup security configurations
curl -X POST http://localhost:8080/api/backup/security

# Backup application configurations
curl -X POST http://localhost:8080/api/backup/application

# Backup audit logs
curl -X POST http://localhost:8080/api/backup/logs
```

### Checking Backup Status
```bash
# List all backups
curl http://localhost:8080/api/backup/list

# Get backup details
curl http://localhost:8080/api/backup/{backup_id}
```

## Recovery Procedures

### 1. Configuration Recovery
```bash
# Restore security configuration from backup
curl -X POST http://localhost:8080/api/recovery/security/{backup_id}

# Restore application configuration
curl -X POST http://localhost:8080/api/recovery/application/{backup_id}
```

### 2. Emergency Recovery Mode
In case of critical failure, the system can be started in recovery mode:

```bash
FIOVANA_RECOVERY_MODE=true ./fiovana-app
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `FIOVANA_BACKUP_ENABLED` | Enable/disable backup system | `true` |
| `FIOVANA_BACKUP_DIRECTORY` | Backup storage directory | `./backups` |
| `FIOVANA_MAX_BACKUPS` | Maximum number of backups to keep | `10` |
| `FIOVANA_RETENTION_DAYS` | Backup retention period | `30` |

## Integrity Verification

All backups include SHA-256 checksums for integrity verification. The system automatically validates backups during restoration:

```rust
fn validate_backup_integrity(backup_path: &Path) -> Result<bool> {
    let expected_checksum = // get from metadata
    let actual_checksum = calculate_checksum(backup_path)?;
    Ok(expected_checksum == actual_checksum)
}
```

## Monitoring and Alerts

The backup system logs all operations to the security audit log with the following events:
- Backup started/completed
- Backup failure
- Integrity check failure
- Restoration operations
- Cleanup operations

## Troubleshooting

### Common Issues

1. **Backup Directory Permission Denied**
   ```bash
   chmod 755 ./backups
   ```

2. **Insufficient Disk Space**
   - Reduce retention days
   - Increase cleanup frequency
   - Specify different backup directory

3. **Integrity Check Failures**
   - Verify disk health
   - Check for file corruption
   - Restore from different backup
