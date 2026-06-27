// Temporary: many public API items are unused by the binary but part of the library surface
#![allow(dead_code)]

//! # rust-assistant
//!
//! A local-first, privacy-focused voice assistant written in Rust.
//!
//! ## Pipeline
//!
//! 1. **Record** — Capture microphone audio via `cpal` + `hound` (WAV)
//! 2. **Transcribe** — Speech-to-text via `whisper-rs` (whisper.cpp bindings)
//! 3. **Understand** — LLM inference via local Ollama server
//! 4. **Speak** — Text-to-speech via `kittentts` (pure Rust, zero system deps)
//!
//! ## Architecture
//!
//! - `cli` — CLI argument parsing (clap)
//! - `assistant` — Conversation loop, config, LLM client
//! - `stt` — Speech-to-text: audio capture + Whisper transcription
//! - `tts` — Text-to-speech: pure Rust KittenTTS engine
//! - `error` — Structured error types
//! - `ui` — Terminal UI helpers (colored output)

pub mod assistant;
pub mod cli;
pub mod error;
pub mod stt;
pub mod tts;
pub mod ui;

// Re-exports for convenience and testing
pub use assistant::config::{
    default_ollama_server, default_tts_speed, default_tts_voice, env_vars, generate_default_toml,
    Config,
};
pub use assistant::conversation::Message;
pub use cli::{Cli, Commands, ShellKind};
pub use error::AssistantError;
pub use stt::audio::AudioConfig;

// Re-export cpal SampleFormat for testing
pub use cpal::SampleFormat;

/// The sample rate used by the TTS engine (24 kHz)
pub const TTS_SAMPLE_RATE: u32 = kittentts::SAMPLE_RATE;

/// The sample rate expected by Whisper (16 kHz)
pub const STT_SAMPLE_RATE: u32 = 16000;
