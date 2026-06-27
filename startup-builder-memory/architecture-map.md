# Architecture Map

## Data Flow
```
[Microphone] → cpal capture → VAD (energy RMS, atomic)
                                    ↓
                             PCM Vec<i16> → WAV assembly (hound)
                                                  ↓
                                        whisper-rs (STT) ← N-channel → mono mixdown
                                                  ↓
                                             [Text]
                                                  ↓
[Ollama LLM] ←── reqwest HTTP API (OnceLock<Client> shared pool)
                                                  ↓
                                             [Response Text]
                                                  ↓
[kittentts TTS] → ONNX Runtime → f32 samples → rodio playback
                                                  ↓
                                             [Speaker]
```

*All audio stays in-memory — no disk I/O.*
*Conversation history bounded at 10 turns.*
*VAD: energy-based RMS threshold, auto-stops on silence, no model needed.*

## Module Boundaries

### `stt/` (Speech-to-Text)
- **Input**: Microphone audio (via cpal)
- **Output**: Text string
- **Dependencies**: cpal, hound, whisper-rs
- **Key types**: `WhisperTranscriber`, `AudioConfig`, `VadState`, `VadConfig`
- **Concerns**: Audio format conversion, resampling, channel mixing, VAD

### `tts/` (Text-to-Speech)
- **Input**: Text string
- **Output**: Audio playback (via rodio)
- **Dependencies**: kittentts, rodio, hound
- **Key types**: `TtsEngine`
- **Concerns**: ONNX inference, voice selection, speed control

### `assistant/` (Orchestration)
- **Input**: User voice (via stt)
- **Output**: Spoken response (via tts)
- **Dependencies**: stt, tts, reqwest
- **Key types**: `Assistant`, `Config`, `Message`
- **Concerns**: Conversation state, config management, LLM API

## CLI Commands
```
rust-assistant
├── run                 # Voice assistant loop (default)
├── setup              # Download models, create dirs
├── doctor             # System health check
├── generate-config    # Create default Assistant.toml
├── generate-completion <shell>  # Shell completions
└── env                # Show env variable docs
```

## Configuration Priority
1. Environment variables (highest)
2. `Assistant.toml` file
3. Hardcoded defaults (lowest)

## Build Targets (CI)
| Target | OS | Arch |
|--------|----|------|
| x86_64-unknown-linux-gnu | Linux | x86_64 |
| aarch64-unknown-linux-gnu | Linux | ARM64 |
| x86_64-apple-darwin | macOS | Intel |
| aarch64-apple-darwin | macOS | Apple Silicon |
| x86_64-pc-windows-msvc | Windows | x86_64 |

## Docker
- Base: `debian:bookworm-slim`
- Build: `rust:1.84-slim-bookworm`
- Runtime: ALSA + ca-certificates
- Entrypoint: `rust-assistant`
