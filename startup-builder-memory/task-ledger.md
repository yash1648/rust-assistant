# Task Ledger

## Legend
- ✅ Done
- 🔄 In Progress
- ⏳ Pending
- ❌ Blocked

## Current Sprint: Hardening & Polish

| ID | Task | Status | Notes |
|----|------|--------|-------|
| T01 | Replace Python TTS with pure Rust | ✅ | Switched any-tts/kokoro → kittentts |
| T02 | Release profile optimization | ✅ | LTO=fat, codegen-units=1, strip, panic=abort |
| T03 | Cross-platform CI/CD | ✅ | GitHub Actions matrix for 6 targets |
| T04 | Multi-stage Dockerfile | ✅ | Debian slim, ~45MB binary |
| T05 | Port bash scripts to Rust | ✅ | setup + doctor CLI commands |
| T06 | Test infrastructure | ✅ | 44 tests (unit + integration) |
| T07 | `.cargo/config.toml` | ✅ | Faster linker config (mold/lld) |
| T08 | Clean up dead files | ✅ | Removed requirement.txt, run.sh, config.sh, kokoro.py |
| T09 | Memory artifacts | ✅ | Created startup-builder-memory/ |
| T10 | Code optimization P1–3 | ✅ | Shared HTTP client, in-memory audio, error handling, bounded history |
| T11 | .env auto-loading | ✅ | dotenvy crate, auto-loaded at startup |
| T12 | Voice Activity Detection | ✅ | Energy-based VAD, no model needed, configurable threshold/silence |
| T13 | Hotkey activation | ⏳ | Global shortcut for push-to-talk |
| T14 | Streaming transcription | ⏳ | Show partial results while speaking |
| T15 | Multiple language support | ⏳ | Whisper multilingual model |
| T16 | GUI frontend | ⏳ | Tauri-based desktop app |

## Backlog
- T17: Custom wake word detection
- T18: Conversation history persistence
- T19: LLM streaming response (SSE)
- T20: Audio output device selection
- T21: Configuration profiles
- T22: Plugin system for TTS/STT backends
