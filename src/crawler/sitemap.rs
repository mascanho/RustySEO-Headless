use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

/// Discovers URLs from sitemap.xml (including sitemap indexes) and robots.txt.
pub async fn discover_additional_urls(base_url: &str, client: &Client) -> Vec<String> {
    let base = match Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return Vec::new(),
    };
    let origin = format!(
        "{}://{}",
        base.scheme(),
        base.host_str().unwrap_or_default()
    );

    let mut visited_sitemaps: HashSet<String> = HashSet::new();
    let mut discovered: Vec<String> = Vec::new();

    // Common sitemap locations to probe
    let candidates = [
        format!("{}/sitemap.xml", origin),
        format!("{}/sitemap_index.xml", origin),
        format!("{}/sitemap-index.xml", origin),
    ];

    for url in &candidates {
        if visited_sitemaps.insert(url.clone()) {
            let urls = fetch_sitemap_recursive(url, client, &mut visited_sitemaps).await;
            discovered.extend(urls);
        }
    }

    // robots.txt may declare additional sitemaps
    if let Some(robot_sitemaps) = fetch_robots_sitemaps(&origin, client).await {
        for url in robot_sitemaps {
            if visited_sitemaps.insert(url.clone()) {
                let urls = fetch_sitemap_recursive(&url, client, &mut visited_sitemaps).await;
                discovered.extend(urls);
            }
        }
    }

    discovered.sort_unstable();
    discovered.dedup();
    discovered
}

/// Fetches a sitemap URL. If it's a sitemap index, recurses into child sitemaps.
async fn fetch_sitemap_recursive(
    sitemap_url: &str,
    client: &Client,
    visited: &mut HashSet<String>,
) -> Vec<String> {
    let text = match fetch_text(sitemap_url, client).await {
        Some(t) => t,
        None => return Vec::new(),
    };

    if text.contains("<sitemapindex") {
        // Sitemap index — child <loc> entries point to other sitemaps, not pages.
        // Collect all child URLs synchronously (no await) so Html is dropped before recursion.
        let child_sitemaps: Vec<String> = {
            let doc = Html::parse_document(&text);
            let Ok(sel) = Selector::parse("sitemap loc") else {
                return Vec::new();
            };
            doc.select(&sel)
                .map(|el| el.text().collect::<String>().trim().to_string())
                .filter(|u| !u.is_empty())
                .collect()
        };

        let mut all_urls = Vec::new();
        for child_url in child_sitemaps {
            if !visited.insert(child_url.clone()) {
                continue;
            }
            tracing::info!("[SITEMAP INDEX] Child sitemap: {}", child_url);
            let child_urls =
                Box::pin(fetch_sitemap_recursive(&child_url, client, visited)).await;
            all_urls.extend(child_urls);
        }
        all_urls
    } else {
        // Regular sitemap — collect page URLs synchronously, no await needed.
        let urls: Vec<String> = {
            let doc = Html::parse_document(&text);
            let Ok(sel) = Selector::parse("url loc") else {
                return Vec::new();
            };
            doc.select(&sel)
                .map(|el| el.text().collect::<String>().trim().to_string())
                .filter(|u| !u.is_empty())
                .collect()
        };

        tracing::info!("[SITEMAP] {} URLs in {}", urls.len(), sitemap_url);
        urls
    }
}

async fn fetch_text(url: &str, client: &Client) -> Option<String> {
    tracing::info!("[SITEMAP] Fetching: {}", url);
    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        tracing::debug!("[SITEMAP] {} returned {}", url, resp.status());
        return None;
    }

    resp.text().await.ok()
}

/// Parses robots.txt for Sitemap: directives.
async fn fetch_robots_sitemaps(origin: &str, client: &Client) -> Option<Vec<String>> {
    let robots_url = format!("{}/robots.txt", origin);
    tracing::info!("[ROBOTS] Fetching: {}", robots_url);

    let resp = client
        .get(&robots_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let text = resp.text().await.ok()?;
    let sitemaps: Vec<String> = text
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.to_ascii_lowercase().starts_with("sitemap:") {
                return None;
            }
            // splitn(2) keeps the full URL (including its own colons, e.g. https://)
            let value = line.splitn(2, ':').nth(1)?.trim();
            // Reattach the scheme that was consumed by splitn
            let full_url = if value.starts_with("//") {
                format!("https:{}", value)
            } else if value.starts_with("http") {
                value.to_string()
            } else {
                return None;
            };
            Some(full_url)
        })
        .collect();

    if sitemaps.is_empty() {
        None
    } else {
        tracing::info!("[ROBOTS] Found {} sitemap references", sitemaps.len());
        Some(sitemaps)
    }
}
