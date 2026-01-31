use crate::models::App;
use crate::helpers::issues::IssueAnalyzer;

impl App {
    /// Populate issues_table_data with real crawled data analysis
    pub fn update_issues_from_crawled_data(&mut self) {
        self.issues_table_data = IssueAnalyzer::generate_issues_table_data_with_robots(
            &self.page_summaries, 
            self.robots_disallowed_urls.len()
        );
    }

    /// Get real URLs for a specific issue type
    pub fn get_urls_for_issue(&self, issue_type: &str) -> Vec<String> {
        // Special handling for robots disallow links - use cached results
        if issue_type == " Robots Disallow Links" {
            return self.robots_disallowed_urls.clone();
        }
        
        IssueAnalyzer::get_urls_for_issue(&self.page_summaries, issue_type)
    }

    /// Spawn background task to fetch robots.txt when crawling starts
    pub fn spawn_robots_analysis(&mut self, start_url: &str) {
        // Only spawn if not already loading and no results exist
        if self.robots_urls_loading || !self.robots_disallowed_urls.is_empty() {
            return;
        }

        self.robots_urls_loading = true;
        
        // Create channel for communication
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        self.robots_receiver = Some(rx);
        
        let start_url = start_url.to_string();
        
        // Spawn background task
        tokio::spawn(async move {
            // Fetch robots.txt in background
            let base_url = start_url.split('/').take(3).collect::<Vec<_>>().join("/");
            let robots_url = format!("{}/robots.txt", base_url);
            
            let result = match crate::crawler::helpers::robots::extract_robots_blocked_urls(&robots_url).await {
                Ok(urls) => urls,
                Err(_) => Vec::new(),
            };
            
            // Send result back to main thread
            let _ = tx.send(result).await;
        });
    }

    /// Check for robots analysis results and update app state
    pub fn check_robots_results(&mut self) {
        if let Some(ref mut rx) = self.robots_receiver {
            if let Ok(urls) = rx.try_recv() {
                self.robots_disallowed_urls = urls;
                self.robots_urls_loading = false;
                
                // Update issues table if we have page data
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