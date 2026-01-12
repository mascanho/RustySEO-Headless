use rand::rng;
use rand::Rng;
use reqwest::Client;
use scraper::Html;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::{sleep, Duration};
use url::Url;

use crate::crawler::helpers::{
    html_parser::{extract_page_elements, PageData},
    user_agents::user_agents,
};
use crate::tui_println;

#[derive(Debug, Clone)]
pub struct CrawlEngine {
    client: Client,
    visited: Arc<Mutex<HashSet<String>>>,
    max_pages: usize,
    user_agents: Vec<String>,
    concurrency_limit: usize,
    semaphore: Arc<Semaphore>,
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
            max_pages: 500, // Safety limit
            user_agents: user_agents_vec,
            concurrency_limit,
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
        }
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

        let mut to_visit = vec![start_url.to_string()];
        let mut join_set = tokio::task::JoinSet::new();

        while !to_visit.is_empty() || !join_set.is_empty() {
            // Fill up our concurrency quota
            while !to_visit.is_empty() && join_set.len() < self.concurrency_limit {
                let url = to_visit.pop().unwrap();

                {
                    let mut visited = self.visited.lock().await;
                    if visited.len() >= self.max_pages || visited.contains(&url) {
                        continue;
                    }
                    visited.insert(url.clone());
                }

                let engine = self.clone();
                let base_url_clone = base_url.clone();
                join_set
                    .spawn(async move { engine.fetch_and_process(&url, &base_url_clone).await });
            }

            // Wait for at least one task to complete
            if let Some(res) = join_set.join_next().await {
                match res {
                    Ok(Ok(data)) => {
                        // Extract new links BEFORE sending result to ensure consistency
                        for (link, _) in &data.anchor_links {
                            let visited = self.visited.lock().await;
                            if !visited.contains(link) && visited.len() < self.max_pages {
                                to_visit.push(link.clone());
                            }
                        }

                        // Send result back
                        let _ = tx.send(data).await;
                    }
                    Ok(Err(e)) => {
                        // Log error? For now we just continue
                        tui_println!("Error: {}", e);
                    }
                    Err(e) => {
                        // Log error? For now we just continue
                        tui_println!("Error: {}", e);
                    }
                }
            }

            // If we hit max pages, we stop spawning but finish existing ones
            let visited_count = {
                let v = self.visited.lock().await;
                v.len()
            };
            if visited_count >= self.max_pages {
                to_visit.clear();
            }
        }
    }

    async fn fetch_and_process(&self, url: &str, base_url: &Url) -> Result<PageData, String> {
        // Implement Jitter
        self.apply_jitter().await;

        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await.map_err(|e| e.to_string())?;

        let user_agent = self.pick_random_user_agent();

        let response = self.client
            .get(url)
            .header("User-Agent", user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("DNT", "1")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = format!(
            "{} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        );

        let headers: Vec<String> = response
            .headers()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
            .collect();

        let html_content = response
            .text()
            .await
            .map_err(|e| format!("Failed to read body: {}", e))?;
        let document = Html::parse_document(&html_content);

        let mut page_data = extract_page_elements(&document);
        page_data.url = url.to_string();
        page_data.status = status;
        page_data.headers = headers;

        // Filter and normalize links to stay on same domain
        page_data.anchor_links = page_data
            .anchor_links
            .into_iter()
            .filter_map(|(href, text)| {
                if let Ok(abs_url) = base_url.join(&href) {
                    if abs_url.domain() == base_url.domain() {
                        return Some((abs_url.to_string(), text));
                    }
                }
                None
            })
            .collect();

        Ok(page_data)
    }

    async fn apply_jitter(&self) {
        let jitter = {
            let mut r = rng();
            r.random_range(50..250)
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
}
