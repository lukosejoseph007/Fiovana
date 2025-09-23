# Ollama Setup Instructions for Proxemic

## Overview
Proxemic uses Ollama for local AI processing. This guide will help you set up Ollama to enable the AI features in Proxemic.

## Installation

### Option 1: Automatic Installation (Recommended)
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

### Option 2: Manual Installation

#### Linux
```bash
# Download the binary
curl -L https://ollama.ai/download/ollama-linux-amd64 -o ollama
chmod +x ollama
sudo mv ollama /usr/local/bin/
```

#### macOS
```bash
# Download and install
curl -L https://ollama.ai/download/ollama-darwin -o ollama
chmod +x ollama
sudo mv ollama /usr/local/bin/
```

#### Windows
1. Download the Windows installer from https://ollama.ai/download
2. Run the installer
3. Follow the installation wizard

## Starting Ollama

### Method 1: Start as service (Recommended)
```bash
# Start Ollama service
ollama serve

# Or if you want it to run in the background
nohup ollama serve > ollama.log 2>&1 &
```

### Method 2: System service (Linux)
```bash
# Create systemd service (optional)
sudo tee /etc/systemd/system/ollama.service > /dev/null <<EOF
[Unit]
Description=Ollama AI Server
After=network.target

[Service]
Type=simple
User=ollama
Group=ollama
ExecStart=/usr/local/bin/ollama serve
Environment="HOME=/home/ollama"
Environment="PATH=/usr/local/bin:/usr/bin:/bin"
Restart=always
RestartSec=3
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Create ollama user
sudo useradd -r -s /bin/false -d /home/ollama ollama
sudo mkdir -p /home/ollama
sudo chown ollama:ollama /home/ollama

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable ollama
sudo systemctl start ollama
```

## Installing Models

### Recommended Models for Proxemic

1. **llama3.2-3b** (Default - Fast, good for most tasks)
```bash
ollama pull llama3.2-3b
```

2. **llama3.1-8b** (Better quality, slower)
```bash
ollama pull llama3.1-8b
```

3. **codellama** (For code-related tasks)
```bash
ollama pull codellama
```

4. **mistral** (Alternative general-purpose model)
```bash
ollama pull mistral
```

### Install from Proxemic UI
You can also install models directly from the Proxemic AI settings page.

## Verification

### Check if Ollama is running
```bash
# Check if service is running
curl http://localhost:11434/api/tags

# Should return JSON with available models
```

### Test in Proxemic
1. Open Proxemic
2. Go to Settings → AI Configuration
3. Select "Local (Ollama)" as provider
4. Select a model from the dropdown
5. Test the connection

## Troubleshooting

### Ollama not starting
```bash
# Check if port 11434 is available
netstat -tlnp | grep 11434

# Kill any processes using the port
sudo lsof -ti:11434 | xargs sudo kill -9

# Restart ollama
ollama serve
```

### Models not loading
```bash
# Check available models
ollama list

# Pull a model if missing
ollama pull llama3.2-3b

# Check Ollama logs
journalctl -u ollama -f
```

### Performance issues
```bash
# Check system resources
htop

# Monitor Ollama usage
ollama ps

# Reduce concurrent requests in Proxemic settings
```

### Proxemic can't connect to Ollama
1. Verify Ollama is running: `curl http://localhost:11434/api/tags`
2. Check firewall settings
3. Restart both Ollama and Proxemic
4. Check Proxemic logs for specific error messages

## Alternative AI Providers

If you can't run Ollama locally, Proxemic also supports:

1. **OpenRouter** - Access to many models via API
2. **Anthropic** - Claude models via API

Configure these in Settings → AI Configuration.

## Performance Tips

1. **RAM Requirements**:
   - 3B models: 4GB RAM minimum
   - 8B models: 8GB RAM minimum
   - 13B+ models: 16GB RAM minimum

2. **GPU Acceleration**: Ollama automatically uses GPU if available (NVIDIA/AMD)

3. **Model Selection**:
   - Use smaller models (3B) for faster responses
   - Use larger models (8B+) for better quality

4. **Concurrent Requests**: Limit to 1-2 concurrent requests for smaller models

## Support

For Ollama-specific issues, check:
- Ollama GitHub: https://github.com/ollama/ollama
- Ollama Documentation: https://github.com/ollama/ollama/blob/main/README.md

For Proxemic integration issues, check the application logs in the Settings → Debug section.