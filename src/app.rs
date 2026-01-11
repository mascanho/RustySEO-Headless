use std::sync::mpsc;

use crate::models::{App, AppSettings};
use crate::ui::modals::dashboard_menu;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Crawl,
    Connectors,
    Dashboard,
    Reports,
    Chat,
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
            bookmark_index: 0,
            bookmarks: vec![],
            bookmark_input: String::new(),
            bookmark_cursor: 0,
            table_data,
            page_data,
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
                    Ok(data) => results.push(data),
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
            ];
            self.table_data.push(row);
            self.log(format!("Crawled: {}", data.url));

            // Update overall progress
            let limit = self.settings.as_ref().map(|s| s.crawler.max_pages).unwrap_or(50) as f64;
            self.crawl_progress = (self.table_data.len() as f64 / limit).min(1.0);
        }

        if crawl_finished {
            self.is_crawling = false;
            self.crawl_receiver = None;
            self.crawl_progress = 1.0;
            self.log("SYSTEM - Crawl finished successfully.");
        }

        if self.input_url.is_empty() {
            return;
        }

        // Only do progress simulation if NOT actually crawling
        if !self.is_crawling && self.crawl_progress < 1.0 {
            self.crawl_progress += 0.005;
            if self.crawl_progress > 1.0 {
                self.crawl_progress = 0.0;
            }
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

    pub fn log<S: Into<String>>(&mut self, message: S) {
        self.logs_data.insert(0, message.into());
        if self.logs_data.len() > 100 {
            self.logs_data.pop();
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.table_data.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.table_data.is_empty() {
                        0
                    } else {
                        self.table_data.len() - 1
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
        if self.logs_data.is_empty() {
            return;
        }
        let i = match self.logs_state.selected() {
            Some(i) => {
                if i >= self.logs_data.len() - 1 {
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
        if self.logs_data.is_empty() {
            return;
        }
        let i = match self.logs_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.logs_data.len() - 1
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

        let _ = std::process::Command::new(cmd)
            .arg(path)
            .spawn();
        
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

    pub fn next_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::Connectors,
            AppState::Connectors => AppState::Crawl,
            AppState::Crawl => AppState::Reports,
            AppState::Reports => AppState::Chat,
            AppState::Chat => AppState::Dashboard,
        }
    }

    pub fn previous_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::Chat,
            AppState::Connectors => AppState::Dashboard,
            AppState::Crawl => AppState::Connectors,
            AppState::Reports => AppState::Crawl,
            AppState::Chat => AppState::Reports,
        }
    }

    pub fn get_state_index(&self) -> usize {
        match self.current_state {
            AppState::Dashboard => 0,
            AppState::Connectors => 1,
            AppState::Crawl => 2,
            AppState::Reports => 3,
            AppState::Chat => 4,
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

        self.page_data.clear();
        self.table_data.clear();
        self.table_state.select(None); // Reset table selection when data is cleared
        self.crawl_progress = 0.0;
        self.is_crawling = true;
        self.logs_data
            .insert(0, format!("Starting crawl for: {}", self.input_url));

        let (tx, rx) = mpsc::channel();
        self.crawl_receiver = Some(rx);
        let target_url = self.input_url.clone();
        let max_pages = self.settings.as_ref().map(|s| s.crawler.max_pages).unwrap_or(500);
        let concurrency = self.settings.as_ref().map(|s| s.crawler.concurrency).unwrap_or(10);

        tokio::task::spawn(async move {
            let engine = crate::crawler::CrawlEngine::new().await
                .with_max_pages(max_pages)
                .with_concurrency(concurrency);
            
            let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel(100);
            let engine_clone = engine.clone();
            let target_url_clone = target_url.clone();
            
            tokio::spawn(async move {
                engine_clone.crawl_concurrently(&target_url_clone, tokio_tx).await;
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
        if self.dashboard_menu_selection < 6 {
            self.dashboard_menu_selection += 1;
        }
    }

    pub fn previous_dashboard_menu_item(&mut self) {
        if self.dashboard_menu_selection > 0 {
            self.dashboard_menu_selection = self.dashboard_menu_selection.saturating_sub(1);
        }
    }

    pub fn execute_dashboard_menu_action(&mut self) {
        if self.show_dashboard_menu {
            dashboard_menu::handle_action(self, self.dashboard_menu_selection);
        }
    }
}
