use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::sync::mpsc;

use crate::crawler::CrawlMessage;
use crate::models::{App, AppSettings};
use crate::ui::modals::dashboard_menu;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Dashboard,
    Crawl,
    Connectors,
    Redirects,
    Images,
    Css,
    Javascript,
    Keywords,
    CoreWebVitals,
    CustomSearch,
    Reports,
    Content,
}

impl Default for App {
    fn default() -> Self {
        let table_data = Vec::new();
        let page_data = Vec::new();
        let table_state = ratatui::widgets::TableState::default();

        Self {
            sidebar_visible: false,
            task_panel_visible: false,
            current_state: AppState::Dashboard,
            sidebar_tab: 0,
            bookmarks: vec![],
            bookmark_index: 0,
            bookmark_input: String::new(),
            bookmark_cursor: 0,
            bookmark_subview: 0, // 0=bookmarks, 1=last_crawled
            last_crawled_index: 0,
            table_data,
            page_data,
            total_pages: 0,
            table_state,
            horizontal_scroll: 0,
            logs_data: vec!["System Initialized - Ready for Crawl".to_string()],
            logs_state: {
                let mut state = ratatui::widgets::ListState::default();
                state.select(Some(0));
                state
            },
            logs_horizontal_scroll: 0,
            connectors_data: vec![],
            tab_rect: None,
            sidebar_tab_rect: None,
            keyword_rects: vec![],
            show_help: false,
            show_details: false,
            show_dashboard_menu: false,
            dashboard_menu_selection: 0,
            crawl_progress: 0.0,
            input: String::new(),
            input_mode: false,
            cursor_position: 0,
            detail_tab: 0,
            detail_scroll: 0,
            detail_horizontal_scroll: 0,
            detail_table_state: ratatui::widgets::TableState::default(),
            input_url: String::new(),
            crawl_receiver: None,
            is_crawling: false,
            settings: Some(AppSettings::default()),
            log_receiver: None,
            show_logs: false,
            logs_height: 18,
            show_ai_modal: false,
            ai_input: String::new(),
            ai_chat_history: vec![],
            ai_chat_state: ratatui::widgets::ListState::default(),
            show_search: false,
            search_query: String::new(),
            filtered_table_data: Vec::new(),
            full_filtered_table_data: Vec::new(),
            show_log_search: false,
            log_search_query: String::new(),
            filtered_logs_data: vec![],
            last_settings_mtime: None,
            page_size: 100,
            current_page: 0,
            last_search_time: None,
            last_log_search_time: None,
        }
    }
}

impl App {
    pub fn on_tick(&mut self) {
        // 1. Collect results from background crawler thread
        let mut results = Vec::new();
        let mut crawl_finished = false;
        if let Some(ref rx) = self.crawl_receiver {
            loop {
                match rx.try_recv() {
                    Ok(CrawlMessage::Page(data)) => results.push(data),
                    Ok(CrawlMessage::Progress {
                        scanned,
                        queued,
                        processing,
                    }) => {
                        let total = scanned + queued + processing;
                        self.crawl_progress = if total == 0 {
                            0.0
                        } else {
                            (scanned as f64 / total as f64).min(1.0)
                        };
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        crawl_finished = true;
                        break;
                    }
                }
            }
        }

        // 2. Process collected results
        for data in results {
            let current_id = self.page_data.len() + 1;
            let mut page_data = data.clone();
            page_data.id = current_id;
            self.page_data.push(page_data);

            let row = vec![
                current_id.to_string(),
                data.url.clone(),
                data.title.clone(),
                data.title_len.to_string(),
                data.h1.clone(),
                data.h1_len.to_string(),
                data.description.clone(),
                data.description_len.to_string(),
                data.h2.clone(),
                data.h2_len.to_string(),
                data.status.clone(),
                data.mobile.to_string(),
                data.language.to_string(),
                data.indexability.to_string(),
                data.anchor_links.len().to_string(),
                data.content_type.clone(),
                data.canonicals.len().to_string(),
                data.size.to_string(),
                data.word_count.unwrap_or(0).to_string(),
            ];
            self.table_data.push(row);
            self.log(format!("Crawled: {}", data.url));

            self.apply_filter();
        }

        if crawl_finished {
            self.is_crawling = false;
            self.crawl_receiver = None;
            self.crawl_progress = 1.0;
            self.log("SYSTEM - Crawl finished successfully.");
        }

        // Debounce search filtering
        if let Some(last_time) = self.last_search_time {
            if last_time.elapsed() > std::time::Duration::from_millis(300) {
                self.apply_filter();
                self.last_search_time = None;
            }
        }
        if let Some(last_time) = self.last_log_search_time {
            if last_time.elapsed() > std::time::Duration::from_millis(300) {
                self.apply_log_filter();
                self.last_log_search_time = None;
            }
        }

        if self.input_url.is_empty() {
            return;
        }

        // 3. Process logs from tracing
        let mut tracing_logs = Vec::new();
        if let Some(ref rx) = self.log_receiver {
            while let Ok(log) = rx.try_recv() {
                tracing_logs.push(log);
            }
        }

        for log in tracing_logs {
            self.log(log);
        }
    }

