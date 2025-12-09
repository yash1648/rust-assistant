#!/usr/bin/env bash
set -e

# === 0. Resolve project root ===
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

echo "==> Project root: $PROJECT_ROOT"

# === 1. Install system dependencies (Arch-style hint, adjust if needed) ===
echo "==> Make sure you have these system packages installed (Arch example):"
echo "    sudo pacman -S --needed base-devel cmake git ffmpeg python python-virtualenv cuda cudnn"
echo "    (If you already installed them, you can ignore this message.)"
echo

# === 2. Clone & build whisper.cpp with CUDA (cuBLAS) ===
WHISPER_DIR="$PROJECT_ROOT/repos/whisper.cpp"

if [ ! -d "$WHISPER_DIR" ]; then
  echo "==> Cloning whisper.cpp..."
  mkdir -p "$PROJECT_ROOT/repos"
  git clone https://github.com/ggerganov/whisper.cpp.git "$WHISPER_DIR"
else
  echo "==> whisper.cpp already exists, skipping clone."
fi

echo "==> Building whisper.cpp with CUDA (WHISPER_CUBLAS=1)..."
cd "$WHISPER_DIR"
make clean || true
WHISPER_CUBLAS=1 make -j"$(nproc)"

# After build we expect:
#   build/bin/whisper-cli
#   build/lib/libwhisper.so.*
#   build/lib/libggml.so.*

if [ ! -f "$WHISPER_DIR/build/bin/whisper-cli" ]; then
  echo "ERROR: whisper-cli not found after build. Check make output."
  exit 1
fi

cd "$PROJECT_ROOT"

# === 3. Download Whisper model (base.en) if missing ===
MODELS_DIR="$PROJECT_ROOT/models"
mkdir -p "$MODELS_DIR"

WHISPER_MODEL="$MODELS_DIR/ggml-base.en.bin"
if [ ! -f "$WHISPER_MODEL" ]; then
  echo "==> Downloading Whisper model ggml-base.en.bin ..."
  curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
    -o "$WHISPER_MODEL"
else
  echo "==> Whisper model already exists at $WHISPER_MODEL"
fi

# === 4. Create Python venv for Kokoro TTS ===
VENV_DIR="$PROJECT_ROOT/kokoro"

if [ ! -d "$VENV_DIR" ]; then
  echo "==> Creating Python venv at $VENV_DIR ..."
  python -m venv "$VENV_DIR"
else
  echo "==> Python venv already exists at $VENV_DIR"
fi

# Activate venv
# shellcheck disable=SC1090
source "$VENV_DIR/bin/activate"

echo "==> Python in venv: $(which python)"

# === 5. Install Python dependencies for TTS (Kokoro, torch, numpy) ===
echo "==> Upgrading pip..."
pip install --upgrade pip

echo "==> Installing Python deps for TTS..."
# NOTE: Adjust torch index URL / version if needed for your CUDA version
# For many CUDA 11.8 setups (common for 1650), something like:
# pip install "torch==2.3.0" --index-url https://download.pytorch.org/whl/cu118
# For now, just plain torch and let pip resolve (you can tune later):
pip install -r requirement.txt


echo "==> Python deps installed."

# === 6. Print environment hints for run.sh ===
echo
echo "======================================================"
echo " SETUP DONE."
echo
echo " Add these to your run.sh (you already mostly have this):"
echo
echo "  PYENV_LIB=/home/sysgrim/.pyenv/versions/3.11.9/lib"
echo "  WHISPER_LIB=\$PROJECT_ROOT/repos/whisper.cpp/build/lib"
echo "  export LD_LIBRARY_PATH=\"\$PYENV_LIB:\$WHISPER_LIB:\$LD_LIBRARY_PATH\""
echo
echo " And run your assistant via run.sh so it picks up:"
echo "  - venv (Kokoro TTS)"
echo "  - whisper.cpp CUDA build"
echo "  - libpython + libwhisper + libggml"
echo "======================================================"
