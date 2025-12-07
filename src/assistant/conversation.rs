use anyhow::Result;
use crate::stt;
use crate::tts;
use crate::assistant::llm;
use std::path::PathBuf;
use std::fs;
use anyhow::Context;
use crate::assistant::config::load_or_create_config;
pub fn models_dir() -> PathBuf {
    PathBuf::from("./records")
}

fn ensure_records_dir() -> Result<()> {
    let dir = models_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).context("creating records directory for the temp stt and tts")?;
    }
    Ok(())
}

pub struct Assistant {
    conversation_history: Vec<Message>,
    voice_model: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Assistant {
    pub fn new() -> Result<Self> {
        // Download voice model on startup
        load_or_create_config()?;
        println!("🎧 Initializing voice...");
        let voice = &tts::voices()[0];
        let voice_model = tts::ensure_voice_downloaded(&voice)?;
        
        println!("✅ Assistant ready!\n");

        Ok(Self {
            conversation_history: vec![],
            voice_model,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("🤖 AI Assistant started (Phi 2.7 + Piper TTS)");      
        println!("Say 'exit' to quit.\n");

        loop {
            // Step 1: Listen to user
            println!("🎙 Listening...");
            let user_input = self.listen_to_user()?;
            
            if user_input.to_lowercase().contains("exit") {
                println!("👋 Goodbye!");
                break;
            }

            println!("📝 You: {}\n", user_input);

            // Step 2: Add to conversation history
            self.conversation_history.push(Message {
                role: "user".to_string(),
                content: user_input.clone(),
            });

            // Step 3: Get AI response from Ollama Gemma 3.2
            println!("🧠 Thinking with phi2.7...");
            let response = llm::call_ollama_api(&self.conversation_history).await?;
            println!("🤖 Assistant: {}\n", response);

            // Step 4: Add to history
            self.conversation_history.push(Message {
                role: "assistant".to_string(),
                content: response.clone(),
            });

            // Step 5: Speak response with Piper
            println!("🔊 Speaking...");
            self.speak_response(&response)?;
            
            println!("---\n");


        }

        Ok(())
    }

    fn listen_to_user(&self) -> Result<String> {
        ensure_records_dir();

        stt::recorder::record_to_wav("records/user_input.wav")?;
        
        let text = stt::transcriber::transcribe_with_whisper(
            "./repos/whisper.cpp",
            "models/ggml-base.en.bin",
            "records/user_input.wav",
        )?;

        Ok(text)
    }

    fn speak_response(&self, text: &str) -> Result<()> {
        let output_wav = PathBuf::from("records/assistant_response.wav");
        
        tts::synthesize_with_piper(&self.voice_model, text, &output_wav)?;
        tts::play_wav(&output_wav)?;
        
        Ok(())
    }

  fn cleanup_temp_files(&self) -> Result<()> {
    let dir = models_dir();

    if dir.exists() {
        fs::remove_dir_all(&dir)
            .with_context(|| format!("Failed cleaning temp directory {:?}", dir))?;
    }

    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed recreating temp directory {:?}", dir))?;

    Ok(())
    }

}
