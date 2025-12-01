# Known Issues

- Hardcoded Ollama URL
  - The chat endpoint is hardcoded (`src/assistant/llm.rs:26`). Replace with your host or refactor to read from environment.
- Whisper CLI path assumptions
  - Transcriber expects `repos/whisper.cpp` and standard build output paths (`src/stt/transcriber.rs:41-74`). Make paths configurable.
- Limited voice catalog
  - Only one voice is defined (`src/tts/voice.rs:12`). Extend `voices()` to include more options.
- Sample format restrictions
  - Recorder supports `I16` and `F32`. Other formats bail with an error (`src/stt/recorder.rs:27-39`).
- Platform differences
  - Audio stack behavior varies across OS; device selection may need additional logic.
