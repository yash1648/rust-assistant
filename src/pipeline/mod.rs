//! Streaming pipeline orchestrator for the blazing fast voice assistant.
//!
//! # Architecture
//!
//! The pipeline connects 4 stages via lock-free `crossbeam` channels:
//!
//! ```text
//!  ┌────────┐   crossbeam   ┌──────────┐   tokio   ┌────────┐   crossbeam   ┌────────┐
//!  │Capture │───channel────▶│   STT    │───task────▶│  LLM   │───channel────▶│  TTS   │
//!  │(cpal)  │   (PCM i16)   │(whisper) │ (text)    │(stream)│   (tokens)    │(stream)│
//!  └────┬───┘               └──────────┘            └────────┘               └───┬────┘
//!       │                                                                       │
//!       └─────────────┬─────────────────────────────────────────────────────────┘
//!                     ▼
//!              ┌──────────────┐
//!              │   Playback   │
//!              │ (interrupt)  │
//!              └──────────────┘
//! ```
//!
//! Key design decisions:
//! - Lock-free channels (crossbeam) for audio transport — no mutex contention
//! - Full-duplex: record while playing, VAD interrupt on new speech
//! - Streaming LLM: start TTS on partial tokens for sub-second response start
//! - Bounded channels prevent OOM on long recordings

use crossbeam::channel::{self, Receiver, Sender};

/// Maximum audio chunks buffered before processing (10 seconds at 48kHz stereo)
const AUDIO_CHANNEL_CAPACITY: usize = 512;

/// Maximum TTS chunks buffered before playback (5 seconds)
const TTS_CHANNEL_CAPACITY: usize = 128;

/// Events flowing through the pipeline stages
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    /// Audio chunk from microphone (raw PCM i16, device-native rate)
    AudioChunk(Vec<i16>),
    /// Recording session started (user began speaking or VAD triggered)
    RecordingStarted,
    /// Recording session stopped (VAD silence detected, Enter pressed, or timeout)
    RecordingStopped,
    /// Transcribed text from Whisper
    Transcription(String),
    /// Partial token from streaming LLM response
    LlmToken(String),
    /// Full LLM response complete (all tokens received)
    LlmDone,
    /// TTS audio ready for playback (f32 samples, 24kHz mono)
    TtsAudio(Vec<f32>),
    /// Interrupt current playback — new speech detected
    Interrupt,
    /// Error in any pipeline stage
    Error(String),
    /// Graceful shutdown of all stages
    Shutdown,
}

/// Create pipeline channels for connecting pipeline stages.
/// Returns (audio_tx, stt_tx, tts_tx, tts_rx).
pub fn create_channels() -> (
    Sender<PipelineEvent>,  // audio_tx — send audio chunks from mic
    Receiver<PipelineEvent>, // audio_rx — receive audio for transcription
    Sender<PipelineEvent>,  // tts_tx — send TTS audio for playback
    Receiver<PipelineEvent>, // tts_rx — receive TTS audio for playback
) {
    let (audio_tx, audio_rx) = channel::bounded::<PipelineEvent>(AUDIO_CHANNEL_CAPACITY);
    let (tts_tx, tts_rx) = channel::bounded::<PipelineEvent>(TTS_CHANNEL_CAPACITY);
    (audio_tx, audio_rx, tts_tx, tts_rx)
}
