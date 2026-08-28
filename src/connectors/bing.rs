use crate::models::{BingQueryRow, BingQueryStats};
use serde::Deserialize;

#[derive(Deserialize)]
struct RowRaw {
    #[serde(default, rename = "Query")]
    query: String,
    #[serde(default, rename = "Clicks")]
    clicks: u64,
    #[serde(default, rename = "Impressions")]
    impressions: u64,
    #[serde(default, rename = "AvgClickPosition")]
    avg_click_position: f64,
    #[serde(default, rename = "AvgImpressionPosition")]
    avg_impression_position: f64,
}

/// Fetches top query stats via the Bing Webmaster Tools API.
pub async fn fetch(api_key: &str, site_url: &str) -> Result<BingQueryStats, String> {
    if api_key.is_empty() || site_url.is_empty() {
        return Err("Set connectors.bing.api_key and site_url in cli-settings.toml".to_string());
    }

    let client = reqwest::Client::new();
    let resp = client
        .get("https://ssl.bing.com/webmaster/api.svc/json/GetQueryStats")
        .query(&[("siteUrl", site_url), ("apikey", api_key)])
        .send()
        .await
        .map_err(|e| format!("Bing Webmaster request failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Bing Webmaster API error: {}", text));
    }

    let rows: Vec<RowRaw> = resp
        .json()
        .await
        .map_err(|e| format!("Could not parse Bing Webmaster response: {}", e))?;

    Ok(BingQueryStats {
        rows: rows
            .into_iter()
            .take(50)
            .map(|r| BingQueryRow {
                query: r.query,
                clicks: r.clicks,
                impressions: r.impressions,
                avg_click_position: r.avg_click_position,
                avg_impression_position: r.avg_impression_position,
            })
            .collect(),
    })
}