    pub fn next_page(&mut self) {
        let total_pages =
            (self.full_filtered_table_data.len() + self.page_size - 1) / self.page_size;
        if self.current_page + 1 < total_pages {
            self.current_page += 1;
            self.apply_pagination();
        }
    }

    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.apply_pagination();
        }
    }

    pub fn log<S: Into<String>>(&mut self, message: S) {
        let msg = message.into();
        // Check if it already has a timestamp [HH:MM:SS]
        let log_entry = if msg.starts_with('[')
            && msg.get(9..10) == Some("]")
            && msg.get(1..9).map(|s| s.contains(':')).unwrap_or(false)
        {
            msg
        } else {
            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
            format!("[{}] [SYSTEM] {}", timestamp, msg)
        };

        self.logs_data.insert(0, log_entry);
        if self.logs_data.len() > 100 {
            self.logs_data.pop();
        }
        self.apply_log_filter();
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_row(&mut self) {
        let len = self.filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    // Check if we can move to next page
                    let total_pages =
                        (self.full_filtered_table_data.len() + self.page_size - 1) / self.page_size;
                    if self.current_page + 1 < total_pages {
                        self.current_page += 1;
                        self.apply_pagination();
                        0 // Select first row of new page
                    } else {
                        len - 1 // Stay at last row of current page
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let len = self.filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    // Check if we can move to previous page
                    if self.current_page > 0 {
                        self.current_page -= 1;
                        self.apply_pagination();
                        self.filtered_table_data.len() - 1 // Select last row of new page
                    } else {
                        0 // Stay at first row of current page
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn next_log(&mut self) {
        let len = if self.log_search_query.is_empty() && !self.show_log_search {
            self.logs_data.len()
        } else {
            self.filtered_logs_data.len()
        };
        if len == 0 {
            return;
        }
        let i = match self.logs_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn previous_log(&mut self) {
        let len = if self.log_search_query.is_empty() && !self.show_log_search {
            self.logs_data.len()
        } else {
            self.filtered_logs_data.len()
        };
        if len == 0 {
            return;
        }
        let i = match self.logs_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn set_sidebar_tab(&mut self, index: usize) {
        if index < 5 {
            self.sidebar_tab = index;
            self.sidebar_visible = true;
        }
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

    pub fn submit_ai_message(&mut self) {
        if self.ai_input.trim().is_empty() {
            return;
        }

        let user_msg = crate::models::ChatLog {
            role: "user".to_string(),
            content: self.ai_input.clone(),
        };
        self.ai_chat_history.push(user_msg);
        let input = self.ai_input.clone();
        self.ai_input.clear();

        // Simulate AI thinking/response
        let response = if input.to_lowercase().contains("hi")
            || input.to_lowercase().contains("hello")
        {
            "Hello! I am your RustySEO AI assistant. How can I help you analyze your crawl today?"
                .to_string()
        } else if input.to_lowercase().contains("page") || input.to_lowercase().contains("url") {
            format!(
                "You have crawled {} pages so far. Would you like me to analyze the status codes or heading structures for you?",
                self.page_data.len()
            )
        } else {
            "I'm currently processing your request. In a real implementation, I would analyze your SEO data and provide actionable insights!".to_string()
        };

        self.ai_chat_history.push(crate::models::ChatLog {
            role: "assistant".to_string(),
            content: response.to_string(),
        });

        // Scroll to bottom
        if !self.ai_chat_history.is_empty() {
            self.ai_chat_state
                .select(Some(self.ai_chat_history.len() - 1));
        }
    }

    pub fn clear_ai_chat(&mut self) {
        self.ai_chat_history.clear();
        self.ai_chat_state.select(None);
    }

    pub fn increase_logs_height(&mut self) {
        if self.logs_height < 50 {
            self.logs_height += 2;
        }
    }

    pub fn decrease_logs_height(&mut self) {
        if self.logs_height > 5 {
            self.logs_height = self.logs_height.saturating_sub(2);
        }
    }

    pub fn open_settings_file(&mut self) {
        let path = crate::models::AppSettings::path();
        #[cfg(target_os = "macos")]
        let cmd = "open";
        #[cfg(not(target_os = "macos"))]
        let cmd = "xdg-open";

        let _ = std::process::Command::new(cmd).arg(path).spawn();

        self.log("Opening settings file...".to_string());
    }

    pub fn reset(&mut self) {
        self.sidebar_visible = false;
        self.task_panel_visible = false;
        self.current_state = AppState::Dashboard;
        self.sidebar_tab = 0;
    }

    pub fn next_sidebar_tab(&mut self) {
        self.sidebar_tab = (self.sidebar_tab + 1) % 5;
    }

    pub fn previous_sidebar_tab(&mut self) {
        self.sidebar_tab = if self.sidebar_tab == 0 {
            4
        } else {
            self.sidebar_tab - 1
        };
    }

    pub fn next_bookmark(&mut self) {
        if !self.bookmarks.is_empty() {
            self.bookmark_index = (self.bookmark_index + 1) % self.bookmarks.len();
        }
    }

    pub fn previous_bookmark(&mut self) {
        if !self.bookmarks.is_empty() {
            self.bookmark_index = if self.bookmark_index == 0 {
                self.bookmarks.len() - 1
            } else {
                self.bookmark_index - 1
            };
        }
    }

    pub fn remove_selected_bookmark(&mut self) {
        if !self.bookmarks.is_empty() && self.bookmark_index < self.bookmarks.len() {
            let url = self.bookmarks[self.bookmark_index].clone();
            crate::db::remove_bookmark(&url);
            self.bookmarks = crate::db::load_bookmarks();
            if self.bookmark_index >= self.bookmarks.len() && !self.bookmarks.is_empty() {
                self.bookmark_index = self.bookmarks.len() - 1;
            } else if self.bookmarks.is_empty() {
                self.bookmark_index = 0;
            }
        }
    }

    pub fn toggle_bookmark_subview(&mut self) {
        self.bookmark_subview = if self.bookmark_subview == 0 { 1 } else { 0 };
        self.last_crawled_index = 0;
    }

    pub fn next_last_crawled(&mut self) {
        let recent_urls = self.get_recent_crawled_urls();
        if !recent_urls.is_empty() {
            self.last_crawled_index = (self.last_crawled_index + 1) % recent_urls.len();
        }
    }

    pub fn previous_last_crawled(&mut self) {
        let recent_urls = self.get_recent_crawled_urls();
        if !recent_urls.is_empty() {
            self.last_crawled_index = if self.last_crawled_index == 0 {
                recent_urls.len() - 1
            } else {
                self.last_crawled_index - 1
            };
        }
    }

    pub fn get_recent_crawled_urls(&self) -> Vec<String> {
        self.table_data
            .iter()
            .rev()
            .take(10) // Show last 10 crawled URLs
            .filter_map(|row| {
                if row.len() > 1 {
                    Some(row[1].clone()) // URL is at index 1
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn next_detail_tab(&mut self) {
        self.detail_tab = (self.detail_tab + 1) % 9;
        self.detail_scroll = 0;
        self.detail_horizontal_scroll = 0;
        self.detail_table_state.select(Some(0));
    }

    pub fn previous_detail_tab(&mut self) {
        self.detail_tab = if self.detail_tab == 0 {
            8
        } else {
            self.detail_tab - 1
        };
        self.detail_scroll = 0;
        self.detail_horizontal_scroll = 0;
        self.detail_table_state.select(Some(0));
    }

    pub fn next_detail_row(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.detail_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.detail_table_state.select(Some(i));
    }

    pub fn previous_detail_row(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.detail_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.detail_table_state.select(Some(i));
    }

    pub fn get_current_detail_len(&self) -> usize {
        let selected_idx = self.table_state.selected().unwrap_or(0);
        let full_idx = self.current_page * self.page_size + selected_idx;
        if full_idx >= self.full_filtered_table_data.len() {
            return 0;
        }
        let row_data = &self.full_filtered_table_data[full_idx];
        let original_id = row_data[0].parse::<usize>().unwrap_or(1);

        // Try to find page data in memory first
        if let Some(page_data) = self.page_data.iter().find(|pd| pd.id == original_id) {
            match self.detail_tab {
                3 => page_data.anchor_links.len(),
                4 => page_data.outlinks.len(),
                5 => page_data.images.len(),
                8 => page_data.headings.len(),
                _ => 0,
            }
        } else if let Some(page_data) = crate::db::load_page_data(original_id) {
            // Fall back to database if not in memory
            match self.detail_tab {
                3 => page_data.anchor_links.len(),
                4 => page_data.outlinks.len(),
                5 => page_data.images.len(),
                8 => page_data.headings.len(),
                _ => 0,
            }
        } else {
            0
        }
    }

    pub fn next_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::Crawl,
            AppState::Crawl => AppState::Connectors,
            AppState::Connectors => AppState::Redirects,
            AppState::Redirects => AppState::Images,
            AppState::Images => AppState::Css,
            AppState::Css => AppState::Javascript,
            AppState::Javascript => AppState::Keywords,
            AppState::Keywords => AppState::CoreWebVitals,
            AppState::CoreWebVitals => AppState::CustomSearch,
            AppState::CustomSearch => AppState::Reports,
            AppState::Reports => AppState::Content,
            AppState::Content => AppState::Dashboard,
        }
    }

    pub fn previous_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::Content,
            AppState::Crawl => AppState::Dashboard,
            AppState::Connectors => AppState::Crawl,
            AppState::Redirects => AppState::Connectors,
            AppState::Images => AppState::Redirects,
            AppState::Css => AppState::Images,
            AppState::Javascript => AppState::Css,
            AppState::Keywords => AppState::Javascript,
            AppState::CoreWebVitals => AppState::Keywords,
            AppState::CustomSearch => AppState::CoreWebVitals,
            AppState::Reports => AppState::CustomSearch,
            AppState::Content => AppState::Reports,
        }
    }

    pub fn get_state_index(&self) -> usize {
        match self.current_state {
            AppState::Dashboard => 0,
            AppState::Crawl => 1,
            AppState::Connectors => 2,
            AppState::Redirects => 3,
            AppState::Images => 4,
            AppState::Css => 5,
            AppState::Javascript => 6,
            AppState::Keywords => 7,
            AppState::CoreWebVitals => 8,
            AppState::CustomSearch => 9,
            AppState::Reports => 10,
            AppState::Content => 11,
        }
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position != 0 {
            let from_left_to_cursor_index = self.cursor_position - 1;
            self.input.remove(from_left_to_cursor_index);
            self.move_cursor_left();
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    pub fn add_input(&mut self, new_input: String) {
        self.input = new_input;
    }

    pub fn start_crawl(&mut self) {
        if self.input_url.is_empty() {
            return;
        }

        // Reload settings before starting crawl to get latest values
        self.settings = Some(crate::models::AppSettings::load());

        self.page_data.clear();
        self.table_data.clear();
        self.table_state.select(None); // Reset table selection when data is cleared
        self.crawl_progress = 0.0;
        self.is_crawling = true;
        self.logs_data
            .insert(0, format!("Starting crawl for: {}", self.input_url));

        let (tx, rx): (mpsc::Sender<CrawlMessage>, mpsc::Receiver<CrawlMessage>) = mpsc::channel();
        self.crawl_receiver = Some(rx);
        let target_url = self.input_url.clone();
        let max_pages = self
            .settings
            .as_ref()
            .map(|s| s.crawler.max_pages)
            .unwrap_or(500);
        let concurrency = self
            .settings
            .as_ref()
            .map(|s| s.crawler.concurrency)
            .unwrap_or(10);
        let enable_javascript = self
            .settings
            .as_ref()
            .map(|s| s.crawler.enable_javascript)
            .unwrap_or(false);

        tokio::task::spawn(async move {
            // Save to recent crawls
            crate::settings::utils::create::add_recent_entry(target_url.clone()).await;

            let engine = crate::crawler::CrawlEngine::new()
                .await
                .with_max_pages(max_pages)
                .with_concurrency(concurrency)
                .with_javascript(enable_javascript);

            let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel(100);
            let engine_clone = engine.clone();
            let target_url_clone = target_url.clone();

            tokio::spawn(async move {
                engine_clone
                    .crawl_concurrently(&target_url_clone, tokio_tx)
                    .await;
            });

            while let Some(data) = tokio_rx.recv().await {
                // Bridge tokio channel to std mpsc channel for the TUI loop
                let _ = tx.send(data);
            }
        });
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    pub fn enter_bookmark_char(&mut self, new_char: char) {
        self.bookmark_input.insert(self.bookmark_cursor, new_char);
        self.move_bookmark_cursor_right();
    }

    pub fn delete_bookmark_char(&mut self) {
        if self.bookmark_cursor != 0 {
            let from_left_to_cursor_index = self.bookmark_cursor - 1;
            self.bookmark_input.remove(from_left_to_cursor_index);
            self.move_bookmark_cursor_left();
        }
    }

    pub fn move_bookmark_cursor_left(&mut self) {
        let cursor_moved_left = self.bookmark_cursor.saturating_sub(1);
        self.bookmark_cursor = self.clamp_bookmark_cursor(cursor_moved_left);
    }

    pub fn move_bookmark_cursor_right(&mut self) {
        let cursor_moved_right = self.bookmark_cursor.saturating_add(1);
        self.bookmark_cursor = self.clamp_bookmark_cursor(cursor_moved_right);
    }

    fn clamp_bookmark_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.bookmark_input.len())
    }

    pub fn scroll_left(&mut self) {
        if self.horizontal_scroll > 0 {
            self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
        }
    }

    pub fn scroll_right(&mut self, max_scroll: usize) {
        if self.horizontal_scroll < max_scroll {
            self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
        }
    }

    pub fn scroll_logs_left(&mut self) {
        if self.logs_horizontal_scroll > 0 {
            self.logs_horizontal_scroll = self.logs_horizontal_scroll.saturating_sub(1);
        }
    }

    pub fn scroll_logs_right(&mut self, max_scroll: usize) {
        if self.logs_horizontal_scroll < max_scroll {
            self.logs_horizontal_scroll = self.logs_horizontal_scroll.saturating_add(1);
        }
    }

    pub fn validate_table_state(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.table_data.len() {
                if self.table_data.is_empty() {
                    self.table_state.select(None);
                } else {
                    self.table_state.select(Some(self.table_data.len() - 1));
                }
            }
        }
    }

    pub fn next_dashboard_menu_item(&mut self) {
        // There are 7 items in the menu (0 to 6)
        if self.dashboard_menu_selection >= 6 {
            self.dashboard_menu_selection = 0;
        } else {
            self.dashboard_menu_selection += 1;
        }
    }

    pub fn previous_dashboard_menu_item(&mut self) {
        if self.dashboard_menu_selection == 0 {
            self.dashboard_menu_selection = 6;
        } else {
            self.dashboard_menu_selection = self.dashboard_menu_selection.saturating_sub(1);
        }
    }

    pub fn execute_dashboard_menu_action(&mut self) {
        if self.show_dashboard_menu {
            dashboard_menu::handle_action(self, self.dashboard_menu_selection);
        }
    }

    pub fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.full_filtered_table_data = self.table_data.clone();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for row in &self.table_data {
                let search_blob = format!("{} {} {}", row[1], row[2], row[6]);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.search_query) {
                    scored_data.push((score, row.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.full_filtered_table_data = scored_data.into_iter().map(|(_, row)| row).collect();
        }

        // Reset page if out of bounds after filtering
        let total_pages =
            (self.full_filtered_table_data.len() + self.page_size - 1) / self.page_size;
        if self.current_page >= total_pages {
            self.current_page = total_pages.saturating_sub(1);
        }

        // Apply pagination
        self.apply_pagination();
    }

    pub fn apply_log_filter(&mut self) {
        if self.log_search_query.is_empty() {
            self.filtered_logs_data = self.logs_data.clone();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut matches = Vec::new();
            for log in &self.logs_data {
                if let Some(score) = matcher.fuzzy_match(log, &self.log_search_query) {
                    matches.push((score, log.clone()));
                }
            }
            matches.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered_logs_data = matches.into_iter().map(|(_, log)| log).collect();
        }

        // Adjust selection if it's out of bounds
        let current_selected = self.logs_state.selected().unwrap_or(0);
        if current_selected >= self.filtered_logs_data.len() && !self.filtered_logs_data.is_empty()
        {
            self.logs_state
                .select(Some(self.filtered_logs_data.len().saturating_sub(1)));
        } else if self.filtered_logs_data.is_empty() {
            self.logs_state.select(None);
        }
    }

    pub fn apply_pagination(&mut self) {
        let start = self.current_page * self.page_size;
        let end = (start + self.page_size).min(self.full_filtered_table_data.len());
        self.filtered_table_data = self.full_filtered_table_data[start..end].to_vec();

        // Adjust selection
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_table_data.len() {
                if self.filtered_table_data.is_empty() {
                    self.table_state.select(None);
                } else {
                    self.table_state
                        .select(Some(self.filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn reload_settings_if_changed(&mut self) {
        let settings_path = AppSettings::path();

        // Get file modification time - cheap operation, no file read
        if let Ok(metadata) = std::fs::metadata(&settings_path) {
            if let Ok(mtime) = metadata.modified() {
                // Only reload if file was modified since last check
                if self
                    .last_settings_mtime
                    .map_or(true, |last_mtime| mtime > last_mtime)
                {
                    let current_settings = AppSettings::load();

                    // Only update if settings actually changed
                    let settings_changed = self.settings.as_ref().map_or(true, |stored| {
                        stored.crawler.max_pages != current_settings.crawler.max_pages
                            || stored.crawler.concurrency != current_settings.crawler.concurrency
                            || stored.crawler.stay_on_domain
                                != current_settings.crawler.stay_on_domain
                            || stored.ui.theme != current_settings.ui.theme
                            || stored.ui.refresh_rate_ms != current_settings.ui.refresh_rate_ms
                    });
                    if settings_changed {
                        self.settings = Some(current_settings);
                        self.log("Settings reloaded from file");
                    }

                    self.last_settings_mtime = Some(mtime);
                }
            }
        }
    }
}
