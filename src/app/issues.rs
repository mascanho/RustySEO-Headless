use crate::models::App;
use crate::helpers::issues::IssueAnalyzer;

impl App {
    /// Populate issues_table_data with real crawled data analysis
    pub fn update_issues_from_crawled_data(&mut self) {
        self.issues_table_data = IssueAnalyzer::generate_issues_table_data_with_robots(
            &self.page_summaries,
            self.robots_disallowed_urls.len(),
            &self.internal_table_data,
            &self.redirects_table_data,
            &self.url_to_status,
            &self.input_url,
        );
    }

    /// Get real URLs for a specific issue type
    pub fn get_urls_for_issue(&self, issue_type: &str) -> Vec<String> {
        // Special handling for whole-crawl checks that need link-graph data beyond
        // a single page's summary - use cached results or recompute directly.
        match issue_type {
            " Robots Disallow Links" => return self.robots_disallowed_urls.clone(),
            " Orphan Pages" => {
                return IssueAnalyzer::analyse_orphan_pages(
                    &self.page_summaries,
                    &self.internal_table_data,
                    &self.input_url,
                )
                .1;
            }
            " Redirect Chains (> 1 hop)" => {
                return IssueAnalyzer::analyse_redirect_chains(&self.redirects_table_data).1;
            }
            " Broken Internal Links" => {
                return IssueAnalyzer::analyse_broken_internal_links(
                    &self.internal_table_data,
                    &self.url_to_status,
                )
                .1;
            }
            " Internal Nofollow Links" => {
                return IssueAnalyzer::analyse_internal_nofollow_links(&self.internal_table_data).1;
            }
            " Canonical Points to Broken Page" => {
                return IssueAnalyzer::analyse_canonical_points_to_broken(
                    &self.page_summaries,
                    &self.url_to_status,
                )
                .1;
            }
            " Redirects to Error Page" => {
                return IssueAnalyzer::analyse_redirects_to_error(&self.redirects_table_data).1;
            }
            _ => {}
        }

        IssueAnalyzer::get_urls_for_issue(&self.page_summaries, issue_type)
    }

    /// Spawn background task to fetch robots.txt + sitemaps when crawling starts.
    pub fn spawn_robots_analysis(&mut self, start_url: &str) {
        if self.robots_urls_loading || !self.robots_disallowed_urls.is_empty() {
            return;
        }

        self.robots_urls_loading = true;

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.robots_receiver = Some(rx);

        let start_url = start_url.to_string();

        tokio::spawn(async move {
            let base_url = start_url.split('/').take(3).collect::<Vec<_>>().join("/");
            let robots_url = format!("{}/robots.txt", base_url);

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default();

            // Fetch raw robots.txt
            let raw_content = async {
                let resp = client.get(&robots_url).send().await.ok()?;
                if !resp.status().is_success() {
                    return None;
                }
                resp.text().await.ok()
            }
            .await
            .unwrap_or_default();

            // Parse disallowed URLs from raw content
            let disallowed_urls = {
                let mut urls = Vec::new();
                for line in raw_content.lines() {
                    let line = line.trim();
                    if line.to_ascii_lowercase().starts_with("disallow:") {
                        if let Some(path) = line.splitn(2, ':').nth(1) {
                            let path = path.trim();
                            if !path.is_empty() && path != "/" {
                                let full = if path.starts_with('/') {
                                    format!("{}{}", base_url, path)
                                } else {
                                    path.to_string()
                                };
                                urls.push(full);
                            } else if path == "/" {
                                urls.push("All pages blocked (Disallow: /)".to_string());
                            }
                        }
                    }
                }
                urls
            };

            // Discover sitemap URLs
            let sitemap_urls =
                crate::crawler::sitemap::discover_additional_urls(&start_url, &client).await;

            let _ = tx
                .send(crate::models::RobotsResult {
                    disallowed_urls,
                    raw_content,
                    sitemap_urls,
                })
                .await;
        });
    }

    /// Check for robots/sitemaps results and update app state.
    pub fn check_robots_results(&mut self) {
        if let Some(ref mut rx) = self.robots_receiver {
            if let Ok(result) = rx.try_recv() {
                self.robots_disallowed_urls = result.disallowed_urls;
                self.robots_txt_content = result.raw_content;
                self.sitemap_urls = result.sitemap_urls;
                self.robots_urls_loading = false;

                if !self.page_summaries.is_empty() {
                    self.update_issues_from_crawled_data();
                }
            }
        }
    }

    /// Handle robots issue asynchronously (kept for backward compatibility but shouldn't be used)
    pub async fn load_robots_urls(&mut self) {
        if self.robots_urls_loading {
            return; // Already loading
        }
        
        self.robots_urls_loading = true;
        let urls = IssueAnalyzer::analyze_robots_on_demand(&self.page_summaries).await;
        
        self.robots_disallowed_urls = urls;
        self.robots_urls_loading = false;
        self.issue_urls_state.select(Some(0));
    }
}