use crate::models::App;

impl App {
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_logs(&mut self) {
        self.show_logs = !self.show_logs;
    }

    pub fn toggle_ai_modal(&mut self) {
        self.show_ai_modal = !self.show_ai_modal;
        if self.show_ai_modal {
            self.sidebar_visible = false;
            self.show_help = false;
        }
    }

    pub async fn submit_ai_message(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.ai_input.trim().is_empty() {
            return Ok(());
        }

        let user_msg = crate::models::ChatLog {
            role: "user".to_string(),
            content: self.ai_input.clone(),
        };
        self.ai_chat_history.push(user_msg);
        let input = self.ai_input.clone();
        self.ai_input.clear();

        // Get AI response using Gemini
        let settings = self.settings.as_ref().ok_or("Settings not loaded")?;

        let response = match settings.provider.llm.to_lowercase().as_str() {
            "gemini" => match crate::ai::gemini::ask(&input, settings).await {
                Ok(resp) => resp,
                Err(e) => format!("Error: {}", e),
            },
            "openai" => match crate::ai::openai::ask(&input, settings).await {
                Ok(resp) => resp,
                Err(e) => format!("Error: {}", e),
            },
            _ => return Err("Unsupported AI model".into()),
        };

        self.ai_chat_history.push(crate::models::ChatLog {
            role: "assistant".to_string(),
            content: response,
        });

        // Scroll to bottom
        if !self.ai_chat_history.is_empty() {
            self.ai_chat_state
                .select(Some(self.ai_chat_history.len() - 1));
        }

        Ok(())
    }

    pub fn clear_ai_chat(&mut self) {
        self.ai_chat_history.clear();
        self.ai_chat_state.select(None);
    }

    pub fn handle_issues_enter(&mut self) {
        if let Some(selected) = self.issues_table_state.selected() {
            if selected < self.issues_table_data.len() {
                let issue_title = self.issues_table_data[selected][0].clone();
                self.current_issue_title = issue_title.clone();

                // Check if this is the robots issue
                if issue_title == " Robots Disallow Links" {
                    // For robots, show the cached results directly
                    self.issue_urls_list = self.get_urls_for_issue(&issue_title);
                    if self.issue_urls_list.is_empty() {
                        self.issue_urls_list = vec!["No disallowed URLs found in robots.txt".to_string()];
                    }
                    self.issue_urls_state.select(Some(0));
                    self.show_issue_urls_modal = true;
                } else {
                    // Generate real URLs for other issues
                    self.issue_urls_list = self.get_urls_for_issue(&issue_title);
                    self.issue_urls_state.select(Some(0));
                    self.show_issue_urls_modal = true;
                }

                self.log(format!("Showing URLs for issue: {}", issue_title));
            }
        }
    }

    pub fn close_issue_urls_modal(&mut self) {
        self.show_issue_urls_modal = false;
        self.issue_urls_list.clear();
        self.current_issue_title.clear();
        self.robots_urls_loading = false;
        self.issue_urls_state.select(None);
    }

    pub fn next_issue_url(&mut self) {
        let len = self.issue_urls_list.len();
        if len == 0 {
            return;
        }
        let i = match self.issue_urls_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.issue_urls_state.select(Some(i));
    }

    pub fn previous_issue_url(&mut self) {
        let len = self.issue_urls_list.len();
        if len == 0 {
            return;
        }
        let i = match self.issue_urls_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.issue_urls_state.select(Some(i));
    }

    pub fn open_selected_issue_url(&mut self) {
        if let Some(selected) = self.issue_urls_state.selected() {
            if selected < self.issue_urls_list.len() {
                let raw_string = &self.issue_urls_list[selected];
                
                // Extract just the URL part from various formats:
                // "URL" - plain URL
                // "URL (extra info)" - URL with info in parentheses
                // "URL [ and ] URL" - duplicate content format
                let url = raw_string
                    .split(" (")
                    .next()
                    .unwrap_or(raw_string)
                    .split(" [")
                    .next()
                    .unwrap_or(raw_string)
                    .trim();

                crate::ui::modals::dashboard_menu::open_in_browser(url);
                self.log(format!("Opened URL in browser: {}", url));
            }
        }
    }

    pub fn copy_selected_issue_url(&mut self) {
        if let Some(selected) = self.issue_urls_state.selected() {
            if selected < self.issue_urls_list.len() {
                let raw_string = &self.issue_urls_list[selected];
                
                // Extract just the URL part (same logic as open_selected_issue_url)
                let url = raw_string
                    .split(" (")
                    .next()
                    .unwrap_or(raw_string)
                    .split(" [")
                    .next()
                    .unwrap_or(raw_string)
                    .trim();
                    
                crate::ui::modals::dashboard_menu::copy_to_clipboard(url.to_string());
                self.log(format!("Copied URL to clipboard: {}", url));
            }
        }
    }

    pub fn show_js_pages_for_url(&mut self, js_url: String) {
        if let Some(ref conn) = self.db_conn {
            self.js_pages_list = crate::db::get_pages_for_js(conn, &js_url);
            self.js_pages_state.select(Some(0));
            self.show_js_pages_modal = true;
        }
    }

    pub fn close_js_pages_modal(&mut self) {
        self.show_js_pages_modal = false;
        self.js_pages_list.clear();
        self.js_pages_state.select(None);
    }

    pub fn show_css_pages_for_url(&mut self, css_url: String) {
        if let Some(ref conn) = self.db_conn {
            self.css_pages_list = crate::db::get_pages_for_css(conn, &css_url);
            self.css_pages_state.select(Some(0));
            self.show_css_pages_modal = true;
        }
    }

    pub fn close_css_pages_modal(&mut self) {
        self.show_css_pages_modal = false;
        self.css_pages_list.clear();
        self.css_pages_state.select(None);
    }

