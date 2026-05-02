use anyhow::Result;
use indicatif::ProgressBar;
use std::path::Path;
use std::time::Duration;
use crate::ui;
use crate::stt;
use crate::tts;
use crate::assistant::llm;
use crate::assistant::config::Config;

pub struct Assistant {
    conversation_history: Vec<Message>,
    kokoro_tts: tts::KokoroTts,
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

        pb.set_message("Initializing TTS...");
        let kokoro_tts = tts::KokoroTts::new(
            &config.tts_voice,
            Path::new(&config.tts_model_path),
            Path::new(&config.tts_voices_path),
        )?;

        pb.set_message("Initializing STT...");
        let transcriber = stt::transcriber::WhisperTranscriber::new(&config.stt_model_path)?;

        pb.finish_and_clear();
        ui::success("✅ Assistant ready!\n");

        Ok(Self {
            conversation_history: vec![],
            kokoro_tts,
            transcriber,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        ui::info("🤖 AI Assistant started (Whisper + Kokoro TTS)");
        println!("Say 'exit' to quit.\n");

        loop {
            ui::info("🎙 Listening...");
            let user_input = self.listen_to_user()?;

            if user_input.to_lowercase().contains("exit") {
                ui::info("👋 Goodbye!");
                break;
            }

            println!("📝 You: {}\n", user_input);
            self.conversation_history.push(Message {
                role: "user".into(),
                content: user_input.clone(),
            });

            let response = llm::call_ollama_api(&self.conversation_history).await?;
            println!("🤖 Assistant: {}\n", response);

            self.conversation_history.push(Message {
                role: "assistant".into(),
                content: response.clone(),
            });

            ui::info("🔊 Speaking...");
            self.speak_response(&response)?;
            println!("---\n");
        }
        Ok(())
    }

    fn listen_to_user(&mut self) -> Result<String> {
        stt::recorder::record_to_wav("records/user_input.wav")?;
        let text = self.transcriber.transcribe_wav("records/user_input.wav")?;
        Ok(text)
    }

    fn speak_response(&mut self, text: &str) -> Result<()> {
        self.kokoro_tts.speak_and_play_blocking(text)?;
        Ok(())
    }
}