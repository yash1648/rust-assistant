# Graph Report - rust-assistant  (2026-06-28)

## Corpus Check
- 33 files · ~17,379 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 435 nodes · 596 edges · 26 communities (24 shown, 2 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 2 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `bafa00c2`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 25|Community 25]]

## God Nodes (most connected - your core abstractions)
1. `🎙️ RustAssistant` - 13 edges
2. `🛣️ Roadmap` - 13 edges
3. `TtsEngine` - 12 edges
4. `📦 Installation Guide` - 12 edges
5. `Config` - 11 edges
6. `Assistant` - 11 edges
7. `📐 Technical Architecture` - 11 edges
8. `Architecture Decisions` - 11 edges
9. `WhisperTranscriber` - 10 edges
10. `🔧 Troubleshooting Guide` - 10 edges

## Surprising Connections (you probably didn't know these)
- `Assistant` --references--> `WhisperTranscriber`  [EXTRACTED]
  src/assistant/conversation.rs → src/stt/transcriber.rs
- `Assistant` --references--> `VadConfig`  [EXTRACTED]
  src/assistant/conversation.rs → src/stt/vad.rs
- `record_to_buffer()` --calls--> `wait_enter()`  [INFERRED]
  src/stt/recorder.rs → src/stt/io.rs
- `record_to_buffer_vad()` --calls--> `spawn_enter_listener()`  [INFERRED]
  src/stt/recorder.rs → src/stt/io.rs
- `record_to_buffer_vad()` --references--> `VadConfig`  [EXTRACTED]
  src/stt/recorder.rs → src/stt/vad.rs

## Import Cycles
- None detected.

## Communities (26 total, 2 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.05
Nodes (40): 📝 Best Practices, Clone and Build, Clone Repository, Dependency Versions, Download Model, Download Voice Models, First Run, Installation (+32 more)

### Community 1 - "Community 1"
Cohesion: 0.05
Nodes (39): 🌐 Advanced Features (Future), API Server, Audit Logging, Automated Test Suite, CI/CD Pipeline, ✅ Complete Features, Config File (TOML), Configuration Management (+31 more)

