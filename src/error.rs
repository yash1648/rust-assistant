//! Structured error types for the assistant application.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssistantError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TTS error: {0}")]
    Tts(String),

    #[error("STT error: {0}")]
    Stt(String),
}
