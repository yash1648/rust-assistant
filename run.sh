#!/usr/bin/env bash
set -e

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

# 1) Activate venv for Kokoro TTS
source "$PROJECT_ROOT/kokoro/bin/activate"

# 2) Shared libraries: libpython (pyenv) + whisper (whisper.cpp build)
PYENV_LIB=/home/sysgrim/.pyenv/versions/3.11.9/lib  # Adjust to your pyenv Python version
WHISPER_LIB="$PROJECT_ROOT/repos/whisper.cpp/build/lib"

export LD_LIBRARY_PATH="$PYENV_LIB:$WHISPER_LIB:$LD_LIBRARY_PATH"

# 3) Run your Rust assistant (binary name from Cargo.toml's [package].name)
cargo build --release

./target/release/Assistant "$@"
