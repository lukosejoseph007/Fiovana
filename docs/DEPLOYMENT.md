# Proxemic Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying Proxemic across different environments. Proxemic is a Tauri-based desktop application with security-focused file operations and AI integration capabilities.

## Table of Contents
- [Environment Configuration](#environment-configuration)
- [Build and Packaging](#build-and-packaging)
- [Deployment Scenarios](#deployment-scenarios)
- [Security Hardening](#security-hardening)
- [Monitoring and Logging](#monitoring-and-logging)
- [Troubleshooting](#troubleshooting)
- [CI/CD Integration](#cicd-integration)

## Environment Configuration

### Required Environment Variables

#### Core Application Settings
```bash
# Environment type (development/production)
PROXEMIC_ENV=production

# Log level (debug/info/warn/error)
RUST_LOG=warn

# Debug mode (true/false) - disable in production
PROXEMIC_DEBUG=false
```

#### AI Service Configuration
```bash
# OpenRouter API Key (required for AI features)
OPENROUTER_API_KEY=your_encrypted_openrouter_key

# Anthropic API Key (alternative AI provider)
ANTHROPIC_API_KEY=your_encrypted_anthropic_key
```

#### Database Configuration
```bash
# SQLite database path
DATABASE_URL=sqlite:./Proxemic.db
```

#### Vector Search Configuration
```bash
# Vector index storage path
VECTOR_INDEX_PATH=./vector_index

# Maximum embedding batch size
MAX_EMBEDDING_BATCH_SIZE=50
```

### Security Configuration

#### Security Level (REQUIRED)
```bash
# Options: development, production, high_security
PROXEMIC_SECURITY_LEVEL=production
```

#### File Security Limits
```bash
# Maximum file size in bytes (default: 100MB)
PROXEMIC_MAX_FILE_SIZE=104857600

# Maximum path length in characters (Windows compatibility)
PROXEMIC_MAX_PATH_LENGTH=260

# Maximum concurrent file operations
PROXEMIC_MAX_CONCURRENT_OPERATIONS=10
```

#### Critical Security Features (MUST BE TRUE IN PRODUCTION)
```bash
# Enable magic number validation for file type detection
PROXEMIC_ENABLE_MAGIC_VALIDATION=true

# Enforce workspace boundaries to restrict file operations
PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES=true

# Enable security audit logging
PROXEMIC_AUDIT_LOGGING_ENABLED=true
```

#### Advanced Security Options
```bash
# Enable content security scanning
PROXEMIC_ENABLE_CONTENT_SCANNING=true

# Suspicious file age threshold in seconds (default: 300 = 5 minutes)
PROXEMIC_SUSPICIOUS_FILE_AGE_THRESHOLD=300

# Rate limiting (requests per minute)
PROXEMIC_RATE_LIMIT_PER_MINUTE=60

# Configuration validation strictness (lenient/strict/paranoid)
PROXEMIC_CONFIG_VALIDATION=strict

# Performance monitoring
PROXEMIC_PERFORMANCE_MONITORING=true
```

#### Compliance and Auditing
```bash
# Compliance framework (none/gdpr/hipaa/pci_dss)
PROXEMIC_COMPLIANCE_FRAMEWORK=none

# Data classification level (public/internal/confidential/restricted)
PROXEMIC_DATA_CLASSIFICATION=internal

# Audit log retention period in days
PROXEMIC_AUDIT_LOG_RETENTION_DAYS=90

# Security alert threshold
PROXEMIC_SECURITY_ALERT_THRESHOLD=10

# Alert method (log/email/slack/webhook)
PROXEMIC_ALERT_METHOD=log
```

### Environment Templates

#### Development Environment (.env.development)
```bash
PROXEMIC_ENV=development
RUST_LOG=debug
PROXEMIC_DEBUG=true
PROXEMIC_SECURITY_LEVEL=development
DATABASE_URL=sqlite:./Proxemic_dev.db
VECTOR_INDEX_PATH=./vector_index_dev
ENABLE_TELEMETRY=true
ENABLE_CLOUD_SYNC=true
ENABLE_COLLABORATION=true
```

#### Production Environment (.env.production)
```bash
PROXEMIC_ENV=production
RUST_LOG=warn
PROXEMIC_DEBUG=false
PROXEMIC_SECURITY_LEVEL=production
ENABLE_TELEMETRY=false
ENABLE_CLOUD_SYNC=false
ENABLE_COLLABORATION=false
PROXEMIC_AUDIT_ENABLED=true
PROXEMIC_STRUCTURED_LOGGING=true
PROXEMIC_RATE_LIMIT_ENABLED=true
```

## Build and Packaging

### Prerequisites

#### System Dependencies
- **Rust**: 1.70+ (stable)
- **Node.js**: 18+
- **Tauri CLI**: `npm install -g @tauri-apps/cli`
- **Platform-specific build tools**:
  - **Windows**: Visual Studio Build Tools with C++ workload
  - **macOS**: Xcode Command Line Tools
  - **Linux**:
    ```bash
    sudo apt-get install -y \
      libgtk-3-dev \
      libwebkit2gtk-4.1-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev \
      patchelf
    ```

### Build Commands

#### Development Build
```bash
# Install dependencies
npm ci

# Build frontend and Tauri app
npm run tauri build

# Development mode with hot reload
npm run tauri dev
```

#### Production Build
```bash
# Set production environment
export PROXEMIC_ENV=production

# Build with production optimizations
npm run tauri build -- --release

# Build specific targets
npm run tauri build -- --target x86_64-pc-windows-msi
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

### Packaging Options

#### Windows (MSI/NSIS)
```bash
# Build Windows installer
npm run tauri build -- --target x86_64-pc-windows-msi

# Build NSIS installer
npm run tauri build -- --target x86_64-pc-windows-nsis
```

#### macOS (DMG/App Bundle)
```bash
# Build macOS application bundle
npm run tauri build -- --target x86_64-apple-darwin

# Build DMG installer
npm run tauri build -- --target universal-apple-darwin
```

#### Linux (AppImage/DEB)
```bash
# Build AppImage
npm run tauri build -- --target x86_64-unknown-linux-gnu

# Build DEB package (requires cargo-deb)
cargo install cargo-deb
cargo deb --target x86_64-unknown-linux-gnu
```

## Deployment Scenarios

### Local Development Deployment

1. **Clone and setup**:
   ```bash
   git clone https://github.com/lukosejoseph007/Proxemic.git
   cd Proxemic
   npm ci
   ```

2. **Configure development environment**:
   ```bash
   cp .env.example .env.development
   # Edit .env.development with your API keys
   ```

3. **Run development server**:
   ```bash
   npm run tauri dev
   ```

### Cloud Deployment (AWS/Azure/GCP)

#### Docker Containerization
```dockerfile
# Dockerfile.production
FROM node:18-alpine AS frontend
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM rust:1.70 AS backend
WORKDIR /app
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./
COPY src-tauri/src/ ./src/
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=frontend /app/dist ./dist
COPY --from=backend /app/target/release/proxemic ./
COPY .env.production ./
EXPOSE 3000
CMD ["./proxemic"]
```

#### Kubernetes Deployment
```yaml
# proxemic-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: proxemic
spec:
  replicas: 3
  selector:
    matchLabels:
      app: proxemic
  template:
    metadata:
      labels:
        app: proxemic
    spec:
      containers:
      - name: proxemic
        image: your-registry/proxemic:latest
        ports:
        - containerPort: 3000
        envFrom:
        - configMapRef:
            name: proxemic-config
        - secretRef:
            name: proxemic-secrets
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
```

### On-Premises Deployment

#### Traditional Installation
1. **Build production artifacts**:
   ```bash
   npm run tauri build -- --release
   ```

2. **Distribute installers**:
   - Windows: `src-tauri/target/release/bundle/msi/*.msi`
   - macOS: `src-tauri/target/release/bundle/dmg/*.dmg`
   - Linux: `src-tauri/target/release/bundle/appimage/*.AppImage`

3. **Configure environment**:
   - Create `.env.production` file in installation directory
   - Set appropriate security levels and API keys

#### Enterprise Deployment
```bash
# Silent installation (Windows)
msiexec /i Proxemic-0.1.0-x86_64.msi /quiet

# Configuration management integration
# - Use Group Policy for Windows deployments
# - Use MDM solutions for macOS deployments
# - Use configuration management tools (Ansible/Puppet/Chef)
```

### Hybrid Deployment

#### Cloud-Connected Desktop
```bash
# Local installation with cloud sync
ENABLE_CLOUD_SYNC=true
CLOUD_SYNC_ENDPOINT=https://your-cloud-endpoint/api

# Configuration via environment variables
CLOUD_AUTH_TOKEN=your_encrypted_token
SYNC_INTERVAL=300  # 5 minutes
```

## Security Hardening

### Production Security Checklist

#### Mandatory Settings
- [ ] `PROXEMIC_SECURITY_LEVEL=production`
- [ ] `PROXEMIC_ENABLE_MAGIC_VALIDATION=true`
- [ ] `PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES=true`
- [ ] `PROXEMIC_AUDIT_LOGGING_ENABLED=true`
- [ ] `PROXEMIC_DEBUG=false`

#### Recommended Settings
- [ ] `PROXEMIC_ENABLE_CONTENT_SCANNING=true`
- [ ] `PROXEMIC_CONFIG_VALIDATION=strict`
- [ ] `PROXEMIC_RATE_LIMIT_PER_MINUTE=60`
- [ ] `PROXEMIC_PERFORMANCE_MONITORING=true`

### Encryption Configuration

#### Environment Encryption
```bash
# Generate encryption key (32 characters)
PROXEMIC_ENCRYPTION_KEY=your_secure_32_character_key_here_change_this

# Recommended: Use key management services
# - AWS KMS, Azure Key Vault, Google Cloud KMS
# - HashiCorp Vault
```

#### Secure API Key Management
```bash
# Use encrypted values for API keys
OPENROUTER_API_KEY=encrypted:your_encrypted_key
ANTHROPIC_API_KEY=encrypted:your_encrypted_key

# Or use environment-specific key management:
# - Development: Plain text (with appropriate security)
# - Production: Hardware Security Modules (HSM)
# - Staging: Secure environment variables
```

### Network Security

#### Firewall Configuration
```bash
# Required outgoing connections
# - AI API endpoints (OpenRouter, Anthropic)
# - Cloud sync endpoints (if enabled)

# Recommended inbound rules
# - None (desktop application)
```

#### SSL/TLS Configuration
```bash
# Ensure all external connections use TLS 1.2+
# Validate certificate chains
# Implement certificate pinning for critical services
```

## Monitoring and Logging

### Audit Logging Configuration

#### Log Levels
```bash
# Development: Detailed logging
RUST_LOG=debug

# Production: Security-focused logging
RUST_LOG=warn,proxemic=info,security=warn

# High security: Minimal logging
RUST_LOG=error
```

#### Log Storage
```bash
# Local log files (default)
LOG_FILE_PATH=./logs/proxemic.log

# Remote logging (optional)
LOGSTASH_ENDPOINT=https://your-logstash:5044
ELASTICSEARCH_HOST=your-es-cluster:9200

# Log retention
LOG_RETENTION_DAYS=90
LOG_ROTATION_SIZE=100MB
```

### Performance Monitoring

#### Metrics Collection
```bash
# Enable performance metrics
PROXEMIC_PERFORMANCE_MONITORING=true

# Metrics endpoint (if using Prometheus)
METRICS_PORT=9090
METRICS_PATH=/metrics
```

#### Alerting Configuration
```bash
# Security alert thresholds
PROXEMIC_SECURITY_ALERT_THRESHOLD=10

# Alert methods
PROXEMIC_ALERT_METHOD=log,email

# Email alert configuration
ALERT_EMAIL_FROM=alerts@yourdomain.com
ALERT_EMAIL_TO=security-team@yourdomain.com
```

## Troubleshooting

### Common Deployment Issues

#### Build Failures
**Problem**: Rust compilation errors
**Solution**:
```bash
# Update Rust toolchain
rustup update stable

# Clean build artifacts
cargo clean
npm run clean

# Verify dependencies
cargo check
npm audit
```

**Problem**: Tauri dependency issues
**Solution**:
```bash
# Reinstall Tauri CLI
npm install -g @tauri-apps/cli@latest

# Update Tauri dependencies
npm update @tauri-apps/api @tauri-apps/cli
```

#### Runtime Issues

**Problem**: Application fails to start
**Solution**:
```bash
# Check environment variables
echo $PROXEMIC_ENV
echo $DATABASE_URL

# Verify file permissions
ls -la ./Proxemic.db

# Check log files
tail -f ./logs/proxemic.log
```

**Problem**: File operation permissions
**Solution**:
```bash
# Verify workspace directories exist
mkdir -p ~/Desktop ~/Documents ~/Downloads

# Check file system permissions
chmod 755 ~/Desktop ~/Documents ~/Downloads
```

### Security-related Issues

**Problem**: Magic number validation failures
**Solution**:
```bash
# Check file signatures
file suspicious_file.txt

# Verify MIME types
mimetype suspicious_file.txt

# Temporary disable for troubleshooting (development only)
PROXEMIC_ENABLE_MAGIC_VALIDATION=false
```

**Problem**: Workspace boundary violations
**Solution**:
```bash
# Verify current working directory
pwd

# Check allowed directories configuration
echo $HOME/Desktop
echo $HOME/Documents
echo $HOME/Downloads
```

### Performance Issues

**Problem**: High memory usage
**Solution**:
```bash
# Reduce batch sizes
MAX_EMBEDDING_BATCH_SIZE=25

# Limit concurrent operations
PROXEMIC_MAX_CONCURRENT_OPERATIONS=5

# Enable memory monitoring
PROXEMIC_PERFORMANCE_MONITORING=true
```

**Problem**: Slow file operations
**Solution**:
```bash
# Check disk I/O
iostat -x 1

# Verify file system health
fsck /dev/your_disk

# Consider SSD storage for vector index
VECTOR_INDEX_PATH=/ssd/vector_index
```

## CI/CD Integration

### GitHub Actions Workflow

#### Continuous Integration
```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Tauri dependencies
        run: sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev
      - name: Run tests
        run: cargo test
        working-directory: src-tauri
```

#### Release Builds
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build-tauri:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - name: Build Tauri app
        run: npm run tauri build
      - name: Upload Release Assets
        uses: actions/upload-artifact@v4
        with:
          name: proxemic-${{ matrix.platform }}
          path: src-tauri/target/release/bundle/**
```

### Security Scanning

#### Dependency Scanning
```bash
# npm audit
npm audit --production

# cargo audit
cargo install cargo-audit
cargo audit

# Snyk integration
npm install -g snyk
snyk test
```

#### Code Quality
```bash
# ESLint
npm run lint

# Rust clippy
cargo clippy -- -D warnings

# Format checking
npm run format:check
cargo fmt --check
```

### Deployment Validation

#### Pre-deployment Checks
```bash
# Configuration validation
cargo run --bin config-validator

# Security audit
cargo run --bin security-audit

# Performance baseline
cargo run --bin performance-benchmark
```

#### Post-deployment Verification
```bash
# Health check
curl http://localhost:3000/health

# Metrics validation
curl http://localhost:9090/metrics

# Log verification
tail -n 100 ./logs/proxemic.log | grep -i error
```

## Appendix

### Environment Variable Reference

| Variable | Description | Default | Required |
|----------|-------------|---------|-----------|
| `PROXEMIC_ENV` | Environment type | `development` | Yes |
| `RUST_LOG` | Log level | `info` | No |
| `PROXEMIC_DEBUG` | Debug mode | `false` | No |
| `OPENROUTER_API_KEY` | OpenRouter API key | - | Yes* |
| `ANTHROPIC_API_KEY` | Anthropic API key | - | No |
| `DATABASE_URL` | Database connection | `sqlite:./Proxemic.db` | Yes |
| `VECTOR_INDEX_PATH` | Vector storage path | `./vector_index` | Yes |
| `PROXEMIC_SECURITY_LEVEL` | Security level | `development` | Yes |
| `PROXEMIC_MAX_FILE_SIZE` | Max file size | `104857600` | Yes |
| `PROXEMIC_MAX_PATH_LENGTH` | Max path length | `260` | Yes |

*Required for AI features

### Security Level Definitions

- **development**: Lenient security for testing
- **production**: Standard security for production use
- **high_security**: Enhanced security for sensitive environments

### File Size Limits

| Environment | Max File Size | Recommendation |
|-------------|---------------|----------------|
| Development | 100MB | Suitable for testing |
| Production | 100MB | Balanced security/functionality |
| High Security | 50MB | Enhanced protection |

### Support Resources

- [GitHub Issues](https://github.com/lukosejoseph007/Proxemic/issues)
- [Documentation](https://github.com/lukosejoseph007/Proxemic/docs)
- [Security Advisories](https://github.com/lukosejoseph007/Proxemic/security)

### Version Compatibility

| Component | Minimum Version | Recommended |
|-----------|----------------|-------------|
| Rust | 1.70 | 1.75+ |
| Node.js | 18 | 20+ |
| Tauri | 2.0 | 2.4+ |
| npm | 8 | 10+ |
