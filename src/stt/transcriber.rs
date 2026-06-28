use anyhow::{Context, Result};
use hound::WavReader;
use std::io::Read;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Pure Rust STT using whisper-rs
pub struct WhisperTranscriber {
    ctx: WhisperContext,
    language: String,
}

impl WhisperTranscriber {
    /// Create a new transcriber with the given model path and language.
    ///
    /// GPU acceleration is auto-detected at compile time:
    ///   `cargo build --release --features cuda`  → NVIDIA GPU
    ///   `cargo build --release --features metal` → Apple Silicon
    ///   `cargo build --release --features vulkan` → Cross-platform
    ///
    /// Flash attention is always enabled for faster inference.
    pub fn new(model_path: &str, language: &str) -> Result<Self> {
        let params = WhisperContextParameters {
            flash_attn: true,
            ..Default::default()
        };

        println!("🧠 Loading Whisper model from: {} (lang: {})", model_path, language);
        if params.use_gpu {
            println!("⚡ GPU acceleration enabled — use_gpu=true, flash_attn=true");
            println!("🔧 {}", whisper_rs::print_system_info());
        } else {
            println!("🔧 CPU mode — compile with --features cuda/metal/vulkan for GPU");
        }

        let ctx = WhisperContext::new_with_params(model_path, params)
            .context("failed to load Whisper model")?;

        Ok(Self { ctx, language: language.to_string() })
    }

    /// Transcribe audio from a WAV file path
    pub fn transcribe_wav(&mut self, wav_path: &str) -> Result<String> {
        let reader = WavReader::open(wav_path)
            .with_context(|| format!("failed to open WAV file: {}", wav_path))?;
        let samples = Self::process_reader(reader)?;
        self.transcribe_samples(&samples)
    }

    /// Transcribe audio from an in-memory WAV buffer
    pub fn transcribe_buffer<R: Read>(&mut self, reader: R) -> Result<String> {
        let reader = WavReader::new(reader)
            .context("failed to read WAV from buffer")?;
        let samples = Self::process_reader(reader)?;
        self.transcribe_samples(&samples)
    }

    /// Zero-copy: transcribe directly from raw PCM i16 samples (no WAV encode/decode).
    ///
    /// This is the fast path used by the VAD recorder — avoids encoding to WAV
    /// and decoding back. Resamples to 16kHz and mixes to mono if needed.
    pub fn transcribe_pcm_i16(&mut self, samples: &[i16], sample_rate: u32, channels: u16) -> Result<String> {
        if samples.is_empty() {
            anyhow::bail!("no audio samples found");
        }

        // Convert i16 → f32 inline
        let mut f32_samples: Vec<f32> = Vec::with_capacity(samples.len());
        for &s in samples {
            f32_samples.push(s as f32 / 32768.0);
        }

        // Mix to mono if multiple channels
        let mono = if channels > 1 {
            Self::mix_to_mono(&f32_samples, channels as usize)
        } else {
            f32_samples
        };

        // Resample to 16kHz if needed
        let resampled = if sample_rate != 16000 {
            Self::resample(&mono, sample_rate as f32, 16000.0)?
        } else {
            mono
        };

        self.transcribe_samples(&resampled)
    }

    /// Common transcription pipeline from raw f32 samples
    fn transcribe_samples(&mut self, samples: &[f32]) -> Result<String> {
        if samples.is_empty() {
            anyhow::bail!("no audio samples found — speak into the microphone before pressing Enter");
        }

        let mut state = self.ctx.create_state()
            .context("failed to create Whisper state")?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        // "auto" → let Whisper detect, ISO code → force that language
        if self.language != "auto" {
            params.set_language(Some(&self.language));
        }
        params.set_translate(false);
        params.set_no_context(true);
        params.set_single_segment(true);
        params.set_suppress_blank(true);

        state
            .full(params, samples)
            .context("transcription failed")?;

        let mut transcription = String::new();
        for segment in state.as_iter() {
            let text = segment.to_string().trim().to_string();
            if !text.is_empty() {
                transcription.push_str(&text);
                transcription.push(' ');
            }
        }

        Ok(transcription.trim().to_string())
    }

