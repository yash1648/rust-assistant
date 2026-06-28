use serde::Deserialize;
use std::env;
use std::path::Path;

/// Application configuration loaded from Assistant.toml with env var overrides
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_ollama_server")]
    pub ollama_server: String,

    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,

    #[serde(default = "default_tts_voice")]
    pub tts_voice: String,

    #[serde(default = "default_tts_model_dir")]
    pub tts_model_dir: Option<String>,

    #[serde(default = "default_tts_speed")]
    pub tts_speed: f32,

    #[serde(default = "default_stt_model_path")]
    pub stt_model_path: String,

    // STT language (auto-detect, or ISO code like "en", "fr", "de", "ja")
    #[serde(default = "default_stt_language")]
    pub stt_language: String,

    // Audio output device name (optional — uses system default if unset)
    #[serde(default)]
    pub output_device: Option<String>,

    // VAD settings
    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,

    #[serde(default = "default_vad_silence_ms")]
    pub vad_silence_ms: u64,
}

pub fn default_ollama_server() -> String { "127.0.0.1:11434".into() }
pub fn default_ollama_model() -> String { "qwen3:4b-instruct-2507-q4_K_M".into() }
pub fn default_tts_voice() -> String { "Jasper".into() }
pub fn default_tts_model_dir() -> Option<String> { None }
pub fn default_tts_speed() -> f32 { 1.0 }
pub fn default_stt_model_path() -> String { "models/ggml-tiny.en.bin".into() }
pub fn default_stt_language() -> String { "auto".into() }
pub fn default_vad_threshold() -> f32 { 0.02 }
pub fn default_vad_silence_ms() -> u64 { 800 }

impl Default for Config {
    fn default() -> Self {
        Self::from_toml()
    }
}

impl Config {
    /// Load configuration from Assistant.toml with env overrides
    pub fn from_toml() -> Self {
        let path = Path::new("Assistant.toml");

        let mut config = if path.exists() {
            match toml::from_str::<TomlConfig>(&std::fs::read_to_string(path).unwrap_or_default()) {
                Ok(toml) => Config {
                    ollama_server: toml.ollama.as_ref().and_then(|o| o.server.clone()).unwrap_or_else(default_ollama_server),
                    ollama_model: toml.ollama.as_ref().and_then(|o| o.model.clone()).unwrap_or_else(default_ollama_model),
                    tts_voice: toml.tts.as_ref().and_then(|t| t.voice.clone()).unwrap_or_else(default_tts_voice),
                    tts_model_dir: toml.tts.as_ref().and_then(|t| t.model_dir.clone()).or_else(default_tts_model_dir),
                    tts_speed: toml.tts.as_ref().and_then(|t| t.speed).unwrap_or_else(default_tts_speed),
                    stt_model_path: toml.stt.as_ref().and_then(|s| s.model_path.clone()).unwrap_or_else(default_stt_model_path),
                    stt_language: toml.stt.as_ref().and_then(|s| s.language.clone()).unwrap_or_else(default_stt_language),
                    output_device: toml.audio.as_ref().and_then(|a| a.output_device.clone()),
                    vad_threshold: toml.vad.as_ref().and_then(|v| v.threshold).unwrap_or_else(default_vad_threshold),
                    vad_silence_ms: toml.vad.as_ref().and_then(|v| v.silence_ms).unwrap_or_else(default_vad_silence_ms),
                },
                Err(e) => {
                    eprintln!("⚠️  Failed to parse Assistant.toml: {}", e);
                    Config {
                        ollama_server: default_ollama_server(),
                        ollama_model: default_ollama_model(),
                        tts_voice: default_tts_voice(),
                        tts_model_dir: default_tts_model_dir(),
                        tts_speed: default_tts_speed(),
                        stt_model_path: default_stt_model_path(),
                        stt_language: default_stt_language(),
                        output_device: None,
                        vad_threshold: default_vad_threshold(),
                        vad_silence_ms: default_vad_silence_ms(),
                    }
                }
            }
        } else {
            Config {
                ollama_server: default_ollama_server(),
                ollama_model: default_ollama_model(),
                tts_voice: default_tts_voice(),
                tts_model_dir: default_tts_model_dir(),
                tts_speed: default_tts_speed(),
                stt_model_path: default_stt_model_path(),
                stt_language: default_stt_language(),
                output_device: None,
                vad_threshold: default_vad_threshold(),
                vad_silence_ms: default_vad_silence_ms(),
            }
        };

        // Apply environment variable overrides
        if let Ok(v) = env::var("OLLAMA_SERVER") { config.ollama_server = v; }
        if let Ok(v) = env::var("OLLAMA_MODEL") { config.ollama_model = v; }
        if let Ok(v) = env::var("TTS_VOICE") { config.tts_voice = v; }
        if let Ok(v) = env::var("TTS_MODEL_DIR") { config.tts_model_dir = Some(v); }
        if let Ok(v) = env::var("TTS_SPEED") { config.tts_speed = v.parse().unwrap_or(1.0); }
        if let Ok(v) = env::var("STT_MODEL_PATH") { config.stt_model_path = v; }
        if let Ok(v) = env::var("STT_LANGUAGE") { config.stt_language = v; }
        if let Ok(v) = env::var("AUDIO_OUTPUT_DEVICE") { config.output_device = Some(v); }
        if let Ok(v) = env::var("VAD_THRESHOLD") { config.vad_threshold = v.parse().unwrap_or(0.02); }
        if let Ok(v) = env::var("VAD_SILENCE_MS") { config.vad_silence_ms = v.parse().unwrap_or(800); }

        config
    }