### Community 2 - "Community 2"
Cohesion: 0.10
Nodes (23): Arc, Assistant, Message, test_bounded_history_logic(), call_ollama_api(), get_client(), AtomicBool, Client (+15 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (33): Advanced Usage, Development & Contributing, ❓ FAQ, General Questions, Installation & Setup, Project Info, Q: Build fails with "linking failed", Q: Can I run this on Windows? (+25 more)

### Community 4 - "Community 4"
Cohesion: 0.12
Nodes (25): Config, default_ollama_model(), default_ollama_server(), default_stt_model_path(), default_tts_model_dir(), default_tts_speed(), default_tts_voice(), default_vad_silence_ms() (+17 more)

### Community 5 - "Community 5"
Cohesion: 0.08
Nodes (26): 1. Install Rust, 2. Install Dependencies, 3. Set Up Whisper, 4. Install & Run Ollama, 5. Build & Run, Architecture Overview, ⚙️ Configuration, 🤝 Contributing (+18 more)

### Community 6 - "Community 6"
Cohesion: 0.08
Nodes (25): Add a New Voice, Add Logging, Build Release Binary, 🎨 Code Style, 🔧 Common Development Tasks, Current State, 📦 Deployment, 🔨 Development Environment (+17 more)

### Community 7 - "Community 7"
Cohesion: 0.08
Nodes (24): 🎤 Audio Device Issues, 🏗️ Build Errors, 🔍 Debug Logging, Error: "Cargo not found", Error: "Could not compile `anyhow`" or other dependency, Error: "Linking failed" on Windows, 📞 Getting Help, Manual Component Testing (+16 more)

### Community 8 - "Community 8"
Cohesion: 0.10
Nodes (21): 🔴 Critical Issues, Hardcoded Configuration Paths, 🟠 High Priority Issues, ⚠️ Known Issues, Limited Sample Format Support, Limited Voice Catalog, 🟢 Low Priority Issues / Enhancements, 🟡 Medium Priority Issues (+13 more)

### Community 9 - "Community 9"
Cohesion: 0.10
Nodes (21): 🌐 API Integration, `assistant` Module, Conversation History, Core Modules, 🔄 Data Flow, 🛠️ Error Handling Strategy, Error Propagation Pattern, Error Sources by Module (+13 more)

### Community 10 - "Community 10"
Cohesion: 0.16
Nodes (13): AtomicU64, Default, compute_rms_f32(), compute_rms_i16(), test_rms_f32_silence(), test_rms_full_scale(), test_rms_half_scale(), test_rms_silence() (+5 more)

### Community 11 - "Community 11"
Cohesion: 0.19
Nodes (4): TempDir, test_config_defaults(), test_config_parsing_valid_toml(), with_temp_config()

### Community 12 - "Community 12"
Cohesion: 0.45
Nodes (3): FAQ, Navigation, Known Issues

### Community 13 - "Community 13"
Cohesion: 0.18
Nodes (10): Architecture Map, `assistant/` (Orchestration), Build Targets (CI), CLI Commands, Configuration Priority, Data Flow, Docker, Module Boundaries (+2 more)

### Community 14 - "Community 14"
Cohesion: 0.17
Nodes (11): ADR-001: Pure Rust TTS, ADR-002: Release Optimization Tradeoffs, ADR-003: Ollama as LLM Backend, ADR-004: CLI-first Architecture, ADR-005: In-Memory Audio Pipeline, ADR-006: Bounded Conversation History, ADR-007: Shared HTTP Client (OnceLock), ADR-008: Energy-Based Voice Activity Detection (+3 more)

### Community 15 - "Community 15"
Cohesion: 0.22
Nodes (8): Current State, Handoff — rust-assistant, Key Measurements, Open Issues, Quick Reference, Recently Completed, Still Valid, VAD Architecture

### Community 16 - "Community 16"
Cohesion: 0.54
Nodes (7): Color, error(), info(), print_colored(), success(), test_ui_functions_dont_panic(), warning()

### Community 17 - "Community 17"
Cohesion: 0.25
Nodes (7): Core User Flow, Current Status, MVP Definition of Done, Next Horizons, Product Brief: rust-assistant, Target Users, Vision

### Community 18 - "Community 18"
Cohesion: 0.50
Nodes (4): ColorChoice, Cli, Commands, ShellKind

### Community 19 - "Community 19"
Cohesion: 0.40
Nodes (4): Backlog, Current Sprint: Hardening & Polish, Legend, Task Ledger

### Community 20 - "Community 20"
Cohesion: 0.50
Nodes (3): Key Files (Hot Paths), Repository Map, Top-Level Structure

### Community 25 - "Community 25"
Cohesion: 0.24
Nodes (8): R, test_mix_to_mono_5_channel(), test_mix_to_mono_stereo(), test_resample_basic(), test_resample_empty(), WhisperTranscriber, WavReader, WhisperContext

## Knowledge Gaps
- **210 isolated node(s):** `ShellKind`, `AssistantError`, `Removed Unused Functions`, `✨ Core Features`, `Runtime Flow` (+205 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `📦 Installation Guide` connect `Community 0` to `Community 12`?**
  _High betweenness centrality (0.090) - this node is a cross-community bridge._
- **Why does `🛣️ Roadmap` connect `Community 1` to `Community 12`?**
  _High betweenness centrality (0.088) - this node is a cross-community bridge._
- **Why does `❓ FAQ` connect `Community 3` to `Community 12`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **What connects `ShellKind`, `AssistantError`, `Removed Unused Functions` to the rest of the system?**
  _210 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05128205128205128 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.10336817653890824 - nodes in this community are weakly interconnected._