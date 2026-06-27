use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Parser)]
#[command(
    name = "rust-assistant",
    version,
    about = "Voice assistant — 100% Rust, zero Python, zero cloud",
    propagate_version = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long, default_value = "auto")]
    pub color: clap::ColorChoice,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the voice assistant (default)
    Run,

    /// Download models and set up the project
    Setup {
        /// Path to Whisper model to download
        #[arg(long, default_value = "models/ggml-base.en.bin")]
        whisper_model: String,

        /// Skip Whisper model download
        #[arg(long)]
        skip_whisper: bool,

        /// Local path for TTS model (optional — auto-downloads if not provided)
        #[arg(long)]
        tts_model_dir: Option<String>,

        /// Force re-download of models
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Check system dependencies and configuration
    Doctor,

    /// Generate shell completions
    GenerateCompletion {
        #[arg(value_enum)]
        shell: ShellKind,
    },

    /// Generate default Assistant.toml
    GenerateConfig,

    /// Show environment variables
    Env,
}
