use crate::connectors::google_oauth;
use crate::models::{Ga4Report, GoogleOAuthTokens};
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct RunReportResponse {
    #[serde(default, rename = "dimensionHeaders")]
    dimension_headers: Vec<Header>,
    #[serde(default, rename = "metricHeaders")]
    metric_headers: Vec<Header>,
    #[serde(default)]
    rows: Vec<ReportRow>,
}

#[derive(Deserialize)]
struct Header {
    name: String,
}

#[derive(Deserialize)]
struct ReportRow {
    #[serde(default, rename = "dimensionValues")]
    dimension_values: Vec<MetricValue>,
    #[serde(default, rename = "metricValues")]
    metric_values: Vec<MetricValue>,
}

#[derive(Deserialize)]
struct MetricValue {
    #[serde(default)]
    value: String,
}

/// Fetches a basic daily traffic report (activeUsers/sessions/pageviews over
/// the last 28 days) from the GA4 Data API, refreshing the access token
/// first if it's missing or expired.
pub async fn fetch(
    client_id: &str,
    client_secret: &str,
    property_id: &str,
    tokens: &mut GoogleOAuthTokens,
) -> Result<Ga4Report, String> {
    if property_id.is_empty() {
        return Err("Set connectors.ga4.property_id in cli-settings.toml".to_string());
    }
    google_oauth::refresh_if_needed(client_id, client_secret, tokens).await?;

    let api_url = format!(
        "https://analyticsdata.googleapis.com/v1beta/properties/{}:runReport",
        property_id
    );

    let body = serde_json::json!({
        "dateRanges": [{"startDate": "28daysAgo", "endDate": "today"}],
        "dimensions": [{"name": "date"}],
        "metrics": [
            {"name": "activeUsers"},
            {"name": "sessions"},
            {"name": "screenPageViews"},
        ],
        "orderBys": [{"dimension": {"dimensionName": "date"}}],
        "limit": 30,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&api_url)
        .bearer_auth(&tokens.access_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("GA4 request failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("GA4 API error: {}", text));
    }

    let parsed: RunReportResponse = resp
        .json()
        .await
        .map_err(|e| format!("Could not parse GA4 response: {}", e))?;

    Ok(Ga4Report {
        dimension_headers: parsed
            .dimension_headers
            .into_iter()
            .map(|h| h.name)
            .collect(),
        metric_headers: parsed.metric_headers.into_iter().map(|h| h.name).collect(),
        rows: parsed
            .rows
            .into_iter()
            .map(|r| {
                (
                    r.dimension_values.into_iter().map(|v| v.value).collect(),
                    r.metric_values.into_iter().map(|v| v.value).collect(),
                )
            })
            .collect(),
    })
}
