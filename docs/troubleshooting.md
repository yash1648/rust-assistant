# 🔧 Troubleshooting Guide

Comprehensive solutions for common issues with the Piper Female voice assistant.

## Quick Diagnosis Flowchart

```
Is the app running?
├─ No → Check Rust compilation errors (see "Build Errors")
└─ Yes
   ├─ No microphone input?
   │  └─ → Audio Device Issues
   ├─ Can't understand speech?
   │  └─ → Whisper/Transcription Issues
   ├─ No response from AI?
   │  └─ → Ollama Issues
   ├─ No audio output?
   │  └─ → Piper/Playback Issues
   └─ Other errors?
      └─ → Check specific error message below
```

---

## 🎤 Audio Device Issues

### Symptom: "No default input device available"

**Error Message**:
```
Error: No input device found
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value'
```

**Root Causes**:
- No microphone connected or disabled
- Microphone not set as default in system settings
- Audio permissions denied

**Solutions**:

**Linux (PulseAudio/ALSA)**
```bash
# List input devices
pactl list sources

# Set default device (replace number with your mic)
pactl set-default-source alsa_input.pci-0000_00_1f.3.analog-stereo

# Test recording
arecord -d 5 test.wav
```

**macOS**
```bash
# List devices
system_profiler SPAudioDataType

# Change default input in System Settings:
# System Settings → Sound → Input → Select microphone
```

**Windows**
```powershell
# In Settings → Sound → Input
# Select your microphone as the default device

# Test:
.\target\release\rust_assistant.exe
```

---

## 📝 Whisper / Transcription Issues

### Symptom: "Whisper binary not found"

**Error Message**:
```
Error: failed to find whisper binary
```

**Root Cause**: Binary not built or path is wrong

**Solution**:

```bash
# Verify the binary exists
ls repos/whisper.cpp/build/bin/main      # Linux/macOS
dir repos\whisper.cpp\build\Release\     # Windows

# If missing, rebuild
cd repos/whisper.cpp/build
cmake .. && make -j$(nproc)

# Update the path in src/stt/transcriber.rs if needed
```

**File**: `src/stt/transcriber.rs` (lines 41-74)
```rust
// The code searches for:
// - On macOS: "whisper-cli" or "main" in repos/whisper.cpp/build/bin/
// - On Linux: same as macOS
// - On Windows: "main.exe" in repos/whisper.cpp/build/Release/
```

### Symptom: "Whisper model not found"

**Error Message**:
```
Error: No such file or directory (os error 2): models/ggml-base.en.bin
```

**Root Cause**: Model file missing or wrong path

**Solution**:

```bash
# Create models directory
mkdir -p models

# Download model (~140 MB)
cd models
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

# Verify
ls -lh ggml-base.en.bin  # Should show ~140MB
```

**Update path**: `src/assistant/conversation.rs` (lines 76-87)
```rust
let text = stt::transcriber::transcribe_with_whisper(
    "./repos/whisper.cpp",
    "models/ggml-base.en.bin",  // Adjust path here
    "user_input.wav",
)?;
```

### Symptom: "Recording starts but no transcription appears"

**Error Message**:
```
Transcribed text:
(blank)
```

**Root Causes**:
- Audio too quiet or background noise
- Microphone not capturing properly
- Whisper model not detecting speech

**Solutions**:

1. **Test microphone quality**:
   ```bash
   # Record a test file
   arecord -d 5 test.wav
   
   # Try manual transcription
   ./repos/whisper.cpp/build/bin/main -m models/ggml-base.en.bin -f test.wav
   ```

2. **Speak louder and clearer**:
   - Position microphone 6-12 inches from mouth
   - Reduce background noise
   - Speak at natural volume

3. **Check microphone levels**:
   ```bash
   # Linux: check levels
   alsamixer
   ```

---

## 🤖 Ollama / LLM Issues

### Symptom: "Cannot connect to Ollama"

**Error Message**:
```
Error: Ollama API error: connection refused
```

**Root Causes**:
- Ollama server not running
- Wrong endpoint configured
- Firewall blocking connection

**Solutions**:

1. **Start Ollama server**:
   ```bash
   # Linux/macOS
   ollama serve
   
   # Windows
   ollama serve
   # (or restart the Ollama app from system tray)
   ```

2. **Verify server is responding**:
   ```bash
   curl http://localhost:11434/api/tags
   # Should return JSON with installed models
   ```

3. **Update endpoint if not localhost**:
   **File**: `src/assistant/llm.rs` (line 26)
   ```rust
   .post("http://192.168.1.100:11434/api/chat")  // Change IP
   ```

4. **Check firewall**:
   - Windows: Allow Ollama through Windows Defender
   - Linux: `sudo ufw allow 11434`

### Symptom: "Model not found" (Gemma)

**Error Message**:
```
Error: model 'gemma3' not found
```

**Root Cause**: Model not downloaded

**Solution**:

```bash
# Pull the model (this downloads ~5-8 GB)
ollama pull gemma3

# Verify installation
ollama list
# Output should show: gemma3 ...

# Alternative models to try:
ollama pull llama3
ollama pull mistral
```

### Symptom: "Ollama response is slow"

**Symptom**: Takes 30+ seconds to get a response

**Root Causes**:
- Model running on CPU (no GPU)
- Model too large for available RAM
- System is resource-constrained

**Solutions**:

