# Product Brief: rust-assistant

## Vision
A local-first, privacy-focused voice assistant that runs entirely on-device — no cloud dependencies, no data leaks. Currently built as a CLI tool with STT → LLM → TTS pipeline.

## Target Users
- Developers who want hands-free coding assistance
- Privacy-conscious users who refuse cloud voice assistants
- Anyone needing offline voice interaction

## Core User Flow
1. User speaks into microphone
2. Audio is transcribed locally (Whisper)
3. Text is sent to local LLM (Ollama)
4. Response is synthesized to speech (KittenTTS)
5. Audio plays through speakers

## Current Status
- **Phase**: MVP Complete / Hardening
- **Stack**: 100% Rust (zero Python, zero cloud)
- **STT**: whisper-rs (whisper.cpp bindings)
- **LLM**: Ollama API (local)
- **TTS**: kittentts (pure Rust, ONNX-based)
- **CLI**: clap with run/setup/doctor/env/completion commands
- **CI/CD**: GitHub Actions (Linux/macOS/Windows + ARM)
- **Container**: Multi-stage Dockerfile

## MVP Definition of Done
- [x] Voice recording (cpal + hound WAV)
- [x] Speech-to-text (whisper-rs)
- [x] LLM conversation (Ollama API)
- [x] Text-to-speech (kittentts)
- [x] Conversation loop
- [x] CLI with subcommands
- [x] Cross-platform build (CI matrix)
- [x] Container support (Docker)
- [x] Unit/integration tests
- [x] Pure Rust — no Python dependencies

## Next Horizons
- Global hotkey activation
- Streaming transcription
- Voice activity detection (VAD)
- Multiple languages
- GUI (Tauri?)
