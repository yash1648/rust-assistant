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
    Run,
    GenerateCompletion { #[arg(value_enum)] shell: ShellKind },
    GenerateConfig,
}
