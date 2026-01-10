use crate::{app::AppState, crawler::PageData, settings};

use std::sync::mpsc::{self, Receiver};

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub max_pages: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { max_pages: 50 }
    }
}

impl AppSettings {
    pub async fn new() -> Self {
        Self { max_pages: 50 }
    }
}

pub struct App {
    pub sidebar_visible: bool,
    pub task_panel_visible: bool,
    pub current_state: AppState,
    pub sidebar_tab: usize,
    pub bookmark_index: usize,
    pub bookmarks: Vec<String>,
    pub bookmark_input: String,
    pub bookmark_cursor: usize,
    pub table_data: Vec<Vec<String>>,
    pub table_state: ratatui::widgets::TableState,
    pub horizontal_scroll: usize,
    pub logs_data: Vec<String>,
    pub logs_state: ratatui::widgets::ListState,
    pub logs_horizontal_scroll: usize,
    pub connectors_data: Vec<(String, bool)>,
    pub tab_rect: Option<ratatui::layout::Rect>,
    pub sidebar_tab_rect: Option<ratatui::layout::Rect>,
    pub keyword_rects: Vec<(String, ratatui::layout::Rect)>,
    pub show_help: bool,
    pub show_details: bool,
    pub crawl_progress: f64,
    pub input: String,
    pub input_mode: bool,
    pub cursor_position: usize,
    pub detail_tab: usize,
    pub input_url: String,
    pub crawl_receiver: Option<Receiver<PageData>>,
    pub is_crawling: bool,
    pub settings: Option<AppSettings>,
    pub log_receiver: Option<Receiver<String>>,
}
