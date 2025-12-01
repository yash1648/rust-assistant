# 📐 Technical Architecture

Deep technical documentation for developers extending or debugging the voice assistant.

## 🏗️ System Architecture

The application implements a classic pipeline architecture with three independent subsystems:

```
┌─────────────┐
│  Audio In   │
└──────┬──────┘
       │
       ▼
┌──────────────────────┐
│   STT Pipeline       │
│ (Audio → Text)       │
│ • Record audio       │
│ • Transcribe via     │
│   whisper.cpp        │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│  LLM Pipeline        │
│ (Text → Response)    │
│ • Ollama Chat API    │
│ • Gemma 3.x model    │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│   TTS Pipeline       │
│ (Text → Audio)       │
│ • Synthesize via     │
│   piper-tts          │
│ • Playback with rodio│
└──────┬───────────────┘
       │
       ▼
┌─────────────┐
│ Audio Out   │
└─────────────┘
```

### Core Modules

#### `assistant` Module
**Responsibility**: Orchestration and conversation management

- **`conversation.rs`** — Main event loop
  - `run()`: Drives the interaction cycle
  - `listen_to_user()`: Records and transcribes user input
  - `speak_response()`: Synthesizes and plays LLM response
  - Maintains `conversation_history: Vec<Message>`

- **`llm.rs`** — Ollama integration
  - `chat_with_ollama()`: Calls the Ollama Chat API
  - Structures messages in OpenAI-compatible format
  - Handles HTTP errors with detailed context

#### `stt` Module
**Responsibility**: Speech-to-text pipeline

- **`audio.rs`** — Device detection
  - `get_default_input_device()`: Queries system for the default microphone
  - Handles platform-specific audio subsystem APIs via `cpal`

- **`recorder.rs`** — Audio capture
  - `record_audio()`: Captures PCM samples to WAV using `hound`
  - Supports `I16` and `F32` sample formats
  - Returns `Result<(), anyhow::Error>`

- **`transcriber.rs`** — Whisper integration
  - `transcribe_with_whisper()`: Spawns the whisper.cpp CLI
  - Parses the generated `.txt` output file
  - Locates the binary across OS variants (macOS, Linux, Windows)

#### `tts` Module
**Responsibility**: Text-to-speech pipeline

- **`engine.rs`** — Piper synthesis & playback
  - `synthesize()`: Spawns piper-tts subprocess
  - `play_audio()`: Uses `rodio` for system audio output
  - Captures stderr for detailed error reporting

- **`voice.rs`** — Voice catalog
  - `voices()`: Registry of available speaker models
  - Currently: `en_GB-cori-high` (UK English, female, high quality)
  - Extensible for additional voices

- **`models.rs`** — Model caching & downloads
  - Downloads ONNX models on first run
  - Caches to `./models/` directory
  - Verifies file integrity before use

## 🔄 Data Flow

### Request-Response Cycle

```
User Input
    ↓
[1] Record Audio (cpal/hound)
    ↓
user_input.wav
    ↓
[2] Transcribe (whisper.cpp CLI)
    ↓
user_input.wav.txt
    ↓
[3] Chat Request (Ollama HTTP API)
    ├─ model: "gemma3"
    ├─ messages: [system, history, user]
    └─ stream: false
    ↓
[4] LLM Response (Gemma 3.x)
    ├─ role: "assistant"
    └─ content: "Natural language response"
    ↓
[5] Synthesize (piper-tts subprocess)
    ├─ input: response text
    └─ output: assistant_response.wav
    ↓
[6] Playback (rodio)
    ↓
Audio Output
```

## 🌐 API Integration

### Ollama Chat API

**Endpoint**: `POST http://localhost:11434/api/chat`

**Request Format** (OpenAI-compatible):
```json
{
  "model": "gemma3",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "What is Rust?"
    }
  ],
  "stream": false
}
```

**Response Format**:
```json
{
  "model": "gemma3",
  "created_at": "2024-12-01T10:30:45.123Z",
  "message": {
    "role": "assistant",
    "content": "Rust is a systems programming language..."
  },
  "done": true,
  "total_duration": 5000000000,
  "load_duration": 1000000000,
  "prompt_eval_count": 15,
  "prompt_eval_duration": 2000000000,
  "eval_count": 120,
  "eval_duration": 2000000000
}
```

**Error Handling**:
- Non-2xx HTTP status codes trigger detailed error messages
- Response body is included in error context
- Network timeouts handled via `tokio::timeout`

## 💾 State Management

### Memory Architecture

