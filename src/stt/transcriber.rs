use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn transcribe_with_whisper(
    whisper_dir: &str,
    model_path: &str,
    wav_path: &str,
) -> Result<String> {
    let bin = find_whisper_binary(whisper_dir)?;
    println!("🧠 Running Whisper...");
    println!("   Binary: {}", bin.display());

    let output = Command::new(&bin)
        .args(["-m", model_path, "-f", wav_path, "-otxt", "-t", "8", "-pp"])
        .output()
        .with_context(|| format!("failed to run whisper at {}", bin.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        crate::ui::error(&format!("transcription failed: {}", stderr));
        anyhow::bail!("transcription failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() || !stderr.is_empty() {
        println!("Whisper output:\n{}{}", stdout, stderr);
    }

    read_transcription(wav_path)
}

fn find_whisper_binary(whisper_dir: &str) -> Result<PathBuf> {
    let cli_bin = if cfg!(target_os = "windows") {
        Path::new(whisper_dir)
            .join("build").join("bin").join("Release").join("whisper-cli.exe")
    } else {
        Path::new(whisper_dir)
            .join("build").join("bin").join("whisper-cli")
    };

    let fallback_bin = if cfg!(target_os = "windows") {
        Path::new(whisper_dir)
            .join("build").join("bin").join("Release").join("main.exe")
    } else {
        Path::new(whisper_dir)
            .join("build").join("bin").join("main")
    };

    if cli_bin.exists() {
        Ok(cli_bin)
    } else if fallback_bin.exists() {
        Ok(fallback_bin)
    } else {
        crate::ui::error("Whisper binary missing");
        Err(crate::error::AssistantError::WhisperNotFound { cli: cli_bin, fallback: fallback_bin }.into())
    }
}

fn read_transcription(wav_path: &str) -> Result<String> {
    let txt_path = format!("{}.txt", wav_path);

    if !Path::new(&txt_path).exists() {
        crate::ui::warn(&format!("⚠️  Output file not created at: {}", txt_path));
        eprintln!("Checking directory contents...");
        for entry in std::fs::read_dir(".")?.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.contains("record") {
                    eprintln!("  Found: {}", name);
                }
            }
        }
        anyhow::bail!("transcription output not found");
    }

    let text = std::fs::read_to_string(&txt_path)?;
    Ok(text.trim().to_string())
}
