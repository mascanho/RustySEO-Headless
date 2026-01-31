use crate::models::App;

impl App {
    pub fn toggle_options_modal(&mut self) {
        self.options_modal = !self.options_modal;
    }

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

                crate::ui::modals::options::open_in_browser(url);
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
                    
                crate::ui::modals::options::copy_to_clipboard(url.to_string());
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
        self.dashboard_menu_selection = (self.dashboard_menu_selection + 1) % 4;
    }

    pub fn previous_dashboard_menu_item(&mut self) {
        self.dashboard_menu_selection = if self.dashboard_menu_selection == 0 {
            3
        } else {
            self.dashboard_menu_selection - 1
        };
    }

    pub fn execute_dashboard_menu_action(&mut self) {
        let action = self.dashboard_menu_selection;
        self.show_dashboard_menu = false;

        // Get the selected row URL
        let url = if let Some(selected) = self.table_state.selected() {
            let full_idx = self.current_page * self.page_size + selected;
            if let Some(row) = self.full_filtered_table_data.get(full_idx) {
                row[1].clone()
            } else {
                return;
            }
        } else {
            return;
        };

        match action {
            0 => {
                // Copy URL
                crate::ui::modals::options::copy_to_clipboard(url.clone());
                self.log(format!("Copied URL to clipboard: {}", url));
            }
            1 => {
                // Open URL in Browser
                crate::ui::modals::options::open_in_browser(&url);
                self.log(format!("Opened URL in browser: {}", url));
            }
            2 => {
                // Open in Google
                let google_url = format!("https://www.google.com/search?q={}", url);
                crate::ui::modals::options::open_in_browser(&google_url);
                self.log(format!("Opening URL in Google: {}", url));
            }
            3 => {
                // Check Keywords
                self.log(format!("Keywords check for: {}", url));
            }
            4 => {
                // View SEO Score
                self.log(format!("SEO Score for: {}", url));
            }
            5 => {
                // Extract Links
                self.log(format!("Extracting links from: {}", url));
            }
            6 => {
                // Screenshot
                self.log(format!("Taking screenshot of: {}", url));
            }
            7 => {
                // Export Data
                self.log(format!("Exporting data for: {}", url));
            }
            _ => {}
        }
    }
}
