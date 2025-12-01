mod tts;

use anyhow::Result;
use std::path::PathBuf;

use crate::tts::{ensure_voice_downloaded, play_wav, synthesize_with_piper, voices};
fn main() -> Result<()> {
    let test_text = "Hey, this is the Cori voice running in Rust with Piper. \
                     I'm trying to sound as natural and human as possible for you. \
                     Let me know if you like this voice, or if you want to try a different style.";

    for voice in voices() {
        println!("\n==============================");
        println!("🎧 Voice: {}", voice.id);
        println!("==============================");

        // 1. Ensure model is downloaded
        let model_path = ensure_voice_downloaded(&voice)?;

        // 2. Synthesize to wav
        let output_wav = PathBuf::from(format!("{}.wav", voice.id));
        synthesize_with_piper(&model_path, test_text, &output_wav)?;

        // 3. Play it
        play_wav(&output_wav)?;

        println!("✅ Finished voice: {}", voice.id);
    }

    println!("\nAll voices played. Done.");
    Ok(())
}
