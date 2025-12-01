use anyhow::{Context, Result};
use serde_json::json;
use super::conversation::Message;

/// Call Ollama with Gemma 3.2 model
pub async fn call_ollama_api(history: &[Message]) -> Result<String> {
    let client = reqwest::Client::new();

    // Build messages for Ollama
    let mut messages = vec![];

    for msg in history {
        messages.push(json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    let request_body = json!({
        "model": "phi:2.7b",  // Change to your model name if different
        "messages": messages,
        "stream": false,
    });

    let response = client
        .post("http://172.22.32.1:11434/api/chat")
        .json(&request_body)
        .send()
        .await
        .context("Failed to call Ollama API. Is it running? (ollama serve)")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Ollama API error: {} - {}",
            status,
            error_text
        ));
    }

    let body: serde_json::Value = response.json().await?;

    let content = body
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .context("Failed to extract response from Ollama. Check model name.")?;

    Ok(content.to_string())
}