```
┌────────────────────────────────────────┐
│  Assistant Runtime State               │
├────────────────────────────────────────┤
│  conversation_history: Vec<Message>    │
│  ├─ { role: "user", content: "..." }   │
│  ├─ { role: "assistant", content: "..." }
│  └─ ...                                │
├────────────────────────────────────────┤
│  Current Message Buffers               │
│  ├─ user_input.wav (audio file)        │
│  ├─ user_input.wav.txt (transcript)    │
│  ├─ assistant_response.wav (audio)     │
│  └─ Generated at runtime               │
├────────────────────────────────────────┤
│  Cached Models                         │
│  ├─ ./models/ggml-base.en.bin          │
│  ├─ ./models/en_GB-cori-high.onnx      │
│  ├─ ./models/en_GB-cori-high.onnx.json │
│  └─ Downloaded on first use            │
└────────────────────────────────────────┘
```

### Conversation History

Maintained as a `Vec<Message>` in memory:
```rust
pub struct Message {
    pub role: String,    // "user" or "assistant"
    pub content: String, // The actual text
}
```

Each interaction appends two messages (user + assistant) to the history. The entire history is sent to Ollama for contextual understanding.

## ⚡ Performance Characteristics

| Operation | Typical Duration | Bottleneck |
|-----------|-----------------|-----------|
| Audio Recording (5 seconds) | 5s | User timing |
| Whisper Transcription | 2-5s | Model inference (CPU-bound) |
| Ollama Chat Request | 1-10s | Model generation speed |
| Piper TTS Synthesis | 0.5-2s | Model inference |
| rodio Playback | Variable | Audio stream duration |
| **Total Round-trip** | **10-30s** | Inference speed |

**Optimization Opportunities**:
- Streaming STT to start transcription before recording completes
- Streaming TTS to begin playback before full synthesis finishes
- Quantized models for faster inference
- GPU acceleration for Ollama and Whisper

## 🛠️ Error Handling Strategy

The project uses `anyhow` for ergonomic error handling with context:

### Error Propagation Pattern
```rust
// Example from transcriber.rs
let output = Command::new(whisper_bin)
    .args(&[...])
    .output()
    .context("failed to execute whisper")?;

if !output.status.success() {
    anyhow::bail!(
        "whisper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
```

### Error Sources by Module

| Module | Common Errors | Recovery |
|--------|--------------|----------|
| `recorder` | No input device, format unsupported | Bail with helpful message |
| `transcriber` | Binary not found, model missing | Bail with path suggestions |
| `llm` | Ollama unreachable, HTTP error | Bail with endpoint hint |
| `engine` | Piper not in PATH, synthesis failed | Bail with stderr context |

## 🔐 Security Considerations

1. **No External Calls**: All processing is local; no telemetry or cloud API calls
2. **File Permissions**: Audio files are world-readable by default (consider restrictive umask)
3. **Model Integrity**: No checksum verification of downloaded models
4. **Subprocess Spawning**: Piper and Whisper are executed via `std::process::Command` without shell=true (safe)

## 📊 Sequence Diagram

```
User    →  Recorder  →  Whisper  →  Assistant  →  Ollama  →  Piper  →  Speaker
 │           │           │           │            │         │        │
 ├─Speak─────→           │           │            │         │        │
 │           ├─Record────→           │            │         │        │
 │           │   (WAV)   │           │            │         │        │
 │           │           ├─Transcribe│            │         │        │
 │           │           │ (TXT)     │            │         │        │
 │           │           ├───────────→            │         │        │
 │           │           │           ├─POST chat─→         │        │
 │           │           │           │            ├─Response→        │
 │           │           │           │←───────────┤         │        │
 │           │           │           ├─Synthesize────────→ │        │
 │           │           │           │            │  (WAV)  │        │
 │           │           │           │            │         ├─Play──→
 │←──────────────────────────────────────────────────────────────────┤
 │                       Audio Response                               │
```

## 🧪 Testing Approach

The project currently relies on manual testing. Recommended automated test structure:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_transcriber_with_fixture() {
        // Transcribe a known WAV file
        let text = transcribe_with_whisper("./test_audio.wav");
        assert!(text.contains("expected text"));
    }

    #[test]
    fn test_ollama_mock() {
        // Replace reqwest with mock HTTP response
        let response = chat_with_ollama(&messages);
        assert_eq!(response.message.role, "assistant");
    }

    #[test]
    fn test_piper_synthesis() {
        // Verify .wav file is created and valid
        synthesize("Hello world");
        assert!(Path::new("assistant_response.wav").exists());
    }
}
```

## 🔗 Navigation

- [Overview](./README.md) — Project introduction
- [Installation](./installation.md) — Setup guide
- [Development](./development.md) — Contributing
- [Troubleshooting](./troubleshooting.md) — Problem solving
- [FAQ](./faq.md) — Common questions
- [Roadmap](./roadmap.md) — Future plans
- [Known Issues](./known-issues.md) — Current limitations

---

**Last Updated**: December 2025