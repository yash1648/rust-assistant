# Architecture Decisions

## ADR-001: Pure Rust TTS
- **Status**: Accepted
- **Context**: Original TTS backend (any-tts/kokoro) required Python 3.x + PyTorch, creating massive portability issues
- **Decision**: Replace with `kittentts` (pure Rust, ONNX-based, zero system dependencies)
- **Consequences**:
  - ✅ Zero system dependencies for TTS
  - ✅ Cross-platform compilation
  - ✅ Single binary deployment
  - ⚠️ ONNX Runtime adds ~45MB to binary size

## ADR-002: Release Optimization Tradeoffs
- **Status**: Accepted
- **Decision**: `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`
- **Consequences**: 45MB release binary, max performance, ~2.5x slower build

## ADR-003: Ollama as LLM Backend
- **Status**: Accepted
- **Decision**: Use Ollama HTTP API for local LLM inference
- **Consequences**: Local privacy, model flexibility, requires running Ollama separately

## ADR-004: CLI-first Architecture
- **Status**: Accepted
- **Decision**: CLI as primary interface (clap), GUI as optional future layer
- **Consequences**: Works in any terminal, scriptable, less accessible to non-technical users

## ADR-005: In-Memory Audio Pipeline
- **Status**: Accepted
- **Decision**: Record PCM samples to `Arc<Mutex<Vec<i16>>>`, assemble WAV in `Cursor<Vec<u8>>`
- **Consequences**: Zero disk I/O, no temp file cleanup

## ADR-006: Bounded Conversation History
- **Status**: Accepted
- **Decision**: Cap at 10 turns (20 messages) with FIFO eviction
- **Consequences**: Predictable memory, prevents context overflow

## ADR-007: Shared HTTP Client (OnceLock)
- **Status**: Accepted
- **Decision**: Use `std::sync::OnceLock<Client>` as global singleton
- **Consequences**: Connection reuse, lazy init

## ADR-008: Energy-Based Voice Activity Detection
- **Status**: Accepted
- **Context**: Need hands-free recording without pressing Enter, but don't want to add heavy ML dependencies
- **Decision**: Use simple RMS threshold VAD (pure signal processing, no model needed)
- **Consequences**:
  - ✅ Zero additional dependencies
  - ✅ Works on any platform
  - ⚠️ Less accurate than Silero VAD in noisy environments
  - ⚠️ Threshold may need tuning per microphone
  - Configurable via `VAD_THRESHOLD` and `VAD_SILENCE_MS` env vars

## ADR-009: .env File Auto-Loading
- **Status**: Accepted
- **Context**: Users want to configure via `.env` file without `export`
- **Decision**: Use `dotenvy` crate to auto-load `.env` at startup
- **Consequences**:
  - ✅ Simple config management
  - ✅ Shell env vars still take precedence
  - ✅ Small dependency (~20KB)

## Anti-Patterns Learned
1. **Python-in-Rust TTS**: Avoid Python FFI for ML inference — pure Rust alternatives exist
2. **Bash setup scripts**: Always prefer Rust-native setup commands
3. **Platform-specific hacks**: LD_LIBRARY_PATH and venv activation are maintenance nightmares
4. **Cursor::clone() for shared writing**: Clones inner Vec; use `&mut Cursor` or collect-and-assemble approach
5. **Unsized match on non_exhaustive enums**: Always include wildcard `_ =>` arm
6. **VAD model dependency trap**: Don't assume you need Silero/ONNX for VAD — energy-based RMS works well enough for quiet environments and avoids 45MB+ of deps
