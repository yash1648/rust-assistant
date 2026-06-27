use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use super::audio::AudioConfig;
use super::io::wait_enter;

/// Record audio from the default microphone to an in-memory buffer (WAV format)
///
/// Returns a `Cursor<Vec<u8>>` containing the WAV data, avoiding disk I/O.
pub fn record_to_buffer() -> Result<Cursor<Vec<u8>>> {
    let host = cpal::default_host();
    let device = host.default_input_device().context("no default input device available")?;
    let config = device.default_input_config().context("no default input config")?;
    let audio_config = AudioConfig::from_device()?;
    let stream_config = config.config();

    let pcm: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));

    let stream = match audio_config.sample_format {
        cpal::SampleFormat::I16 => {
            let pcm_clone = Arc::clone(&pcm);
            device.build_input_stream(&stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    pcm_clone.lock().expect("Audio PCM buffer lock poisoned").extend_from_slice(data);
                },
                move |err| eprintln!("❌ Stream error: {:?}", err), None,
            ).context("failed to build input stream for I16")?
        }
        cpal::SampleFormat::F32 => {
            let pcm_clone = Arc::clone(&pcm);
            device.build_input_stream(&stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut guard = pcm_clone.lock().expect("Audio PCM buffer lock poisoned");
                    for &sample in data {
                        let s = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                        guard.push(s);
                    }
                },
                move |err| eprintln!("❌ Stream error: {:?}", err), None,
            ).context("failed to build input stream for F32")?
        }
        cpal::SampleFormat::I8 | cpal::SampleFormat::U8 | cpal::SampleFormat::U16 => {
            anyhow::bail!("unsupported input sample format: {:?}", audio_config.sample_format);
        }
        _ => anyhow::bail!("unsupported input sample format: {:?}", audio_config.sample_format),
    };

    stream.play()?;
    println!("🎙 Recording... press ENTER to stop.");
    wait_enter()?;
    drop(stream);

    let samples = pcm.lock().expect("Audio PCM buffer lock poisoned during finalize").clone();
    let spec = WavSpec {
        channels: audio_config.channels,
        sample_rate: audio_config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::with_capacity(samples.len() * 2 + 44));
    let mut writer = WavWriter::new(&mut cursor, spec)
        .context("failed to create in-memory WAV writer")?;
    for &sample in &samples {
        writer.write_sample(sample).context("failed to write audio sample")?;
    }
    writer.finalize().context("failed to finalize WAV recording")?;
    cursor.set_position(0);
    Ok(cursor)
}

/// Legacy wrapper: records to a WAV file on disk
pub fn record_to_wav(path: &str) -> Result<()> {
    let buffer = record_to_buffer()?;
    std::fs::write(path, buffer.into_inner())
        .with_context(|| format!("failed to write WAV to {}", path))?;
    Ok(())
}
