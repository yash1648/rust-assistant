# 🛣️ Roadmap

Strategic improvements and features planned for the Piper Female voice assistant.

## Vision

Build a **local-first, high-quality voice AI assistant** that prioritizes privacy, responsiveness, and user customization.

---

## 📍 Current Status (v1.0)

### ✅ Complete Features

- [x] Speech-to-text via whisper.cpp
- [x] LLM integration with Ollama
- [x] Text-to-speech with Piper
- [x] Full conversation history
- [x] Multi-platform support (Linux, macOS, Windows)
- [x] Local-only processing (no cloud dependencies)

### 🔄 In Progress

- [ ] User testing and feedback collection
- [ ] Documentation refinement
- [ ] Performance profiling

---

## 🚀 Planned Features (v1.1 - Q1 2025)

### Configuration Management

**Goal**: Eliminate hardcoded settings

#### Environment Variables
```bash
# Usage
export PIPER_BIN="piper-tts"
export WHISPER_PATH="./repos/whisper.cpp"
export WHISPER_MODEL="models/ggml-base.en.bin"
export OLLAMA_ENDPOINT="http://localhost:11434/api/chat"
export OLLAMA_MODEL="gemma3"

cargo run
```

#### Config File (TOML)
```toml
# .voice-assistant.toml
[tts]
binary = "piper-tts"
voice = "en_GB-cori-high"

[stt]
whisper_path = "./repos/whisper.cpp"
model = "models/ggml-base.en.bin"

[llm]
endpoint = "http://localhost:11434/api/chat"
model = "gemma3"

[audio]
input_device = "default"
output_device = "default"
```

**Benefits**:
- No code recompilation needed
- Easy deployment across machines
- User-friendly configuration

---

## 🎙️ Voice & Audio Enhancements (v1.2 - Q2 2025)

### Extended Voice Catalog

Support multiple speakers:

```rust
// src/tts/voice.rs
pub fn voices() -> HashMap<&str, VoiceMetadata> {
    map! {
        "cori-female-gb" => VoiceMetadata {
            name: "Cori (British, Female)",
            language: "en-GB",
            gender: "Female",
            quality: "High",
            file_size: "100MB",
        },
        "lessac-male-us" => VoiceMetadata {
            name: "Lessac (US, Male)",
            language: "en-US",
            gender: "Male",
            quality: "High",
            file_size: "95MB",
        },
        // ... more voices
    }
}
```

### Runtime Voice Selection

Interactive voice picker:

```
Available voices:
1. Cori (British, Female, High)
2. Lessac (US, Male, High)
3. Alba (British, Female, Medium)

Select voice (1-3):
```

### Multi-language Support

Expand beyond English:

```rust
// Support for Spanish, French, German, etc.
pub fn supported_languages() -> Vec<&str> {
    vec!["en", "es", "fr", "de", "it", "ja"]
}
```

---

## ⚡ Performance Optimization (v1.3 - Q3 2025)

### Streaming STT

Real-time transcription as user speaks:

```
User speaks...
[Recording chunk 1] → Transcribe → "Hello"
[Recording chunk 2] → Transcribe → "Hello, how"
[Recording chunk 3] → Transcribe → "Hello, how are you"
```

**Benefits**: Reduced latency, faster feedback

### Streaming TTS

Begin playback before synthesis completes:

```
LLM response: "This is a long response..."
[Synthesize chunk 1] → Play immediately
[Synthesize chunk 2] → Queue for playback
[Synthesize chunk 3] → Play next
```

**Benefits**: Natural, responsive experience

### Model Quantization

Support quantized (smaller, faster) models:

```bash
# Smaller models run faster
ollama pull orca-mini:3b    # 3B params
ollama pull mistral:7b-q4   # 7B quantized to Q4

# Performance improvements:
# - 50% smaller RAM footprint
# - 2-3x faster inference
# - Slight quality reduction
```

---

## 🎯 User Experience (v1.4 - Q4 2025)

### Interactive Configuration Wizard

```bash
$ cargo run -- --setup

🎙️ Welcome to Piper Female Setup

1. Audio Configuration
   [✓] Microphone: USB Audio Device
   [✓] Speaker: HDMI Output
   
2. STT Configuration
   [?] Whisper location: ./repos/whisper.cpp ✓
   [?] Model: ggml-base.en.bin ✓
   
3. LLM Configuration
   [?] Ollama endpoint: http://localhost:11434 ✓
   [?] Model: gemma3 ✓
   
4. TTS Configuration
   [?] Voice: en_GB-cori-high ✓
   
Setup complete! Run: cargo run
```

### Interactive Device Selection

```
Available Input Devices:
1. Built-in Microphone
2. USB Audio Device (recommended)
3. Bluetooth Headset

Select (1-3): 2
✓ Selected: USB Audio Device
```

### Conversation Management

```bash
# View history
$ rust-assistant --history

# Save conversation
$ rust-assistant --save my-conversation.json

# Load previous conversation
$ rust-assistant --load my-conversation.json
```

---

