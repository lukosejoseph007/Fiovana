# Proxemic Development Guide

## Development Workflows

### Building the Application
```bash
# Install dependencies
npm install

# Build frontend and backend
npm run build

# Build Tauri application
cargo tauri build
```

### Running the Application
```bash
# Start development server (frontend on :3000, backend on :1420)
npm run tauri dev
```

### Testing
```bash
# Run Rust tests
cd src-tauri && cargo test

# Run frontend tests
npm run test

# Run lint checks
npm run lint
npm run format:check
```

### Debugging
1. Set breakpoints in VSCode:
   - For Rust: Use the "Rust Backend Debug" configuration
   - For TypeScript: Use the "Tauri Development Debug" configuration
2. Start debugging session in VSCode

### Hot Reload
- Frontend changes: Automatically reloads when saving files
- Backend changes: Requires restart of `tauri dev` process

### Environment Configuration
Create `.env` file with:
```env
OPENROUTER_API_KEY=your_key
ANTHROPIC_API_KEY=your_key
RUST_LOG=info
TAURI_DEV_PORT=3000
DATABASE_URL=sqlite:./Proxemic.db
```

## CI/CD Pipeline
Workflows located in `.github/workflows`:
- `ci.yml`: Main CI pipeline
- `security.yml`: Security scans
- `release.yml`: Release automation

## Directory Structure
```
src/              # Frontend (React/TypeScript)
src-tauri/        # Backend (Rust)
  src/            # Rust modules
    ai/           # AI integration
    vector/       # Vector search
    document/     # Document processing
projectinfo/      # Project documentation
```

## Troubleshooting
```bash
# Fix dependency issues
cargo clean && npm cache clean --force
npm install && cd src-tauri && cargo update

# Reset database
rm Proxemic.db
