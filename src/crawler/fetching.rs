use crate::crawler::helpers::html_parser::{PageData, extract_page_elements};
use headless_chrome::Browser;
use rand::Rng;
use reqwest::Client;
use scraper::Html;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use url::Url;

pub async fn fetch_and_process(
    client: &Client,
    browser: &Option<Arc<Browser>>,
    user_agents: &[String],
    pagespeed_config: &Option<crate::models::PageSpeedConfig>,
    enable_javascript: bool,
    url: &str,
    base_url: &Url,
    referer: Option<String>,
) -> Result<PageData, String> {
    apply_jitter().await;

    let mut page_data = if enable_javascript {
        if let Some(browser) = browser {
            fetch_js(url, base_url, browser.clone()).await?
        } else {
            fetch_standard(client, user_agents, url, base_url, referer).await?
        }
    } else {
        fetch_standard(client, user_agents, url, base_url, referer).await?
    };

    if let Some(config) = pagespeed_config {
        if config.status && !config.api_key.is_empty() {
            let is_html = page_data.content_type.to_lowercase().contains("text/html");
            let is_ok = page_data.status.starts_with('2');
            if is_html && is_ok {
                tracing::info!("[CWV] Fetching PageSpeed for {}", url);
                match fetch_pagespeed_data(client, url, "desktop", &config.api_key).await {
                    Ok(cwv) => page_data.cwv_desktop = Some(cwv),
                    Err(e) => tracing::error!("[CWV] Desktop failed for {}: {}", url, e),
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                match fetch_pagespeed_data(client, url, "mobile", &config.api_key).await {
                    Ok(cwv) => page_data.cwv_mobile = Some(cwv),
                    Err(e) => tracing::error!("[CWV] Mobile failed for {}: {}", url, e),
                }
            }
        }
    }

    Ok(page_data)
}

async fn fetch_standard(
    client: &Client,
    user_agents: &[String],
    url: &str,
    base_url: &Url,
    referer: Option<String>,
) -> Result<PageData, String> {
    let mut current_url = url.to_string();
    let mut redirect_chain = Vec::new();
    let mut hops = 0;
    let max_hops = 10;
    
    // Retry configuration
    let max_retries = 4;
    let mut retry_count = 0;
    let mut backoff = Duration::from_secs(2);

    let response = loop {
        let user_agent = pick_random_user_agent(user_agents);
        let mut request = client
            .get(&current_url)
            .header("User-Agent", user_agent)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "same-origin")
            .header("DNT", "1")
            .header("Upgrade-Insecure-Requests", "1");

        if let Some(ref_url) = referer.as_ref() {
            request = request.header("Referer", ref_url);
        }

        match request.send().await {
            Ok(res) => {
                let status = res.status();

                if status.is_redirection() && hops < max_hops {
                    if let Some(location) = res.headers().get("location") {
                        if let Ok(location_str) = location.to_str() {
                            // Resolve redirect URL against the CURRENT URL, not the base_url
                            let current_url_parsed = Url::parse(&current_url)
                                .map_err(|e| format!("Failed to parse current URL {}: {}", current_url, e))?;
                            
                            let next_url = match Url::parse(location_str) {
                                Ok(u) => u.to_string(),
                                Err(_) => match current_url_parsed.join(location_str) {
                                    Ok(u) => u.to_string(),
                                    Err(_) => current_url.clone(), 
                                },
                            };

                            redirect_chain.push(crate::models::RedirectHop {
                                url: next_url.clone(),
                                status: status.as_u16(),
                            });

                            current_url = next_url;
                            hops += 1;
                            retry_count = 0;
                            backoff = Duration::from_secs(2); 
                            continue;
                        }
                    }
                }
                
                // Handle Rate Limiting (429) and Server Errors (5xx)
                if status.as_u16() == 429 || status.is_server_error() {
                    // ... (keep existing retry logic)
                    if retry_count < max_retries {
                        retry_count += 1;
                        tracing::warn!(
                            "[RETRY] {} - {} (Attempt {}/{}). Waiting {:?}...",
                            url,
                            status,
                            retry_count,
                            max_retries,
                            backoff
                        );
                        sleep(backoff).await;
                        backoff *= 2; 
                        continue;
                    } else {
                        tracing::error!("[FAIL] {} - Max retries exceeded for status {}", url, status);
                        return Err(format!("Max retries exceeded for status {}", status));
                    }
                }

                break res;
            }
            Err(e) => {
               // ... (keep existing retry logic)
                if retry_count < max_retries {
                    retry_count += 1;
                    tracing::warn!(
                        "[RETRY] {} - Network Error: {} (Attempt {}/{}). Waiting {:?}...",
                        url,
                        e,
                        retry_count,
                        max_retries,
                        backoff
                    );
                    sleep(backoff).await;
                    backoff *= 2;
                    continue;
                } else {
                     return Err(format!("Request failed after retries: {}", e));
                }
            }
        }
    };

    let status_code = response.status();
    tracing::info!("[FETCH] {} -> STATUS: {}", url, status_code);

    let headers_list: Vec<String> = response
        .headers()
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("[invalid]")))
        .collect();

    let status = format!(
        "{} {}",
        status_code.as_u16(),
        status_code.canonical_reason().unwrap_or("")
    );

    let content_type_header = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let html_content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?;
    let document = Html::parse_document(&html_content);

    let mut page_data = extract_page_elements(&document);
    // Normalize the final (post-redirect) URL so it matches the form other pages'
    // anchor_links/outlinks use when referencing this page - otherwise redirected
    // pages never match url_to_status lookups (e.g. everything looks "unvisited"
    // on a site that trailing-slash- or https-redirects every internal link).
    page_data.url = crate::crawler::url_normalizer::normalize_url(&current_url)
        .unwrap_or_else(|| current_url.clone());
    page_data.requested_url = crate::crawler::url_normalizer::normalize_url(url)
        .unwrap_or_else(|| url.to_string());
    page_data.status = status;
    page_data.headers = headers_list;
    page_data.redirect_chain = redirect_chain;

    if let Some(ct) = content_type_header {
        page_data.content_type = ct;
    }
    
    // Parse the final URL to use as base for resolving relative links
    let final_url = Url::parse(&current_url)
        .map_err(|e| format!("Failed to parse final URL {}: {}", current_url, e))?;

    // Store all original links as outlinks before filtering
    page_data.outlinks = page_data
        .anchor_links
        .iter()
        .map(|link| {
            // Resolve against final_url (the page we are on)
            if let Ok(abs_url) = final_url.join(&link.href) {
                crate::crawler::helpers::html_parser::AnchorLink {
                    href: abs_url.to_string(),
                    text: link.text.clone(),
                    rel: link.rel.clone(),
                }
            } else {
                link.clone()
            }
        })
        .collect();

    // Filter and normalize links to stay on same domain
    let mut seen_urls = std::collections::HashSet::new();
    page_data.anchor_links = page_data
        .anchor_links
        .into_iter()
        .filter_map(|link| {
            // Resolve against final_url (the page we are on)
            if let Ok(mut abs_url) = final_url.join(&link.href) {
                // Check domain against base_url (crawl scope) using loose comparison
                if crate::crawler::url_normalizer::is_same_domain(abs_url.domain(), base_url.domain()) {
                    // Start by clearing fragment
                    abs_url.set_fragment(None);
                    
                    // Normalize the URL string using the centralized normalizer
                    if let Some(normalized_url) = crate::crawler::url_normalizer::normalize_url(abs_url.as_str()) {
                         // Check if we should crawl this URL type
                         if crate::crawler::url_normalizer::should_crawl_url(&normalized_url) {
                             // Deduplicate within this page
                             if seen_urls.insert(normalized_url.clone()) {
                                 return Some(crate::crawler::helpers::html_parser::AnchorLink {
                                     href: normalized_url,
                                     text: link.text,
                                     rel: link.rel,
                                 });
                             }
                         }
                    }
                }
            }
            None
        })
        .collect();

    tracing::info!(
        "[LINKS] Found {} unique same-domain links on {}",
        page_data.anchor_links.len(),
        url
    );

    Ok(page_data)
}

