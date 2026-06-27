//! Text-to-speech module — pure Rust, no system dependencies.
//!
//! Uses [KittenTTS](https://crates.io/crates/kittentts) for neural TTS
//! with ONNX Runtime. Zero Python, zero C libraries, zero shared objects.

pub mod engine;
pub use engine::TtsEngine;
