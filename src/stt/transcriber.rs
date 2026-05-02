use anyhow::{Context, Result};
use hound::WavReader;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Pure Rust STT using whisper-rs
pub struct WhisperTranscriber {
    ctx: WhisperContext,
}

impl WhisperTranscriber {
    /// Create a new transcriber with the given model path
    pub fn new(model_path: &str) -> Result<Self> {
        println!("🧠 Loading Whisper model from: {}", model_path);

        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        )
        .context("failed to load Whisper model")?;

        Ok(Self { ctx })
    }

    /// Transcribe audio from a WAV file
    pub fn transcribe_wav(&mut self, wav_path: &str) -> Result<String> {
        // Read WAV file
        let samples = Self::load_wav_samples(wav_path)?;

        if samples.is_empty() {
            anyhow::bail!("no audio samples found in WAV file");
        }

        // Create whisper state
        let mut state = self.ctx.create_state()
            .context("failed to create Whisper state")?;

        // Configure transcription parameters
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_no_context(true);
        params.set_single_segment(true);

        // Run transcription
        state
            .full(params, &samples[..])
            .context("transcription failed")?;

        // Collect all text from segments
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

    /// Load WAV file and convert to f32 mono 16kHz samples
    fn load_wav_samples(wav_path: &str) -> Result<Vec<f32>> {
        let reader = WavReader::open(wav_path)
            .with_context(|| format!("failed to open WAV file: {}", wav_path))?;

        let spec = reader.spec();
        println!("📼 WAV: {}Hz, {} channels, {} bits",
            spec.sample_rate, spec.channels, spec.bits_per_sample);

        // Convert to mono f32 at 16kHz
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

        // Mix stereo to mono if needed
        let samples = if spec.channels == 2 {
            Self::mix_to_mono(&samples)
        } else {
            samples
        };

        Ok(samples)
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

    /// Mix stereo samples to mono
    fn mix_to_mono(samples: &[f32]) -> Vec<f32> {
        samples.chunks(2)
            .map(|chunk| {
                match chunk {
                    &[left, right] => (left + right) / 2.0,
                    &[single] => single,
                    _ => 0.0,
                }
            })
            .collect()
    }
}