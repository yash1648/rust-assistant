// src/tts/mod.rs
pub mod voice;
pub mod models;
pub mod engine;

// Optional re-exports so main.rs can do `use tts::...`
pub use voice::{Voice, voices};
pub use models::ensure_voice_downloaded;
pub use engine::{synthesize_with_piper, play_wav};