    // Dashboard Menu methods
    pub fn next_dashboard_menu_item(&mut self) {
        let count = crate::ui::modals::dashboard_menu::MENU_ITEMS.len();
        self.dashboard_menu_selection = (self.dashboard_menu_selection + 1) % count;
    }

    pub fn previous_dashboard_menu_item(&mut self) {
        let count = crate::ui::modals::dashboard_menu::MENU_ITEMS.len();
        self.dashboard_menu_selection = if self.dashboard_menu_selection == 0 {
            count - 1
        } else {
            self.dashboard_menu_selection - 1
        };
    }

    pub fn execute_dashboard_menu_action(&mut self) {
        let action = self.dashboard_menu_selection;
        self.show_dashboard_menu = false;

        // Get the selected row (index must line up with MENU_ITEMS in dashboard_menu.rs)
        let row = if let Some(selected) = self.table_state.selected() {
            let full_idx = self.current_page * self.page_size + selected;
            match self.full_filtered_table_data.get(full_idx) {
                Some(row) => row.clone(),
                None => return,
            }
        } else {
            return;
        };
        let url = row[1].clone();

        match action {
            0 => {
                // Copy URL
                crate::ui::modals::dashboard_menu::copy_to_clipboard(url.clone());
                self.log(format!("Copied URL to clipboard: {}", url));
            }
            1 => {
                // Open URL in Browser
                crate::ui::modals::dashboard_menu::open_in_browser(&url);
                self.log(format!("Opened URL in browser: {}", url));
            }
            2 => {
                // Open in Google
                let google_url = format!("https://www.google.com/search?q={}", url);
                crate::ui::modals::dashboard_menu::open_in_browser(&google_url);
                self.log(format!("Opening URL in Google: {}", url));
            }
            3 => {
                // View SEO Score
                let link_score = self.link_scores.get(&url).copied();
                self.seo_score_data = Some(crate::app::menu_actions::calculate(&url, &row, link_score));
                self.show_seo_score_modal = true;
            }
            4 => {
                // Extract Links
                self.open_page_links_for_url(&url);
            }
            5 => {
                // Screenshot
                self.spawn_screenshot(url);
            }
            6 => {
                // Export Data
                self.export_overview_csv();
            }
            _ => {}
        }
    }

    pub fn close_seo_score_modal(&mut self) {
        self.show_seo_score_modal = false;
        self.seo_score_data = None;
    }

    /// Populate `page_links_list` with every link found on `url` (internal + external)
    /// and open the "Extract Links" modal.
    fn open_page_links_for_url(&mut self, url: &str) {
        let mut links: Vec<crate::models::PageLinkEntry> = self
            .internal_table_data
            .iter()
            .filter(|l| l.source == url)
            .map(|l| crate::models::PageLinkEntry {
                destination: l.destination.clone(),
                anchor: l.anchor.clone(),
                rel: l.rel.clone(),
                is_internal: true,
            })
            .chain(self.external_table_data.iter().filter(|l| l.source == url).map(|l| {
                crate::models::PageLinkEntry {
                    destination: l.destination.clone(),
                    anchor: l.anchor.clone(),
                    rel: l.rel.clone(),
                    is_internal: false,
                }
            }))
            .collect();
        links.sort_by(|a, b| b.is_internal.cmp(&a.is_internal));

        self.page_links_state
            .select(if links.is_empty() { None } else { Some(0) });
        self.page_links_list = links;
        self.show_page_links_modal = true;
    }

    pub fn close_page_links_modal(&mut self) {
        self.show_page_links_modal = false;
        self.page_links_list.clear();
        self.page_links_state.select(None);
    }

    pub fn next_page_link(&mut self) {
        let len = self.page_links_list.len();
        if len == 0 {
            return;
        }
        let i = match self.page_links_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        self.page_links_state.select(Some(i));
    }

    pub fn previous_page_link(&mut self) {
        let len = self.page_links_list.len();
        if len == 0 {
            return;
        }
        let i = match self.page_links_state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.page_links_state.select(Some(i));
    }

    pub fn open_selected_page_link(&mut self) {
        if let Some(i) = self.page_links_state.selected() {
            if let Some(link) = self.page_links_list.get(i) {
                let destination = link.destination.clone();
                crate::ui::modals::dashboard_menu::open_in_browser(&destination);
                self.log(format!("Opened URL in browser: {}", destination));
            }
        }
    }

    /// Kick off a background screenshot capture for `url`. Result is picked up by
    /// `check_screenshot_results` (polled from `on_tick`) and written to the log.
    fn spawn_screenshot(&mut self, url: String) {
        self.log(format!("Capturing screenshot of: {}", url));

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.screenshot_receiver = Some(rx);

        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || crate::app::menu_actions::capture_screenshot(&url))
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
            let _ = tx.send(result).await;
        });
    }

    /// Poll for a completed screenshot capture and log the outcome. Called every tick.
    pub fn check_screenshot_results(&mut self) {
        if let Some(ref mut rx) = self.screenshot_receiver {
            match rx.try_recv() {
                Ok(Ok(path)) => {
                    self.log(format!("Screenshot saved: {}", path));
                    self.screenshot_receiver = None;
                }
                Ok(Err(e)) => {
                    self.log(format!("Screenshot failed: {}", e));
                    self.screenshot_receiver = None;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    self.screenshot_receiver = None;
                }
            }
        }
    }

    /// Export the full Overview table to a timestamped CSV file.
    fn export_overview_csv(&mut self) {
        match crate::app::menu_actions::write_overview_csv(&self.table_data) {
            Ok(path) => self.log(format!("Exported {} rows to: {}", self.table_data.len(), path)),
            Err(e) => self.log(format!("Export failed: {}", e)),
        }
    }
}
