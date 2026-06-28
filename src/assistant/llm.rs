use anyhow::{Context, Result};
use crossbeam::channel::Sender;
use futures::StreamExt;
use serde_json::json;
use std::sync::OnceLock;
use crate::assistant::config::Config;
use crate::pipeline::PipelineEvent;
use super::conversation::Message;

/// Shared HTTP client with connection pooling and keep-alive.
/// Avoids DNS + TCP + TLS handshake on every call.
fn get_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .http1_only()   // Ollama uses HTTP/1.1, not HTTP/2
            .build()
            .expect("Failed to build HTTP client")
    })
}

/// Warm up the Ollama connection during initialization.
/// This avoids the cold-start DNS + TCP handshake on the first user query.
/// Call this once after loading models to shave ~100-300ms off the first LLM call.
pub async fn warm_up_connection(server: &str) {
    let url = format!("http://{}/api/tags", server);
    if let Ok(resp) = get_client().get(&url).send().await {
        if resp.status().is_success() {
            tracing::info!("Ollama connection warmed up at {}", server);
        }
    }
    // Silently ignore failures — Ollama might not be running yet
}

/// Call Ollama with streaming response.
/// Sends partial tokens through the pipeline channel for immediate TTS processing.
/// Returns the full accumulated response text for conversation history.
pub async fn call_ollama_streaming(
    history: &[Message],
    tts_tx: Sender<PipelineEvent>,
) -> Result<String> {
    let client = get_client();
    let config = Config::from_toml();

    let messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| json!({"role": &m.role, "content": &m.content}))
        .collect();

    let request_body = json!({
        "model": config.ollama_model,
        "messages": messages,
        "stream": true,
    });

    let url = format!("http://{}/api/chat", config.ollama_server);
    let resp = client
        .post(url)
        .json(&request_body)
        .send()
        .await
        .context("request to Ollama failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_text = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Ollama API error: {} - {}", status, error_text));
    }

    // Stream the NDJSON response — send tokens through pipeline as they arrive
    let mut stream = resp.bytes_stream();
    let mut full_response = String::new();
    let mut line_buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("error reading Ollama stream")?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for byte in chunk_str.bytes() {
            let c = byte as char;
            if c == '\n' {
                if line_buffer.is_empty() {
                    continue;
                }
                // Parse NDJSON line
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line_buffer) {
                    if let Some(content) = val.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                        full_response.push_str(content);
                        // Send token through pipeline for streaming TTS
                        let _ = tts_tx.try_send(PipelineEvent::LlmToken(content.to_string()));
                    }
                    if val.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                        break;
                    }
                }
                line_buffer.clear();
            } else {
                line_buffer.push(c);
            }
        }
    }

    let _ = tts_tx.try_send(PipelineEvent::LlmDone);

    if full_response.is_empty() {
        anyhow::bail!("Ollama returned empty response");
    }

    Ok(full_response)
}

/// Non-streaming call (kept for backward compat, not used in the new pipeline).
pub async fn call_ollama_api(history: &[Message]) -> Result<String> {
    let client = get_client();
    let config = Config::from_toml();

    let messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| json!({"role": &m.role, "content": &m.content}))
        .collect();

    let request_body = json!({
        "model": config.ollama_model,
        "messages": messages,
        "stream": false,
    });

    let url = format!("http://{}/api/chat", config.ollama_server);
    let resp = client
        .post(url)
        .json(&request_body)
        .send()
        .await
        .context("request to Ollama failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_text = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Ollama API error: {} - {}", status, error_text));
    }

    let body: serde_json::Value = resp.json().await.context("invalid JSON from Ollama")?;

    // Return an error if the LLM response is missing content (don't silently speak empty string)
    let content = body
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::anyhow!("Ollama response missing 'message.content' field"))?;

    Ok(content.to_string())
}
