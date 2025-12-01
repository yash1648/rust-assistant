# 🎙️ Piper Female — Voice Assistant

[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](#license)
[![Status](https://img.shields.io/badge/status-active-success)](#)

A sophisticated Rust application that transforms voice input into intelligent conversational responses with natural speech output. This is a fully-functional voice assistant running entirely on your local machine.

> **Intelligent. Private. Yours.**

## ✨ Core Features

- **🎧 Speech Recognition** — Powered by `whisper.cpp` for accurate speech-to-text
- **🤖 AI Responses** — Leverages `Ollama` with Gemma 3.x LLM models
- **🗣️ Natural Speech** — High-quality female voice synthesis via `piper-tts`
- **🔒 Complete Privacy** — All processing occurs locally; no cloud dependencies
- **⚡ Low Latency** — Optimized async Rust runtime for responsive interactions

## 🔄 How It Works

The voice assistant follows a streamlined pipeline:

```
Speak → Record Audio → Transcribe → Process with LLM → Synthesize Speech → Play Response
```

### Runtime Flow

1. **Audio Capture**: Records microphone input to `user_input.wav` using `cpal` and `hound`
2. **Transcription**: Processes the WAV file with `whisper.cpp` to extract text
3. **LLM Processing**: Sends conversation history to `Ollama` (Gemma 3.x) for intelligent responses
4. **Speech Synthesis**: Generates spoken response using `piper-tts` with the Cori voice model
5. **Audio Playback**: Outputs `assistant_response.wav` through system speakers via `rodio`

### Architecture Overview

| Module | Purpose |
|--------|---------|
| `assistant` | Orchestrates the conversation loop and maintains chat history |
| `stt` | Speech-to-text recording and transcription pipeline |
| `tts` | Text-to-speech synthesis and audio playback |

## 📋 System Requirements

### Hardware
- **Processor**: Intel/AMD x86-64 or Apple Silicon (M1+)
- **RAM**: 4GB minimum (8GB+ recommended for Gemma models)
- **Storage**: 2GB+ for models and dependencies
- **Audio**: Working microphone and speaker configuration

### Software
- **OS**: Linux, macOS 11+, or Windows 10/11
- **Rust**: 1.80 or later (via [rustup.rs](https://rustup.rs))
- **Build Tools**: CMake, GCC/Clang (for whisper.cpp compilation)

### Dependencies
```toml
anyhow = "1"
reqwest = { version = "0.12", features = ["json"] }
rodio = "0.17"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cpal = "0.15"
hound = "3"
tokio = { version = "1", features = ["full"] }
```

## 🚀 Quick Start

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install Dependencies

**Linux (Ubuntu/Debian)**
```bash
sudo apt-get update
sudo apt-get install -y build-essential cmake git
pip install piper-tts  # or build from source
```

**macOS**
```bash
brew install cmake piper-tts
```

**Windows**
- Download Piper from [releases](https://github.com/rhasspy/piper/releases)
- Add to `PATH`

### 3. Set Up Whisper

```bash
git clone https://github.com/ggerganov/whisper.cpp repos/whisper.cpp
cd repos/whisper.cpp && mkdir build && cd build
cmake .. && make -j$(nproc)
cd ../..

# Download a model
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin \
  -O models/ggml-base.en.bin
```

### 4. Install & Run Ollama

```bash
# Install from https://ollama.com
ollama serve &

# In another terminal, pull Gemma
ollama pull gemma3
```

### 5. Build & Run

```bash
cargo run
```

When prompted, speak naturally. Say **"exit"** to quit.

## ⚙️ Configuration

### Piper Binary Location
Edit `src/tts/engine.rs:9`:
```rust
pub const PIPER_BIN: &str = "piper-tts";  // Change to "piper" if needed
```

### Whisper Model Path
Edit `src/assistant/conversation.rs:76-87`:
```rust
let text = stt::transcriber::transcribe_with_whisper(
    "./repos/whisper.cpp",
    "models/ggml-base.en.bin",
    "user_input.wav",
)?;
```

### Ollama Endpoint
Edit `src/assistant/llm.rs:26`:
```rust
.post("http://localhost:11434/api/chat")
```

Change `localhost` if Ollama runs on a different host.

## 📁 Project Structure

```
rust-assistant/
├── src/
│   ├── main.rs
│   ├── assistant/
│   │   ├── mod.rs
│   │   ├── conversation.rs    # Main loop & orchestration
│   │   └── llm.rs             # Ollama API client
│   ├── stt/
│   │   ├── mod.rs
│   │   ├── audio.rs           # Microphone detection
│   │   ├── recorder.rs        # Audio capture to WAV
│   │   └── transcriber.rs     # Whisper.cpp integration
│   └── tts/
│       ├── mod.rs
│       ├── engine.rs          # Piper synthesis & playback
│       ├── voice.rs           # Voice catalog
│       └── models.rs          # Model caching
├── Cargo.toml
├── models/                    # Whisper & Piper models
├── repos/whisper.cpp/         # Cloned whisper.cpp repo
└── docs/
    ├── README.md
    ├── installation.md
    ├── technical.md
    ├── troubleshooting.md
    ├── development.md
    └── faq.md
```

## 🔧 Troubleshooting

| Issue | Solution |
|-------|----------|
| **No microphone detected** | Ensure a default input device is configured in system settings |
| **Whisper binary not found** | Build whisper.cpp and verify paths in `src/stt/transcriber.rs` |
| **Piper errors** | Confirm `piper-tts` is in `PATH` and executable |
| **Ollama connection failed** | Verify `ollama serve` is running and endpoint is reachable |
| **No audio playback** | Check output device settings and audio permissions |

See the complete [Troubleshooting Guide](./docs/troubleshooting.md) for detailed diagnostics.

## 📚 Documentation

- **[Installation Guide](./docs/installation.md)** — Platform-specific setup
- **[Technical Architecture](./docs/technical.md)** — Deep dive into the codebase
- **[Development Guide](./docs/development.md)** — Contributing and extending
- **[FAQ](./docs/faq.md)** — Common questions and answers
- **[Roadmap](./docs/roadmap.md)** — Planned improvements
- **[Known Issues](./docs/known-issues.md)** — Current limitations

## 🚦 Getting Started Next Steps

1. ✅ Install all dependencies (see Quick Start)
2. ✅ Configure paths in source files (see Configuration)
3. ✅ Run `cargo run` and test with a short phrase
4. ✅ Refer to [docs](./docs) for advanced usage

## 🤝 Contributing

Contributions are welcome! Please:
- Keep commits focused and descriptive
- Follow Rust idioms and style guidelines
- Run `cargo fmt` and `cargo clippy` before submitting

See [Development Guide](./docs/development.md) for details.

## 📄 License

This project is licensed under the **MIT License**. See [LICENSE](./LICENSE) file for details.

## 🎯 Roadmap Highlights

- Configuration via environment variables or `config.toml`
- Extended voice catalog with runtime selection
- Streaming STT/TTS for real-time responsiveness
- Cross-platform device selection UI
- Comprehensive test suite and CI/CD

See the full [Roadmap](./docs/roadmap.md) for more.

---

**Questions?** Check the [FAQ](./docs/faq.md) or open an issue on GitHub.

**Last Updated**: December 2025