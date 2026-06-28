//! Interruptible audio playback using `rodio`.
//!
//! Supports clearing the current playback queue when new speech is detected
//! (full-duplex: the assistant can be interrupted mid-sentence).

use anyhow::{Context, Result};
use rodio::{OutputStream, Sink, buffer::SamplesBuffer};

/// Play f32 audio samples through the default output device.
/// Blocks until playback completes or sink is interrupted.
pub fn play_samples(samples: &[f32], sink: &Sink, _sample_rate: u32) -> Result<()> {
    let source = SamplesBuffer::new(1, _sample_rate, samples.to_vec());
    sink.append(source);
    Ok(())
}

/// Create an audio output stream and sink for interruptible playback.
pub fn create_playback() -> Result<(OutputStream, Sink)> {
    let (stream, stream_handle) = OutputStream::try_default()
        .context("failed to open audio output stream")?;
    let sink = Sink::try_new(&stream_handle)
        .context("failed to create audio sink")?;
    Ok((stream, sink))
}

/// Stop all current playback and clear the queue.
/// Called when new speech is detected (interrupt).
pub fn interrupt_playback(sink: &Sink) {
    sink.stop();
    sink.clear();
    tracing::info!("Playback interrupted");
}
