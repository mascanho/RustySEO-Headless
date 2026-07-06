use crate::app::AppState;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Payload sent back from the background robots/sitemaps fetch task.
pub struct RobotsResult {
    pub disallowed_urls: Vec<String>,
    pub raw_content: String,
    pub sitemap_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    pub crawler: CrawlerConfig,
    pub ui: UiConfig,
    pub system: SystemConfig,
    pub connectors: ConnectorsConfig,
    pub provider: LLMprovider,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(default)]
    pub extractor: bool,
    #[serde(default)]
    pub extractor_text: String,
    #[serde(default)]
    pub extractor_type: String,
    #[serde(default)]
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    pub theme: String,
    pub show_logs_on_start: bool,
    pub sidebar_width_percentage: u16,
    pub refresh_rate_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemConfig {
    pub database_path: String,
    pub log_level: String,
    pub export_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectorsConfig {
    pub pagespeed: PageSpeedConfig,
    pub search_console: SearchConsoleConfig,
    pub gemini: GeminiConfig,
    pub openai: OpenAiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageSpeedConfig {
    pub api_key: String,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchConsoleConfig {
    pub token: String,
    pub project_id: String,
    pub project_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
                extractor: false,
                extractor_type: "".to_string(),
                extractor_text: "".to_string(),
                batch_size: 50,
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
                    status: false,
                },
                search_console: SearchConsoleConfig {
                    token: "".to_string(),
                    project_id: "".to_string(),
                    project_name: "".to_string(),
                },
                gemini: GeminiConfig {
                    api_key: "".to_string(),
                    model: "gemini-pro".to_string(),
                    status: false,
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

#[derive(Debug, Clone)]
pub struct InternalLink {
    pub id: usize,
    pub source: String,
    pub destination: String,
    pub anchor: String,
    pub rel: String,
}

#[derive(Debug, Clone)]
pub struct ExternalLink {
    pub id: usize,
    pub source: String,
    pub destination: String,
    pub anchor: String,
    pub rel: String,
}

#[derive(Debug, Clone)]
pub struct CssUrl {
    pub id: usize,
    pub url: String,
    pub page_count: usize, // Number of pages that reference this CSS URL
}

#[derive(Debug, Clone)]
pub struct JsUrl {
    pub id: usize,
    pub url: String,
    pub script_type: String,
    pub is_async: bool,
    pub is_defer: bool,
    pub page_count: usize,
}

/// Entry for the Custom Search extraction results table
#[derive(Debug, Clone)]
pub struct ExtractionTableEntry {
    pub id: usize,
    pub url: String,
    pub element: String,
    pub snippet: String,
}

/// Entry for the Images results table
#[derive(Debug, Clone)]
pub struct ImageTableEntry {
    pub id: usize,
    pub url: String,
    pub alt: String,
    pub status: String,
    pub size: String,
    pub page_count: usize,
}

/// Entry for the Files results table
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: usize,
    pub url: String,
    pub filetype: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedirectHop {
    pub url: String,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedirectEntry {
    pub id: usize,
    pub initial_url: String,
    pub status_code: u16,
    pub chain: Vec<RedirectHop>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RobotsEntry {
    pub id: usize,
    pub url: String,
    pub blocked_urls: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PageSummary {
    pub id: usize,
    pub url: String,
    pub title: String,
    pub title_len: usize,
    pub description: String,
    pub description_len: usize,
    pub status: String,
    pub h1_len: usize,
    pub h1_count: usize,
    pub h2_count: usize,
    pub h3_count: usize,
    pub h4_count: usize,
    pub h5_count: usize,
    pub h6_count: usize,
    pub has_schema: bool,
    pub schema_count: usize,
    pub size: usize,
    pub word_count: usize,
    pub internal_link_count: usize,
    pub external_link_count: usize,
    pub images_count: usize,
    pub images_missing_alt: usize,
    pub is_canonical: bool,
    pub has_png_jpg: bool,
    pub mobile: bool,
    pub indexability: String,
    pub language: String,
    pub cwv_performance_desktop: Option<f64>,
    pub cwv_performance_mobile: Option<f64>,
    pub has_generic_anchors: bool,
}

pub struct App {
    pub options_modal: bool,
    pub sidebar_visible: bool,
    pub task_panel_visible: bool,
    pub current_state: AppState,
    pub sidebar_tab: usize,
    pub sidebar_scroll: usize,
    pub bookmarks: Vec<String>,
    pub bookmark_index: usize,
    pub bookmark_input: String,
    pub bookmark_cursor: usize,
    pub bookmark_subview: usize, // 0=bookmarks, 1=last_crawled
    pub bookmarks_state: ratatui::widgets::ListState,
    pub last_crawled_index: usize,
    pub table_data: Vec<Vec<String>>,
    pub page_summaries: Vec<PageSummary>,
    pub selected_page_details: Option<crate::crawler::PageData>,
    pub total_pages: usize,
    pub table_state: ratatui::widgets::TableState,
    pub horizontal_scroll: usize,
    pub logs_data: Vec<String>,
    pub logs_state: ratatui::widgets::ListState,
    pub logs_horizontal_scroll: usize,
    pub connectors_data: Vec<(String, bool)>,
    pub tab_rect: Option<ratatui::layout::Rect>,
    pub table_rect: Option<ratatui::layout::Rect>,
    pub sidebar_tab_rect: Option<ratatui::layout::Rect>,
    pub keyword_rects: Vec<(String, ratatui::layout::Rect)>,
    pub show_help: bool,
    pub show_details: bool,
    pub show_dashboard_menu: bool,
    pub dashboard_menu_selection: usize,
    pub crawl_progress: f64,
    pub queued_urls: usize,
    pub input: String,
    pub input_mode: bool,
    pub cursor_position: usize,
    pub detail_tab: usize,
    pub detail_scroll: u16,
    pub detail_horizontal_scroll: usize,
    pub detail_table_state: ratatui::widgets::TableState,
    pub input_url: String,
    pub crawl_receiver: Option<tokio::sync::mpsc::Receiver<crate::crawler::CrawlMessage>>,
    pub is_crawling: bool,
    pub settings: Option<AppSettings>,
    pub settings_receiver: Option<std::sync::mpsc::Receiver<()>>,
    pub log_receiver: Option<std::sync::mpsc::Receiver<String>>,
    pub show_logs: bool,
    pub logs_height: u16,
    pub show_ai_modal: bool,
    pub ai_input: String,
    pub ai_chat_history: Vec<ChatLog>,
    pub ai_chat_state: ratatui::widgets::ListState,
    pub ai_chat_scroll: usize,
    pub ai_chat_auto_scroll: bool,
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
    pub last_search_time: Option<std::time::Instant>,
    pub last_log_search_time: Option<std::time::Instant>,
    pub recent_crawls: Result<Vec<String>, Box<dyn std::error::Error>>,
    // Internal Links Tab State
    pub internal_table_data: Vec<InternalLink>,
    pub internal_table_state: ratatui::widgets::TableState,
    pub internal_filtered_table_data: Vec<InternalLink>,
    pub internal_full_filtered_table_data: Vec<InternalLink>,

    // External Links Tab State
    pub external_table_data: Vec<ExternalLink>,
    pub external_table_state: ratatui::widgets::TableState,
    pub external_filtered_table_data: Vec<ExternalLink>,
    pub external_full_filtered_table_data: Vec<ExternalLink>,
    pub external_current_page: usize,
    pub external_page_size: usize,
    pub external_horizontal_scroll: usize,
    pub external_search_query: String,
    pub show_external_search: bool,

    // CSS URLs Tab State
    pub css_urls_table_data: Vec<CssUrl>,
    pub css_urls_table_state: ratatui::widgets::TableState,
    pub css_urls_filtered_table_data: Vec<CssUrl>,
    pub css_urls_full_filtered_table_data: Vec<CssUrl>,
    pub css_urls_current_page: usize,
    pub css_urls_page_size: usize,
    pub css_urls_horizontal_scroll: usize,
    pub css_urls_search_query: String,
    pub show_css_urls_search: bool,
    pub internal_current_page: usize,
    pub internal_page_size: usize,
    pub internal_horizontal_scroll: usize,
    pub internal_search_query: String,
    pub show_internal_search: bool,
    pub url_to_status: HashMap<String, String>,
    // Javascript URLs Tab State
    pub js_urls_table_data: Vec<JsUrl>,
    pub js_urls_table_state: ratatui::widgets::TableState,
    pub js_urls_filtered_table_data: Vec<JsUrl>,
    pub js_urls_full_filtered_table_data: Vec<JsUrl>,
    pub js_urls_current_page: usize,
    pub js_urls_page_size: usize,
    pub js_urls_horizontal_scroll: usize,
    pub js_urls_search_query: String,
    pub show_js_urls_search: bool,
    // Content Tab State
    pub content_table_state: ratatui::widgets::TableState,
    pub content_filtered_table_data: Vec<Vec<String>>,
    pub content_full_filtered_table_data: Vec<Vec<String>>,
    pub content_current_page: usize,
    pub content_page_size: usize,
    pub content_horizontal_scroll: usize,
    pub content_search_query: String,
    pub show_content_search: bool,
    // Javascript Pages Modal State
    pub show_js_pages_modal: bool,
    pub js_pages_list: Vec<String>,
    pub js_pages_state: ratatui::widgets::ListState,
    // CSS Pages Modal State
    pub show_css_pages_modal: bool,
    pub css_pages_list: Vec<String>,
    pub css_pages_state: ratatui::widgets::ListState,
    // Custom Search/Extractor Tab State
    pub extractor_table_data: Vec<ExtractionTableEntry>,
    pub extractor_table_state: ratatui::widgets::TableState,
    pub extractor_filtered_table_data: Vec<ExtractionTableEntry>,
    pub extractor_full_filtered_table_data: Vec<ExtractionTableEntry>,
    pub extractor_current_page: usize,
    pub extractor_page_size: usize,
    pub extractor_horizontal_scroll: usize,
    pub extractor_search_query: String,
    pub show_extractor_search: bool,
    // Images Tab State
    pub images_table_data: Vec<ImageTableEntry>,
    pub images_table_state: ratatui::widgets::TableState,
    pub images_filtered_table_data: Vec<ImageTableEntry>,
    pub images_full_filtered_table_data: Vec<ImageTableEntry>,
    pub images_current_page: usize,
    pub images_page_size: usize,
    pub images_horizontal_scroll: usize,
    pub images_search_query: String,
    pub show_images_search: bool,
    // Tree View State
    pub tree_view_state: ratatui::widgets::ListState,
    pub tree_view_selected_index: usize,
    pub tree_view_expanded_nodes: std::collections::HashSet<String>,
    // Issues Tab State
    pub issues_table_data: Vec<Vec<String>>,
    pub issues_table_state: ratatui::widgets::TableState,
    pub issues_current_page: usize,
    pub issues_page_size: usize,
    // Issues URLs Modal State
    pub show_issue_urls_modal: bool,
    pub issue_urls_list: Vec<String>,
    pub issue_urls_state: ratatui::widgets::ListState,
    pub current_issue_title: String,
    pub robots_urls_loading: bool,
    pub robots_disallowed_urls: Vec<String>,
    pub robots_txt_content: String,
    pub sitemap_urls: Vec<String>,
    pub robots_receiver: Option<tokio::sync::mpsc::Receiver<RobotsResult>>,
    // Files Tab State
    pub files_table_data: Vec<FileEntry>,
    pub files_table_state: ratatui::widgets::TableState,
    pub files_filtered_table_data: Vec<FileEntry>,
    pub files_full_filtered_table_data: Vec<FileEntry>,
    pub files_current_page: usize,
    pub files_page_size: usize,
    pub files_search_query: String,
    pub show_files_search: bool,
    // Redirects Tab State
    pub redirects_table_data: Vec<RedirectEntry>,
    pub redirects_table_state: ratatui::widgets::TableState,
    pub redirects_filtered_table_data: Vec<RedirectEntry>,
    pub redirects_full_filtered_table_data: Vec<RedirectEntry>,
    pub redirects_current_page: usize,
    pub redirects_page_size: usize,
    pub redirects_horizontal_scroll: usize,
    pub redirects_search_query: String,
    pub show_redirects_search: bool,
    // Robots Tab State
    pub robots_table_data: Vec<RobotsEntry>,
    pub robots_table_state: ratatui::widgets::TableState,
    pub robots_filtered_table_data: Vec<RobotsEntry>,
    pub robots_full_filtered_table_data: Vec<RobotsEntry>,
    pub robots_current_page: usize,
    pub robots_page_size: usize,
    pub robots_horizontal_scroll: usize,
    pub robots_search_query: String,
    pub show_robots_search: bool,
    // Add Sets for O(1) membership checks during large crawls
    pub seen_files: std::collections::HashSet<String>,
    pub seen_css: std::collections::HashSet<String>,
    pub seen_js: std::collections::HashSet<String>,
    pub seen_images: std::collections::HashSet<String>,
    // Persistent Database Connection
    pub db_conn: Option<rusqlite::Connection>,
    // Faster lookups for aggregate tables during crawl
    pub css_counts: HashMap<String, usize>,
    pub js_counts: HashMap<String, usize>,
    pub image_counts: HashMap<String, usize>,

    // Link Score (Crawl Analysis)
    /// Maps a requested (pre-redirect) URL to the final URL it resolves to.
    pub redirect_map: HashMap<String, String>,
    /// Maps a URL to the (different) URL its canonical tag points to.
    pub canonical_map: HashMap<String, String>,
    /// Final Link Score (1-100) per eligible URL, populated by Crawl Analysis.
    pub link_scores: HashMap<String, u32>,
}
