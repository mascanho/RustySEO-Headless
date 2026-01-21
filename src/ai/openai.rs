use crate::models::AppSettings;
use reqwest::Client;
use serde_json::{json, Value};

pub async fn ask(
    question: &str,
    settings: &AppSettings,
) -> Result<String, Box<dyn std::error::Error>> {
    if settings.connectors.openai.model.trim().is_empty()
        || settings.connectors.openai.api_key.trim().is_empty()
    {
        return Err("OpenAI is not configured or disabled, check your settings file and see if \"openai\" or \"gemini\" are selected as the main LLM.".into());
    }

    let client = Client::new();

    let url = "https://api.openai.com/v1/chat/completions";

    let payload = json!({
        "model": settings.connectors.openai.model,
        "messages": [{"role": "user", "content": question}]
    });

    let response = client
        .post(url)
        .header(
            "Authorization",
            format!("Bearer {}", settings.connectors.openai.api_key),
        )
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("OpenAI API error: {}", error_text).into());
    }

    let response_json: Value = response.json().await?;

    if let Some(choices) = response_json.get("choices").and_then(|c| c.as_array()) {
        if let Some(first_choice) = choices.first() {
            if let Some(message) = first_choice.get("message") {
                if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                    return Ok(content.to_string());
                }
            }
        }
    }

    Err("Invalid response format from OpenAI API".into())
}
