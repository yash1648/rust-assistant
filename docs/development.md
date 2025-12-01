# 🛠️ Development Guide

Guidelines for contributing to and extending the Piper Female voice assistant.

## 📋 Table of Contents

- [Development Environment](#development-environment)
- [Code Style](#code-style)
- [Testing](#testing)
- [Common Tasks](#common-tasks)
- [Deployment](#deployment)
- [FAQ](#faq)

---

## 🔨 Development Environment

### Setup

```bash
# Clone the repository
git clone https://github.com/yash1648/rust-assistant
cd rust-assistant

# Install Rust (if not already done)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Open in your favorite editor
code .  # VS Code
# or
nvim src/main.rs  # Neovim
```

### Recommended Tools

| Tool | Purpose | Install |
|------|---------|---------|
| **rust-analyzer** | IDE support | VS Code extension |
| **cargo-watch** | Auto-rebuild on changes | `cargo install cargo-watch` |
| **cargo-edit** | Manage dependencies | `cargo install cargo-edit` |
| **clippy** | Linting | Included with rustup |

### Quick Commands

```bash
# Format code
cargo fmt --all

# Check for issues
cargo clippy --all-targets --all-features

# Build (debug)
cargo build

# Build (optimized release)
cargo build --release

# Run with debug output
RUST_BACKTRACE=1 cargo run

# Watch for changes (requires cargo-watch)
cargo watch -x run
```

---

## 🎨 Code Style

### Rust Conventions

Follow standard Rust idioms:

```rust
// ✅ Good: Use idiomatic error handling
fn transcribe(path: &str) -> anyhow::Result<String> {
    let output = Command::new("whisper")
        .arg(path)
        .output()
        .context("failed to run whisper")?;
    
    // ...
    Ok(text)
}

// ❌ Avoid: Unwrapping or panicking in library code
fn transcribe(path: &str) -> String {
    Command::new("whisper")
        .arg(path)
        .output()
        .unwrap()  // Bad!
        // ...
}
```

### Module Organization

```rust
// src/assistant/mod.rs
pub mod conversation;
pub mod llm;

pub use conversation::run;
pub use llm::chat_with_ollama;
```

### Documentation

```rust
/// Transcribes audio using whisper.cpp.
///
/// # Arguments
/// * `whisper_path` - Path to whisper.cpp repository
/// * `model_path` - Path to GGML model file
/// * `audio_path` - Path to WAV file to transcribe
///
/// # Returns
/// The transcribed text, or an error if transcription failed.
///
/// # Example
/// ```
/// let text = transcribe_with_whisper(
///     "./repos/whisper.cpp",
///     "models/ggml-base.en.bin",
///     "input.wav"
/// )?;
/// ```
pub fn transcribe_with_whisper(
    whisper_path: &str,
    model_path: &str,
    audio_path: &str,
) -> anyhow::Result<String> {
    // ...
}
```

### Format Before Committing

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 🧪 Testing

### Current State

The project uses **manual testing** during development. Automated tests are planned.

### Manual Testing Workflow

```bash
# 1. Start Ollama
ollama serve &

# 2. Build and run
cargo run

# 3. Test a phrase
🎙️ Listening...
> *Speak: "Hello, what time is it?"*

# 4. Verify each step
✅ user_input.wav created
✅ Transcription: "Hello, what time is it?"
✅ Ollama response received
✅ assistant_response.wav created
✅ Audio played back
```

### Recommended Test Cases

1. **Happy Path**:
   - Record clear speech
   - Expect accurate transcription
   - Get relevant LLM response
   - Hear natural speech output

2. **Edge Cases**:
   - Whisper with background noise
   - Very short utterance ("ok")
   - Very long utterance (>30 seconds)
   - Say "exit" to quit cleanly

3. **Component Testing**:
   - Microphone detection
   - Model file downloads
   - Ollama connectivity
   - Speaker output

### Future: Automated Tests

Structure for upcoming test suite:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcriber_with_fixture() {
        // Create a test WAV file
        let text = transcribe_with_whisper(
            "./repos/whisper.cpp",
            "models/ggml-base.en.bin",
            "test_fixtures/hello.wav",
        ).unwrap();
        
        assert!(text.to_lowercase().contains("hello"));
    }

    #[tokio::test]
    async fn test_ollama_mock() {
        // Mock HTTP response
        let response = chat_with_ollama(
            &[Message {
                role: "user".to_string(),
                content: "test".to_string(),
            }],
        ).await.unwrap();
        
        assert_eq!(response.message.role, "assistant");
    }

    #[test]
    fn test_piper_synthesis() {
        synthesize("Hello world").unwrap();
        assert!(Path::new("assistant_response.wav").exists());
    }
}
```

---

## 🔧 Common Development Tasks

### Add a New Voice

**File**: `src/tts/voice.rs`

```rust
pub fn voices() -> Vec<&'static str> {
    vec![
        "en_GB-cori-high",      // Current
        "en_US-lessac-high",    // New
        "en_GB-alba-medium",    // New
    ]
}
```

### Support a Different LLM

**File**: `src/assistant/llm.rs`

```rust
pub async fn chat_with_ollama(
    messages: &[Message],
) -> anyhow::Result<ChatResponse> {
    // Change model here:
    let json = serde_json::json!({
        "model": "llama3",  // Was "gemma3"
        "messages": messages,
        "stream": false,
    });
    
    // ... rest of function
}
```

### Make Config Externalized

**Goal**: Read from environment or `config.toml`

```rust
// In src/lib.rs or src/config.rs
pub struct Config {
    pub piper_bin: String,
    pub whisper_path: String,
    pub whisper_model: String,
    pub ollama_endpoint: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            piper_bin: std::env::var("PIPER_BIN")
                .unwrap_or_else(|_| "piper-tts".to_string()),
            whisper_path: std::env::var("WHISPER_PATH")
                .unwrap_or_else(|_| "./repos/whisper.cpp".to_string()),
            // ... etc
        }
    }
}
```

### Add Logging

Add `env_logger` to `Cargo.toml`:

```toml
[dependencies]
env_logger = "0.11"
log = "0.4"
```

In `src/main.rs`:

```rust
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .init();
    
    log::info!("Starting voice assistant");
    // ...
}
```

Then run with:

```bash
RUST_LOG=debug cargo run
```

---

## 📦 Deployment

### Build Release Binary

```bash
# Optimized, production-ready binary
cargo build --release

# Binary location:
# - Linux/macOS: ./target/release/rust_assistant
# - Windows: .\target\release\rust_assistant.exe
```

### Runtime Dependencies Checklist

Before distributing, ensure users have:

- [ ] Rust (or prebuilt binary)
- [ ] `piper-tts` binary in PATH
- [ ] `whisper.cpp` built at `./repos/whisper.cpp`
- [ ] Whisper model at `./models/ggml-base.en.bin`
- [ ] `ollama serve` running
- [ ] Gemma model pulled (`ollama pull gemma3`)
- [ ] Microphone and speaker configured

### Distribution Package (Linux Example)

```bash
#!/bin/bash
# build-release.sh

# Build binary
cargo build --release

# Create package
mkdir -p dist/rust-assistant
cp target/release/rust_assistant dist/rust-assistant/
cp -r docs dist/rust-assistant/
cp -r models dist/rust-assistant/
cp Cargo.toml dist/rust-assistant/

# Create tarball
tar czf rust-assistant-1.0.0-linux-x64.tar.gz dist/

# Create installation script
cat > dist/install.sh << 'EOF'
#!/bin/bash
mkdir -p ~/rust-assistant
cd ~/rust-assistant
tar xzf rust-assistant-*.tar.gz
echo "Installation complete. Run: ./rust-assistant"
EOF

chmod +x dist/install.sh
```

---

# ❓ FAQ

## General Questions

### Q: Do I need an internet connection?

**A**: Ollama can run entirely offline once models are downloaded. However, the first-time setup requires downloading models (~5-8 GB for Gemma). Whisper.cpp and Piper also download models on first use.

### Q: Which LLMs are supported?

**A**: Any model available in Ollama:
- Gemma 3.x (current default)
- Llama 3.x
- Mistral
- Neural Chat
- And many more

Change the `model` field in `src/assistant/llm.rs:26` to try different models.

### Q: Can I use a different TTS voice?

**A**: Yes! The voice catalog is in `src/tts/voice.rs:12`. Add voices like:
- `en_US-lessac-high` (US English, male)
- `en_GB-alba-medium` (UK English, female)
- Any other Piper voice

Download models from [Piper releases](https://github.com/rhasspy/piper/releases).

### Q: Is my audio data private?

**A**: **Yes**. Everything runs locally on your machine. No audio, transcriptions, or responses leave your computer. This is a key design principle.

---

## Installation & Setup

### Q: What if Piper is called `piper` instead of `piper-tts`?

**A**: Edit `src/tts/engine.rs:9`:
```rust
pub const PIPER_BIN: &str = "piper";
```

Then rebuild: `cargo build --release`

### Q: Can I run this on Windows?

**A**: Yes! Install:
1. Rust (via `rustup-init.exe`)
2. Visual Studio Build Tools (C++ support)
3. Piper (download binary, add to PATH)
4. Build whisper.cpp with CMake

See [Installation Guide](./installation.md) for Windows steps.

### Q: Does this work on Apple Silicon (M1/M2)?

**A**: Yes! Ollama automatically detects Metal acceleration. Build whisper.cpp normally:
```bash
cd repos/whisper.cpp && mkdir build && cd build
cmake .. && make -j$(sysctl -n hw.ncpu)
```

### Q: How much disk space do I need?

**A**: Approximately:
- Whisper model: 140 MB (ggml-base.en.bin)
- Ollama Gemma3: 5-8 GB
- Piper voice: 50-100 MB
- **Total: ~6-9 GB**

Use smaller models to reduce space (e.g., `ggml-tiny.en.bin` is 75 MB).

---

## Runtime & Performance

### Q: Why is transcription slow?

**A**: Whisper runs on your CPU by default. To speed up:
1. Use a smaller model (`ggml-tiny.en.bin`)
2. Use a newer/faster CPU
3. GPU acceleration (experimental for whisper.cpp)

### Q: Can I speed up the LLM response?

**A**: Several options:
1. Use a smaller model: `ollama pull orca-mini:3b`
2. Enable GPU acceleration in Ollama
3. Increase system RAM
4. Close other applications

### Q: What's the typical latency?

**A**: Full round-trip (speak → response) typically takes:
- Recording: 5 seconds (user-dependent)
- Transcription: 2-5 seconds
- LLM: 5-20 seconds (model-dependent)
- TTS: 1-3 seconds
- **Total: 15-40 seconds**

Faster setups with GPU and quantized models can achieve 10-20 seconds.

---

## Troubleshooting

### Q: Recording starts but transcription is empty

**A**: 
1. Speak louder and clearer
2. Reduce background noise
3. Test microphone independently: `arecord test.wav`
4. Verify model file is correct: `ls -lh models/ggml-base.en.bin`

### Q: "Ollama API error: connection refused"

**A**:
```bash
# Start Ollama (new terminal)
ollama serve

# Verify it's running
curl http://localhost:11434/api/tags
```

### Q: No audio playback

**A**:
1. Test speaker: `afplay assistant_response.wav` (macOS) or `paplay` (Linux)
2. Check volume isn't muted
3. Set correct output device in system settings

### Q: Build fails with "linking failed"

**A**:
- Linux: Install `build-essential` and `cmake`
- macOS: Install Xcode Command Line Tools
- Windows: Install Visual Studio Build Tools (C++ workload)

---

## Development & Contributing

### Q: How do I add a new feature?

**A**:
1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make changes and test thoroughly
3. Run `cargo fmt && cargo clippy`
4. Commit with descriptive message
5. Open a pull request

### Q: Can I use this in a commercial product?

**A**: Yes! The project is MIT licensed. You're free to use, modify, and distribute it. Include the license file as required by MIT.

### Q: How do I contribute?

**A**:
1. Fork the repository
2. Create a feature branch
3. Make focused changes
4. Follow the code style guide
5. Submit a pull request

See [Development Guide](./development.md) for details.

---

## Advanced Usage

### Q: Can I stream audio instead of recording?

**A**: Planned improvement! Currently, the app records full audio before transcribing. Streaming would enable real-time transcription.

### Q: Can I use a GPU for faster inference?

**A**: 
- **Ollama**: GPU support is automatic (NVIDIA CUDA, AMD ROCm, Apple Metal)
- **Whisper**: Partial GPU support available; see upstream docs
- **Piper**: CPU-only currently

### Q: How do I deploy this to a server?

**A**: Build a release binary and ensure dependencies are installed:
```bash
cargo build --release
scp target/release/rust_assistant user@server:~/
```

The server needs: Ollama, whisper.cpp, piper-tts, and audio hardware.

---

## Project Info

### Q: Who maintains this?

**A**: This is an open-source project. See contributors on GitHub.

### Q: What's the roadmap?

**A**: See [Roadmap](./roadmap.md) for planned features:
- Configurable settings (env vars, config file)
- Extended voice options
- Streaming STT/TTS
- Better device management
- Test suite
- Packaging and installers

### Q: Where can I report bugs?

**A**: Open an issue on [GitHub](https://github.com/yash1648/rust-assistant/issues) with:
- Error message and backtrace
- OS and version
- Steps to reproduce
- Expected vs. actual behavior

---

## 🔗 Quick Links

- **Repository**: https://github.com/yash1648/rust-assistant
- **Installation**: [Installation Guide](./installation.md)
- **Technical Details**: [Architecture Docs](./technical.md)
- **Troubleshooting**: [Troubleshooting Guide](./troubleshooting.md)
- **Roadmap**: [Future Plans](./roadmap.md)

---

**Last Updated**: December 2025