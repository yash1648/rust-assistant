use anyhow::Result;
use indicatif::ProgressBar;
use std::path::Path;
use std::time::Duration;
use crate::ui;
use crate::stt;
use crate::tts;
use crate::assistant::llm;
use crate::assistant::config::Config;

/// Maximum conversation turns to keep in context (to bound memory & Ollama context)
const MAX_HISTORY_TURNS: usize = 10;

pub struct Assistant {
    conversation_history: Vec<Message>,
    tts_engine: tts::TtsEngine,
    transcriber: stt::transcriber::WhisperTranscriber,
}

#[derive(Debug, Clone)]
pub struct Message { pub role: String, pub content: String }

impl Assistant {
    pub fn new() -> Result<Self> {
        let config = Config::default();
        config.print();

        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(80));

        pb.set_message("Initializing TTS (pure Rust, no Python)...");
        let tts_engine = tts::TtsEngine::new(
            &config.tts_voice,
            config.tts_model_dir.as_ref().map(Path::new),
            config.tts_speed,
        )?;

        pb.set_message("Initializing STT...");
        let transcriber = stt::transcriber::WhisperTranscriber::new(&config.stt_model_path)?;

        pb.finish_and_clear();
        ui::success("✅ Assistant ready!\n");

        // Ensure records directory exists
        std::fs::create_dir_all("records").ok();

        Ok(Self {
            conversation_history: vec![],
            tts_engine,
            transcriber,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        ui::info("🤖 AI Assistant started (100% Rust — Whisper STT + KittenTTS)");
        println!("Say 'exit' to quit.\n");

        loop {
            ui::info("🎙 Listening...");
            let user_input = self.listen_to_user()?;

            if user_input.to_lowercase().contains("exit") {
                ui::info("👋 Goodbye!");
                break;
            }

            println!("📝 You: {}\n", user_input);
            self.add_message(Message {
                role: "user".into(),
                content: user_input.clone(),
            });

            let response = llm::call_ollama_api(&self.conversation_history).await?;
            println!("🤖 Assistant: {}\n", response);

            self.add_message(Message {
                role: "assistant".into(),
                content: response.clone(),
            });

            ui::info("🔊 Speaking...");
            self.speak_response(&response)?;
            println!("---\n");
        }
        Ok(())
    }

    /// Add a message to history with bounded context window
    fn add_message(&mut self, msg: Message) {
        self.conversation_history.push(msg);
        // Keep only the last N turns (each turn = user + assistant = 2 messages)
        while self.conversation_history.len() > MAX_HISTORY_TURNS * 2 {
            self.conversation_history.remove(0);
        }
    }

    fn listen_to_user(&mut self) -> Result<String> {
        // Use in-memory recording for zero disk I/O
        let buffer = stt::recorder::record_to_buffer()?;
        // Transcribe directly from the buffer
        let text = self.transcriber.transcribe_buffer(buffer)?;
        Ok(text)
    }

    fn speak_response(&mut self, text: &str) -> Result<()> {
        self.tts_engine.speak_blocking(text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message {
            role: "user".into(),
            content: "Hello".into(),
        };
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_bounded_history_logic() {
        // Test the bounded history algorithm directly (no model dependency)
        let mut history: Vec<Message> = Vec::new();
        let mut add_message = |msg: Message| {
            history.push(msg);
            while history.len() > MAX_HISTORY_TURNS * 2 {
                history.remove(0);
            }
        };

        // Add more than MAX_HISTORY_TURNS rounds
        for i in 0..MAX_HISTORY_TURNS + 5 {
            add_message(Message {
                role: "user".into(),
                content: format!("msg {}", i),
            });
            add_message(Message {
                role: "assistant".into(),
                content: format!("resp {}", i),
            });
        }

        // History should be bounded
        assert!(
            history.len() <= MAX_HISTORY_TURNS * 2,
            "History should be bounded to {} messages, got {}",
            MAX_HISTORY_TURNS * 2,
            history.len()
        );

        // The oldest messages should have been dropped
        let first = &history[0];
        assert_ne!(first.content, "msg 0", "Oldest message should have been dropped");
    }
}
