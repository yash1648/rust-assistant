# 📦 Installation Guide

Complete platform-specific setup instructions for the Piper Female voice assistant.

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Install Rust](#step-1-install-rust)
- [Step 2: Install Piper TTS](#step-2-install-piper-tts)
- [Step 3: Build Whisper.cpp](#step-3-buildwhisper)
- [Step 4: Set Up Ollama](#step-4-set-up-ollama)
- [Step 5: Configure the Project](#step-5-configure-the-project)
- [Step 6: Build & Verify](#step-6-build--verify)
- [Troubleshooting](#troubleshooting)

## ✅ Prerequisites

### System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **OS** | Linux (Ubuntu 20.04+), macOS 11+, Windows 10 | Recent stable release |
| **RAM** | 4 GB | 8 GB+ |
| **Storage** | 2 GB free | 5 GB+ |
| **CPU** | 2-core x86-64 or ARM64 | 4+ core modern processor |
| **Audio** | USB microphone or built-in | Dedicated audio interface |

### Dependency Versions

```
Rust:        1.80+ (stable recommended)
Python:      3.8+ (for piper-tts)
CMake:       3.16+
Git:         2.20+
Ollama:      0.1+ (any recent version)
```

---

## Step 1: Install Rust

### Linux / macOS

```bash
# Download and run the Rust installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should print "rustc 1.xx.x"
cargo --version
```

### Windows 11

1. Download `rustup-init.exe` from [rustup.rs](https://rustup.rs)
2. Run the installer and follow prompts
3. Verify in PowerShell:
   ```powershell
   rustc --version
   ```

### Update Rust (All Platforms)

```bash
rustup update stable
```

---

## Step 2: Install Piper TTS

### Linux (Ubuntu / Debian)

**Option A: Package Manager**
```bash
sudo apt-get update
sudo apt-get install -y piper-tts
piper --version
```

**Option B: Build from Source**
```bash
git clone https://github.com/rhasspy/piper
cd piper
cd src/cpp && ./build.sh
# Binary: ./piper
```

**Option C: Pre-built Binary**
```bash
# Download from releases
wget https://github.com/rhasspy/piper/releases/download/2024.1/piper_linux_x86_64.tar.gz
tar xzf piper_linux_x86_64.tar.gz
export PATH="$PATH:$(pwd)/piper"
```

### macOS

```bash
# Using Homebrew (recommended)
brew install piper-tts

# Verify
piper-tts --version
# or
piper --version
```

### Windows 11

1. Download the latest release: [Piper Releases](https://github.com/rhasspy/piper/releases)
2. Extract to a folder (e.g., `C:\piper`)
3. Add to PATH:
   - Open **System Properties** → **Environment Variables**
   - Click **New** (under System Variables)
   - Variable Name: `PATH`
   - Variable Value: `C:\piper`
   - Click **OK** and restart terminal

4. Verify:
   ```powershell
   piper-tts --version
   ```

---

## Step 3: Build Whisper

Whisper.cpp requires compilation from source. Follow these steps:

### Clone Repository

```bash
# Navigate to project root
cd /path/to/rust-assistant

# Clone whisper.cpp
git clone https://github.com/ggerganov/whisper.cpp repos/whisper.cpp
cd repos/whisper.cpp
```

### Linux / macOS

```bash
# Create build directory
mkdir build && cd build

# Configure with CMake
cmake ..

# Build (using all available cores)
make -j$(nproc)  # Linux
make -j$(sysctl -n hw.ncpu)  # macOS

# Verify binary was created
ls bin/main  # Should exist
```

### Windows 11 (PowerShell)

```powershell
# Create build directory
mkdir build; cd build

# Configure with CMake
cmake .. -G "Visual Studio 17 2022"

# Build
cmake --build . --config Release

# Verify
ls build\Release\main.exe
```

### Download Model

```bash
# Navigate back to project root
cd /path/to/rust-assistant

# Create models directory
mkdir -p models

# Download the base model (~140 MB)
# Linux/macOS:
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin \
  -O models/ggml-base.en.bin

# Windows (PowerShell):
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" `
  -OutFile "models/ggml-base.en.bin"
```

> **Tip**: Available models: `ggml-tiny.en.bin` (75M, faster), `ggml-base.en.bin` (140M, balanced), `ggml-small.en.bin` (466M, more accurate)

---

## Step 4: Set Up Ollama

Ollama provides the LLM inference engine.

### Installation

**Linux**
```bash
# Download and install
curl -fsSL https://ollama.ai/install.sh | sh

# Start the service
ollama serve
```

**macOS**
```bash
# Download from https://ollama.com
# Or use Homebrew:
brew install ollama

# Start in the background
ollama serve &
```

**Windows**
1. Download from [ollama.com](https://ollama.com)
2. Run the installer
3. Open new PowerShell and run:
   ```powershell
   ollama serve
   ```

### Pull Gemma Model

Open a **new terminal** (keep `ollama serve` running) and run:

```bash
# Pull the Gemma 3.x model (~5-8 GB)
ollama pull gemma3

# Verify it's installed
ollama list
```

> **Alternative Models**: Try `ollama pull llama3` or `ollama pull mistral` for different responses

### Test Ollama

```bash
# Quick test (in new terminal)
curl http://localhost:11434/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gemma3",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": false
  }'
```

Expected response: JSON with the model's reply.

---

## Step 5: Configure the Project

Edit source files to match your setup:

### Update Piper Binary Name

**File**: `src/tts/engine.rs` (line 9)

```rust
// Change this to match your installation:
pub const PIPER_BIN: &str = "piper-tts";  // or "piper" if that's what you have
```

### Update Whisper Path (if needed)

**File**: `src/assistant/conversation.rs` (lines 76-87)

```rust
// Adjust paths if you cloned whisper.cpp elsewhere:
let text = stt::transcriber::transcribe_with_whisper(
    "./repos/whisper.cpp",              // Path to cloned repo
    "models/ggml-base.en.bin",          // Path to downloaded model
    "user_input.wav",
)?;
```

### Update Ollama Endpoint (if not localhost)

**File**: `src/assistant/llm.rs` (line 26)

```rust
// If Ollama runs on a remote machine:
.post("http://192.168.1.100:11434/api/chat")  // Change IP as needed
```

---

## Step 6: Build & Verify

### Clone and Build

```bash
# Clone the repository (if not already done)
git clone https://github.com/yash1648/rust-assistant
cd rust-assistant

# Build the project
cargo build --release

# This generates: target/release/rust_assistant (or .exe on Windows)
```

### Run the Application

```bash
# Simple way (debug mode, slower)
cargo run

# Or run the release binary (faster)
./target/release/rust_assistant
# Windows: .\target\release\rust_assistant.exe
```

### First Run

1. You should see:
   ```
   🎙️  Listening... Speak now! (or say 'exit' to quit)
   ```

2. **Speak clearly** into your microphone (5-10 second phrase)

3. The assistant will:
   - Record your audio to `user_input.wav`
   - Print your transcription
   - Show the LLM response
   - Play the spoken reply

4. Say **"exit"** to quit

### Download Voice Models

On first run, Piper downloads the Cori voice model (~50-100 MB) to `./models/`. This is automatic.

---

## 🔍 Troubleshooting

### "Piper not found" Error

**Problem**: Binary not in PATH  
**Solution**:
- Verify `piper-tts --version` works in terminal
- Update `PIPER_BIN` in `src/tts/engine.rs`
- On Windows, restart terminal or computer after adding to PATH

### "Whisper binary not found"

**Problem**: Build failed or wrong path  
**Solution**:
```bash
# Verify binary exists:
ls repos/whisper.cpp/build/bin/  # Linux/macOS
dir repos\whisper.cpp\build\Release\  # Windows

# If missing, rebuild:
cd repos/whisper.cpp/build
make clean && make -j
```

### "No microphone detected"

**Problem**: System audio not configured  
**Solution**:
- Test with: `arecord test.wav` (Linux) or `recordaudio test.wav` (macOS)
- In system settings, select a default input device
- Check microphone volume isn't muted

### "Ollama connection refused"

**Problem**: Server not running  
**Solution**:
```bash
# Start Ollama server (in new terminal)
ollama serve

# Test connectivity:
curl http://localhost:11434/api/tags
```

### "Model not found" (Gemma)

**Problem**: Model not pulled  
**Solution**:
```bash
# Pull the model
ollama pull gemma3

# List installed models
ollama list
```

### Windows: "python not found"

**Problem**: Piper needs Python  
**Solution**:
```powershell
# Install Python via Microsoft Store or python.org
python --version  # Verify installation
```

---

## 📝 Best Practices

1. **Keep dependencies updated** (but within Cargo.toml major versions)
2. **Use a dedicated microphone** for better transcription quality
3. **Test each component independently**:
   - Record a WAV: `arecord test.wav`
   - Transcribe: `./repos/whisper.cpp/build/bin/main -m models/ggml-base.en.bin -f test.wav`
   - Query Ollama: `curl http://localhost:11434/api/tags`
   - Synthesize: `echo "hello" | piper-tts --model en_GB-cori-high --output_file test.wav`

4. **Monitor model storage**: Downloaded models can use 1-2 GB
5. **Refactor config to env vars** for easier deployment

---

## 🔗 Navigation

- [README](./README.md) — Project overview
- [Technical Architecture](./technical.md) — Deep dive
- [Development](./development.md) — Contributing
- [Troubleshooting](./troubleshooting.md) — Detailed diagnostics
- [FAQ](./faq.md) — Quick answers
- [Roadmap](./roadmap.md) — Future plans

---

**Stuck?** Check [Troubleshooting](./troubleshooting.md) or open a GitHub issue.

**Last Updated**: December 2025