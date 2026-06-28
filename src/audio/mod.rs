//! Audio subsystem — lock-free capture + interruptible playback.
//!
//! # Capture
//!
//! `capture::record_to_channel()` sends PCM i16 chunks through a crossbeam
//! channel. No mutex, no blocking in the audio callback — just a lock-free
//! `try_send()`. The receiver accumulates chunks for GPU transcription.
//!
//! # Playback
//!
//! `playback::create_playback()` returns a `rodio::Sink` that supports
//! queueing multiple audio buffers. `playback::interrupt_playback()` clears
//! the queue — used when VAD detects new speech during playback.
//!
//! # Config
//!
//! `AudioConfig` is shared between capture and the rest of the pipeline.

pub mod capture;
pub mod playback;

pub use super::stt::audio::AudioConfig;
