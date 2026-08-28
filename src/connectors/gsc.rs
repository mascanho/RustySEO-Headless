use crate::connectors::google_oauth;
use crate::models::{GoogleOAuthTokens, GscReport, GscRow};
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct QueryResponse {
    #[serde(default)]
    rows: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    #[serde(default)]
    keys: Vec<String>,
    #[serde(default)]
    clicks: f64,
    #[serde(default)]
    impressions: f64,
    #[serde(default)]
    ctr: f64,
    #[serde(default)]
    position: f64,
}

/// Fetches the last 28 days of Search Analytics data grouped by query,
/// refreshing the access token first if it's missing or expired.
pub async fn fetch(
    client_id: &str,
    client_secret: &str,
    site_url: &str,
    tokens: &mut GoogleOAuthTokens,
) -> Result<GscReport, String> {
    if site_url.is_empty() {
        return Err("Set connectors.search_console.site_url in cli-settings.toml".to_string());
    }
    google_oauth::refresh_if_needed(client_id, client_secret, tokens).await?;

    let end = chrono::Utc::now().date_naive();
    let start = end - chrono::Duration::days(28);
    let encoded_site: String =
        url::form_urlencoded::byte_serialize(site_url.as_bytes()).collect();
    let api_url = format!(
        "https://www.googleapis.com/webmasters/v3/sites/{}/searchAnalytics/query",
        encoded_site
    );

    let body = serde_json::json!({
        "startDate": start.format("%Y-%m-%d").to_string(),
        "endDate": end.format("%Y-%m-%d").to_string(),
        "dimensions": ["query"],
        "rowLimit": 25,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&api_url)
        .bearer_auth(&tokens.access_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Search Console request failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Search Console API error: {}", text));
    }

    let parsed: QueryResponse = resp
        .json()
        .await
        .map_err(|e| format!("Could not parse Search Console response: {}", e))?;

    Ok(GscReport {
        rows: parsed
            .rows
            .into_iter()
            .map(|r| GscRow {
                keys: r.keys,
                clicks: r.clicks,
                impressions: r.impressions,
                ctr: r.ctr,
                position: r.position,
            })
            .collect(),
    })
}
