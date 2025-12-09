use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};
use std::sync::{Arc, Mutex};
use super::audio::AudioConfig;
use super::io::wait_enter;

pub fn record_to_wav(path: &str) -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("no default input device available")?;

    let config = device
        .default_input_config()
        .context("no default input config")?;

    let audio_config = AudioConfig::from_device()?;

    let spec = WavSpec {
        channels: audio_config.channels,
        sample_rate: audio_config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = Arc::new(Mutex::new(Some(WavWriter::create(path, spec)?)));

    let stream_config = config.config();

    match audio_config.sample_format {
        cpal::SampleFormat::I16 => {
            record_i16_stream(&device, &stream_config, Arc::clone(&writer))?;
        }
        cpal::SampleFormat::F32 => {
            record_f32_stream(&device, &stream_config, Arc::clone(&writer))?;
        }
        other => {
            anyhow::bail!("unsupported input sample format: {:?}", other);
        }
    }

    if let Ok(mut guard) = writer.lock() {
        if let Some(w) = guard.take() {
            w.finalize()?;
        }
    }

    Ok(())
}

fn record_i16_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    writer: Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
) -> Result<()> {
    let stream = device
        .build_input_stream(
            config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer.lock() {
                    if let Some(ref mut w) = *guard {
                        for &sample in data {
                            let _ = w.write_sample(sample);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("❌ Stream error: {:?}", err);
            },
            None,
        )
        .context("failed to build input stream for I16")?;

    stream.play()?;
    println!("🎙 Recording... press ENTER to stop.");
    wait_enter()?;
    drop(stream);

    Ok(())
}

fn record_f32_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    writer: Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
) -> Result<()> {
    let stream = device
        .build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer.lock() {
                    if let Some(ref mut w) = *guard {
                        for &sample in data {
                            let s = (sample * i16::MAX as f32)
                                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                            let _ = w.write_sample(s);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("❌ Stream error: {:?}", err);
            },
            None,
        )
        .context("failed to build input stream for F32")?;

    stream.play()?;
    println!("🎙 Recording... press ENTER to stop.");
    wait_enter()?;
    drop(stream);

    Ok(())
}
