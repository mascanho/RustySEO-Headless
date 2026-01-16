use headless_chrome::{Browser, LaunchOptions};
use rand::Rng;
use rand::rng;
use reqwest::Client;
use scraper::Html;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio::time::{Duration, sleep};
use url::Url;

use crate::crawler::helpers::{
    html_parser::{PageData, extract_page_elements},
    user_agents::user_agents,
};

#[derive(Clone)]
pub struct CrawlEngine {
    client: Client,
    visited: Arc<Mutex<HashSet<String>>>,
    success_count: Arc<AtomicUsize>,
    max_pages: usize,
    user_agents: Vec<String>,
    concurrency_limit: usize,
    semaphore: Arc<Semaphore>,
    stats: crate::crawler::stats::CrawlStats,
    enable_javascript: bool,
    browser: Option<Arc<Browser>>,
}

impl std::fmt::Debug for CrawlEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrawlEngine")
            .field("client", &self.client)
            .field("visited", &self.visited)
            .field("success_count", &self.success_count)
            .field("max_pages", &self.max_pages)
            .field("user_agents", &self.user_agents)
            .field("concurrency_limit", &self.concurrency_limit)
            .field("semaphore", &self.semaphore)
            .field("stats", &self.stats)
            .field("enable_javascript", &self.enable_javascript)
            .field("browser", &self.browser.is_some())
            .finish()
    }
}

