# Proxemic - AI-Powered Content Intelligence Platform

## Getting Started

### Prerequisites
- Rust (>=1.70)
- Node.js (>=18.0)
- Tauri CLI (`cargo install tauri-cli`)

### Installation
1. Clone the repository: `git clone https://github.com/lukosejoseph007/Proxemic.git`
2. Install dependencies: `npm install`
3. Build and run: `cargo tauri dev`

### Usage
- Import documents via drag-and-drop
- Use the chat interface to analyze and transform content
- Generate multi-format outputs (Word, PDF, HTML)
- Collaborate in real-time with team members

### Configuration
Set environment variables in `.env`:
```env
# AI Service Configuration
OPENROUTER_API_KEY=your_openrouter_api_key
ANTHROPIC_API_KEY=your_anthropic_api_key

# Development Settings
RUST_LOG=info
TAURI_DEV_PORT=3000
```

## Key Features
- Semantic document analysis
- Style-preserving content updates
- Enterprise integration (SharePoint, Google Drive)
- Real-time collaboration
- Multi-format output generation
