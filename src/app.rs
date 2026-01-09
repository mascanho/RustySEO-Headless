use std::sync::mpsc;

use crate::models::{App, AppSettings};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Crawl,
    Logs,
    Connectors,
    Dashboard,
    Reports,
    Chat,
}

impl Default for App {
    fn default() -> Self {
        let table_data = Vec::new();
        let table_state = ratatui::widgets::TableState::default();

        Self {
            sidebar_visible: false,
            task_panel_visible: false,
            current_state: AppState::Crawl,
            sidebar_tab: 0,
            table_data,
            table_state,
            logs_data: vec!["System Initialized - Ready for Crawl".to_string()],
            logs_state: {
                let mut state = ratatui::widgets::ListState::default();
                state.select(Some(0));
                state
            },
            connectors_data: vec![],
            tab_rect: None,
            sidebar_tab_rect: None,
            keyword_rects: vec![],
            show_help: false,
            show_details: false,
            crawl_progress: 0.0,
            input: String::new(),
            input_mode: false,
            cursor_position: 0,
            detail_tab: 0,
            input_url: String::new(),
            crawl_receiver: None,
            is_crawling: false,
            settings: Some(AppSettings::default()),
        }
    }
}

impl App {
    pub fn on_tick(&mut self) {
        // Collect results from background crawler thread
        let mut finished = false;
        if let Some(ref rx) = self.crawl_receiver {
            loop {
                match rx.try_recv() {
                    Ok(data) => {
                        // ID, URL, Title, Title Len, H1, H1 Len, Meta Desc, Meta Len, Status
                        let row = vec![
                            data.id.to_string(),
                            data.url.clone(),
                            data.title.clone(),
                            data.title_len.to_string(),
                            data.h1.clone(),
                            data.h1_len.to_string(),
                            data.description.clone(),
                            data.description_len.to_string(),
                            data.status.clone(),
                        ];
                        self.table_data.push(row);
                        self.logs_data.insert(0, format!("Crawled: {}", data.url));
                        if self.logs_data.len() > 100 {
                            self.logs_data.pop();
                        }

                        // Smoothly update overall progress based on some limit (e.g. 50 pages)
                        self.crawl_progress = (self.table_data.len() as f64 / 50.0).min(1.0);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        finished = true;
                        break;
                    }
                }
            }
        }

        if finished {
            self.is_crawling = false;
            self.crawl_receiver = None;
            self.crawl_progress = 1.0;
            self.logs_data
                .insert(0, "SYSTEM - Crawl finished successfully.".to_string());
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
        if index < 4 {
            self.sidebar_tab = index;
            self.sidebar_visible = true;
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn reset(&mut self) {
        self.sidebar_visible = false;
        self.task_panel_visible = false;
        self.current_state = AppState::Crawl;
        self.sidebar_tab = 0;
    }

    pub fn next_sidebar_tab(&mut self) {
        self.sidebar_tab = (self.sidebar_tab + 1) % 4;
    }

    pub fn previous_sidebar_tab(&mut self) {
        self.sidebar_tab = if self.sidebar_tab == 0 {
            3
        } else {
            self.sidebar_tab - 1
        };
    }

    pub fn next_detail_tab(&mut self) {
        self.detail_tab = (self.detail_tab + 1) % 3;
    }

    pub fn previous_detail_tab(&mut self) {
        self.detail_tab = if self.detail_tab == 0 {
            2
        } else {
            self.detail_tab - 1
        };
    }

    pub fn next_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Crawl => AppState::Logs,
            AppState::Logs => AppState::Connectors,
            AppState::Connectors => AppState::Dashboard,
            AppState::Dashboard => AppState::Reports,
            AppState::Reports => AppState::Chat,
            AppState::Chat => AppState::Crawl,
        }
    }

    pub fn previous_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Crawl => AppState::Chat,
            AppState::Logs => AppState::Crawl,
            AppState::Connectors => AppState::Logs,
            AppState::Dashboard => AppState::Connectors,
            AppState::Reports => AppState::Dashboard,
            AppState::Chat => AppState::Reports,
        }
    }

    pub fn get_state_index(&self) -> usize {
        match self.current_state {
            AppState::Crawl => 0,
            AppState::Logs => 1,
            AppState::Connectors => 2,
            AppState::Dashboard => 3,
            AppState::Reports => 4,
            AppState::Chat => 5,
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

        self.table_data.clear();
        self.crawl_progress = 0.0;
        self.is_crawling = true;
        self.logs_data
            .insert(0, format!("Starting crawl for: {}", self.input_url));

        let (tx, rx) = mpsc::channel();
        self.crawl_receiver = Some(rx);
        let target_url = self.input_url.clone();

        std::thread::spawn(move || {
            let mut engine = crate::crawler::CrawlEngine::new();
            let results = engine.crawl(&target_url);
            for data in results {
                let _ = tx.send(data);
                // Slight delay to make TUI look "real-time" and not overwhelm
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
}
