use crate::{app::AppState, crawler::PageData};

use std::sync::mpsc::Receiver;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub crawler: CrawlerConfig,
    pub ui: UiConfig,
    pub system: SystemConfig,
    pub connectors: ConnectorsConfig,
    pub provider: LLMprovider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerConfig {
    pub max_pages: usize,
    pub concurrency: usize,
    pub user_agent: String,
    pub stay_on_domain: bool,
    pub follow_redirects: bool,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub enable_javascript: bool,
    #[serde(default)]
    pub max_memory_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_logs_on_start: bool,
    pub sidebar_width_percentage: u16,
    pub refresh_rate_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub database_path: String,
    pub log_level: String,
    pub export_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorsConfig {
    pub pagespeed: PageSpeedConfig,
    pub search_console: SearchConsoleConfig,
    pub gemini: GeminiConfig,
    pub openai: OpenAiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSpeedConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsoleConfig {
    pub token: String,
    pub project_id: String,
    pub project_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMprovider {
    pub llm: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            crawler: CrawlerConfig {
                max_pages: 300,
                concurrency: 10,
                user_agent: "RustySEO/0.1.0".to_string(),
                stay_on_domain: true,
                follow_redirects: true,
                timeout_seconds: 15,
                enable_javascript: false,
                max_memory_pages: 1000,
            },
            ui: UiConfig {
                theme: "Oceanic".to_string(),
                show_logs_on_start: false,
                sidebar_width_percentage: 33,
                refresh_rate_ms: 100,
            },
            system: SystemConfig {
                database_path: "./rustyseo.db".to_string(),
                log_level: "info".to_string(),
                export_format: "csv".to_string(),
            },

            provider: LLMprovider {
                llm: "Not selected".to_string(),
            },
            connectors: ConnectorsConfig {
                pagespeed: PageSpeedConfig {
                    api_key: "".to_string(),
                },
                search_console: SearchConsoleConfig {
                    token: "".to_string(),
                    project_id: "".to_string(),
                    project_name: "".to_string(),
                },
                gemini: GeminiConfig {
                    api_key: "".to_string(),
                    model: "gemini-pro".to_string(),
                },
                openai: OpenAiConfig {
                    api_key: "".to_string(),
                    model: "gpt-4-turbo".to_string(),
                },
            },
        }
    }
}

impl AppSettings {
    pub fn path() -> std::path::PathBuf {
        let project_dirs = directories::ProjectDirs::from("", "", "rustyseo").unwrap();
        project_dirs.data_dir().join("cli-settings.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatLog {
    pub role: String,
    pub content: String,
}

pub struct App {
    pub sidebar_visible: bool,
    pub task_panel_visible: bool,
    pub current_state: AppState,
    pub sidebar_tab: usize,
    pub bookmarks: Vec<String>,
    pub bookmark_index: usize,
    pub bookmark_input: String,
    pub bookmark_cursor: usize,
    pub bookmark_subview: usize, // 0=bookmarks, 1=last_crawled
    pub last_crawled_index: usize,
    pub table_data: Vec<Vec<String>>,
    pub page_data: Vec<crate::crawler::PageData>,
    pub total_pages: usize,
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
    pub show_dashboard_menu: bool,
    pub dashboard_menu_selection: usize,
    pub crawl_progress: f64,
    pub input: String,
    pub input_mode: bool,
    pub cursor_position: usize,
    pub detail_tab: usize,
    pub detail_scroll: u16,
    pub detail_horizontal_scroll: usize,
    pub detail_table_state: ratatui::widgets::TableState,
    pub input_url: String,
    pub crawl_receiver: Option<Receiver<PageData>>,
    pub is_crawling: bool,
    pub settings: Option<AppSettings>,
    pub log_receiver: Option<Receiver<String>>,
    pub show_logs: bool,
    pub logs_height: u16,
    pub show_ai_modal: bool,
    pub ai_input: String,
    pub ai_chat_history: Vec<ChatLog>,
    pub ai_chat_state: ratatui::widgets::ListState,
    pub show_search: bool,
    pub search_query: String,
    pub filtered_table_data: Vec<Vec<String>>,
    pub full_filtered_table_data: Vec<Vec<String>>,
    pub show_log_search: bool,
    pub log_search_query: String,
    pub filtered_logs_data: Vec<String>,
    pub last_settings_mtime: Option<std::time::SystemTime>,
    pub page_size: usize,
    pub current_page: usize,
}
