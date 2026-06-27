use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};
use std::io::Cursor;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use super::audio::AudioConfig;
use super::io::{spawn_enter_listener, wait_enter};
use super::vad::{VadConfig, VadState};

/// Record audio from the default microphone to an in-memory buffer (WAV format)
///
/// Returns a `Cursor<Vec<u8>>` containing the WAV data, avoiding disk I/O.
pub fn record_to_buffer() -> Result<Cursor<Vec<u8>>> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .context("no default input device available")?;

    let config = device
        .default_input_config()
        .context("no default input config")?;

    let audio_config = AudioConfig::from_device()?;
    let stream_config = config.config();

    // Collect raw PCM I16 samples in a shared buffer (avoids needing Seek + Write sharing)
    let pcm: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));

    let stream = match audio_config.sample_format {
        cpal::SampleFormat::I16 => {
            let pcm_clone = Arc::clone(&pcm);
            device
                .build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        pcm_clone
                            .lock()
                            .expect("Audio PCM buffer lock poisoned")
                            .extend_from_slice(data);
                    },
                    move |err| {
                        eprintln!("❌ Stream error: {:?}", err);
                    },
                    None,
                )
                .context("failed to build input stream for I16")?
        }
        cpal::SampleFormat::F32 => {
            let pcm_clone = Arc::clone(&pcm);
            device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut guard = pcm_clone
                            .lock()
                            .expect("Audio PCM buffer lock poisoned");
                        for &sample in data {
                            let s = (sample * i16::MAX as f32)
                                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                            guard.push(s);
                        }
                    },
                    move |err| {
                        eprintln!("❌ Stream error: {:?}", err);
                    },
                    None,
                )
                .context("failed to build input stream for F32")?
        }
        cpal::SampleFormat::I8 | cpal::SampleFormat::U8 | cpal::SampleFormat::U16 => {
            anyhow::bail!(
                "unsupported input sample format: {:?}. Try setting your system audio to 16-bit signed or 32-bit float.",
                audio_config.sample_format
            );
        }
        _ => anyhow::bail!(
            "unsupported input sample format: {:?}",
            audio_config.sample_format
        ),
    };

    stream.play()?;
    println!("🎙 Recording... press ENTER to stop.");
    wait_enter()?;
    drop(stream);

    // Write collected PCM samples as WAV via hound (using &mut Cursor as the inner writer)
    let samples = pcm
        .lock()
        .expect("Audio PCM buffer lock poisoned during finalize")
        .clone();

    let spec = WavSpec {
        channels: audio_config.channels,
        sample_rate: audio_config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::with_capacity(
        samples.len() * 2 + 44, // PCM data + WAV header estimate
    ));
    let mut writer = WavWriter::new(&mut cursor, spec)
        .context("failed to create in-memory WAV writer")?;
    for &sample in &samples {
        writer
            .write_sample(sample)
            .context("failed to write audio sample")?;
    }
    writer.finalize().context("failed to finalize WAV recording")?;
    cursor.set_position(0);
    Ok(cursor)
}

/// Maximum recording duration in seconds (safety limit to prevent infinite recording).
const MAX_RECORDING_SECS: u64 = 120;

/// Record audio with Voice Activity Detection (auto-stops on silence).
///
/// Falls back to Enter key if VAD doesn't trigger (e.g., noisy environment).
/// Includes a 2-minute safety timeout to prevent infinite recording.
/// Returns a `Cursor<Vec<u8>>` containing WAV data.
pub fn record_to_buffer_vad(vad_config: VadConfig) -> Result<Cursor<Vec<u8>>> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .context("no default input device available")?;

    let config = device
        .default_input_config()
        .context("no default input config")?;

    let audio_config = AudioConfig::from_device()?;
    let stream_config = config.config();

    // Start a background thread to listen for Enter press
    let (enter_pressed, _enter_handle) = spawn_enter_listener();

    // Shared PCM buffer and VAD state
    let pcm: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
    let vad = Arc::new(VadState::new(vad_config));

    let stream = match audio_config.sample_format {
        cpal::SampleFormat::I16 => {
            let pcm_clone = Arc::clone(&pcm);
            let vad_clone = Arc::clone(&vad);
            device
                .build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Accumulate samples
                        if let Ok(mut guard) = pcm_clone.lock() {
                            guard.extend_from_slice(data);
                        }
                        // VAD processing (atomic, no blocking)
                        vad_clone.process_audio(data);
                    },
                    move |err| {
                        eprintln!("❌ Stream error: {:?}", err);
                    },
                    None,
                )
                .context("failed to build input stream for I16")?
        }
        cpal::SampleFormat::F32 => {
            let pcm_clone = Arc::clone(&pcm);
            let vad_clone = Arc::clone(&vad);
            device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut guard) = pcm_clone.lock() {
                            for &sample in data {
                                let s = (sample * i16::MAX as f32)
                                    .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                                guard.push(s);
                            }
                        }
                        vad_clone.process_audio_f32(data);
                    },
                    move |err| {
                        eprintln!("❌ Stream error: {:?}", err);
                    },
                    None,
                )
                .context("failed to build input stream for F32")?
        }
        cpal::SampleFormat::I8 | cpal::SampleFormat::U8 | cpal::SampleFormat::U16 => {
            anyhow::bail!(
                "unsupported input sample format: {:?}. Try setting your system audio to 16-bit signed or 32-bit float.",
                audio_config.sample_format
            );
        }
        _ => anyhow::bail!(
            "unsupported input sample format: {:?}",
            audio_config.sample_format
        ),
    };

    stream.play()?;
    println!("🎙 Recording... (VAD auto-stop, press ENTER to force-stop, {}s timeout)", MAX_RECORDING_SECS);

    // Poll until VAD triggers, Enter is pressed, or safety timeout
    let start = std::time::Instant::now();
    while !vad.is_stopped() && !enter_pressed.load(Ordering::SeqCst) {
        if start.elapsed().as_secs() > MAX_RECORDING_SECS {
            println!("⚠️  Recording timeout ({}s) reached.", MAX_RECORDING_SECS);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    drop(stream);

    // Write collected PCM samples as WAV
    let samples = pcm
        .lock()
        .expect("Audio PCM buffer lock poisoned during finalize")
        .clone();

    let spec = WavSpec {
        channels: audio_config.channels,
        sample_rate: audio_config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::with_capacity(
        samples.len() * 2 + 44,
    ));
    let mut writer = WavWriter::new(&mut cursor, spec)
        .context("failed to create in-memory WAV writer")?;
    for &sample in &samples {
        writer
            .write_sample(sample)
            .context("failed to write audio sample")?;
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
