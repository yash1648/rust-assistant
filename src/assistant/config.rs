use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaConfig {
    pub server: String, // e.g. "172.30.176.1:11434"
    pub model: String,  // e.g. "lucy:latest"
}

pub fn load_or_create_config() -> Result<OllamaConfig> {
    let path = "ollama_config.json";

    // If config exists, just load it
    if Path::new(path).exists() {
        let data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let cfg: OllamaConfig =
            serde_json::from_str(&data).context("Failed to parse config JSON")?;
        return Ok(cfg);
    }

    // Otherwise, ask the user once and create it
    println!("No config found. Let's set up your Ollama connection.");

    // Ask for server
    print!("Enter Ollama server IP:PORT (example: 172.30.176.1:11434): ");
    io::stdout().flush().unwrap();
    let mut server = String::new();
    io::stdin().read_line(&mut server).unwrap();
    let server = server.trim().to_string();

    // Ask for model
    print!("Enter model name (example: lucy:latest): ");
    io::stdout().flush().unwrap();
    let mut model = String::new();
    io::stdin().read_line(&mut model).unwrap();
    let model = model.trim().to_string();

    let cfg = OllamaConfig { server, model };

    let json = serde_json::to_string_pretty(&cfg).context("Failed to serialize config")?;
    fs::write(path, json).with_context(|| format!("Failed to write config file: {}", path))?;

    println!("Config saved to {}", path);

    Ok(cfg)
}
pub fn get_ollama_config() -> Result<OllamaConfig> {
    load_or_create_config()
}
