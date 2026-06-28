use anyhow::{Context, Result};
use serde_json::json;
use std::sync::OnceLock;
use crate::assistant::config::Config;
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
            .http2_prior_knowledge()
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
