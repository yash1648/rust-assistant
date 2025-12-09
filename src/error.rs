use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum AssistantError {
    #[error("Whisper binary not found at {cli} or {fallback}")]
    WhisperNotFound { cli: PathBuf, fallback: PathBuf },
}
