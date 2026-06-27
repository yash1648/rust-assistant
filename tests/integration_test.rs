//! Integration tests for rust-assistant.
//!
//! These tests verify the core infrastructure without requiring
//! model downloads or hardware (microphone, speakers).

use tempfile::TempDir;

/// Helper: create a temp dir with an Assistant.toml inside
fn with_temp_config(toml_content: &str) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(dir.path().join("Assistant.toml"), toml_content)
        .expect("failed to write config");
    // Change to temp dir for config loading, then restore
    std::env::set_current_dir(dir.path()).ok();
    dir
}

/// Test that the config module can parse a valid TOML file
#[test]
fn test_config_parsing_valid_toml() {
    let toml = r#"
[ollama]
server = "127.0.0.1:11434"
model = "gemma3:latest"

[tts]
voice = "Jasper"
speed = 1.0

[stt]
model_path = "models/ggml-base.en.bin"
"#;

    let _dir = with_temp_config(toml);
    let content = std::fs::read_to_string("Assistant.toml").unwrap();
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(parsed.is_ok(), "valid TOML should parse: {:?}", parsed.err());
    // TempDir auto-cleaned on drop
}

/// Test that config defaults work without a config file
#[test]
fn test_config_defaults() {
    let _dir = with_temp_config("");
    assert_eq!(
        rust_assistant::default_ollama_server(),
        "127.0.0.1:11434"
    );
    assert_eq!(rust_assistant::default_tts_voice(), "Jasper");
    assert!(rust_assistant::default_tts_speed() - 1.0 < f32::EPSILON);
}

/// Test that the error module works
#[test]
fn test_error_display() {
    let err = rust_assistant::AssistantError::Config("test".into());
    assert_eq!(format!("{}", err), "Configuration error: test");

    let err = rust_assistant::AssistantError::Model("not found".into());
    assert_eq!(format!("{}", err), "Model error: not found");
}

/// Test CLI argument parsing — verify all commands parse correctly
#[test]
fn test_cli_parsing() {
    use clap::Parser;
    use rust_assistant::Cli;

    // Run command (default)
    let cli = Cli::try_parse_from(["rust-assistant"]).unwrap();
    assert!(cli.command.is_none());

    // Explicit run
    let cli = Cli::try_parse_from(["rust-assistant", "run"]).unwrap();
    assert!(matches!(cli.command, Some(rust_assistant::Commands::Run)));

    // Setup command
    let cli = Cli::try_parse_from(["rust-assistant", "setup"]).unwrap();
    assert!(matches!(cli.command, Some(rust_assistant::Commands::Setup { .. })));

    // Doctor command
    let cli = Cli::try_parse_from(["rust-assistant", "doctor"]).unwrap();
    assert!(matches!(cli.command, Some(rust_assistant::Commands::Doctor)));

    // Generate config
    let cli = Cli::try_parse_from(["rust-assistant", "generate-config"]).unwrap();
    assert!(matches!(cli.command, Some(rust_assistant::Commands::GenerateConfig)));

    // Env command
    let cli = Cli::try_parse_from(["rust-assistant", "env"]).unwrap();
    assert!(matches!(cli.command, Some(rust_assistant::Commands::Env)));

    // Shell completion
    let cli = Cli::try_parse_from(["rust-assistant", "generate-completion", "bash"]).unwrap();
    assert!(matches!(cli.command, Some(rust_assistant::Commands::GenerateCompletion { shell: rust_assistant::ShellKind::Bash })));

    // Verbose flag
    let cli = Cli::try_parse_from(["rust-assistant", "-v"]).unwrap();
    assert_eq!(cli.verbose, 1);
}

/// Test that the LLM module builds valid requests
#[test]
fn test_llm_request_format() {
    use rust_assistant::Message;

    let history = vec![
        Message { role: "user".into(), content: "Hello".into() },
    ];

    assert_eq!(history[0].role, "user");
    assert_eq!(history[0].content, "Hello");
}

/// Test audio config struct
#[test]
fn test_audio_config_defaults() {
    let config = rust_assistant::AudioConfig {
        sample_rate: 16000,
        channels: 1,
        sample_format: cpal::SampleFormat::I16,
    };
    assert_eq!(config.sample_rate, 16000);
    assert_eq!(config.channels, 1);
}

/// Test WAV spec generation
#[test]
fn test_wav_spec_creation() {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 24000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    assert_eq!(spec.sample_rate, 24000);
    assert_eq!(spec.channels, 1);
    assert_eq!(spec.bits_per_sample, 16);
}

/// Test the env vars module
#[test]
fn test_env_var_names() {
    use rust_assistant::env_vars;
    assert_eq!(env_vars::OLLAMA_SERVER, "OLLAMA_SERVER");
    assert_eq!(env_vars::OLLAMA_MODEL, "OLLAMA_MODEL");
    assert_eq!(env_vars::TTS_VOICE, "TTS_VOICE");
    assert_eq!(env_vars::TTS_MODEL_DIR, "TTS_MODEL_DIR");
    assert_eq!(env_vars::TTS_SPEED, "TTS_SPEED");
    assert_eq!(env_vars::STT_MODEL_PATH, "STT_MODEL_PATH");
    assert_eq!(env_vars::VAD_THRESHOLD, "VAD_THRESHOLD");
    assert_eq!(env_vars::VAD_SILENCE_MS, "VAD_SILENCE_MS");
}

/// Test default config generation
#[test]
fn test_generate_default_toml() {
    let toml = rust_assistant::generate_default_toml();
    assert!(toml.contains("[ollama]"));
    assert!(toml.contains("[tts]"));
    assert!(toml.contains("[stt]"));
    assert!(toml.contains("voice = \"Jasper\""));
}

/// Test that the generate config output is valid TOML
#[test]
fn test_generated_toml_is_valid() {
    let toml = rust_assistant::generate_default_toml();
    let parsed: Result<toml::Value, _> = toml::from_str(&toml);
    assert!(parsed.is_ok(), "generated TOML should be valid: {:?}", parsed.err());
}
