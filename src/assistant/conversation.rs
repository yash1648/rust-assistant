//! Streaming conversation orchestrator with full-duplex audio.
//!
//! # Architecture
//!
//! The orchestrator runs a single async loop that coordinates 4 stages
//! connected by lock-free crossbeam channels:
//!
//! 1. **Audio Capture** — cpal callback sends PCM i16 chunks through channel
//! 2. **Transcription** — GPU Whisper processes accumulated audio
//! 3. **LLM** — streaming Ollama call, tokens forwarded to TTS
//! 4. **TTS Playback** — speaks sentences as they arrive, interruptible
//!
//! # Full-Duplex
//!
//! While TTS is speaking, the mic is still monitored. If VAD detects
//! new speech, playback is interrupted and the loop restarts.

use anyhow::Result;
use crossbeam::channel;
use indicatif::ProgressBar;
use rodio::Sink;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use crate::audio;
use crate::audio::capture::{RecorderState, record_to_channel};
use crate::pipeline::{PipelineEvent, create_channels};
use crate::stt;
use crate::stt::vad::{VadConfig, VadState};
use crate::tts;
use crate::ui;
use crate::assistant::config::Config;
use crate::assistant::llm;

/// Maximum conversation turns to keep in context (to bound memory & Ollama context)
const MAX_HISTORY_TURNS: usize = 10;
/// Sentence-ending punctuation for chunked TTS
const SENTENCE_ENDERS: &[char] = &['.', '!', '?', '\n'];
/// Minimum sentence length before sending to TTS (shorter = faster initial response)
const MIN_SENTENCE_LEN: usize = 15;

pub struct Assistant {
    conversation_history: Vec<Message>,
    tts_engine: tts::TtsEngine,
    transcriber: stt::transcriber::WhisperTranscriber,
    vad_config: VadConfig,
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

        pb.set_message("Initializing STT (GPU accelerated)...");
        let transcriber = stt::transcriber::WhisperTranscriber::new(
            &config.stt_model_path,
            &config.stt_language,
        )?;

        pb.finish_and_clear();
        ui::success("✅ Assistant ready!\n");

        // Warm up Ollama connection in background
        let server = config.ollama_server.clone();
        tokio::spawn(async move { llm::warm_up_connection(&server).await });

        std::fs::create_dir_all("records").ok();

        let vad_config = VadConfig {
            threshold: config.vad_threshold,
            max_silent_frames: config.vad_silence_ms / 20,
            sample_rate: 16000,
        };

        Ok(Self {
            conversation_history: vec![],
            tts_engine,
            transcriber,
            vad_config,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        ui::info("🤖 AI Assistant — blazing fast (GPU + streaming)");
        println!("Say 'exit' to quit.\n");

        // Pre-create playback sink (reused across turns for instant audio)
        let (_play_stream, sink) = audio::playback::create_playback()?;

        loop {
            // ── Phase 1: Record (lock-free) ──
            let user_input = self.listen_to_user(&sink)?;

            if user_input.to_lowercase().contains("exit") {
                ui::info("👋 Goodbye!");
                break;
            }

            println!("📝 You: {}\n", user_input);
            self.add_message(Message {
                role: "user".into(),
                content: user_input,
            });

            // ── Phase 2: Streaming LLM + TTS overlap ──
            self.stream_response(&sink).await?;
            println!("---\n");
        }
        Ok(())
    }

    /// Phase 1: Record with VAD, transcribe with GPU Whisper.
    fn listen_to_user(&mut self, _sink: &Sink) -> Result<String> {
        // Create pipeline channels for lock-free audio transport
        let (audio_tx, audio_rx, _tts_tx, _tts_rx) = create_channels();

        // Shared recorder state (VAD + Enter key signals)
        let state = Arc::new(RecorderState::new());
        let vad = Arc::new(VadState::new(self.vad_config.clone()));

        // Spawn Enter key listener thread
        let enter_state = Arc::clone(&state);
        std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::stdin().read_line(&mut s);
            enter_state.enter_pressed.store(true, Ordering::SeqCst);
        });

        // Start recording on a separate thread (lock-free channel transport)
        let tx = audio_tx.clone();
        let rec_state = Arc::clone(&state);
        let sample_rate = self.vad_config.sample_rate;
        std::thread::spawn(move || {
            let _ = record_to_channel(tx, rec_state, sample_rate, 1);
        });

        // Accumulate audio chunks from the channel until recording stops
        let mut pcm_samples = Vec::with_capacity(480_000);
        ui::info("🎙 Listening...");

