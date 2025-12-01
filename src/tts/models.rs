// src/tts/models.rs
use crate::tts::voice::Voice;
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::get;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn models_dir() -> PathBuf {
    PathBuf::from("./models")
}

fn ensure_models_dir() -> Result<()> {
    let dir = models_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).context("creating models directory")?;
    }
    Ok(())
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    println!("⬇️  Downloading {} -> {}", url, dest.display());
    let resp = get(url).with_context(|| format!("requesting {}", url))?;
    if !resp.status().is_success() {
        return Err(anyhow!("failed to download {}: {}", url, resp.status()));
    }
    let bytes = resp.bytes()?;
    let mut file = File::create(dest)?;
    file.write_all(&bytes)?;
    Ok(())
}

pub fn ensure_voice_downloaded(voice: &Voice) -> Result<PathBuf> {
    ensure_models_dir()?;
    let dir = models_dir();

    let model_path = dir.join(format!("{}.onnx", voice.id));
    let config_path = dir.join(format!("{}.onnx.json", voice.id));

    if !model_path.exists() {
        download_file(voice.model_url, &model_path)
            .with_context(|| format!("downloading model for {}", voice.id))?;
    } else {
        println!("✅ Model already present: {}", model_path.display());
    }

    if !config_path.exists() {
        download_file(voice.config_url, &config_path)
            .with_context(|| format!("downloading config for {}", voice.id))?;
    } else {
        println!("✅ Config already present: {}", config_path.display());
    }

    Ok(model_path)
}
