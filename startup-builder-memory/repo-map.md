# Repository Map

## Top-Level Structure
```
rust-assistant/
├── src/
│   ├── main.rs              # Entry point, CLI dispatch, setup/doctor commands
│   ├── lib.rs               # Public API re-exports for testing
│   ├── cli.rs               # Clap CLI definition (7 subcommands)
│   ├── error.rs             # Structured error types (thiserror)
│   ├── ui.rs                # Colored terminal output
│   ├── assistant/
│   │   ├── mod.rs           # Module re-exports
│   │   ├── config.rs        # TOML + env var configuration
│   │   ├── conversation.rs  # Main conversation loop
│   │   └── llm.rs           # Ollama API client
│   ├── stt/
│   │   ├── mod.rs           # Module declarations
│   │   ├── audio.rs         # Audio device configuration
│   │   ├── io.rs            # stdin helper
│   │   ├── recorder.rs      # WAV recording via cpal
│   │   └── transcriber.rs   # Whisper transcription
│   └── tts/
│       ├── mod.rs           # Module exports
│       └── engine.rs        # Pure Rust KittenTTS engine
├── tests/
│   └── integration_test.rs  # 10 integration tests
├── .github/workflows/
│   └── ci.yml               # Cross-platform CI/CD matrix
├── .cargo/
│   └── config.toml          # Linker configuration
├── Dockerfile               # Multi-stage build
├── Cargo.toml               # Manifest + release profiles
├── Assistant.toml           # Runtime configuration
└── startup-builder-memory/  # AI-assisted development memory
    ├── product-brief.md
    ├── task-ledger.md
    ├── decisions.md
    ├── repo-map.md
    └── architecture-map.md
```

## Key Files (Hot Paths)
- **Config loading**: `src/assistant/config.rs` (TOML + env vars)
- **Conversation loop**: `src/assistant/conversation.rs` (record → transcribe → LLM → speak)
- **TTS**: `src/tts/engine.rs` (kittentts wrapper)
- **STT**: `src/stt/transcriber.rs` (whisper-rs wrapper)
- **Recording**: `src/stt/recorder.rs` (cpal audio capture)
