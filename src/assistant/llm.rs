use anyhow::{Context, Result};
use serde_json::json;
use crate::assistant::config::get_ollama_config;
use super::conversation::Message;

pub async fn call_ollama_api(history: &[Message]) -> Result<String> {
    let client = reqwest::Client::new();
    let cfg = get_ollama_config()?;

    let messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| json!({"role": &m.role, "content": &m.content}))
        .collect();

    let request_body = json!({
        "model": cfg.model,
        "messages": messages,
        "stream": false,
    });

    let url = format!("http://{}/api/chat", cfg.server);
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
    let content = body
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(content)
}
