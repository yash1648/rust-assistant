// Many re-exported public API items are unused by the binary itself
#![allow(dead_code)]

use anyhow::Result;
use clap::{Parser, CommandFactory};
use clap_complete::{generate_to, shells};
use cpal::traits::{DeviceTrait, HostTrait};
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
use cli::{Cli, Commands, ShellKind};

mod assistant;
mod stt;
mod tts;
mod ui;
mod error;

fn init_tracing(verbosity: u8) {
    let level = match verbosity { 0 => "info", 1 => "debug", _ => "trace" };
    let filter = EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| level.to_string()));
    fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Auto-load .env file if it exists (silently ignore if not)
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => {
            let mut assistant = assistant::Assistant::new()?;
            assistant.run().await?;
        }
        Commands::Setup { whisper_model, skip_whisper, tts_model_dir, force } => {
            setup(whisper_model, skip_whisper, tts_model_dir, force).await?;
        }
        Commands::Doctor => {
            doctor();
        }
        Commands::GenerateCompletion { shell } => {
            let mut cmd = Cli::command();
            std::fs::create_dir_all("completions/bash")?;
            std::fs::create_dir_all("completions/zsh")?;
            std::fs::create_dir_all("completions/fish")?;
            match shell {
                ShellKind::Bash => { generate_to(shells::Bash, &mut cmd, "rust-assistant", "completions/bash")?; }
                ShellKind::Zsh => { generate_to(shells::Zsh, &mut cmd, "rust-assistant", "completions/zsh")?; }
                ShellKind::Fish => { generate_to(shells::Fish, &mut cmd, "rust-assistant", "completions/fish")?; }
            }
            println!("✅ Generated completions for {:?}", shell);
        }
        Commands::Env => {
            assistant::config::print_env_help();
        }
        Commands::GenerateConfig => {
            let config = assistant::config::generate_default_toml();
            std::fs::write("Assistant.toml", config)?;
            println!("✅ Generated Assistant.toml");
        }
    }

    Ok(())
}

/// Set up the project: create directories, download models
async fn setup(
    whisper_model: String,
    skip_whisper: bool,
    tts_model_dir: Option<String>,
    force: bool,
) -> Result<()> {
    use std::path::Path;
    use indicatif::ProgressBar;

    println!("🔧 rust-assistant setup\n");

    // Create necessary directories
    let dirs = ["models", "records", "completions/bash", "completions/zsh", "completions/fish"];
    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }
    println!("✅ Created directories");

    // Download Whisper model if needed
    if !skip_whisper {
        let whisper_path = Path::new(&whisper_model);
        if whisper_path.exists() && !force {
            println!("✅ Whisper model already exists at: {}", whisper_model);
        } else {
            println!("📥 Downloading Whisper model to {}...", whisper_model);

            // Ensure parent directory exists
            if let Some(parent) = whisper_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let model_name = whisper_path.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let url = format!(
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
                model_name
            );

            let pb = ProgressBar::new_spinner();
            pb.set_message(format!("Downloading {}...", model_name));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()?;
            let response = client.get(&url).send().await?;

            if !response.status().is_success() {
                anyhow::bail!("Failed to download model: HTTP {}", response.status());
            }

            let bytes = response.bytes().await?;
            std::fs::write(whisper_path, &bytes)?;

            pb.finish_and_clear();
            println!("✅ Downloaded {} ({} MB)", model_name, bytes.len() / 1024 / 1024);
        }
    }

    // Verify TTS model is available
    if let Some(dir) = &tts_model_dir {
        let tts_path = Path::new(dir);
        if tts_path.exists() && !force {
            println!("✅ TTS model directory exists: {}", dir);
        } else if let Some(parent) = tts_path.parent() {
            std::fs::create_dir_all(parent)?;
            println!("📥 TTS model will be auto-downloaded on first run from HuggingFace");
            println!("   Model: KittenML/kitten-tts-mini-0.8");
        }
    } else {
        println!("📥 TTS model will be auto-downloaded on first run from HuggingFace");
        println!("   Model: KittenML/kitten-tts-mini-0.8 (~80 MB)");
    }

    // Generate default config if not exists
    if !Path::new("Assistant.toml").exists() {
        let config = assistant::config::generate_default_toml();
        std::fs::write("Assistant.toml", config)?;
        println!("✅ Generated default Assistant.toml");
    }

    println!("\n✅ Setup complete!");
    Ok(())
}

