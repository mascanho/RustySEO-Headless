use crate::crawler::helpers::html_parser::CwvData;

/// On-demand PageSpeed Insights run for a single URL, separate from the
/// during-crawl CWV fetch in `crawler::fetching` (same underlying API call).
pub async fn fetch(api_key: &str, url: &str, strategy: &str) -> Result<CwvData, String> {
    if api_key.is_empty() {
        return Err("Set connectors.pagespeed.api_key in cli-settings.toml".to_string());
    }
    if url.is_empty() {
        return Err("No URL to test - crawl a site first or set one on this tab".to_string());
    }
    let client = reqwest::Client::new();
    crate::crawler::fetching::fetch_pagespeed_data(&client, url, strategy, api_key).await
}
