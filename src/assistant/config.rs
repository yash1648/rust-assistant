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
}

pub fn default_ollama_server() -> String { "127.0.0.1:11434".into() }
pub fn default_ollama_model() -> String { "qwen3:4b-instruct-2507-q4_K_M".into() }
pub fn default_tts_voice() -> String { "Jasper".into() }
pub fn default_tts_model_dir() -> Option<String> { None }
pub fn default_tts_speed() -> f32 { 1.0 }
pub fn default_stt_model_path() -> String { "models/ggml-base.en.bin".into() }

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
            }
        };

        // Apply environment variable overrides
        if let Ok(v) = env::var("OLLAMA_SERVER") { config.ollama_server = v; }
        if let Ok(v) = env::var("OLLAMA_MODEL") { config.ollama_model = v; }
        if let Ok(v) = env::var("TTS_VOICE") { config.tts_voice = v; }
        if let Ok(v) = env::var("TTS_MODEL_DIR") { config.tts_model_dir = Some(v); }
        if let Ok(v) = env::var("TTS_SPEED") { config.tts_speed = v.parse().unwrap_or(1.0); }
        if let Ok(v) = env::var("STT_MODEL_PATH") { config.stt_model_path = v; }

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
        println!("   STT Model: {}", self.stt_model_path);
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
}

/// Environment variable names
pub mod env_vars {
    pub const OLLAMA_SERVER: &str = "OLLAMA_SERVER";
    pub const OLLAMA_MODEL: &str = "OLLAMA_MODEL";
    pub const TTS_VOICE: &str = "TTS_VOICE";
    pub const TTS_MODEL_DIR: &str = "TTS_MODEL_DIR";
    pub const TTS_SPEED: &str = "TTS_SPEED";
    pub const STT_MODEL_PATH: &str = "STT_MODEL_PATH";
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
model_path = "models/ggml-base.en.bin"
"#.to_string()
}
