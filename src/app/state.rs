use crate::models::{App, AppSettings};
use crate::settings::utils::read::recent_crawls;
use crate::helpers::issues::IssueAnalyzer;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RustyColors {
    Primary,
    Secondary,
}

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
    Files,
}

impl Default for App {
    fn default() -> Self {
        let table_data = Vec::new();
        let page_data = Vec::new();
        let table_state = ratatui::widgets::TableState::default();

        Self {
            options_modal: false,
            sidebar_visible: false,
            task_panel_visible: false,
            current_state: AppState::Dashboard,
            sidebar_tab: 0,
            bookmarks: vec![],
            bookmark_index: 0,
            bookmark_input: String::new(),
            bookmark_cursor: 0,
            bookmark_subview: 0, // 0=bookmarks, 1=last_crawled
            bookmarks_state: {
                let mut state = ratatui::widgets::ListState::default();
                state.select(Some(0));
                state
            },
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
            // Tree View State
            tree_view_state: {
                let mut state = ratatui::widgets::ListState::default();
                state.select(Some(0));
                state
            },
            tree_view_selected_index: 0,
            tree_view_expanded_nodes: std::collections::HashSet::new(),
            // Issues Tab State
            issues_table_data: IssueAnalyzer::generate_issues_table_data(&[]),
            issues_table_state: {
                let mut state = ratatui::widgets::TableState::default();
                state.select(Some(0));
                state
            },
            issues_current_page: 0,
            issues_page_size: 100,
            // Issues URLs Modal State
            show_issue_urls_modal: false,
            issue_urls_list: vec![],
            issue_urls_state: {
                let mut state = ratatui::widgets::ListState::default();
                state.select(Some(0));
                state
            },
            current_issue_title: String::new(),
            // Files Tab State
            files_table_data: Vec::new(),
            files_table_state: ratatui::widgets::TableState::default(),
            files_filtered_table_data: Vec::new(),
            files_full_filtered_table_data: Vec::new(),
            files_current_page: 0,
            files_page_size: 100,
            files_search_query: String::new(),
            show_files_search: false,
            // Redirects Tab State
            redirects_table_data: Vec::new(),
            redirects_table_state: ratatui::widgets::TableState::default(),
            redirects_filtered_table_data: Vec::new(),
            redirects_full_filtered_table_data: Vec::new(),
            redirects_current_page: 0,
            redirects_page_size: 100,
            redirects_horizontal_scroll: 0,
            redirects_search_query: String::new(),
            show_redirects_search: false,
        }
    }
}
