use headless_chrome::{Browser, LaunchOptions};
use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio::time::Duration;
use url::Url;

use crate::crawler::fetching::fetch_and_process;
use crate::crawler::helpers::html_parser::PageData;
use crate::crawler::helpers::user_agents::user_agents;
use crate::settings::utils::create::add_recent_entry;

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
    pagespeed_config: Option<crate::models::PageSpeedConfig>,
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
            .field("pagespeed", &self.pagespeed_config.is_some())
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
            pagespeed_config: None,
        }
    }

    pub fn with_pagespeed(mut self, config: Option<crate::models::PageSpeedConfig>) -> Self {
        self.pagespeed_config = config;
        self
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
        // SET THE RECENTLY CRAWLED INTO THE LIST OF RECENTLY CRAWLED URLS
        add_recent_entry(start_url.to_string()).await;
        let (tx, mut rx) = mpsc::channel(self.max_pages);
        let start_url = start_url.to_string();
        let engine = self.clone();

        // Start the crawl in a background task
        tokio::spawn(async move {
            engine.crawl_concurrently(&start_url, tx).await;
        });

        let mut results = Vec::new();
        while let Some(data) = rx.recv().await {
            match data {
                crate::crawler::CrawlMessage::Page(p) => {
                    if headless {
                        println!(
                            "[{}] {} - {} links found",
                            p.status,
                            p.url,
                            p.anchor_links.len()
                        );
                    }
                    results.push(p);
                }
                crate::crawler::CrawlMessage::Progress { .. } => {} // ignore progress for CLI
            }
        }

        results
    }

    /// concurrent crawler logic using JoinSet for task management
    pub async fn crawl_concurrently(
        &self,
        start_url: &str,
        tx: mpsc::Sender<crate::crawler::CrawlMessage>,
    ) {
        let base_url = match Url::parse(start_url) {
            Ok(url) => url,
            Err(_) => return,
        };

        //  new queue module for breadth-first crawling
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
                    let _permit = engine
                        .semaphore
                        .acquire()
                        .await
                        .map_err(|e| e.to_string())?;
                    fetch_and_process(
                        &engine.client,
                        &engine.browser,
                        &engine.user_agents,
                        &engine.pagespeed_config,
                        engine.enable_javascript,
                        &normalized_url,
                        &base_url_clone,
                        referer,
                    )
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
                                for link in &data.anchor_links {
                                    // Normalize each discovered link
                                    let normalized_link =
                                        match crate::crawler::url_normalizer::normalize_url(
                                            &link.href,
                                        ) {
                                            Some(u) => u,
                                            None => {
                                                normalization_failed += 1;
                                                tracing::debug!(
                                                    "[LINK] Failed to normalize: {}",
                                                    link.href
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
                        let _ = tx.send(crate::crawler::CrawlMessage::Page(data)).await;

                        // Send progress update
                        let scanned = self.success_count.load(Ordering::SeqCst) as usize;
                        let queued = queue.len();
                        let processing = join_set.len();
                        let _ = tx
                            .send(crate::crawler::CrawlMessage::Progress {
                                scanned,
                                queued,
                                processing,
                            })
                            .await;
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
}
