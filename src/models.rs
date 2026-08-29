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
    #[serde(default)]
    pub check_external_links: bool,
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
    #[serde(default)]
    pub ga4: Ga4Config,
    #[serde(default)]
    pub gbp: GbpConfig,
    #[serde(default)]
    pub clarity: ClarityConfig,
    #[serde(default)]
    pub bing: BingWebmasterConfig,
    #[serde(default)]
    pub ahrefs: AhrefsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageSpeedConfig {
    pub api_key: String,
    pub status: bool,
}

/// OAuth2 access/refresh tokens for a connected Google product (Search
/// Console, GA4, Business Profile). All three run the same "installed app"
/// authorization-code flow via `crate::connectors::google_oauth`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GoogleOAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix timestamp (seconds) the access token expires at. 0 = never fetched.
    pub expires_at: i64,
}

impl GoogleOAuthTokens {
    pub fn is_connected(&self) -> bool {
        !self.refresh_token.is_empty()
    }
}

/// A user-registered Google Cloud OAuth "Desktop app" client. Google requires
/// each developer to register their own client (Cloud Console > APIs &
/// Credentials) - there is no shared client this CLI can embed - so these are
/// pasted into `cli-settings.toml` by the user, same convention as the
/// existing PageSpeed/Gemini API keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SearchConsoleConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    /// The exact property as it appears in Search Console (e.g.
    /// `sc-domain:example.com` or `https://example.com/`).
    #[serde(default)]
    pub site_url: String,
    #[serde(default)]
    pub tokens: GoogleOAuthTokens,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Ga4Config {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    /// Numeric GA4 property ID (Admin > Property Settings), without the
    /// "properties/" prefix.
    #[serde(default)]
    pub property_id: String,
    #[serde(default)]
    pub tokens: GoogleOAuthTokens,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GbpConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    /// Business Profile account resource id (Account Management API), e.g.
    /// `106234...`, without the "accounts/" prefix.
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub tokens: GoogleOAuthTokens,
}

/// Microsoft Clarity Data Export API token, generated per-project in the
/// Clarity dashboard under Settings > Data Export.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ClarityConfig {
    #[serde(default)]
    pub api_token: String,
}

/// Bing Webmaster Tools API key, from the Webmaster Tools UI under Settings >
/// API Access.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BingWebmasterConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub site_url: String,
}

/// Ahrefs API v3 token (requires a paid Ahrefs plan with API access).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AhrefsConfig {
    #[serde(default)]
    pub api_token: String,
    /// Domain or URL to report on, e.g. `example.com`.
    #[serde(default)]
    pub target: String,
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
                check_external_links: false,
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
                search_console: SearchConsoleConfig::default(),
                gemini: GeminiConfig {
                    api_key: "".to_string(),
                    model: "gemini-pro".to_string(),
                    status: false,
                },
                openai: OpenAiConfig {
                    api_key: "".to_string(),
                    model: "gpt-4-turbo".to_string(),
                },
                ga4: Ga4Config::default(),
                gbp: GbpConfig::default(),
                clarity: ClarityConfig::default(),
                bing: BingWebmasterConfig::default(),
                ahrefs: AhrefsConfig::default(),
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

    /// Persists the full settings file, e.g. after a Data & Insights connector
    /// finishes an OAuth exchange and needs to save its refresh token.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
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

