use anyhow::Result;
use clap::{Parser, CommandFactory};
use clap_complete::{generate_to, shells};
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
use cli::{Cli, Commands, ShellKind};

mod assistant;
mod stt;
mod tts;
mod ui;
mod error;

fn init_tracing(verbosity: u8) {
    let level = match verbosity { 0 => "info", 1 => "debug", _ => "trace" };
    let filter = EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| level.to_string()));
    fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => {
            let mut assistant = assistant::Assistant::new()?;
            assistant.run().await?;
        }
        Commands::GenerateCompletion { shell } => {
            let mut cmd = Cli::command();
            std::fs::create_dir_all("completions/bash")?;
            std::fs::create_dir_all("completions/zsh")?;
            std::fs::create_dir_all("completions/fish")?;
            match shell {
                ShellKind::Bash => { generate_to(shells::Bash, &mut cmd, "assistant", "completions/bash")?; }
                ShellKind::Zsh => { generate_to(shells::Zsh, &mut cmd, "assistant", "completions/zsh")?; }
                ShellKind::Fish => { generate_to(shells::Fish, &mut cmd, "assistant", "completions/fish")?; }
            }
            println!("Generated completions for {:?}", shell);
        }
        Commands::GenerateConfig => {
            let default = assistant::config::default_config();
            let toml = toml::to_string_pretty(&default)?;
            std::fs::write("Assistant.toml", toml)?;
            println!("Wrote Assistant.toml");
        }
    }

    Ok(())
}
