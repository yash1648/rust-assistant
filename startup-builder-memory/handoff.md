# Handoff — rust-assistant

## Current State
Project is fully functional at MVP+ level. Voice Activity Detection added — recording now auto-stops on silence.

### Recently Completed
- ✅ **Voice Activity Detection (VAD)** — Energy-based RMS threshold, no model needed
- ✅ **Zero external VAD dependencies** — pure signal processing, zero new deps
- ✅ **Safety timeout** — 120s max recording prevents infinite loops
- ✅ **VAD configuration** — threshold + silence duration in `Assistant.toml` + env vars
- ✅ **.env auto-loading** — `dotenvy` loads `.env` file at startup
- ✅ **44 tests** (was 30) — 7 VAD + 7 VAD binary tests added

### Still Valid
- ✅ **STT**: whisper-rs with 16kHz WAV input, automatic resampling, N-channel mixdown
- ✅ **LLM**: Ollama HTTP API with conversation history
- ✅ **TTS**: Pure Rust KittenTTS, auto-downloads model, 8 voices
- ✅ **CI/CD**: GitHub Actions matrix build for 6 targets
- ✅ **Docker**: Multi-stage build (amd64 + arm64)
- ✅ **100% Rust**: No Python, no cloud, no system deps beyond ALSA
- ✅ **Optimized pipeline**: In-memory audio, shared HTTP client, bounded history (10 turns)

### VAD Architecture
```
Audio callback → RMS computed per buffer → silence counter (atomic)
                                                  ↓
                    counter > max_silent_frames? → set should_stop flag
                                                  ↓
                    Main loop polls should_stop + Enter key + 120s timeout
                                                  ↓
                    Stop → assemble WAV → transcribe
```

VAD config options (all with env var overrides):
- `VAD_THRESHOLD`: RMS energy threshold (0.0–1.0, default 0.02)
- `VAD_SILENCE_MS`: Silence duration before stop (default 800ms)

## Open Issues
- Dead code warnings for public API functions are intentional and safe

## Quick Reference
```bash
cargo run                   # Run assistant (now with VAD auto-stop)
cargo run setup             # Download models
cargo run doctor            # System health check
cargo run env               # Show all env vars (including VAD_THRESHOLD, VAD_SILENCE_MS)
```

## Key Measurements
- Tests: **44** (17 lib unit + 17 bin unit + 10 integration)
- All passing
- VAD: 7 unit tests (RMS calculation, silence trigger, speech reset, state reset)
