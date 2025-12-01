use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn transcribe_with_whisper(
    whisper_dir: &str,
    model_path: &str,
    wav_path: &str,
) -> Result<String> {
    let bin_to_use = find_whisper_binary(whisper_dir)?;
    
    println!("🧠 Running Whisper...");
    println!("   Binary: {}", bin_to_use.display());
    
    let output = Command::new(&bin_to_use)
        .args([
            "-m", model_path,
            "-f", wav_path,
            "-otxt",
            "-t", "8",
            "-pp",
        ])
        .output()
        .with_context(|| format!("failed to run whisper at {}", bin_to_use.display()))?;
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if !stdout.is_empty() || !stderr.is_empty() {
        println!("Whisper output:\n{}{}", stdout, stderr);
    }
    
    if !output.status.success() {
        anyhow::bail!("whisper transcription failed with exit code: {:?}", output.status.code());
    }
    
    read_transcription(wav_path)
}

fn find_whisper_binary(whisper_dir: &str) -> Result<std::path::PathBuf> {
    let cli_bin = if cfg!(target_os = "windows") {
        Path::new(whisper_dir)
            .join("build")
            .join("bin")
            .join("Release")
            .join("whisper-cli.exe")
    } else {
        Path::new(whisper_dir)
            .join("build")
            .join("bin")
            .join("whisper-cli")
    };
    
    let fallback_bin = if cfg!(target_os = "windows") {
        Path::new(whisper_dir)
            .join("build")
            .join("bin")
            .join("Release")
            .join("main.exe")
    } else {
        Path::new(whisper_dir)
            .join("build")
            .join("bin")
            .join("main")
    };
    
    if cli_bin.exists() {
        Ok(cli_bin)
    } else if fallback_bin.exists() {
        Ok(fallback_bin)
    } else {
        anyhow::bail!(
            "whisper binary not found at {} or {}",
            cli_bin.display(),
            fallback_bin.display()
        )
    }
}

fn read_transcription(wav_path: &str) -> Result<String> {
    let txt_path = format!("{}.txt", wav_path);
    
    if !Path::new(&txt_path).exists() {
        eprintln!("⚠️  Output file not created at: {}", txt_path);
        eprintln!("Checking directory contents...");
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.contains("record") {
                        eprintln!("  Found: {}", name);
                    }
                }
            }
        }
        anyhow::bail!("whisper did not create output file at {}", txt_path);
    }
    
    let text = std::fs::read_to_string(&txt_path)
        .with_context(|| format!("reading whisper output from {}", txt_path))?;
    
    Ok(text.trim().to_string())
}