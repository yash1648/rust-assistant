//! Lock-free audio capture using `crossbeam` channels.
//!
//! The cpal audio callback sends PCM i16 samples through a bounded,
//! lock-free channel instead of `Arc<Mutex<Vec<i16>>>`. This eliminates
//! mutex contention on every sample callback (zero-wait producer path).

use anyhow::{Context, Result};
use crossbeam::channel::Sender;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use crate::audio::AudioConfig;
use crate::pipeline::PipelineEvent;

/// Maximum recording duration in seconds (safety limit to prevent infinite recording).
pub const MAX_RECORDING_SECS: u64 = 120;

/// Shared state for interrupting recording
#[derive(Default)]
pub struct RecorderState {
    /// Signal from VAD: stop recording
    pub vad_stop: AtomicBool,
    /// Signal from Enter key listener: force stop
    pub enter_pressed: AtomicBool,
}

impl RecorderState {
    pub fn new() -> Self {
        Self {
            vad_stop: AtomicBool::new(false),
            enter_pressed: AtomicBool::new(false),
        }
    }

    pub fn should_stop(&self) -> bool {
        self.vad_stop.load(Ordering::SeqCst) || self.enter_pressed.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.vad_stop.store(false, Ordering::SeqCst);
        self.enter_pressed.store(false, Ordering::SeqCst);
    }
}

/// Record audio from the default microphone, sending PCM i16 chunks through
/// a lock-free crossbeam channel to the pipeline.
///
/// Returns when VAD triggers, Enter is pressed, or safety timeout reached.
/// The audio chunks are accumulated by the receiver for transcription.
pub fn record_to_channel(
    audio_tx: Sender<PipelineEvent>,
    state: Arc<RecorderState>,
    _sample_rate: u32,
    _channels: u16,
) -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device()
        .context("no default input device available")?;
    let config = device.default_input_config()
        .context("no default input config")?;
    let audio_config = AudioConfig::from_device()?;
    let stream_config = config.config();

    let _ = audio_tx.send(PipelineEvent::RecordingStarted);

    let stream = match audio_config.sample_format {
        cpal::SampleFormat::I16 => {
            let tx = audio_tx.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    // Lock-free send to the pipeline channel
                    let _ = tx.try_send(PipelineEvent::AudioChunk(data.to_vec()));
                },
                move |err| eprintln!("❌ Stream error: {:?}", err),
                None,
            ).context("failed to build input stream for I16")?
        }
        cpal::SampleFormat::F32 => {
            let tx = audio_tx.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Convert f32 → i16 in the callback, send through channel
                    let chunk: Vec<i16> = data.iter().map(|&s|
                        (s * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16
                    ).collect();
                    let _ = tx.try_send(PipelineEvent::AudioChunk(chunk));
                },
                move |err| eprintln!("❌ Stream error: {:?}", err),
                None,
            ).context("failed to build input stream for F32")?
        }
        _ => anyhow::bail!("unsupported input sample format: {:?}", audio_config.sample_format),
    };

    stream.play()?;
    println!("🎙 Recording... (VAD auto-stop, ENTER force-stop, {}s timeout)", MAX_RECORDING_SECS);

    // Poll until stop signal, Enter key, or timeout
    let start = std::time::Instant::now();
    while !state.should_stop() {
        if start.elapsed().as_secs() > MAX_RECORDING_SECS {
            println!("⚠️  Recording timeout ({}s) reached.", MAX_RECORDING_SECS);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    drop(stream);
    let _ = audio_tx.send(PipelineEvent::RecordingStopped);
    Ok(())
}