/// Check system dependencies and configuration (synchronous)
fn doctor() {
    use std::process::Command;

    println!("🔍 rust-assistant doctor\n");

    // Check Rust version
    println!("🔧 Rust toolchain:");
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("   ✅ rustc {}", version.trim());
    } else {
        println!("   ❌ rustc not found — install from https://rustup.rs");
    }

    if let Ok(output) = Command::new("cargo").arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("   ✅ cargo {}", version.trim());
    } else {
        println!("   ❌ cargo not found");
    }

    // Check Ollama (blocking HTTP call)
    println!("\n🤖 Ollama (local LLM):");
    let config = assistant::config::Config::from_toml();
    match check_ollama(&config) {
        Ok(()) => {},
        Err(e) => println!("   ❌ {}", e),
    }

    // Check audio devices
    println!("\n🎙 Audio system:");
    match cpal::default_host().default_input_device() {
        Some(dev) => {
            match dev.name() {
                Ok(name) => println!("   ✅ Input device: {}", name),
                Err(_) => println!("   ✅ Input device available"),
            }
        }
        None => println!("   ❌ No default input device found"),
    }
    match cpal::default_host().default_output_device() {
        Some(dev) => {
            match dev.name() {
                Ok(name) => println!("   ✅ Output device: {}", name),
                Err(_) => println!("   ✅ Output device available"),
            }
        }
        None => println!("   ❌ No default output device found"),
    }

    // Check config
    println!("\n📋 Configuration:");
    let config_path = std::path::Path::new("Assistant.toml");
    if config_path.exists() {
        println!("   ✅ Assistant.toml found");
        config.print();
    } else {
        println!("   ⚠️  Assistant.toml not found — run: cargo run generate-config");
    }

    // Check model files
    println!("\n📦 Model files:");
    let stt_path = std::path::Path::new(&config.stt_model_path);
    if stt_path.exists() {
        match std::fs::metadata(stt_path) {
            Ok(m) => println!("   ✅ STT model ({}): {} MB", config.stt_model_path, m.len() / 1024 / 1024),
            Err(e) => println!("   ❌ Cannot read STT model: {}", e),
        }
    } else {
        println!("   ❌ STT model not found: {} — run: cargo run setup", config.stt_model_path);
    }

    match &config.tts_model_dir {
        Some(dir) => {
            let path = std::path::Path::new(dir);
            if path.exists() {
                println!("   ✅ TTS model directory: {}", dir);
            } else {
                println!("   ⚠️  TTS model directory '{}' not found (will auto-download)", dir);
            }
        }
        None => {
            println!("   ℹ️  TTS model will auto-download from HuggingFace on first run");
        }
    }

    println!("\n✅ Doctor check complete!");
}

/// Check Ollama connectivity (blocking)
fn check_ollama(config: &assistant::config::Config) -> Result<(), String> {
    let url = format!("http://{}/api/tags", config.ollama_server);
    let resp = reqwest::blocking::get(&url)
        .map_err(|e| format!("Cannot connect to Ollama at {}: {}", config.ollama_server, e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned status: {}", resp.status()));
    }

    println!("   ✅ Ollama running at {}", config.ollama_server);

    if let Ok(body) = resp.json::<serde_json::Value>() {
        if let Some(models) = body["models"].as_array() {
            let names: Vec<&str> = models.iter()
                .filter_map(|m| m["name"].as_str())
                .collect();
            println!("   📦 Available models: {}", names.join(", "));
            let has_model = models.iter().any(|m| {
                m["name"].as_str().is_some_and(|n| n.contains(&config.ollama_model))
            });
            if has_model {
                println!("   ✅ Default model '{}' is available", config.ollama_model);
            } else {
                println!("   ⚠️  Default model '{}' not found — run: ollama pull {}", config.ollama_model, config.ollama_model);
            }
        }
    }

    Ok(())
}