/// A single on-page SEO factor as shown in the "View SEO Score" action.
#[derive(Debug, Clone)]
pub struct SeoScoreFactor {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

/// Composite on-page SEO score for a single crawled page, shown by the
/// Actions Menu "View SEO Score" action.
#[derive(Debug, Clone)]
pub struct SeoScoreBreakdown {
    pub url: String,
    pub score: u32,
    pub factors: Vec<SeoScoreFactor>,
}

/// A link found on a specific page, shown by the Actions Menu "Extract Links" action.
#[derive(Debug, Clone)]
pub struct PageLinkEntry {
    pub destination: String,
    pub anchor: String,
    pub rel: String,
    pub is_internal: bool,
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

/// Generic holder for a Data & Insights connector's last fetch: the data (if
/// any), the last error (if any), and whether a fetch is currently running.
/// Shared by all seven connector tabs so `App` doesn't need three near-
/// identical fields repeated per connector.
#[derive(Debug, Clone)]
pub struct ConnectorState<T> {
    pub data: Option<T>,
    pub error: Option<String>,
    pub loading: bool,
}

impl<T> Default for ConnectorState<T> {
    fn default() -> Self {
        Self {
            data: None,
            error: None,
            loading: false,
        }
    }
}

/// One row of a Google Search Console Search Analytics response.
#[derive(Debug, Clone)]
pub struct GscRow {
    /// Dimension values in the order requested (e.g. just `[query]`).
    pub keys: Vec<String>,
    pub clicks: f64,
    pub impressions: f64,
    pub ctr: f64,
    pub position: f64,
}

#[derive(Debug, Clone, Default)]
pub struct GscReport {
    pub rows: Vec<GscRow>,
}

/// A GA4 Data API `runReport` response, kept in its native
/// dimensions/metrics-header + row shape rather than modeled per-metric,
/// since the requested dimensions/metrics (and therefore the shape) vary.
#[derive(Debug, Clone, Default)]
pub struct Ga4Report {
    pub dimension_headers: Vec<String>,
    pub metric_headers: Vec<String>,
    pub rows: Vec<(Vec<String>, Vec<String>)>,
}

/// Microsoft Clarity's Data Export API returns a list of metric blocks whose
/// inner shape differs per metric (Traffic vs EngagementTime vs ScrollDepth,
/// etc.), so this keeps the raw JSON rather than modeling every variant.
#[derive(Debug, Clone, Default)]
pub struct ClarityInsights {
    pub metrics: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct BingQueryRow {
    pub query: String,
    pub clicks: u64,
    pub impressions: u64,
    pub avg_click_position: f64,
    pub avg_impression_position: f64,
}

#[derive(Debug, Clone, Default)]
pub struct BingQueryStats {
    pub rows: Vec<BingQueryRow>,
}

#[derive(Debug, Clone)]
pub struct GbpLocation {
    pub title: String,
    pub address: String,
    pub phone: String,
    pub website: String,
}

#[derive(Debug, Clone, Default)]
pub struct GbpLocations {
    pub locations: Vec<GbpLocation>,
}

#[derive(Debug, Clone, Default)]
pub struct AhrefsReport {
    pub domain_rating: f64,
    pub ahrefs_rank: u64,
    pub backlinks: u64,
    pub referring_domains: u64,
}

/// Word n-grams (contiguous phrases of 1-4 words) extracted from a page's
/// visible body text. Each list is sorted by frequency descending and capped
/// to the top 15 - the same bounded-output shape as the existing single-word
/// `PageData::keywords`, so per-page cost and memory stay in that same order.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NgramData {
    pub unigrams: Vec<(String, usize)>,
    pub bigrams: Vec<(String, usize)>,
    pub trigrams: Vec<(String, usize)>,
    pub quadgrams: Vec<(String, usize)>,
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
    pub h1: String,
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
    /// True if an `X-Robots-Tag` response header contains `noindex` - a page can be
    /// blocked from indexing this way even when its meta robots tag looks fine.
    pub has_noindex_header: bool,
    /// Normalized href of the page's `rel="canonical"` tag, if present and it
    /// points somewhere other than the page's own URL.
    pub canonical_target: Option<String>,
    /// Number of `rel="canonical"` tags found (should be 0 or 1; >1 is a bug).
    pub canonical_count: usize,
    /// True if an HTTPS page loads an image, stylesheet, or script over plain HTTP.
    pub has_mixed_content: bool,
    /// Top 1/2/3/4-word phrase frequencies from the page's body text.
    pub ngrams: NgramData,
    /// 64-bit SimHash of the page's body text - compare via
    /// `crawler::helpers::simhash::hamming_distance` to find near-duplicates.
    pub content_fingerprint: u64,
}

/// A pair of crawled pages (by `PageSummary::id`) whose content SimHash
/// fingerprints are within `simhash::NEAR_DUPLICATE_THRESHOLD` bits of each
/// other. `distance == 0` means byte-identical body text.
#[derive(Debug, Clone)]
pub struct DuplicatePair {
    pub id_a: usize,
    pub id_b: usize,
    pub distance: u32,
}

pub struct App {
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
    pub tab_hitboxes: Vec<ratatui::layout::Rect>,
    pub table_rect: Option<ratatui::layout::Rect>,
    pub sidebar_tab_rect: Option<ratatui::layout::Rect>,
    pub keyword_rects: Vec<(String, ratatui::layout::Rect)>,
    pub show_help: bool,
    pub show_details: bool,
    pub show_dashboard_menu: bool,
    pub dashboard_menu_selection: usize,
    // SEO Score Modal State (Actions Menu -> View SEO Score)
    pub show_seo_score_modal: bool,
    pub seo_score_data: Option<SeoScoreBreakdown>,
    // Page Links Modal State (Actions Menu -> Extract Links)
    pub show_page_links_modal: bool,
    pub page_links_list: Vec<PageLinkEntry>,
    pub page_links_state: ratatui::widgets::ListState,
    // Screenshot capture (Actions Menu -> Screenshot), resolved in on_tick
    pub screenshot_receiver: Option<tokio::sync::mpsc::Receiver<Result<String, String>>>,
    // "Complete" modal shown once a background/long-running Actions Menu task
    // (Screenshot, Export Data) finishes.
    pub show_action_result_modal: bool,
    pub action_result_title: String,
    pub action_result_message: String,
    pub action_result_success: bool,
    pub crawl_progress: f64,
    pub queued_urls: usize,
    pub input: String,
    pub input_mode: bool,
    pub cursor_position: usize,
    pub detail_tab: usize,
    pub detail_scroll: u16,
    pub detail_horizontal_scroll: usize,
    pub detail_table_state: ratatui::widgets::TableState,
    // Overview tab docked sub-table (Screaming Frog / RustySEO style lower pane).
    // Mirrors the Page Details modal for the currently highlighted row.
    pub show_overview_subtable: bool,
    pub overview_subtable_height: u16,
    pub overview_subtable_rect: Option<ratatui::layout::Rect>,
    pub overview_subtable_last_key: Option<(usize, usize)>,
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
    pub external_status_receiver: Option<tokio::sync::mpsc::Receiver<(String, String)>>,
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
    /// Near/exact-duplicate content pairs, found incrementally as each page
    /// is crawled (see `App::detect_duplicate_content` in `app::actions`).
    pub duplicate_pairs: Vec<DuplicatePair>,

