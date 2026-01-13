use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

/// Attempts to discover additional URLs from common sources
pub async fn discover_additional_urls(
    base_url: &str,
    client: &Client,
) -> Vec<String> {
    let mut discovered = Vec::new();
    
    // Try to fetch sitemap.xml
    if let Some(sitemap_urls) = fetch_sitemap(base_url, client).await {
        discovered.extend(sitemap_urls);
    }
    
    // Try robots.txt for sitemap references
    if let Some(robots_sitemaps) = fetch_robots_sitemaps(base_url, client).await {
        for sitemap_url in robots_sitemaps {
            if let Some(urls) = fetch_sitemap(&sitemap_url, client).await {
                discovered.extend(urls);
            }
        }
    }
    
    discovered
}

/// Fetches and parses a sitemap.xml file
async fn fetch_sitemap(sitemap_url: &str, client: &Client) -> Option<Vec<String>> {
    let url = if sitemap_url.ends_with(".xml") {
        sitemap_url.to_string()
    } else {
        format!("{}/sitemap.xml", sitemap_url.trim_end_matches('/'))
    };
    
    tracing::info!("[SITEMAP] Attempting to fetch: {}", url);
    
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;
    
    if !response.status().is_success() {
        tracing::debug!("[SITEMAP] Failed to fetch {}: {}", url, response.status());
        return None;
    }
    
    let text = response.text().await.ok()?;
    let mut urls = Vec::new();
    
    // Parse XML sitemap
    let doc = Html::parse_document(&text);
    let loc_selector = Selector::parse("loc").ok()?;
    
    for element in doc.select(&loc_selector) {
        let url_text = element.text().collect::<String>();
        if !url_text.is_empty() {
            urls.push(url_text.trim().to_string());
        }
    }
    
    if !urls.is_empty() {
        tracing::info!("[SITEMAP] Found {} URLs in sitemap", urls.len());
        Some(urls)
    } else {
        None
    }
}

/// Fetches robots.txt and extracts sitemap URLs
async fn fetch_robots_sitemaps(base_url: &str, client: &Client) -> Option<Vec<String>> {
    let base = Url::parse(base_url).ok()?;
    let robots_url = format!("{}://{}/robots.txt", base.scheme(), base.host_str()?);
    
    tracing::info!("[ROBOTS] Attempting to fetch: {}", robots_url);
    
    let response = client.get(&robots_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;
    
    if !response.status().is_success() {
        return None;
    }
    
    let text = response.text().await.ok()?;
    let mut sitemaps = Vec::new();
    
    for line in text.lines() {
        let line = line.trim();
        if line.to_lowercase().starts_with("sitemap:") {
            if let Some(sitemap_url) = line.split(':').nth(1) {
                let sitemap_url = sitemap_url.trim();
                if sitemap_url.starts_with("http") {
                    sitemaps.push(sitemap_url.to_string());
                }
            }
        }
    }
    
    if !sitemaps.is_empty() {
        tracing::info!("[ROBOTS] Found {} sitemap references", sitemaps.len());
        Some(sitemaps)
    } else {
        None
    }
}
