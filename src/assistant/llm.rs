use anyhow::{Context, Result};
use serde_json::json;
use std::sync::OnceLock;
use crate::assistant::config::Config;
use super::conversation::Message;

/// Shared HTTP client — avoids creating a new TCP connection pool per call
fn get_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client")
    })
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
