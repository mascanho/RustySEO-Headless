use directories::ProjectDirs;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde_json;
use std::collections::HashMap;
use std::sync::mpsc;

use crate::crawler::CrawlMessage;
use crate::models::{App, AppSettings};
use crate::settings::utils::read::recent_crawls;
use crate::ui::modals::dashboard_menu;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Dashboard,
    Crawl,
    Internal,
    Redirects,
    Images,
    Css,
    Javascript,
    Keywords,
    CoreWebVitals,
    CustomExtractor,
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
            table_rect: None,
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
            settings_receiver: None,
            log_receiver: None,
            show_logs: false,
            logs_height: 18,
            show_ai_modal: false,
            ai_input: String::new(),
            ai_chat_history: vec![],
            ai_chat_state: ratatui::widgets::ListState::default(),
            ai_chat_scroll: 0,
            ai_chat_auto_scroll: true,
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
            recent_crawls: recent_crawls(),
            internal_table_data: Vec::new(),
            internal_table_state: ratatui::widgets::TableState::default(),
            internal_filtered_table_data: Vec::new(),
            internal_full_filtered_table_data: Vec::new(),
            internal_current_page: 0,
            internal_page_size: 100,
            internal_horizontal_scroll: 0,
            internal_search_query: String::new(),
            show_internal_search: false,
            css_urls_table_data: Vec::new(),
            css_urls_table_state: ratatui::widgets::TableState::default(),
            css_urls_filtered_table_data: Vec::new(),
            css_urls_full_filtered_table_data: Vec::new(),
            css_urls_current_page: 0,
            css_urls_page_size: 100,
            css_urls_horizontal_scroll: 0,
            css_urls_search_query: String::new(),
            show_css_urls_search: false,
            url_to_status: HashMap::new(),
            js_urls_table_data: Vec::new(),
            js_urls_table_state: ratatui::widgets::TableState::default(),
            js_urls_filtered_table_data: Vec::new(),
            js_urls_full_filtered_table_data: Vec::new(),
            js_urls_current_page: 0,
            js_urls_page_size: 100,
            js_urls_horizontal_scroll: 0,
            js_urls_search_query: String::new(),
            show_js_urls_search: false,
            content_table_state: ratatui::widgets::TableState::default(),
            content_filtered_table_data: Vec::new(),
            content_full_filtered_table_data: Vec::new(),
            content_current_page: 0,
            content_page_size: 100,
            content_horizontal_scroll: 0,
            content_search_query: String::new(),
            show_content_search: false,
            show_js_pages_modal: false,
            js_pages_list: Vec::new(),
            js_pages_state: ratatui::widgets::ListState::default(),
            show_css_pages_modal: false,
            css_pages_list: Vec::new(),
            css_pages_state: ratatui::widgets::ListState::default(),
            // Custom Search/Extractor Tab State
            extractor_table_data: Vec::new(),
            extractor_table_state: ratatui::widgets::TableState::default(),
            extractor_filtered_table_data: Vec::new(),
            extractor_full_filtered_table_data: Vec::new(),
            extractor_current_page: 0,
            extractor_page_size: 100,
            extractor_horizontal_scroll: 0,
            extractor_search_query: String::new(),
            show_extractor_search: false,
            // Images Tab State
            images_table_data: Vec::new(),
            images_table_state: ratatui::widgets::TableState::default(),
            images_filtered_table_data: Vec::new(),
            images_full_filtered_table_data: Vec::new(),
            images_current_page: 0,
            images_page_size: 100,
            images_horizontal_scroll: 0,
            images_search_query: String::new(),
            show_images_search: false,
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
        for data in &results {
            let current_id = self.page_data.len() + 1;
            let mut page_data = data.clone();
            page_data.id = current_id;
            self.page_data.push(page_data);

            let mut row = vec![
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
                data.css
                    .as_ref()
                    .map_or("0 B".to_string(), |css| css.total_size_formatted.clone()),
                data.css
                    .as_ref()
                    .map_or("0".to_string(), |css| css.external_css_count.to_string()),
                data.css.as_ref().map_or("0 B".to_string(), |css| {
                    css.inline_css_size_formatted.clone()
                }),
                data.css
                    .as_ref()
                    .and_then(|css| css.css_urls.first())
                    .map_or("inline only".to_string(), |url| url.clone()),
            ];

            // 2b. Add top 10 keywords to the row
            let mut keywords = data.keywords.clone().unwrap_or_default();
            keywords.resize(10, String::new()); // Ensure we have 10 slots
            for kw in keywords {
                row.push(kw);
            }

            // 2c. Add CWV data
            let d = data.cwv_desktop.clone().unwrap_or_default();
            row.push(d.performance_score);
            row.push(d.fcp);
            row.push(d.lcp);
            row.push(d.cls);
            row.push(d.tbt);
            row.push(d.speed_index);

            let m = data.cwv_mobile.clone().unwrap_or_default();
            row.push(m.performance_score);
            row.push(m.fcp);
            row.push(m.lcp);
            row.push(m.cls);
            row.push(m.tbt);
            row.push(m.speed_index);

            self.table_data.push(row);

            // Populate internal links table
            for link in &data.anchor_links {
                let normalized_to = crate::crawler::url_normalizer::normalize_url(&link.href)
                    .unwrap_or_else(|| link.href.clone());

                let internal_link = crate::models::InternalLink {
                    id: self.internal_table_data.len() + 1,
                    source: data.url.clone(),
                    destination: normalized_to,
                    anchor: link.text.clone(),
                    rel: link.rel.clone(),
                };
                self.internal_table_data.push(internal_link);
            }

            // Collect CSS URLs for CSS URLs table
            if let Some(css_info) = &data.css {
                for css_url in &css_info.css_urls {
                    // Normalize the CSS URL if possible
                    let normalized_css_url = crate::crawler::url_normalizer::normalize_url(css_url)
                        .unwrap_or_else(|| css_url.clone());

                    // Check if this URL is already in our collection
                    let existing_index = self
                        .css_urls_table_data
                        .iter()
                        .position(|css| css.url == normalized_css_url);

                    if let Some(index) = existing_index {
                        // Increment the page count for existing URL
                        self.css_urls_table_data[index].page_count += 1;
                    } else {
                        // Add new unique CSS URL
                        let css_url_entry = crate::models::CssUrl {
                            id: self.css_urls_table_data.len() + 1,
                            url: normalized_css_url,
                            page_count: 1,
                        };
                        self.css_urls_table_data.push(css_url_entry);
                    }
                }
            }

            // Collect JS URLs for JS URLs table
            if let Some(js_info) = &data.javascript {
                for script in &js_info.scripts {
                    if let Some(js_url) = &script.src {
                        // Normalize the JS URL if possible
                        let normalized_js_url =
                            crate::crawler::url_normalizer::normalize_url(js_url)
                                .unwrap_or_else(|| js_url.clone());

                        // Check if this URL is already in our collection
                        let existing_index = self
                            .js_urls_table_data
                            .iter()
                            .position(|js| js.url == normalized_js_url);

                        if let Some(index) = existing_index {
                            // Increment the page count for existing URL
                            self.js_urls_table_data[index].page_count += 1;
                        } else {
                            // Add new unique JS URL
                            let js_url_entry = crate::models::JsUrl {
                                id: self.js_urls_table_data.len() + 1,
                                url: normalized_js_url,
                                script_type: script.script_type.clone(),
                                is_async: script.is_async,
                                is_defer: script.is_defer,
                                page_count: 1,
                            };
                            self.js_urls_table_data.push(js_url_entry);
                        }
                    }
                }
            }

            // Collect Extraction Results for Custom Search table
            if let Some(extraction) = &data.extraction {
                if extraction.found {
                    for match_item in &extraction.matches {
                        let entry = crate::models::ExtractionTableEntry {
                            id: self.extractor_table_data.len() + 1,
                            url: data.url.clone(),
                            element: match_item.element.clone(),
                            snippet: match_item.snippet.clone(),
                        };
                        self.extractor_table_data.push(entry);
                    }
                }
            }

            // Collect Images for Images table
            for image in &data.images {
                let normalized_img_url = image.src.clone();

                if let Some(existing) = self
                    .images_table_data
                    .iter_mut()
                    .find(|i| i.url == normalized_img_url)
                {
                    existing.page_count += 1;
                } else {
                    self.images_table_data.push(crate::models::ImageTableEntry {
                        id: self.images_table_data.len() + 1,
                        url: normalized_img_url,
                        alt: image.alt.clone(),
                        status: "-".to_string(),
                        size: image.size_formatted.clone(),
                        page_count: 1,
                    });
                }
            }

            self.url_to_status
                .insert(data.url.clone(), data.status.clone());
            self.log(format!("Crawled: {}", data.url));
        }

        if !results.is_empty() {
            self.apply_filter();
            self.apply_internal_filter();
            self.apply_css_urls_filter();
            self.apply_js_urls_filter();
            self.apply_extractor_filter();
            self.apply_content_filter();
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
                self.apply_internal_filter();
                self.apply_css_urls_filter();
                self.apply_js_urls_filter();
                self.apply_extractor_filter();
                self.apply_images_filter();
                self.apply_content_filter();
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
        self.sidebar_tab = (self.sidebar_tab + 1) % 6;
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
        let project_dirs = ProjectDirs::from("", "", "rustyseo").unwrap();
        let recent_crawls_path = project_dirs.data_dir().join("recent-crawls.json");

        if recent_crawls_path.exists() {
            std::fs::read_to_string(&recent_crawls_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
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

    pub fn move_detail_row_up(&mut self) {
        let selected_idx = self.table_state.selected().unwrap_or(0);
        let full_idx = self.current_page * self.page_size + selected_idx;
        if full_idx >= self.full_filtered_table_data.len() {
            return;
        }
        let row_data = &self.full_filtered_table_data[full_idx];
        let original_id = row_data[0].parse::<usize>().unwrap_or(1);
        let page_idx = original_id.saturating_sub(1);

        // Ensure page data is in memory
        if page_idx >= self.page_data.len() {
            if let Some(pd) = crate::db::load_page_data(original_id) {
                self.page_data.push(pd);
            } else {
                return;
            }
        }

        let selected = self.detail_table_state.selected().unwrap_or(0);

        match self.detail_tab {
            3 => {
                // inlinks
                if selected > 0 {
                    self.page_data[page_idx]
                        .anchor_links
                        .swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
            }
            4 => {
                // outlinks
                if selected > 0 {
                    self.page_data[page_idx]
                        .outlinks
                        .swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
            }
            5 => {
                // images
                if selected > 0 {
                    self.page_data[page_idx].images.swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
            }
            8 => {
                // headings
                if selected > 0 {
                    self.page_data[page_idx]
                        .headings
                        .swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
            }
            _ => {}
        }
    }

    pub fn move_detail_row_down(&mut self) {
        let selected_idx = self.table_state.selected().unwrap_or(0);
        let full_idx = self.current_page * self.page_size + selected_idx;
        if full_idx >= self.full_filtered_table_data.len() {
            return;
        }
        let row_data = &self.full_filtered_table_data[full_idx];
        let original_id = row_data[0].parse::<usize>().unwrap_or(1);
        let page_idx = original_id.saturating_sub(1);

        // Ensure page data is in memory
        if page_idx >= self.page_data.len() {
            if let Some(pd) = crate::db::load_page_data(original_id) {
                self.page_data.push(pd);
            } else {
                return;
            }
        }

        let selected = self.detail_table_state.selected().unwrap_or(0);
        let len = self.get_current_detail_len();

        match self.detail_tab {
            3 => {
                // inlinks
                if selected < len.saturating_sub(1) {
                    self.page_data[page_idx]
                        .anchor_links
                        .swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected + 1));
                }
            }
            4 => {
                // outlinks
                if selected < len.saturating_sub(1) {
                    self.page_data[page_idx]
                        .outlinks
                        .swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected + 1));
                }
            }
            5 => {
                // images
                if selected < len.saturating_sub(1) {
                    self.page_data[page_idx].images.swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected + 1));
                }
            }
            8 => {
                // headings
                if selected < len.saturating_sub(1) {
                    self.page_data[page_idx]
                        .headings
                        .swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected + 1));
                }
            }
            _ => {}
        }
    }

    pub fn next_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::Crawl,
            AppState::Crawl => AppState::Internal,
            AppState::Internal => AppState::Redirects,
            AppState::Redirects => AppState::Images,
            AppState::Images => AppState::Css,
            AppState::Css => AppState::Javascript,
            AppState::Javascript => AppState::Keywords,
            AppState::Keywords => AppState::CoreWebVitals,
            AppState::CoreWebVitals => AppState::CustomExtractor,
            AppState::CustomExtractor => AppState::Reports,
            AppState::Reports => AppState::Content,
            AppState::Content => AppState::Dashboard,
        }
    }

    pub fn previous_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::Content,
            AppState::Crawl => AppState::Dashboard,
            AppState::Internal => AppState::Crawl,
            AppState::Redirects => AppState::Internal,
            AppState::Images => AppState::Redirects,
            AppState::Css => AppState::Images,
            AppState::Javascript => AppState::Css,
            AppState::Keywords => AppState::Javascript,
            AppState::CoreWebVitals => AppState::Keywords,
            AppState::CustomExtractor => AppState::CoreWebVitals,
            AppState::Reports => AppState::CustomExtractor,
            AppState::Content => AppState::Reports,
        }
    }

    pub fn get_state_index(&self) -> usize {
        match self.current_state {
            AppState::Dashboard => 0,
            AppState::Crawl => 1,
            AppState::Internal => 2,
            AppState::Redirects => 3,
            AppState::Images => 4,
            AppState::Css => 5,
            AppState::Javascript => 6,
            AppState::Keywords => 7,
            AppState::CoreWebVitals => 8,
            AppState::CustomExtractor => 9,
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
        let loaded_settings = crate::models::AppSettings::load();
        let settings_path = crate::models::AppSettings::path();
        self.settings = Some(loaded_settings);

        let pconfig = self
            .settings
            .as_ref()
            .map(|s| s.connectors.pagespeed.clone());
        let status_info = match &pconfig {
            Some(c) => format!("status={}, key_len={}", c.status, c.api_key.len()),
            None => "None".to_string(),
        };
        self.log(format!(
            "SYSTEM - Settings loaded from: {:?}",
            settings_path
        ));
        self.log(format!("SYSTEM - PageSpeed Config: {}", status_info));

        self.page_data.clear();
        self.table_data.clear();
        self.internal_table_data.clear();
        self.css_urls_table_data.clear();
        self.css_urls_filtered_table_data.clear();
        self.css_urls_full_filtered_table_data.clear();
        self.url_to_status.clear();
        self.table_state.select(None); // Reset table selection when data is cleared
        self.internal_table_state.select(None);
        self.css_urls_table_state.select(None);
        self.css_urls_current_page = 0;
        self.css_urls_search_query.clear();
        self.show_css_urls_search = false;
        self.js_urls_table_data.clear();
        self.js_urls_filtered_table_data.clear();
        self.js_urls_full_filtered_table_data.clear();
        self.js_urls_table_state.select(None);
        self.js_urls_current_page = 0;
        self.js_urls_search_query.clear();
        self.show_js_urls_search = false;
        self.content_table_state.select(None);
        self.content_current_page = 0;
        self.content_search_query.clear();
        self.show_content_search = false;
        self.content_filtered_table_data.clear();
        self.content_full_filtered_table_data.clear();
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
        let pagespeed_config = pconfig;
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
                .with_javascript(enable_javascript)
                .with_pagespeed(pagespeed_config);

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

    pub fn scroll_content_left(&mut self) {
        if self.content_horizontal_scroll > 0 {
            self.content_horizontal_scroll = self.content_horizontal_scroll.saturating_sub(1);
        }
    }

    pub fn scroll_content_right(&mut self, max_scroll: usize) {
        if self.content_horizontal_scroll < max_scroll {
            self.content_horizontal_scroll = self.content_horizontal_scroll.saturating_add(1);
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

    pub fn apply_internal_filter(&mut self) {
        if self.internal_search_query.is_empty() {
            // Only update if lengths differ to avoid redundant massive clones
            if self.internal_full_filtered_table_data.len() != self.internal_table_data.len() {
                self.internal_full_filtered_table_data = self.internal_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for link in &self.internal_table_data {
                let search_blob = format!("{} {} {}", link.source, link.destination, link.anchor);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.internal_search_query)
                {
                    scored_data.push((score, link.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.internal_full_filtered_table_data =
                scored_data.into_iter().map(|(_, link)| link).collect();
        }

        let total_pages = (self.internal_full_filtered_table_data.len() + self.internal_page_size
            - 1)
            / self.internal_page_size;
        if self.internal_current_page >= total_pages {
            self.internal_current_page = total_pages.saturating_sub(1);
        }

        self.apply_internal_pagination();
    }

    pub fn apply_internal_pagination(&mut self) {
        let start = self.internal_current_page * self.internal_page_size;
        let end =
            (start + self.internal_page_size).min(self.internal_full_filtered_table_data.len());
        self.internal_filtered_table_data =
            self.internal_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.internal_table_state.selected() {
            if selected >= self.internal_filtered_table_data.len() {
                if self.internal_filtered_table_data.is_empty() {
                    self.internal_table_state.select(None);
                } else {
                    self.internal_table_state
                        .select(Some(self.internal_filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn apply_css_urls_filter(&mut self) {
        if self.css_urls_search_query.is_empty() {
            // Only update if lengths differ to avoid redundant massive clones
            if self.css_urls_full_filtered_table_data.len() != self.css_urls_table_data.len() {
                self.css_urls_full_filtered_table_data = self.css_urls_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for css_url in &self.css_urls_table_data {
                if let Some(score) = matcher.fuzzy_match(&css_url.url, &self.css_urls_search_query)
                {
                    scored_data.push((score, css_url.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.css_urls_full_filtered_table_data = scored_data
                .into_iter()
                .map(|(_, css_url)| css_url)
                .collect();
        }

        let total_pages = (self.css_urls_full_filtered_table_data.len() + self.css_urls_page_size
            - 1)
            / self.css_urls_page_size;
        if self.css_urls_current_page >= total_pages {
            self.css_urls_current_page = total_pages.saturating_sub(1);
        }

        self.apply_css_urls_pagination();
    }

    pub fn apply_css_urls_pagination(&mut self) {
        let start = self.css_urls_current_page * self.css_urls_page_size;
        let end =
            (start + self.css_urls_page_size).min(self.css_urls_full_filtered_table_data.len());
        self.css_urls_filtered_table_data =
            self.css_urls_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.css_urls_table_state.selected() {
            if selected >= self.css_urls_filtered_table_data.len() {
                if self.css_urls_filtered_table_data.is_empty() {
                    self.css_urls_table_state.select(None);
                } else {
                    self.css_urls_table_state
                        .select(Some(self.css_urls_filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn apply_js_urls_filter(&mut self) {
        if self.js_urls_search_query.is_empty() {
            if self.js_urls_full_filtered_table_data.len() != self.js_urls_table_data.len() {
                self.js_urls_full_filtered_table_data = self.js_urls_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for js_url in &self.js_urls_table_data {
                if let Some(score) = matcher.fuzzy_match(&js_url.url, &self.js_urls_search_query) {
                    scored_data.push((score, js_url.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.js_urls_full_filtered_table_data =
                scored_data.into_iter().map(|(_, js_url)| js_url).collect();
        }

        let total_pages = (self.js_urls_full_filtered_table_data.len() + self.js_urls_page_size
            - 1)
            / self.js_urls_page_size;
        if self.js_urls_current_page >= total_pages {
            self.js_urls_current_page = total_pages.saturating_sub(1);
        }

        self.apply_js_urls_pagination();
    }

    pub fn apply_js_urls_pagination(&mut self) {
        let start = self.js_urls_current_page * self.js_urls_page_size;
        let end = (start + self.js_urls_page_size).min(self.js_urls_full_filtered_table_data.len());
        self.js_urls_filtered_table_data =
            self.js_urls_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.js_urls_table_state.selected() {
            if selected >= self.js_urls_filtered_table_data.len() {
                if self.js_urls_filtered_table_data.is_empty() {
                    self.js_urls_table_state.select(None);
                } else {
                    self.js_urls_table_state
                        .select(Some(self.js_urls_filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn apply_content_filter(&mut self) {
        if self.content_search_query.is_empty() {
            // Only update if lengths differ to avoid redundant massive clones
            if self.content_full_filtered_table_data.len() != self.table_data.len() {
                self.content_full_filtered_table_data = self.table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for row in &self.table_data {
                let search_blob = format!("{} {} {}", row[1], row[2], row[6]);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.content_search_query) {
                    scored_data.push((score, row.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.content_full_filtered_table_data =
                scored_data.into_iter().map(|(_, row)| row).collect();
        }

        let total_pages = (self.content_full_filtered_table_data.len() + self.content_page_size
            - 1)
            / self.content_page_size;
        if self.content_current_page >= total_pages {
            self.content_current_page = total_pages.saturating_sub(1);
        }

        self.apply_content_pagination();
    }

    pub fn apply_content_pagination(&mut self) {
        let start = self.content_current_page * self.content_page_size;
        let end = (start + self.content_page_size).min(self.content_full_filtered_table_data.len());
        self.content_filtered_table_data =
            self.content_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.content_table_state.selected() {
            if selected >= self.content_filtered_table_data.len() {
                if self.content_filtered_table_data.is_empty() {
                    self.content_table_state.select(None);
                } else {
                    self.content_table_state
                        .select(Some(self.content_filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn apply_extractor_filter(&mut self) {
        if self.extractor_search_query.is_empty() {
            // Only update if lengths differ to avoid redundant massive clones
            if self.extractor_full_filtered_table_data.len() != self.extractor_table_data.len() {
                self.extractor_full_filtered_table_data = self.extractor_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();

            for entry in &self.extractor_table_data {
                let search_blob = format!("{} {} {}", entry.url, entry.element, entry.snippet);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.extractor_search_query)
                {
                    scored_data.push((score, entry.clone()));
                }
            }

            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.extractor_full_filtered_table_data =
                scored_data.into_iter().map(|(_, row)| row).collect();
        }

        let total_pages =
            (self.extractor_full_filtered_table_data.len() + self.extractor_page_size - 1)
                / self.extractor_page_size;

        // Handle pagination reset if search reduced pages
        if self.extractor_current_page >= total_pages && total_pages > 0 {
            self.extractor_current_page = total_pages.saturating_sub(1);
        } else if total_pages == 0 {
            self.extractor_current_page = 0;
        }

        self.apply_extractor_pagination();
    }

    pub fn apply_extractor_pagination(&mut self) {
        let start = self.extractor_current_page * self.extractor_page_size;
        let end =
            (start + self.extractor_page_size).min(self.extractor_full_filtered_table_data.len());

        if start < self.extractor_full_filtered_table_data.len() {
            self.extractor_filtered_table_data =
                self.extractor_full_filtered_table_data[start..end].to_vec();
        } else {
            self.extractor_filtered_table_data = Vec::new();
        }

        if let Some(selected) = self.extractor_table_state.selected() {
            if selected >= self.extractor_filtered_table_data.len() {
                if self.extractor_filtered_table_data.is_empty() {
                    self.extractor_table_state.select(None);
                } else {
                    self.extractor_table_state
                        .select(Some(self.extractor_filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn next_internal_row(&mut self) {
        let len = self.internal_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.internal_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    let total_pages = (self.internal_full_filtered_table_data.len()
                        + self.internal_page_size
                        - 1)
                        / self.internal_page_size;
                    if self.internal_current_page + 1 < total_pages {
                        self.internal_current_page += 1;
                        self.apply_internal_pagination();
                        0
                    } else {
                        len - 1
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.internal_table_state.select(Some(i));
    }

    pub fn previous_internal_row(&mut self) {
        let len = self.internal_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.internal_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.internal_current_page > 0 {
                        self.internal_current_page -= 1;
                        self.apply_internal_pagination();
                        self.internal_filtered_table_data.len().saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.internal_table_state.select(Some(i));
    }

    pub fn next_internal_page(&mut self) {
        let total_pages = (self.internal_full_filtered_table_data.len() + self.internal_page_size
            - 1)
            / self.internal_page_size;
        if self.internal_current_page + 1 < total_pages {
            self.internal_current_page += 1;
            self.apply_internal_pagination();
        }
    }

    pub fn previous_internal_page(&mut self) {
        if self.internal_current_page > 0 {
            self.internal_current_page -= 1;
            self.apply_internal_pagination();
        }
    }

    pub fn next_content_row(&mut self) {
        let len = self.content_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.content_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    let total_pages =
                        (self.content_full_filtered_table_data.len() + self.content_page_size - 1)
                            / self.content_page_size;
                    if self.content_current_page + 1 < total_pages {
                        self.content_current_page += 1;
                        self.apply_content_pagination();
                        0
                    } else {
                        len - 1
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.content_table_state.select(Some(i));
    }

    pub fn previous_content_row(&mut self) {
        let len = self.content_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.content_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.content_current_page > 0 {
                        self.content_current_page -= 1;
                        self.apply_content_pagination();
                        self.content_filtered_table_data.len().saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.content_table_state.select(Some(i));
    }

    pub fn next_content_page(&mut self) {
        let total_pages = (self.content_full_filtered_table_data.len() + self.content_page_size
            - 1)
            / self.content_page_size;
        if self.content_current_page + 1 < total_pages {
            self.content_current_page += 1;
            self.apply_content_pagination();
        }
    }

    pub fn previous_content_page(&mut self) {
        if self.content_current_page > 0 {
            self.content_current_page -= 1;
            self.apply_content_pagination();
        }
    }

    pub fn reload_settings_if_changed(&mut self) {
        // First, check if we received any file change notifications from the watcher
        let mut should_reload = false;

        if let Some(ref rx) = self.settings_receiver {
            // Non-blocking check for settings change events
            // Drain all pending events (in case multiple were queued)
            loop {
                match rx.try_recv() {
                    Ok(()) => {
                        should_reload = true;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // Watcher died, fall back to mtime checking
                        should_reload = self.check_settings_mtime();
                        break;
                    }
                }
            }
        } else {
            // No watcher connected, fall back to mtime checking
            should_reload = self.check_settings_mtime();
        }

        if should_reload {
            let current_settings = AppSettings::load();

            // Only update if settings actually changed (using PartialEq to compare ALL fields)
            let settings_changed = self
                .settings
                .as_ref()
                .map_or(true, |stored| stored != &current_settings);

            if settings_changed {
                self.settings = Some(current_settings);
                self.log("Settings reloaded from file");
            }
        }
    }

    /// Fallback mtime-based check for settings changes
    fn check_settings_mtime(&mut self) -> bool {
        let settings_path = AppSettings::path();

        if let Ok(metadata) = std::fs::metadata(&settings_path) {
            if let Ok(mtime) = metadata.modified() {
                // Only reload if file was modified since last check
                if self
                    .last_settings_mtime
                    .map_or(true, |last_mtime| mtime > last_mtime)
                {
                    self.last_settings_mtime = Some(mtime);
                    return true;
                }
            }
        }
        false
    }

    pub fn show_js_pages_for_url(&mut self, js_url: String) {
        let mut pages = Vec::new();
        for page in &self.page_data {
            if let Some(js_info) = &page.javascript {
                if js_info.js_urls.contains(&js_url) {
                    pages.push(page.url.clone());
                }
            }
        }
        self.js_pages_list = pages;
        self.js_pages_state.select(Some(0));
        self.show_js_pages_modal = true;
    }

    pub fn close_js_pages_modal(&mut self) {
        self.show_js_pages_modal = false;
        self.js_pages_list.clear();
        self.js_pages_state.select(None);
    }

    pub fn show_css_pages_for_url(&mut self, css_url: String) {
        let mut pages = Vec::new();
        for page in &self.page_data {
            if let Some(css_info) = &page.css {
                if css_info.css_urls.contains(&css_url) {
                    pages.push(page.url.clone());
                }
            }
        }
        self.css_pages_list = pages;
        self.css_pages_state.select(Some(0));
        self.show_css_pages_modal = true;
    }

    pub fn close_css_pages_modal(&mut self) {
        self.show_css_pages_modal = false;
        self.css_pages_list.clear();
        self.css_pages_state.select(None);
    }

    pub fn next_extractor_page(&mut self) {
        let total_pages =
            (self.extractor_full_filtered_table_data.len() + self.extractor_page_size - 1)
                / self.extractor_page_size;
        if self.extractor_current_page + 1 < total_pages {
            self.extractor_current_page += 1;
            self.apply_extractor_pagination();
        }
    }

    pub fn previous_extractor_page(&mut self) {
        if self.extractor_current_page > 0 {
            self.extractor_current_page -= 1;
            self.apply_extractor_pagination();
        }
    }

    pub fn next_extractor_row(&mut self) {
        let len = self.extractor_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.extractor_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    let total_pages = (self.extractor_full_filtered_table_data.len()
                        + self.extractor_page_size
                        - 1)
                        / self.extractor_page_size;
                    if self.extractor_current_page + 1 < total_pages {
                        self.extractor_current_page += 1;
                        self.apply_extractor_pagination();
                        0
                    } else {
                        len - 1
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.extractor_table_state.select(Some(i));
    }

    pub fn previous_extractor_row(&mut self) {
        let len = self.extractor_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.extractor_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.extractor_current_page > 0 {
                        self.extractor_current_page -= 1;
                        self.apply_extractor_pagination();
                        self.extractor_filtered_table_data.len() - 1
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.extractor_table_state.select(Some(i));
    }

    pub fn apply_images_filter(&mut self) {
        if self.images_search_query.is_empty() {
            if self.images_full_filtered_table_data.len() != self.images_table_data.len() {
                self.images_full_filtered_table_data = self.images_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();

            for entry in &self.images_table_data {
                let search_blob = format!("{} {}", entry.url, entry.alt);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.images_search_query) {
                    scored_data.push((score, entry.clone()));
                }
            }

            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.images_full_filtered_table_data =
                scored_data.into_iter().map(|(_, row)| row).collect();
        }

        let total_pages = (self.images_full_filtered_table_data.len() + self.images_page_size - 1)
            / self.images_page_size;

        if self.images_current_page >= total_pages && total_pages > 0 {
            self.images_current_page = total_pages.saturating_sub(1);
        } else if total_pages == 0 {
            self.images_current_page = 0;
        }

        self.apply_images_pagination();
    }

    pub fn apply_images_pagination(&mut self) {
        let start = self.images_current_page * self.images_page_size;
        let end = (start + self.images_page_size).min(self.images_full_filtered_table_data.len());

        if start < self.images_full_filtered_table_data.len() {
            self.images_filtered_table_data =
                self.images_full_filtered_table_data[start..end].to_vec();
        } else {
            self.images_filtered_table_data = Vec::new();
        }

        if let Some(selected) = self.images_table_state.selected() {
            if selected >= self.images_filtered_table_data.len() {
                if self.images_filtered_table_data.is_empty() {
                    self.images_table_state.select(None);
                } else {
                    self.images_table_state
                        .select(Some(self.images_filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn next_images_page(&mut self) {
        let total_pages = (self.images_full_filtered_table_data.len() + self.images_page_size - 1)
            / self.images_page_size;
        if self.images_current_page + 1 < total_pages {
            self.images_current_page += 1;
            self.apply_images_pagination();
        }
    }

    pub fn previous_images_page(&mut self) {
        if self.images_current_page > 0 {
            self.images_current_page -= 1;
            self.apply_images_pagination();
        }
    }

    pub fn next_images_row(&mut self) {
        let len = self.images_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.images_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    let total_pages =
                        (self.images_full_filtered_table_data.len() + self.images_page_size - 1)
                            / self.images_page_size;
                    if self.images_current_page + 1 < total_pages {
                        self.images_current_page += 1;
                        self.apply_images_pagination();
                        0
                    } else {
                        len - 1
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.images_table_state.select(Some(i));
    }

    pub fn previous_images_row(&mut self) {
        let len = self.images_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.images_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.images_current_page > 0 {
                        self.images_current_page -= 1;
                        self.apply_images_pagination();
                        self.images_filtered_table_data.len() - 1
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.images_table_state.select(Some(i));
    }
}
