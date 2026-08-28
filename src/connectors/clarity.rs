use crate::models::ClarityInsights;

/// Fetches Microsoft Clarity's live project insights (Data Export API). The
/// response is a list of metric blocks whose inner shape varies per metric
/// (Traffic vs EngagementTime vs ScrollDepth, etc.), so it's kept as raw JSON
/// rather than modeled per-variant - the UI renders each block's metric name
/// plus a flattened summary of its fields.
pub async fn fetch(api_token: &str) -> Result<ClarityInsights, String> {
    if api_token.is_empty() {
        return Err("Set connectors.clarity.api_token in cli-settings.toml".to_string());
    }

    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.clarity.ms/export-data/api/v1/project-live-insights")
        .bearer_auth(api_token)
        .query(&[("numOfDays", "3")])
        .send()
        .await
        .map_err(|e| format!("Clarity request failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Clarity API error: {}", text));
    }

    let metrics: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| format!("Could not parse Clarity response: {}", e))?;

    Ok(ClarityInsights { metrics })
}