    // ---- Data & Insights tab (GSC/GA4/Clarity/Bing/PageSpeed/GBP/Ahrefs) ----
    /// Index of the selected connector sub-tab: 0=GSC 1=GA4 2=Clarity 3=Bing
    /// 4=PageSpeed 5=GBP 6=Ahrefs.
    pub data_insights_tab: usize,

    pub gsc_state: ConnectorState<GscReport>,
    pub ga4_state: ConnectorState<Ga4Report>,
    pub clarity_state: ConnectorState<ClarityInsights>,
    pub bing_state: ConnectorState<BingQueryStats>,
    pub pagespeed_insights_state: ConnectorState<crate::crawler::helpers::html_parser::CwvData>,
    pub gbp_state: ConnectorState<GbpLocations>,
    pub ahrefs_state: ConnectorState<AhrefsReport>,

    // GSC/GA4/GBP fetches also refresh the OAuth access token when it's
    // stale, so their results carry the (possibly updated) tokens back for
    // App to persist - the alternative, refreshing synchronously before
    // spawning, would block the render loop on a network round trip.
    pub gsc_receiver:
        Option<tokio::sync::mpsc::Receiver<Result<(GscReport, GoogleOAuthTokens), String>>>,
    pub ga4_receiver:
        Option<tokio::sync::mpsc::Receiver<Result<(Ga4Report, GoogleOAuthTokens), String>>>,
    pub clarity_receiver: Option<tokio::sync::mpsc::Receiver<Result<ClarityInsights, String>>>,
    pub bing_receiver: Option<tokio::sync::mpsc::Receiver<Result<BingQueryStats, String>>>,
    pub pagespeed_insights_receiver: Option<
        tokio::sync::mpsc::Receiver<
            Result<crate::crawler::helpers::html_parser::CwvData, String>,
        >,
    >,
    pub gbp_receiver:
        Option<tokio::sync::mpsc::Receiver<Result<(GbpLocations, GoogleOAuthTokens), String>>>,
    pub ahrefs_receiver: Option<tokio::sync::mpsc::Receiver<Result<AhrefsReport, String>>>,

    /// Shared by the three Google-OAuth connectors (GSC/GA4/GBP): which one
    /// the in-flight loopback listener belongs to, so the result can be
    /// written back into the right settings slot.
    pub google_oauth_receiver:
        Option<tokio::sync::mpsc::Receiver<(crate::connectors::google_oauth::GoogleService, Result<GoogleOAuthTokens, String>)>>,
    pub google_oauth_in_progress: bool,
}
