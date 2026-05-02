use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
}


#[derive(Debug, Parser)]
#[command(name = "assistant", version, about = "Voice assistant (STT + LLM + TTS)", propagate_version = true)]
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
    /// Run the voice assistant
    Run,
    /// Generate shell completions
    GenerateCompletion { #[arg(value_enum)] shell: ShellKind },
    /// Generate default Assistant.toml
    GenerateConfig,
    /// Show environment variables
    Env,
}