impl CrawlEngine {
    pub async fn new() -> Self {
        let user_agents_vec = match user_agents() {
            Ok(agents) if !agents.is_empty() => agents,
            _ => vec!["Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()],
        };

        let default_ua = user_agents_vec[0].clone();
        let concurrency_limit = 10; // Default limit

        Self {
            client: Client::builder()
                .user_agent(default_ua)
                .timeout(Duration::from_secs(15))
                .pool_max_idle_per_host(concurrency_limit)
                .tcp_keepalive(Some(Duration::from_secs(60)))
                .build()
                .unwrap(),
            visited: Arc::new(Mutex::new(HashSet::new())),
            success_count: Arc::new(AtomicUsize::new(0)),
            max_pages: 500, // Safety limit
            user_agents: user_agents_vec,
            concurrency_limit,
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
            stats: crate::crawler::stats::CrawlStats::new(),
            enable_javascript: false,
            browser: None,
        }
    }

    pub fn with_javascript(mut self, enable: bool) -> Self {
        self.enable_javascript = enable;
        if enable {
            let options = LaunchOptions {
                headless: true,
                ..Default::default()
            };
            match Browser::new(options) {
                Ok(b) => self.browser = Some(Arc::new(b)),
                Err(e) => tracing::error!("Failed to launch browser: {}", e),
            }
        }
        self
    }

    /// Sets the maximum number of pages to crawl
    pub fn with_max_pages(mut self, max: usize) -> Self {
        self.max_pages = max;
        self
    }

    /// Sets the concurrency limit
    pub fn with_concurrency(mut self, limit: usize) -> Self {
        self.concurrency_limit = limit;
        self.semaphore = Arc::new(Semaphore::new(limit));
        self
    }

    /// Primary entry point for crawling.
    /// Returns a vector of PageData for backward compatibility.
    pub async fn crawl(&self, start_url: &str, headless: bool) -> Vec<PageData> {
        let (tx, mut rx) = mpsc::channel(self.max_pages);
        let start_url = start_url.to_string();
        let engine = self.clone();

        // Start the crawl in a background task
        tokio::spawn(async move {
            engine.crawl_concurrently(&start_url, tx).await;
        });

        let mut results = Vec::new();
        while let Some(data) = rx.recv().await {
            if headless {
                println!(
                    "[{}] {} - {} links found",
                    data.status,
                    data.url,
                    data.anchor_links.len()
                );
            }
            results.push(data);
        }

        results
    }

    /// Robust concurrent crawler logic using JoinSet for task management
    pub async fn crawl_concurrently(&self, start_url: &str, tx: mpsc::Sender<PageData>) {
        let base_url = match Url::parse(start_url) {
            Ok(url) => url,
            Err(_) => return,
        };

        // Use our new queue module for breadth-first crawling
        let mut queue = crate::crawler::queue::CrawlQueue::new();
        queue.push(start_url.to_string(), None);

        // Try to discover URLs from sitemap.xml and robots.txt
        tracing::info!("[DISCOVERY] Attempting to discover URLs from sitemap and robots.txt...");
        let sitemap_urls =
            crate::crawler::sitemap::discover_additional_urls(start_url, &self.client).await;
        if !sitemap_urls.is_empty() {
            tracing::info!(
                "[DISCOVERY] Found {} URLs from sitemaps, adding to queue",
                sitemap_urls.len()
            );
            for url in sitemap_urls {
                // Only add URLs from the same domain
                if let Ok(parsed) = Url::parse(&url) {
                    if parsed.domain() == base_url.domain() {
                        queue.push(url, None);
                    }
                }
            }
        }

        let mut join_set = tokio::task::JoinSet::new();

        while !queue.is_empty() || !join_set.is_empty() {
            // Fill up our concurrency quota
            while !queue.is_empty() && join_set.len() < self.concurrency_limit {
                if self.success_count.load(Ordering::SeqCst) >= self.max_pages {
                    break;
                }

                let (url, referer) = queue.pop().unwrap();

                // Normalize URL before checking if visited
                let normalized_url = match crate::crawler::url_normalizer::normalize_url(&url) {
                    Some(u) => u,
                    None => {
                        tracing::warn!("[SKIP] Failed to normalize URL: {}", url);
                        continue;
                    }
                };

                // Check if we should crawl this URL
                if !crate::crawler::url_normalizer::should_crawl_url(&normalized_url) {
                    tracing::debug!("[SKIP] URL filtered out: {}", normalized_url);
                    continue;
                }

                {
                    let mut visited = self.visited.lock().await;
                    if visited.contains(&normalized_url) {
                        continue;
                    }
                    visited.insert(normalized_url.clone());
                }

                let engine = self.clone();
                let base_url_clone = base_url.clone();
                join_set.spawn(async move {
                    engine
                        .fetch_and_process(&normalized_url, &base_url_clone, referer)
                        .await
                });
            }

            // Wait for at least one task to complete
            if let Some(res) = join_set.join_next().await {
                match res {
                    Ok(Ok(data)) => {
                        // Extract new links BEFORE sending result to ensure consistency
                        let current_url = data.url.clone();
                        let current_success = self.success_count.load(Ordering::SeqCst);
                        let links_found = data.anchor_links.len();

                        self.stats.add_discovered(links_found);

                        // Only add new links if we haven't reached max successful crawls
                        if current_success < self.max_pages {
                            let mut new_links = Vec::new();
                            let mut filtered_count = 0;
                            let mut duplicate_count = 0;
                            let mut normalization_failed = 0;

                            {
                                let visited = self.visited.lock().await;
                                for (link, _) in &data.anchor_links {
                                    // Normalize each discovered link
                                    let normalized_link =
                                        match crate::crawler::url_normalizer::normalize_url(link) {
                                            Some(u) => u,
                                            None => {
                                                normalization_failed += 1;
                                                tracing::debug!(
                                                    "[LINK] Failed to normalize: {}",
                                                    link
                                                );
                                                continue;
                                            }
                                        };

                                    if !crate::crawler::url_normalizer::should_crawl_url(
                                        &normalized_link,
                                    ) {
                                        filtered_count += 1;
                                        tracing::debug!(
                                            "[LINK] Filtered (non-HTML): {}",
                                            normalized_link
                                        );
                                        continue;
                                    }

                                    if visited.contains(&normalized_link) {
                                        duplicate_count += 1;
                                        tracing::debug!("[LINK] Duplicate: {}", normalized_link);
                                    } else {
                                        new_links.push(normalized_link.clone());
                                        tracing::debug!("[LINK] NEW -> Queue: {}", normalized_link);
                                    }
                                }
                            } // Release lock before pushing to queue

                            self.stats.add_filtered(filtered_count);
                            self.stats.add_duplicate(duplicate_count);

                            // Add all new links to the queue
                            queue.push_batch(new_links.clone(), Some(current_url.clone()));

                            tracing::info!(
                                "[QUEUE] Page: {} | Found: {} links | New: {} | Filtered: {} | Duplicates: {} | Failed Norm: {} | Queue: {} | Success: {}/{}",
                                current_url,
                                links_found,
                                new_links.len(),
                                filtered_count,
                                duplicate_count,
                                normalization_failed,
                                queue.len(),
                                current_success + 1,
                                self.max_pages
                            );

                            // Print summary stats every 50 pages
                            if (current_success + 1) % 50 == 0 {
                                tracing::warn!("[STATS] {}", self.stats.get_summary());
                            }

                            // Warning if no new links found
                            if new_links.is_empty() && links_found > 0 {
                                tracing::warn!(
                                    "[WARNING] Page {} had {} links but ALL were duplicates/filtered. Queue size: {}",
                                    current_url,
                                    links_found,
                                    queue.len()
                                );
                            }
                        }

                        // Send result back
                        self.success_count.fetch_add(1, Ordering::SeqCst);
                        self.stats.increment_crawled();
                        let _ = tx.send(data).await;
                    }
                    Ok(Err(e)) => {
                        self.stats.increment_failed();
                        tracing::error!("Crawl Error: {}", e);
                    }
                    Err(e) => {
                        self.stats.increment_failed();
                        tracing::error!("Task Panic/Error: {}", e);
                    }
                }
            }

            // If we hit max pages, we stop spawning but finish existing ones
            if self.success_count.load(Ordering::SeqCst) >= self.max_pages {
                queue.clear();
            }

            // Diagnostic: Check if queue is empty but we haven't hit max pages
            if queue.is_empty() && join_set.is_empty() {
                let current = self.success_count.load(Ordering::SeqCst);
                if current < self.max_pages {
                    tracing::warn!(
                        "[QUEUE EXHAUSTED] Crawl stopped at {} pages (target: {}). No more links to discover. {}",
                        current,
                        self.max_pages,
                        self.stats.get_summary()
                    );
                }
            }
        }

        // Final stats
        tracing::warn!("[CRAWL COMPLETE] {}", self.stats.get_summary());
    }

    async fn fetch_and_process(
        &self,
        url: &str,
        base_url: &Url,
        referer: Option<String>,
    ) -> Result<PageData, String> {
        // Implement Jitter
        self.apply_jitter().await;

        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await.map_err(|e| e.to_string())?;

        // Logic split for JS crawling
        if self.enable_javascript {
            if let Some(ref browser) = self.browser {
                return self.fetch_js(url, base_url, browser.clone()).await;
            }
        }

        let user_agent = self.pick_random_user_agent();
        tracing::debug!("[UA] Using agent: {} for {}", user_agent, url);

        let mut request = self.client
            .get(url)
            .header("User-Agent", user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "same-origin")
            .header("DNT", "1")
            .header("Upgrade-Insecure-Requests", "1");

        if let Some(ref_url) = referer {
            request = request.header("Referer", ref_url);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status_code = response.status();

        // Detailed response logging for debugging blocks/CDNs
        tracing::info!("[FETCH] {} -> STATUS: {}", url, status_code);

        let headers_list: Vec<String> = response
            .headers()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("[invalid]")))
            .collect();

        // Check for specific security headers or CDN signals
        for h in &headers_list {
            let h_low = h.to_lowercase();
            if h_low.contains("server: cloudflare") {
                tracing::warn!("--- Blocking Signal: Cloudflare detected at {}", url);
            }
            if h_low.contains("x-cache: hit") || h_low.contains("x-cache: miss") {
                tracing::debug!("--- Cache Info: {} for {}", h, url);
            }
        }

        tracing::debug!(
            "<<< RESPONSE HEADERS ({}):\n{}",
            url,
            headers_list.join("\n")
        );

        if status_code.as_u16() == 429 {
            tracing::error!("Rate limited (429) at {}. Waiting 5s...", url);
            sleep(Duration::from_secs(5)).await;
            return Err(format!("Rate limited (429) at {}", url));
        }

        if status_code.as_u16() == 403 {
            tracing::error!("Forbidden (403) at {}. Might be blocked.", url);
            return Err(format!("Forbidden (403) at {}", url));
        }

        let status = format!(
            "{} {}",
            status_code.as_u16(),
            status_code.canonical_reason().unwrap_or("")
        );

        let headers = headers_list;

        let html_content = response
            .text()
            .await
            .map_err(|e| format!("Failed to read body: {}", e))?;
        let document = Html::parse_document(&html_content);

        let mut page_data = extract_page_elements(&document);
        page_data.url = url.to_string();
        page_data.status = status;
        page_data.headers = headers;

        // Store all original links as outlinks before filtering
        page_data.outlinks = page_data
            .anchor_links
            .iter()
            .filter_map(|(href, text)| {
                if let Ok(abs_url) = base_url.join(href) {
                    Some((abs_url.to_string(), text.clone()))
                } else {
                    Some((href.clone(), text.clone()))
                }
            })
            .collect();

        // Filter and normalize links to stay on same domain
        let mut seen_urls = HashSet::new();
        page_data.anchor_links = page_data
            .anchor_links
            .into_iter()
            .filter_map(|(href, text)| {
                if let Ok(mut abs_url) = base_url.join(&href) {
                    // Remove fragment (e.g., #section) to avoid duplicate crawls
                    abs_url.set_fragment(None);

                    if abs_url.domain() == base_url.domain() {
                        let url_str = abs_url.to_string();
                        // Deduplicate within this page
                        if seen_urls.insert(url_str.clone()) {
                            return Some((url_str, text));
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

    async fn apply_jitter(&self) {
        let jitter = {
            let mut r = rng();
            r.random_range(300..1200)
        };
        sleep(Duration::from_millis(jitter)).await;
    }

    fn pick_random_user_agent(&self) -> String {
        let mut r = rng();
        if self.user_agents.is_empty() {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        } else {
            let idx = r.random_range(0..self.user_agents.len());
            self.user_agents[idx].clone()
        }
    }

    async fn fetch_js(
        &self,
        url: &str,
        base_url: &Url,
        browser: Arc<Browser>,
    ) -> Result<PageData, String> {
        tracing::debug!("[JS-FETCH] Navigating to {}", url);

        let url_str = url.to_string();
        let browser = browser.clone();

        // Blocking interaction with headless_chrome
        let (html_content, status) = tokio::task::spawn_blocking(move || {
            let tab = browser
                .new_tab()
                .map_err(|e| format!("Tab creation failed: {}", e))?;

            // Set User Agent? (Headless chrome might have default, or we can set it)
            // tab.set_user_agent(...)

            tab.navigate_to(&url_str)
                .map_err(|e| format!("Navigation failed: {}", e))?;
            tab.wait_until_navigated()
                .map_err(|e| format!("Wait failed: {}", e))?;

            // Wait extra time for JS to render?
            // std::thread::sleep(Duration::from_millis(1000));

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
        page_data.status = status;
        page_data.headers = vec!["Requested-Mode: JavaScript".to_string()];

        // Same Link Processing Logic
        // Store all original links as outlinks before filtering
        page_data.outlinks = page_data
            .anchor_links
            .iter()
            .filter_map(|(href, text)| {
                if let Ok(abs_url) = base_url.join(href) {
                    Some((abs_url.to_string(), text.clone()))
                } else {
                    Some((href.clone(), text.clone()))
                }
            })
            .collect();

        // Filter and normalize links...
        let mut seen_urls = HashSet::new();
        page_data.anchor_links = page_data
            .anchor_links
            .into_iter()
            .filter_map(|(href, text)| {
                if let Ok(mut abs_url) = base_url.join(&href) {
                    abs_url.set_fragment(None);
                    if abs_url.domain() == base_url.domain() {
                        let url_str = abs_url.to_string();
                        if seen_urls.insert(url_str.clone()) {
                            return Some((url_str, text));
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
}
