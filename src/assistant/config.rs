use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig { pub server: String, pub model: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantConfig { pub ollama: OllamaConfig }

pub fn default_config() -> AssistantConfig {
    AssistantConfig { ollama: get_ollama_config().unwrap_or(OllamaConfig {
        server: "localhost:11434".into(),
        model: "phi-2.7".into(),
    }) }
}

pub fn load_or_create_config() -> Result<AssistantConfig> {
    let path = "Assistant.toml";
    if std::path::Path::new(path).exists() {
        let s = std::fs::read_to_string(path)?;
        let cfg: AssistantConfig = toml::from_str(&s)?;
        Ok(cfg)
    } else {
        let cfg = default_config();
        let s = toml::to_string_pretty(&cfg)?;
        std::fs::write(path, s)?;
        Ok(cfg)
    }
}

pub fn get_ollama_config() -> Result<OllamaConfig> {
    Ok(load_or_create_config()?.ollama)
}
