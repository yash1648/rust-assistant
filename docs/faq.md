# FAQ

> Quick answers with links and consistent formatting.

- What voice does this use?
  - The Cori high‑quality English GB voice (`src/tts/voice.rs:12`). You can add more voices to `voices()`.
- Piper binary is called `piper` on my system. Is that okay?
  - Yes. Change `PIPER_BIN` in `src/tts/engine.rs:9` to `piper`.
- Do I need internet access?
  - `Ollama` can run models locally; however the first pull (`ollama pull gemma3`) downloads weights.
- Can I use a different LLM?
  - Update the `model` field and endpoint in `src/assistant/llm.rs:26`.
- Where are audio files stored?
  - Generated in the project root as `user_input.wav` and `assistant_response.wav`.
- How do I quit?
  - Say `exit` during a prompt.

## Navigation

- Overview: `docs/README.md`
- Installation: `docs/installation.md`
- Architecture: `docs/technical.md`
- Troubleshooting: `docs/troubleshooting.md`
