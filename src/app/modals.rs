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
                let issue_title = &self.issues_table_data[selected][0];
                self.current_issue_title = issue_title.clone();

                // Generate real URLs for the selected issue
                self.issue_urls_list = self.get_urls_for_issue(issue_title);

                // Reset the list state to select the first item
                self.issue_urls_state.select(Some(0));

                // Show the modal
                self.show_issue_urls_modal = true;

                self.log(format!("Showing URLs for issue: {}", issue_title));
            }
        }
    }

    pub fn close_issue_urls_modal(&mut self) {
        self.show_issue_urls_modal = false;
        self.issue_urls_list.clear();
        self.current_issue_title.clear();
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
                let url = &self.issue_urls_list[selected];
                crate::ui::modals::options::open_in_browser(url);
                self.log(format!("Opened URL in browser: {}", url));
            }
        }
    }

    pub fn copy_selected_issue_url(&mut self) {
        if let Some(selected) = self.issue_urls_state.selected() {
            if selected < self.issue_urls_list.len() {
                let url = self.issue_urls_list[selected].clone();
                crate::ui::modals::options::copy_to_clipboard(url.clone());
                self.log(format!("Copied URL to clipboard: {}", url));
            }
        }
    }

    pub fn show_js_pages_for_url(&mut self, js_url: String) {
        self.js_pages_list = self
            .page_data
            .iter()
            .filter(|p| {
                p.javascript.as_ref().map_or(false, |js| {
                    js.scripts.iter().any(|s| s.src.as_ref() == Some(&js_url))
                })
            })
            .map(|p| p.url.clone())
            .collect();
        self.js_pages_state.select(Some(0));
        self.show_js_pages_modal = true;
    }

    pub fn close_js_pages_modal(&mut self) {
        self.show_js_pages_modal = false;
        self.js_pages_list.clear();
        self.js_pages_state.select(None);
    }

    pub fn show_css_pages_for_url(&mut self, css_url: String) {
        self.css_pages_list = self
            .page_data
            .iter()
            .filter(|p| {
                p.css
                    .as_ref()
                    .map_or(false, |css| css.css_urls.contains(&css_url))
            })
            .map(|p| p.url.clone())
            .collect();
        self.css_pages_state.select(Some(0));
        self.show_css_pages_modal = true;
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

        match action {
            0 => {
                // Open in browser
                if let Some(selected) = self.table_state.selected() {
                    let full_idx = self.current_page * self.page_size + selected;
                    if let Some(row) = self.full_filtered_table_data.get(full_idx) {
                        crate::ui::modals::options::open_in_browser(&row[1]);
                    }
                }
            }
            1 => {
                // Copy to clipboard
                if let Some(selected) = self.table_state.selected() {
                    let full_idx = self.current_page * self.page_size + selected;
                    if let Some(row) = self.full_filtered_table_data.get(full_idx) {
                        crate::ui::modals::options::copy_to_clipboard(row[1].clone());
                    }
                }
            }
            2 => {
                // View Details
                self.show_details = true;
            }
            3 => {
                // Export
                // Add export logic here
            }
            _ => {}
        }
    }
}
