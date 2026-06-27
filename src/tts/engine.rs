//! Pure Rust text-to-speech engine using `kittentts`.
//!
//! Zero system dependencies — no Python, no C libraries, no shared objects.
//! Works on every platform: Linux, macOS, Windows, iOS, Android.
//! Uses ONNX Runtime for fast CPU inference with models as small as 25 MB.

use anyhow::{Context, Result};
use kittentts::KittenTTS;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::path::Path;

/// Pure Rust TTS engine using KittenTTS (ONNX-based, zero system deps)
pub struct TtsEngine {
    inner: KittenTTS,
    voice: String,
    speed: f32,
}

impl TtsEngine {
    /// Load a TTS model from a local directory.
    ///
    /// The model directory should contain:
    /// - `model.onnx` (or `model.int8.onnx` for quantized)
    /// - `voices.npz` (voice embeddings)
    /// - `config.json` (model configuration)
    ///
    /// If `model_dir` doesn't exist, the model will be auto-downloaded
    /// from HuggingFace Hub to the system cache directory.
    pub fn new(voice: &str, model_dir: Option<&Path>, speed: f32) -> Result<Self> {
        let inner = match model_dir {
            Some(dir) if dir.exists() => {
                tracing::info!("Loading TTS model from: {:?}", dir);
                Self::load_local(dir)?
            }
            _ => {
                tracing::info!("Downloading TTS model from HuggingFace Hub...");
                // Auto-download the mini model (~80 MB) — cached after first download
                kittentts::download::load_from_hub("KittenML/kitten-tts-mini-0.8")
                    .context("failed to download/load KittenTTS model from HuggingFace")?
            }
        };

        tracing::info!(
            "TTS ready — {} voice(s): {:?}",
            inner.available_voices.len(),
            inner.available_voices
        );

        Ok(Self {
            inner,
            voice: voice.to_string(),
            speed,
        })
    }

    /// Load a KittenTTS model from local ONNX + NPZ files
    fn load_local(model_dir: &Path) -> Result<KittenTTS> {
        let onnx_path = model_dir.join("model.onnx");
        let int8_path = model_dir.join("model.int8.onnx");
        let npz_path = model_dir.join("voices.npz");

        // Prefer int8 quantized model (faster, smaller)
        let model_path = if int8_path.exists() {
            int8_path
        } else {
            onnx_path
        };

        if !model_path.exists() {
            anyhow::bail!(
                "Model file not found at {:?}. Looking for model.onnx or model.int8.onnx.",
                model_path
            );
        }
        if !npz_path.exists() {
            anyhow::bail!("Voices file not found at {:?}", npz_path);
        }

        kittentts::model::KittenTtsOnnx::load(
            &model_path,
            &npz_path,
            Default::default(),
            Default::default(),
        )
        .context("failed to load KittenTTS model from local files")
    }

    /// Synthesize text to audio samples (f32, 24 kHz, mono)
    pub fn synthesize(&self, text: &str) -> Result<Vec<f32>> {
        self.inner
            .generate(text, &self.voice, self.speed, true)
            .context("TTS synthesis failed")
    }

    /// Synthesize and play audio through the default output device
    pub fn speak_blocking(&self, text: &str) -> Result<()> {
        let samples = self.synthesize(text)?;
        self.play_audio(&samples)
    }

    /// Play f32 audio samples through the default output device
    fn play_audio(&self, samples: &[f32]) -> Result<()> {
        let (_stream, stream_handle) =
            OutputStream::try_default().context("failed to open audio output stream")?;
        let sink = Sink::try_new(&stream_handle).context("failed to create audio sink")?;

        let source = SamplesBuffer::new(1, kittentts::SAMPLE_RATE, samples.to_vec());
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    /// Synthesize and save to a WAV file
    pub fn save_to_wav(&self, text: &str, path: &Path) -> Result<()> {
        let samples = self.synthesize(text)?;
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: kittentts::SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer =
            hound::WavWriter::create(path, spec).context("failed to create WAV file")?;
        for &sample in &samples {
            let clamped = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer
                .write_sample(clamped)
                .context("failed to write WAV sample")?;
        }
        writer.finalize().context("failed to finalize WAV file")?;
        Ok(())
    }

    /// List available voices from the loaded model
    pub fn available_voices(&self) -> &[String] {
        &self.inner.available_voices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_available_voices() {
        // This test requires a model to be downloaded — skip if not available
        let tts = TtsEngine::new("Jasper", None, 1.0);
        if let Ok(engine) = tts {
            assert!(
                !engine.available_voices().is_empty(),
                "should have at least one voice"
            );
        }
        // If download fails (no network), test is skipped gracefully
    }

    #[test]
    fn test_synthesize_short_text() {
        let tts = TtsEngine::new("Jasper", None, 1.0);
        if let Ok(engine) = tts {
            let result = engine.synthesize("Hello world");
            assert!(result.is_ok(), "synthesis should succeed");
            if let Ok(audio) = result {
                assert!(!audio.is_empty(), "audio should not be empty");
                // 24 kHz sample rate → at least ~8000 samples for "Hello world"
                assert!(
                    audio.len() > 1000,
                    "audio too short: {} samples",
                    audio.len()
                );
            }
        }
    }

    #[test]
    fn test_synthesize_empty_text() {
        let tts = TtsEngine::new("Jasper", None, 1.0);
        if let Ok(engine) = tts {
            let result = engine.synthesize("");
            // Should either succeed with silence or return an error
            if let Ok(audio) = result {
                assert!(audio.is_empty() || audio.len() < 1000);
            }
        }
    }
}
