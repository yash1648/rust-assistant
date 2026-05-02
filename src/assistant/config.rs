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

    #[serde(default = "default_tts_model_path")]
    pub tts_model_path: String,

    #[serde(default = "default_tts_voices_path")]
    pub tts_voices_path: String,

    #[serde(default = "default_stt_model_path")]
    pub stt_model_path: String,
}

fn default_ollama_server() -> String { "127.0.0.1:11434".into() }
fn default_ollama_model() -> String { "gemma3:latest".into() }
fn default_tts_voice() -> String { "af_heart".into() }
fn default_tts_model_path() -> String { "models/kokoro".into() }
fn default_tts_voices_path() -> String { "models/kokoro/voices".into() }
fn default_stt_model_path() -> String { "models/ggml-base.en.bin".into() }

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
                    tts_model_path: toml.tts.as_ref().and_then(|t| t.model_path.clone()).unwrap_or_else(default_tts_model_path),
                    tts_voices_path: toml.tts.as_ref().and_then(|t| t.voices_path.clone()).unwrap_or_else(default_tts_voices_path),
                    stt_model_path: toml.stt.as_ref().and_then(|s| s.model_path.clone()).unwrap_or_else(default_stt_model_path),
                },
                Err(e) => {
                    eprintln!("⚠️  Failed to parse Assistant.toml: {}", e);
                    Config {
                        ollama_server: default_ollama_server(),
                        ollama_model: default_ollama_model(),
                        tts_voice: default_tts_voice(),
                        tts_model_path: default_tts_model_path(),
                        tts_voices_path: default_tts_voices_path(),
                        stt_model_path: default_stt_model_path(),
                    }
                }
            }
        } else {
            Config {
                ollama_server: default_ollama_server(),
                ollama_model: default_ollama_model(),
                tts_voice: default_tts_voice(),
                tts_model_path: default_tts_model_path(),
                tts_voices_path: default_tts_voices_path(),
                stt_model_path: default_stt_model_path(),
            }
        };

        // Apply environment variable overrides
        config.ollama_server = env::var("OLLAMA_SERVER").unwrap_or(config.ollama_server);
        config.ollama_model = env::var("OLLAMA_MODEL").unwrap_or(config.ollama_model);
        config.tts_voice = env::var("TTS_VOICE").unwrap_or(config.tts_voice);
        config.tts_model_path = env::var("TTS_MODEL_PATH").unwrap_or(config.tts_model_path);
        config.tts_voices_path = env::var("TTS_VOICES_PATH").unwrap_or(config.tts_voices_path);
        config.stt_model_path = env::var("STT_MODEL_PATH").unwrap_or(config.stt_model_path);

        config
    }

    /// Print current configuration
    pub fn print(&self) {
        println!("📋 Configuration:");
        println!("   Ollama Server: {}", self.ollama_server);
        println!("   Ollama Model: {}", self.ollama_model);
        println!("   TTS Voice: {}", self.tts_voice);
        println!("   TTS Model: {}", self.tts_model_path);
        println!("   TTS Voices: {}", self.tts_voices_path);
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
    model_path: Option<String>,
    voices_path: Option<String>,
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
    pub const TTS_MODEL_PATH: &str = "TTS_MODEL_PATH";
    pub const TTS_VOICES_PATH: &str = "TTS_VOICES_PATH";
    pub const STT_MODEL_PATH: &str = "STT_MODEL_PATH";
}

/// Print all supported environment variables
pub fn print_env_help() {
    println!("\n🔧 Supported Environment Variables (override Assistant.toml):");
    println!("   {} - Ollama server address", env_vars::OLLAMA_SERVER);
    println!("   {} - Ollama model name", env_vars::OLLAMA_MODEL);
    println!("   {} - TTS voice name", env_vars::TTS_VOICE);
    println!("   {} - Path to TTS model", env_vars::TTS_MODEL_PATH);
    println!("   {} - Path to voice files", env_vars::TTS_VOICES_PATH);
    println!("   {} - Path to Whisper STT model", env_vars::STT_MODEL_PATH);
    println!();
}

/// Generate default Assistant.toml
pub fn generate_default_toml() -> String {
    r#"# Assistant Configuration
# Environment variables override these settings

[ollama]
server = "127.0.0.1:11434"
model = "gemma3:latest"

[tts]
voice = "af_heart"
model_path = "models/kokoro"
voices_path = "models/kokoro/voices"

[stt]
model_path = "models/ggml-base.en.bin"
"#.to_string()
}