async fn fetch_js(url: &str, base_url: &Url, browser: Arc<Browser>) -> Result<PageData, String> {
    tracing::debug!("[JS-FETCH] Navigating to {}", url);

    let url_str = url.to_string();

    let (html_content, status) = tokio::task::spawn_blocking(move || {
        let tab = browser
            .new_tab()
            .map_err(|e| format!("Tab creation failed: {}", e))?;
        tab.navigate_to(&url_str)
            .map_err(|e| format!("Navigation failed: {}", e))?;
        tab.wait_until_navigated()
            .map_err(|e| format!("Wait failed: {}", e))?;
        let content = tab
            .get_content()
            .map_err(|e| format!("Content fetch failed: {}", e))?;
        Ok::<(String, String), String>((content, "200 OK (JS)".to_string()))
    })
    .await
    .map_err(|e| e.to_string())??;

    let document = Html::parse_document(&html_content);
    let mut page_data = extract_page_elements(&document);
    page_data.url = url.to_string();
    page_data.requested_url = url.to_string();
    page_data.status = status;
    page_data.headers = vec!["Requested-Mode: JavaScript".to_string()];

    // Parse the current page URL to use as base for resolving relative links
    let current_page_url = Url::parse(url)
        .map_err(|e| format!("Failed to parse current URL {}: {}", url, e))?;

    // Store all original links as outlinks before filtering
    page_data.outlinks = page_data
        .anchor_links
        .iter()
        .map(|link| {
            if let Ok(abs_url) = current_page_url.join(&link.href) {
                crate::crawler::helpers::html_parser::AnchorLink {
                    href: abs_url.to_string(),
                    text: link.text.clone(),
                    rel: link.rel.clone(),
                }
            } else {
                link.clone()
            }
        })
        .collect();

    // Filter and normalize links...
    let mut seen_urls = std::collections::HashSet::new();
    page_data.anchor_links = page_data
        .anchor_links
        .into_iter()
        .filter_map(|link| {
            if let Ok(mut abs_url) = current_page_url.join(&link.href) {
                if crate::crawler::url_normalizer::is_same_domain(abs_url.domain(), base_url.domain()) {
                   abs_url.set_fragment(None);
                   
                   // Normalize using centralized logic
                   if let Some(normalized_url) = crate::crawler::url_normalizer::normalize_url(abs_url.as_str()) {
                       if crate::crawler::url_normalizer::should_crawl_url(&normalized_url) {
                            if seen_urls.insert(normalized_url.clone()) {
                                return Some(crate::crawler::helpers::html_parser::AnchorLink {
                                    href: normalized_url,
                                    text: link.text,
                                    rel: link.rel,
                                });
                            }
                       }
                   }
                }
            }
            None
        })
        .collect();

    tracing::info!(
        "[JS-LINKS] Found {} unique same-domain links on {}",
        page_data.anchor_links.len(),
        url
    );

    Ok(page_data)
}

