// src/tts/engine.rs
use anyhow::{anyhow, Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub const PIPER_BIN: &str = "piper-tts"; // or "piper"

pub fn synthesize_with_piper(model_path: &Path, text: &str, output_wav: &Path) -> Result<()> {
    println!(
        "🗣  Synthesizing with model {} -> {}",
        model_path.display(),
        output_wav.display()
    );

    let mut child = Command::new(PIPER_BIN)
        .args([
            "-m",
            model_path.to_str().unwrap(),
            // human-ish Cori tuning:
            "--length-scale", "1.2",
            "--noise-scale", "0.5",
            "--noise-w", "0.6",
            "--sentence-silence", "0.25",
            "--output_file",
            output_wav.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn piper")?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("failed to open piper stdin"))?;
        stdin.write_all(text.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("piper failed: {}", stderr));
    }

    Ok(())
}

pub fn play_wav(path: &Path) -> Result<()> {
    println!("▶️  Playing {}", path.display());
    let (_stream, handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&handle)?;
    let file = File::open(path)?;
    let source = Decoder::new(BufReader::new(file))?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