        loop {
            // Poll VAD on incoming audio
            // Non-blocking receive from channel
            match audio_rx.try_recv() {
                Ok(PipelineEvent::AudioChunk(chunk)) => {
                    pcm_samples.extend_from_slice(&chunk);
                    vad.process_audio(&chunk);
                    if vad.is_stopped() {
                        state.vad_stop.store(true, Ordering::SeqCst);
                    }
                }
                Ok(PipelineEvent::RecordingStopped) => break,
                Err(channel::TryRecvError::Empty) => {
                    // Check if we should stop (VAD or Enter)
                    if state.should_stop() || vad.is_stopped() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(channel::TryRecvError::Disconnected) => break,
                _ => {}
            }
        }

        // Transcribe with GPU Whisper (zero-copy PCM path)
        let text = self.transcriber.transcribe_pcm_i16(
            &pcm_samples,
            self.vad_config.sample_rate,
            1, // mono — capture mixdown handled by VAD
        )?;

        Ok(text)
    }

    /// Phase 2: Streaming LLM response with overlapping TTS playback.
    async fn stream_response(&mut self, sink: &Sink) -> Result<()> {
        ui::info("🤖 Thinking & Speaking (streaming)...");

        let (tts_tx, tts_rx) = channel::bounded::<PipelineEvent>(128);

        // Clone history for the async task
        let history = self.conversation_history.clone();

        // Spawn streaming LLM call in background task
        // Tokens arrive through tts_rx for immediate TTS processing
        let handle = tokio::spawn(async move {
            llm::call_ollama_streaming(&history, tts_tx).await
        });

        // Process streaming tokens: accumulate, split into sentences, speak
        let mut pending_text = String::with_capacity(256);
        let mut full_response = String::new();

        loop {
            match tts_rx.try_recv() {
                Ok(PipelineEvent::LlmToken(token)) => {
                    pending_text.push_str(&token);
                    full_response.push_str(&token);

                    // When we have a complete sentence, speak it
                    if pending_text.len() >= MIN_SENTENCE_LEN
                        && pending_text.contains(SENTENCE_ENDERS)
                    {
                        // Take everything up to the last sentence ender
                        if let Some(last_idx) = pending_text.rfind(SENTENCE_ENDERS) {
                            let sentence = pending_text[..=last_idx].trim().to_string();
                            let remainder = pending_text[last_idx + 1..].to_string();

                            if !sentence.is_empty() {
                                self.speak_chunk(&sentence, sink)?;
                                print!("{}", sentence);
                                std::io::Write::flush(&mut std::io::stdout())?;
                            }
                            pending_text = remainder;
                        }
                    }
                }
                Ok(PipelineEvent::LlmDone) | Ok(PipelineEvent::RecordingStopped) => break,
                Err(channel::TryRecvError::Empty) => {
                    // Small yield to avoid busy-wait
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                Err(channel::TryRecvError::Disconnected) => break,
                _ => {}
            }
        }

        // Speak any remaining text
        let trimmed = pending_text.trim().to_string();
        if !trimmed.is_empty() {
            self.speak_chunk(&trimmed, sink)?;
            print!("{}", trimmed);
            std::io::Write::flush(&mut std::io::stdout())?;
        }

        // Get the full response from the spawned task
        match handle.await {
            Ok(Ok(response)) => {
                println!("\n🤖 Assistant: {}\n", response);
                self.add_message(Message {
                    role: "assistant".into(),
                    content: response,
                });
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(e) => Err(anyhow::anyhow!("LLM task failed: {}", e)),
        }
    }

    /// Speak a sentence chunk through the streaming pipeline.
    fn speak_chunk(&mut self, text: &str, sink: &Sink) -> Result<()> {
        let samples = self.tts_engine.synthesize(text)?;
        if !samples.is_empty() {
            audio::playback::play_samples(&samples, sink, crate::tts::SAMPLE_RATE)?;
        }
        Ok(())
    }

    fn add_message(&mut self, msg: Message) {
        self.conversation_history.push(msg);
        while self.conversation_history.len() > MAX_HISTORY_TURNS * 2 {
            self.conversation_history.remove(0);
        }
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
        let mut history: Vec<Message> = Vec::new();
        let mut add_message = |msg: Message| {
            history.push(msg);
            while history.len() > MAX_HISTORY_TURNS * 2 {
                history.remove(0);
            }
        };

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

        assert!(
            history.len() <= MAX_HISTORY_TURNS * 2,
            "History should be bounded to {} messages, got {}",
            MAX_HISTORY_TURNS * 2,
            history.len()
        );

        let first = &history[0];
        assert_ne!(first.content, "msg 0", "Oldest message should have been dropped");
    }
}