## 🔒 Security & Privacy (v1.5 - 2025)

### Data Encryption

- Encrypt stored conversation history
- Optional password protection for sensitive chats

### Audit Logging

```
[2024-12-01 10:30:45] INPUT: "What is my password?"
[2024-12-01 10:30:50] OUTPUT: "I don't see a password in the documents..."
[2024-12-01 10:31:02] MODEL: gemma3 (via Ollama)
```

### Permission Management

```rust
// Explicit opt-in for file access, internet, etc.
pub enum Permission {
    Microphone,
    Speaker,
    FileAccess,
    NetworkAccess,
}
```

---

## 🧪 Testing & Quality (v1.6 - 2025)

### Automated Test Suite

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Coverage report
cargo tarpaulin --out Html
```

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

### Performance Benchmarks

```bash
# Track latency over time
cargo bench

# Profile CPU/memory usage
cargo flamegraph
```

---

## 📦 Packaging & Distribution (v2.0 - 2025)

### Platform-Specific Installers

```bash
# macOS: .dmg installer
cargo install cargo-bundle
cargo bundle --release --target aarch64-apple-darwin

# Windows: .msi installer
cargo wix

# Linux: .deb package
cargo deb
```

### Prebuilt Binaries

Distribute ready-to-run executables:
- `rust-assistant-v1.1-macos-arm64`
- `rust-assistant-v1.1-windows-x64.exe`
- `rust-assistant-v1.1-linux-x64`

### Docker Support

```dockerfile
FROM rust:latest
RUN apt-get install -y piper-tts
COPY . /app
WORKDIR /app
RUN cargo build --release
ENTRYPOINT ["./target/release/rust_assistant"]
```

---

## 🌐 Advanced Features (Future)

### Web UI

Browser-based interface for configuration and interaction:

```
http://localhost:8080/
├─ Dashboard
├─ Settings
├─ Chat History
└─ Admin Panel
```

### API Server

REST API for external integrations:

```bash
POST /api/chat
{
  "message": "What's the weather?",
  "model": "gemma3",
  "stream": true
}
```

### Pluggable Backends

Support alternate implementations:

```rust
pub trait STTBackend {
    fn transcribe(&self, audio: &[u8]) -> Result<String>;
}

pub trait TTSBackend {
    fn synthesize(&self, text: &str) -> Result<Vec<u8>>;
}

pub trait LLMBackend {
    fn chat(&self, messages: &[Message]) -> Result<String>;
}
```

---

## 📊 Milestone Timeline

```
v1.0 ✓ Current
├─ Core STT/LLM/TTS
├─ Multi-platform
└─ Full documentation

v1.1 → Q1 2025
├─ Config files
├─ Environment variables
└─ Extended testing

v1.2 → Q2 2025
├─ Multiple voices
├─ Language support
└─ Voice selection UI

v1.3 → Q3 2025
├─ Streaming STT/TTS
├─ Model quantization
└─ Performance optimizations

v1.4 → Q4 2025
├─ Setup wizard
├─ Device selection
└─ Conversation management

v2.0 → 2025-2026
├─ Web UI
├─ REST API
├─ Desktop app
└─ Installers
```

---

## 🤝 How to Contribute

Interested in helping? See [Development Guide](./development.md) for:
- Code style guidelines
- Testing requirements
- Pull request process

Priority areas for contributions:
1. ✅ Performance optimization
2. ✅ Testing and CI/CD
3. ✅ Documentation improvements
4. ✅ New features from roadmap
5. ✅ Bug fixes

---

---

# ⚠️ Known Issues

List of current limitations and workarounds.

## 🔴 Critical Issues

### Hardcoded Configuration Paths

**Issue**: Settings must be edited in source code  
**Severity**: High  
**Status**: Planned fix in v1.1  
**Workaround**: See [Installation Guide](./installation.md#step-5-configure-the-project)

**Affected Files**:
- `src/tts/engine.rs:9` — Piper binary name
- `src/stt/transcriber.rs:41-74` — Whisper binary location
- `src/assistant/conversation.rs:76-87` — Whisper model path
- `src/assistant/llm.rs:26` — Ollama endpoint

**Planned Solution**:
```rust
// v1.1: Read from environment
let piper_bin = env::var("PIPER_BIN").unwrap_or("piper-tts".to_string());
let whisper_path = env::var("WHISPER_PATH").unwrap_or("./repos/whisper.cpp".to_string());
let ollama_endpoint = env::var("OLLAMA_ENDPOINT").unwrap_or("http://localhost:11434".to_string());
```

---

## 🟠 High Priority Issues

### Limited Voice Catalog

**Issue**: Only one voice available (`en_GB-cori-high`)  
**Severity**: Medium  
**Status**: Planned for v1.2  
**Impact**: Users can't personalize voice output

**Workaround**:
Edit `src/tts/voice.rs:12` to add voices manually:
```rust
pub fn voices() -> Vec<&str> {
    vec![
        "en_GB-cori-high",
        "en_US-lessac-high",  // Add this
        "en_GB-alba-medium",  // And this
    ]
}
```

Then download model files:
```bash
wget https://huggingface.co/rhasspy/piper/resolve/main/voices/en/en_US/lessac/high/en_US-lessac-high.onnx \
  -O models/en_US-lessac-high.onnx
