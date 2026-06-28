//! Interruptible audio playback with configurable output device.
//!
//! Supports device selection by name: `AUDIO_OUTPUT_DEVICE` env var or
//! `[audio] output_device` in Assistant.toml. Falls back to system default.
//! Lists available devices via `list_output_devices()`.

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{OutputStream, Sink, buffer::SamplesBuffer};

/// Create an audio output stream and sink from a device name.
///
/// If `device_name` is `None`, uses the system default output device.
/// If specified, enumerates cpal output devices and selects by substring match
/// (case-insensitive). Falls back to default if no match found.
pub fn create_playback(device_name: Option<&str>) -> Result<(OutputStream, Sink)> {
    let (stream, stream_handle) = match device_name {
        Some(name) if !name.is_empty() => {
            let devices = cpal::default_host().output_devices()
                .context("failed to enumerate audio output devices")?;
            let name_lower = name.to_lowercase();
            match devices.into_iter().find(|d| {
                d.name().map(|n| n.to_lowercase().contains(&name_lower)).unwrap_or(false)
            }) {
                Some(device) => {
                    println!("🔊 Audio output: {} (configured)", device.name().unwrap_or_default());
                    OutputStream::try_from_device(&device)
                        .context("failed to open audio output stream")?
                }
                None => {
                    eprintln!("⚠️  Output device '{}' not found — using default", name);
                    OutputStream::try_default()
                        .context("failed to open default audio output stream")?
                }
            }
        }
        _ => {
            OutputStream::try_default()
                .context("failed to open default audio output stream")?
        }
    };
    let sink = Sink::try_new(&stream_handle)
        .context("failed to create audio sink")?;
    Ok((stream, sink))
}

/// Play f32 audio samples through the given sink.
pub fn play_samples(samples: &[f32], sink: &Sink, sample_rate: u32) -> Result<()> {
    let source = SamplesBuffer::new(1, sample_rate, samples.to_vec());
    sink.append(source);
    Ok(())
}

/// Stop all current playback and clear the queue (speech interrupt).
pub fn interrupt_playback(sink: &Sink) {
    sink.stop();
    sink.clear();
    tracing::info!("Playback interrupted");
}

/// List all available output devices (used by doctor command).
pub fn list_output_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices = host.output_devices()
        .context("failed to enumerate audio output devices")?;
    let names: Vec<String> = devices
        .filter_map(|d| d.name().ok())
        .collect();
    Ok(names)
}