1. **Check if GPU is being used**:
   ```bash
   # Windows (NVIDIA)
   nvidia-smi  # Should show process using ollama
   
   # macOS (Metal)
   # Automatically detected for Apple Silicon
   ```

2. **Try a smaller model**:
   ```bash
   ollama pull orca-mini:3b  # Smaller, faster
   ```

3. **Increase available RAM**:
   - Close other applications
   - Increase system swap if needed

4. **Optimize Ollama settings**:
   - Edit `~/.ollama/models/manifest.json` or config file
   - Adjust `num_gpu: 1` to use more GPU layers

---

## 🔊 Piper / Text-to-Speech Issues

### Symptom: "Piper: command not found"

**Error Message**:
```
Error: failed to run piper: No such file or directory
```

**Root Causes**:
- Binary not installed
- Not in PATH
- Wrong binary name

**Solutions**:

1. **Verify installation**:
   ```bash
   piper-tts --version
   # or
   piper --version
   ```

2. **Add to PATH** (if installed but not found):
   
   **Linux**:
   ```bash
   export PATH="$PATH:/path/to/piper/bin"
   ```
   
   **macOS**:
   ```bash
   which piper-tts  # Should show /usr/local/bin/piper-tts
   ```
   
   **Windows**:
   - Right-click **This PC** → Properties
   - Click **Environment Variables**
   - Add the piper directory to PATH

3. **Update binary name in code**:
   **File**: `src/tts/engine.rs` (line 9)
   ```rust
   pub const PIPER_BIN: &str = "piper";  // Try "piper" instead of "piper-tts"
   ```

### Symptom: "No audio playback after synthesis"

**Symptom**: WAV file is created but no sound

**Root Causes**:
- Speaker not set as default
- Audio permissions denied
- Speaker volume is muted

**Solutions**:

1. **Verify speaker setup**:
   ```bash
   # Linux: check output devices
   pactl list sinks
   
   # macOS: System Settings → Sound → Output
   # Windows: Settings → Sound → Output device
   ```

2. **Test playback manually**:
   ```bash
   # Linux
   paplay assistant_response.wav
   
   # macOS
   afplay assistant_response.wav
   
   # Windows
   powershell -c "(New-Object Media.SoundPlayer 'assistant_response.wav').PlaySync()"
   ```

3. **Check volume levels**:
   - Unmute speaker in system settings
   - Increase volume to 50%+
   - Check speaker is not muted at hardware level

### Symptom: "Piper synthesis fails"

**Error Message**:
```
Error: piper exited with error: ...
```

**Root Causes**:
- Voice model not downloaded
- Incompatible audio format
- Missing Python dependencies

**Solutions**:

1. **Download voice models** (automatic on first run):
   ```bash
   # Manually download if needed
   mkdir -p models
   wget -O models/en_GB-cori-high.onnx \
     https://huggingface.co/.../en_GB-cori-high.onnx
   ```

2. **Verify Piper has access to models**:
   ```bash
   piper-tts --list-speakers
   ```

3. **Test synthesis manually**:
   ```bash
   echo "Hello world" | piper-tts \
     --model models/en_GB-cori-high.onnx \
     --output-file test.wav
   ```

---

## 🏗️ Build Errors

### Error: "Cargo not found"

**Solution**: Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Error: "Could not compile `anyhow`" or other dependency

**Solution**: Update Rust and dependencies
```bash
rustup update stable
cargo update
cargo build
```

### Error: "Linking failed" on Windows

**Solution**: Install Visual Studio Build Tools
```
Download from: https://visualstudio.microsoft.com/downloads/
Install: Desktop development with C++
```

---

## 🔍 Debug Logging

To get more detailed error messages, rebuild with logging:

```bash
# Run with backtrace (Linux/macOS)
RUST_BACKTRACE=1 cargo run

# Windows PowerShell
$env:RUST_BACKTRACE=1; cargo run

# Full backtrace
RUST_BACKTRACE=full cargo run
```

### Manual Component Testing

Test each part independently:

**Test Microphone**:
```bash
arecord -d 5 test_input.wav
```

**Test Whisper**:
```bash
./repos/whisper.cpp/build/bin/main -m models/ggml-base.en.bin -f test_input.wav
cat test_input.wav.txt
```

**Test Ollama**:
```bash
curl -X POST http://localhost:11434/api/chat \
  -H "Content-Type: application/json" \
  -d '{"model":"gemma3","messages":[{"role":"user","content":"hi"}],"stream":false}'
```

**Test Piper**:
```bash
echo "This is a test" | piper-tts --model en_GB-cori-high --output-file test_output.wav
```

**Test Playback**:
```bash
paplay test_output.wav  # Linux
afplay test_output.wav  # macOS
```

---

## 📞 Getting Help

If issues persist:

1. **Check the logs**: Run with `RUST_BACKTRACE=1`
2. **Verify dependencies**: See [Installation Guide](./installation.md)
3. **Test components**: Use manual testing steps above
4. **Open an issue** on GitHub with:
   - Full error message and backtrace
   - OS and version
   - Output of `rustc --version` and `cargo --version`
   - Which step failed (recording, transcription, LLM, synthesis, playback)

---

## 🔗 Navigation

- [Installation](./installation.md) — Setup help
- [Technical](./technical.md) — Architecture details
- [FAQ](./faq.md) — Quick answers
- [Development](./development.md) — Contributing

---

**Last Updated**: December 2025