async fn fetch_pagespeed_data(
    client: &Client,
    url: &str,
    strategy: &str,
    api_key: &str,
) -> Result<crate::crawler::helpers::html_parser::CwvData, String> {
    let mut api_url =
        Url::parse("https://www.googleapis.com/pagespeedonline/v5/runPagespeed").unwrap();
    api_url
        .query_pairs_mut()
        .append_pair("url", url)
        .append_pair("key", api_key)
        .append_pair("strategy", strategy)
        .append_pair("category", "performance");

    let response = client
        .get(api_url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parse failed: {}", e))?;

    let lighthouse = json.get("lighthouseResult").ok_or("No lighthouseResult")?;
    let categories = lighthouse.get("categories").ok_or("No categories")?;
    let performance = categories
        .get("performance")
        .ok_or("No performance category")?;
    let score = performance
        .get("score")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        * 100.0;
    let audits = lighthouse.get("audits").ok_or("No audits")?;

    let get_display_value = |audit_name: &str| {
        audits
            .get(audit_name)
            .and_then(|a| a.get("displayValue"))
            .and_then(|v| v.as_str())
            .unwrap_or("N/A")
            .to_string()
    };

    Ok(crate::crawler::helpers::html_parser::CwvData {
        fcp: get_display_value("first-contentful-paint"),
        lcp: get_display_value("largest-contentful-paint"),
        cls: get_display_value("cumulative-layout-shift"),
        tbt: get_display_value("total-blocking-time"),
        speed_index: get_display_value("speed-index"),
        performance_score: format!("{:.0}", score),
    })
}

async fn apply_jitter() {
    let jitter = rand::rng().random_range(100..=500);
    sleep(Duration::from_millis(jitter)).await;
}

fn pick_random_user_agent(user_agents: &[String]) -> String {
    if user_agents.is_empty() {
        "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)".to_string()
    } else {
        user_agents[rand::rng().random_range(0..user_agents.len())].clone()
    }
}
