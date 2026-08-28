use crate::models::AhrefsReport;

/// Fetches domain rating and backlink counts from the Ahrefs API v3
/// (requires a paid Ahrefs plan with API access). Response fields are
/// looked up defensively (top-level or nested under `metrics`) since Ahrefs'
/// exact v3 response shape isn't pinned down here against a live account.
pub async fn fetch(api_token: &str, target: &str) -> Result<AhrefsReport, String> {
    if api_token.is_empty() || target.is_empty() {
        return Err("Set connectors.ahrefs.api_token and target in cli-settings.toml".to_string());
    }

    let client = reqwest::Client::new();
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();

    let dr_resp = client
        .get("https://api.ahrefs.com/v3/site-explorer/domain-rating")
        .bearer_auth(api_token)
        .query(&[("target", target.to_string()), ("date", today.clone())])
        .send()
        .await
        .map_err(|e| format!("Ahrefs domain-rating request failed: {}", e))?;
    if !dr_resp.status().is_success() {
        let text = dr_resp.text().await.unwrap_or_default();
        return Err(format!("Ahrefs API error: {}", text));
    }
    let dr_json: serde_json::Value = dr_resp
        .json()
        .await
        .map_err(|e| format!("Could not parse Ahrefs domain-rating response: {}", e))?;

    let backlinks_resp = client
        .get("https://api.ahrefs.com/v3/site-explorer/backlinks-stats")
        .bearer_auth(api_token)
        .query(&[
            ("target", target.to_string()),
            ("mode", "domain".to_string()),
            ("date", today),
        ])
        .send()
        .await
        .map_err(|e| format!("Ahrefs backlinks-stats request failed: {}", e))?;
    if !backlinks_resp.status().is_success() {
        let text = backlinks_resp.text().await.unwrap_or_default();
        return Err(format!("Ahrefs API error: {}", text));
    }
    let bl_json: serde_json::Value = backlinks_resp
        .json()
        .await
        .map_err(|e| format!("Could not parse Ahrefs backlinks-stats response: {}", e))?;

    let find_f64 = |v: &serde_json::Value, keys: &[&str]| -> f64 {
        for key in keys {
            if let Some(n) = v.get(key).and_then(|x| x.as_f64()) {
                return n;
            }
            if let Some(n) = v
                .get("metrics")
                .and_then(|m| m.get(key))
                .and_then(|x| x.as_f64())
            {
                return n;
            }
        }
        0.0
    };

    Ok(AhrefsReport {
        domain_rating: find_f64(&dr_json, &["domain_rating"]),
        ahrefs_rank: find_f64(&dr_json, &["ahrefs_rank"]) as u64,
        backlinks: find_f64(&bl_json, &["backlinks", "live"]) as u64,
        referring_domains: find_f64(&bl_json, &["refdomains", "referring_domains"]) as u64,
    })
}
