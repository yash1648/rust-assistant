use anyhow::Result;
use any_tts::{load_model, AudioSamples, ModelType, SynthesisRequest, TtsConfig};
use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
use std::path::Path;

/// Pure Rust Kokoro TTS using any-tts
pub struct KokoroTts {
    model: Box<dyn any_tts::TtsModel + Send + Sync>,
    voice: String,
}

impl KokoroTts {
    /// Initialize Kokoro TTS engine with explicit paths
    pub fn new(voice: &str, model_path: &Path, voices_path: &Path) -> Result<Self> {
        println!("🎤 Loading Kokoro TTS");
        println!("   Voice: {}", voice);
        println!("   Model: {:?}", model_path);
        println!("   Voices: {:?}", voices_path);

        let model = load_model(
            TtsConfig::new(ModelType::Kokoro)
                .with_model_path(model_path.to_string_lossy().as_ref())
                .with_voices_dir(voices_path.to_string_lossy().as_ref()),
        )
        .map_err(|e| anyhow::anyhow!("failed to load Kokoro model: {}", e))?;

        Ok(Self { model, voice: voice.to_string() })
    }

    /// Generate speech and play it synchronously
    pub fn speak_and_play_blocking(&self, text: &str) -> Result<()> {
        let audio = self.synthesize(text)?;
        self.play_audio_blocking(audio)
    }

    /// Synthesize text to audio
    fn synthesize(&self, text: &str) -> Result<AudioSamples> {
        let request = SynthesisRequest::new(text)
            .with_voice(&self.voice)
            .with_language("en");

        let audio = self.model
            .synthesize(&request)
            .map_err(|e| anyhow::anyhow!("synthesis failed: {}", e))?;

        Ok(audio)
    }

    /// Play audio samples synchronously
    fn play_audio_blocking(&self, audio: AudioSamples) -> Result<()> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        // Convert f32 to i16
        let samples_i16: Vec<i16> = audio.samples
            .iter()
            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();

        let source = SamplesBuffer::new(1, audio.sample_rate as u32, samples_i16);
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }
}