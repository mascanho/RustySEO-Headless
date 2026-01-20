use crate::models::AppSettings;
use reqwest::Client;
use serde_json::{Value, json};

pub async fn ask(
    question: &str,
    settings: &AppSettings,
) -> Result<String, Box<dyn std::error::Error>> {
    if !settings.connectors.gemini.status || settings.connectors.gemini.api_key.is_empty() {
        return Err("Gemini is not configured or disabled".into());
    }

    let client = Client::new();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        settings.connectors.gemini.model, settings.connectors.gemini.api_key
    );

    let payload = json!({
        "contents": [{
            "parts": [{
                "text": question
            }]
        }]
    });

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Gemini API error: {}", error_text).into());
    }

    let response_json: Value = response.json().await?;

    if let Some(candidates) = response_json.get("candidates").and_then(|c| c.as_array()) {
        if let Some(first_candidate) = candidates.first() {
            if let Some(content) = first_candidate.get("content") {
                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                    if let Some(first_part) = parts.first() {
                        if let Some(text) = first_part.get("text").and_then(|t| t.as_str()) {
                            return Ok(text.to_string());
                        }
                    }
                }
            }
        }
    }

    Err("Invalid response format from Gemini API".into())
}
