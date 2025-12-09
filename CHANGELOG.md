## Removed Unused Functions

- src/tts/voice.rs::voices(): listed Piper voices; removed because Kokoro TTS is the supported path.
- src/tts/voice.rs::Voice: voice struct used by Piper download; removed.
- src/tts/models.rs::models_dir(): returned ./models path; unused.
- src/tts/models.rs::ensure_models_dir(): created models dir; unused.
- src/tts/models.rs::download_file(url, dest): HTTP download; unused.
- src/tts/models.rs::ensure_voice_downloaded(voice): ensured model/config; unused.
- src/tts/engine.rs::PIPER_BIN: Piper binary name; unused.
- src/tts/engine.rs::synthesize_with_piper(model_path, text, output_wav): piped text to Piper; unused.
- src/tts/engine.rs::play_wav(path): played WAV via rodio; unused.

Justification: Functions were not referenced anywhere in the current codebase and caused warnings under cargo clippy -D warnings. The Kokoro PyO3 engine is the active TTS path.

Alternatives: Use 	ts::KokoroTts (src/tts/kokoro.rs) for synthesis and playback. If Piper is needed in the future, reintroduce a minimal adapter as a separate module.