    /// Read WAV, resample to 16kHz, mix to mono — returns f32 samples
    fn process_reader<R: Read>(reader: WavReader<R>) -> Result<Vec<f32>> {
        let spec = reader.spec();
        println!("📼 WAV: {}Hz, {} channels, {} bits",
            spec.sample_rate, spec.channels, spec.bits_per_sample);

        let samples: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
            (hound::SampleFormat::Int, 16) => {
                reader.into_samples::<i16>()
                    .filter_map(|s| s.ok())
                    .map(|s| s as f32 / 32768.0)
                    .collect()
            }
            (hound::SampleFormat::Int, 32) => {
                reader.into_samples::<i32>()
                    .filter_map(|s| s.ok())
                    .map(|s| s as f32 / 2147483648.0)
                    .collect()
            }
            (hound::SampleFormat::Float, 32) => {
                reader.into_samples::<f32>()
                    .filter_map(|s| s.ok())
                    .collect()
            }
            _ => anyhow::bail!("unsupported WAV format: {} bits", spec.bits_per_sample),
        };

        // Resample to 16kHz if needed
        let samples = if spec.sample_rate != 16000 {
            println!("🔄 Resampling from {}Hz to 16000Hz...", spec.sample_rate);
            Self::resample(&samples, spec.sample_rate as f32, 16000.0)?
        } else {
            samples
        };

        // Mix to mono (handle any channel count)
        if spec.channels > 1 {
            Ok(Self::mix_to_mono(&samples, spec.channels as usize))
        } else {
            Ok(samples)
        }
    }

    /// Simple linear resampling
    fn resample(samples: &[f32], from_rate: f32, to_rate: f32) -> Result<Vec<f32>> {
        if samples.is_empty() {
            return Ok(vec![]);
        }

        let ratio = to_rate / from_rate;
        let new_len = (samples.len() as f32 * ratio).round() as usize;
        let mut result = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = i as f32 / ratio;
            let idx = src_idx as usize;
            let frac = src_idx - idx as f32;

            if idx + 1 < samples.len() {
                let sample = samples[idx] * (1.0 - frac) + samples[idx + 1] * frac;
                result.push(sample);
            } else if idx < samples.len() {
                result.push(samples[idx]);
            }
        }

        Ok(result)
    }

    /// Mix N-channel audio to mono by averaging all channels
    fn mix_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
        samples.chunks(channels)
            .map(|chunk| {
                let sum: f32 = chunk.iter().sum();
                sum / channels as f32
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_empty() {
        let result = WhisperTranscriber::resample(&[], 44100.0, 16000.0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_resample_basic() {
        let input = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0];
        let result = WhisperTranscriber::resample(&input, 44100.0, 16000.0).unwrap();
        // Downsampling by ~2.76x should produce fewer samples
        assert!(result.len() < input.len());
        assert!(!result.is_empty());
    }

    #[test]
    fn test_mix_to_mono_stereo() {
        let stereo = vec![0.5, -0.5, 1.0, -1.0];
        let mono = WhisperTranscriber::mix_to_mono(&stereo, 2);
        assert_eq!(mono.len(), 2);
        assert!((mono[0] - 0.0).abs() < f32::EPSILON);  // (0.5 + -0.5) / 2 = 0
        assert!((mono[1] - 0.0).abs() < f32::EPSILON);  // (1.0 + -1.0) / 2 = 0
    }

    #[test]
    fn test_mix_to_mono_5_channel() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mono = WhisperTranscriber::mix_to_mono(&input, 5);
        assert_eq!(mono.len(), 1);
        assert!((mono[0] - 3.0).abs() < f32::EPSILON);  // (1+2+3+4+5)/5 = 3
    }
}