    /// Print current configuration
    pub fn print(&self) {
        println!("📋 Configuration:");
        println!("   Ollama Server: {}", self.ollama_server);
        println!("   Ollama Model: {}", self.ollama_model);
        println!("   TTS Voice: {}", self.tts_voice);
        match &self.tts_model_dir {
            Some(d) => println!("   TTS Model: {} (local)", d),
            None => println!("   TTS Model: auto-download (HuggingFace)"),
        }
        println!("   TTS Speed: {}x", self.tts_speed);
        println!("   STT Model: {} (lang: {})", self.stt_model_path, self.stt_language);
        match &self.output_device {
            Some(d) => println!("   Audio Output: {} (configured)", d),
            None => println!("   Audio Output: default system device"),
        }
        println!("   VAD Threshold: {:.3}", self.vad_threshold);
        println!("   VAD Silence: {}ms", self.vad_silence_ms);
    }
}

/// TOML config structure (for parsing Assistant.toml)
#[derive(Debug, Deserialize, Default)]
struct TomlConfig {
    #[serde(default)]
    ollama: Option<OllamaSection>,

    #[serde(default)]
    tts: Option<TtsSection>,

    #[serde(default)]
    stt: Option<SttSection>,

    #[serde(default)]
    audio: Option<AudioSection>,

    #[serde(default)]
    vad: Option<VadSection>,
}

#[derive(Debug, Deserialize)]
struct OllamaSection {
    server: Option<String>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TtsSection {
    voice: Option<String>,
    model_dir: Option<String>,
    speed: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct SttSection {
    model_path: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AudioSection {
    output_device: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VadSection {
    threshold: Option<f32>,
    silence_ms: Option<u64>,
}

/// Environment variable names
pub mod env_vars {
    pub const OLLAMA_SERVER: &str = "OLLAMA_SERVER";
    pub const OLLAMA_MODEL: &str = "OLLAMA_MODEL";
    pub const TTS_VOICE: &str = "TTS_VOICE";
    pub const TTS_MODEL_DIR: &str = "TTS_MODEL_DIR";
    pub const TTS_SPEED: &str = "TTS_SPEED";
    pub const STT_MODEL_PATH: &str = "STT_MODEL_PATH";
    pub const STT_LANGUAGE: &str = "STT_LANGUAGE";
    pub const AUDIO_OUTPUT_DEVICE: &str = "AUDIO_OUTPUT_DEVICE";
    pub const VAD_THRESHOLD: &str = "VAD_THRESHOLD";
    pub const VAD_SILENCE_MS: &str = "VAD_SILENCE_MS";
}

/// Print all supported environment variables
pub fn print_env_help() {
    println!("\n🔧 Supported Environment Variables (override Assistant.toml):");
    println!("   {} - Ollama server address", env_vars::OLLAMA_SERVER);
    println!("   {} - Ollama model name", env_vars::OLLAMA_MODEL);
    println!("   {} - TTS voice name", env_vars::TTS_VOICE);
    println!("   {} - Path to local TTS model directory", env_vars::TTS_MODEL_DIR);
    println!("   {} - TTS speech speed multiplier (0.5-2.0)", env_vars::TTS_SPEED);
    println!("   {} - Path to Whisper STT model", env_vars::STT_MODEL_PATH);
    println!("   {} - STT language (auto-detect or ISO code like en/fr/de)", env_vars::STT_LANGUAGE);
    println!("   {} - Audio output device name (optional, lists devices with doctor)", env_vars::AUDIO_OUTPUT_DEVICE);
    println!("   {} - VAD energy threshold (0.0–1.0, default 0.02)", env_vars::VAD_THRESHOLD);
    println!("   {} - VAD silence timeout in ms (default 800)", env_vars::VAD_SILENCE_MS);
    println!();
}

/// Generate default Assistant.toml
pub fn generate_default_toml() -> String {
    r#"# rust-assistant Configuration
# Environment variables override these settings.
# All paths are relative to the project root.

[ollama]
server = "127.0.0.1:11434"
model = "gemma3:latest"

[tts]
# Voice name (Jasper, Luna, Bella, Bruno, Rosie, Hugo, Kiki, Leo)
voice = "Jasper"
# Local model directory (optional — auto-downloads from HuggingFace if unset)
# model_dir = "models/kitten-tts-mini"
# Speech speed (0.5 = half speed, 2.0 = double speed)
speed = 1.0

[stt]
model_path = "models/ggml-tiny.en.bin"
# Language for transcription (auto-detect, or "en"/"fr"/"de"/"ja"/etc.)
# language = "auto"

[audio]
# Audio output device name (partial match, case-insensitive)
# Run `cargo run doctor` to list available devices
# output_device = "Speaker"
"#.to_string()
}
