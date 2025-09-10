# Proxemic - Secure AI-Powered Document Intelligence Platform

## Overview
Proxemic is a secure desktop application built with Tauri (Rust + TypeScript) that provides AI-powered document analysis and processing with enterprise-grade security features.

## Getting Started

### Prerequisites
- **Rust** (>=1.70) - [Install Rust](https://rustup.rs/)
- **Node.js** (>=18.0) - [Install Node.js](https://nodejs.org/)
- **Tauri CLI** (`cargo install tauri-cli@^2.0.0`)
- **Git** - For version control

### Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/lukosejoseph007/Proxemic.git
   cd Proxemic
   ```

2. Install frontend dependencies:
   ```bash
   npm install
   ```

3. Install Rust dependencies (if not already installed):
   ```bash
   cargo build
   ```

4. Build and run in development mode:
   ```bash
   cargo tauri dev
   ```

### Production Build
```bash
# Build for current platform
cargo tauri build

# Build for specific target
cargo tauri build --target x86_64-pc-windows-msvc
```

## Configuration

### Environment Variables
Copy `.env.example` to `.env` and configure:

```env
# AI Service Configuration
OPENROUTER_API_KEY=your_openrouter_api_key_here
ANTHROPIC_API_KEY=your_anthropic_api_key_here

# Development Configuration
RUST_LOG=info
TAURI_DEV_PORT=3000

# Database Configuration
DATABASE_URL=sqlite:./Proxemic.db

# Security Configuration (Production Hardened)
PROXEMIC_SECURITY_LEVEL=development
PROXEMIC_MAX_FILE_SIZE=104857600
PROXEMIC_MAX_PATH_LENGTH=260
PROXEMIC_MAX_CONCURRENT_OPERATIONS=10
PROXEMIC_ENABLE_MAGIC_VALIDATION=true
PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES=true
PROXEMIC_AUDIT_LOGGING_ENABLED=true
```

## Key Features

### 🔒 Security Features
- **Magic Number Validation** - File type detection and validation
- **Workspace Boundary Enforcement** - Restricts file operations to designated areas
- **Audit Logging** - Comprehensive security event tracking
- **Content Security Scanning** - Optional malicious content detection
- **Rate Limiting** - Prevents abuse and DoS attacks
- **Configuration Validation** - Strict security configuration checks

### 🤖 AI-Powered Capabilities
- Semantic document analysis and processing
- Style-preserving content updates and transformations
- Vector search and embedding capabilities
- Multi-format output generation (Word, PDF, HTML)

### 🛠️ Development Features
- Comprehensive test suite with integration tests
- Performance benchmarking
- Security integration testing
- Log rotation and management
- TypeScript with React frontend
- Rust backend with Tauri integration

## Development

### Project Structure
```
proxemic/
├── src/                 # Frontend (TypeScript/React)
├── src-tauri/          # Backend (Rust)
├── docs/               # Documentation
├── tests/              # Test files
└── projectinfo/        # Project information
```

### Available Scripts
- `npm run dev` - Start development server
- `npm run build` - Build frontend
- `npm run tauri` - Run Tauri commands
- `npm run test` - Run tests
- `npm run lint` - Run ESLint
- `npm run format` - Format code with Prettier

### Testing
```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test integration_tests
cargo test --test performance_benchmarks
cargo test --test json_schema_validation
```

## Documentation

- [Deployment Guide](./docs/DEPLOYMENT.md) - Production deployment instructions
- [Security Documentation](./docs/SECURITY.md) - Security features and configuration
- Development guidelines and best practices

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:
- Check the documentation
- Review existing issues
- Create a new issue with detailed information