```

---

### Platform Differences in Audio Handling

**Issue**: Audio device detection varies across OS  
**Severity**: Medium  
**Status**: Known limitation  
**Affected**: Windows, some Linux distros

**Symptoms**:
- Wrong microphone selected
- No speaker output
- Audio format errors

**Workaround**:
1. Set microphone/speaker as default in system settings
2. Test audio with native tools:
   - Linux: `arecord`, `aplay`
   - macOS: `afplay`
   - Windows: Built-in audio settings

**Future Fix**: Device selection UI in v1.4

---

## 🟡 Medium Priority Issues

### Limited Sample Format Support

**Issue**: Recorder only supports `I16` and `F32` PCM formats  
**Severity**: Low  
**Status**: Will support more formats in v2.0

**Current Support**:
- ✅ 16-bit PCM (I16)
- ✅ 32-bit floating-point (F32)
- ❌ 24-bit PCM
- ❌ Compressed formats (MP3, FLAC)

**Workaround**: Most modern audio hardware uses I16 or F32 by default

---

### No Model Integrity Verification

**Issue**: Downloaded models aren't checksum-verified  
**Severity**: Low  
**Status**: Planned improvement

**Risk**: Corrupted downloads could cause silent failures

**Workaround**: Manually verify model files:
```bash
ls -lh models/ggml-base.en.bin
# Should be exactly 140,544,262 bytes for base.en model
```

---

### Transcription Fails with Background Noise

**Issue**: Whisper has trouble with noisy environments  
**Severity**: Medium  
**Status**: User responsibility (use better mic)

**Causes**:
- Cheap microphone with poor noise rejection
- Loud background noise (fan, traffic, etc.)
- Quiet user voice

**Solutions**:
1. Use a better microphone (USB headset, condenser mic)
2. Reduce background noise
3. Speak at normal volume, closer to mic
4. Try smaller model with less noise sensitivity:
   ```bash
   # Download tiny model (75 MB)
   wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin \
     -O models/ggml-tiny.en.bin
   
   # Update config to use it
   ```

---

## 🟢 Low Priority Issues / Enhancements

### No Streaming Transcription

**Issue**: Full audio must be recorded before transcription starts  
**Severity**: Enhancement  
**Status**: Planned for v1.3

**Impact**: Longer perceived latency

**Workaround**: Keep utterances short (< 30 seconds)

**Future**: Streaming would enable:
- Real-time text display
- Interruption capability
- Faster feedback

---

### No GPU Acceleration for Whisper

**Issue**: Speech-to-text runs on CPU only  
**Severity**: Enhancement  
**Status**: Upstream work (whisper.cpp)

**Impact**: Slow transcription on CPU-only systems

**Status**: Whisper.cpp has partial GPU support; integration pending

**Workaround**: Use smaller model:
```bash
# Faster: tiny model (75 MB, ~1s per 5s audio)
models/ggml-tiny.en.bin

# vs

# Slower: base model (140 MB, ~5s per 5s audio)
models/ggml-base.en.bin
```

---

### Single-Threaded Conversation Loop

**Issue**: App blocks while processing (no responsive UI)  
**Severity**: Low  
**Status**: Not urgent (CLI app)

**Impact**: Can't interrupt mid-processing

**Future**: Will improve with streaming TTS/STT

---

### No Persistent Conversation History

**Issue**: Chat history is lost when app exits  
**Severity**: Low  
**Status**: Enhancement for v1.4

**Workaround**: Manually save conversations (future feature)

**Future**:
```bash
# Save
rust-assistant --save conversation.json

# Load
rust-assistant --load conversation.json
```

---

## 🔵 Upstream Limitations (Out of Scope)

### Whisper.cpp Accuracy

Depends on audio quality and model size. No workaround possible in this project; resolution requires better hardware or larger models.

### Ollama Model Limitations

If LLM responses are irrelevant:
1. Try different model: `ollama pull llama3`
2. Rephrase your question
3. Provide more context

### Piper TTS Quality

Voice synthesis is limited by model training data. Alternative voices available but all are trained on datasets with inherent limitations.

---

## 📋 Reporting Issues

Found a bug not listed here?

1. **Check existing issues**: https://github.com/yash1648/rust-assistant/issues
2. **Provide details**:
   - OS and version
   - Exact error message
   - Steps to reproduce
   - Expected vs. actual behavior
3. **Include context**:
   - Output of `rustc --version` and `cargo --version`
   - Which component failed (STT, LLM, TTS, audio)

---

## 🔗 Navigation

- [Installation](./installation.md) — Setup help
- [Troubleshooting](./troubleshooting.md) — Problem solving
- [Technical](./technical.md) — Architecture
- [Development](./development.md) — Contributing
- [FAQ](./faq.md) — Questions

---

**Last Updated**: December 2025