#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Crawl,
    Logs,
    Connectors,
    Dashboard,
    Reports,
    Chat,
}

pub struct App {
    pub sidebar_visible: bool,
    pub task_panel_visible: bool,
    pub current_state: AppState,
    pub sidebar_tab: usize,
    pub tasks: Vec<String>,
    pub table_data: Vec<Vec<String>>,
    pub logs_data: Vec<String>,
    pub connectors_data: Vec<(String, bool)>,
    pub table_scroll_x: usize,
    pub tab_rect: Option<ratatui::layout::Rect>,
    pub sidebar_tab_rect: Option<ratatui::layout::Rect>,
    pub keyword_rects: Vec<(String, ratatui::layout::Rect)>,
    pub show_help: bool,
}

impl Default for App {
    fn default() -> Self {
        let table_data = vec![
            vec![
                "1".to_string(),
                "Item A".to_string(),
                "Active".to_string(),
                "2023-01-01".to_string(),
                "100".to_string(),
                "Cat1".to_string(),
                "Note1".to_string(),
            ],
            vec![
                "2".to_string(),
                "Item B".to_string(),
                "Inactive".to_string(),
                "2023-01-02".to_string(),
                "200".to_string(),
                "Cat2".to_string(),
                "Note2".to_string(),
            ],
            vec![
                "3".to_string(),
                "Item C".to_string(),
                "Active".to_string(),
                "2023-01-03".to_string(),
                "300".to_string(),
                "Cat3".to_string(),
                "Note3".to_string(),
            ],
            vec![
                "4".to_string(),
                "Item D".to_string(),
                "Inactive".to_string(),
                "2023-01-04".to_string(),
                "400".to_string(),
                "Cat1".to_string(),
                "Note4".to_string(),
            ],
            vec![
                "5".to_string(),
                "Item E".to_string(),
                "Active".to_string(),
                "2023-01-05".to_string(),
                "500".to_string(),
                "Cat2".to_string(),
                "Note5".to_string(),
            ],
            vec![
                "6".to_string(),
                "Item F".to_string(),
                "Inactive".to_string(),
                "2023-01-06".to_string(),
                "600".to_string(),
                "Cat3".to_string(),
                "Note6".to_string(),
            ],
            vec![
                "7".to_string(),
                "Item G".to_string(),
                "Active".to_string(),
                "2023-01-07".to_string(),
                "700".to_string(),
                "Cat1".to_string(),
                "Note7".to_string(),
            ],
            vec![
                "8".to_string(),
                "Item H".to_string(),
                "Inactive".to_string(),
                "2023-01-08".to_string(),
                "800".to_string(),
                "Cat2".to_string(),
                "Note8".to_string(),
            ],
            vec![
                "9".to_string(),
                "Item I".to_string(),
                "Active".to_string(),
                "2023-01-09".to_string(),
                "900".to_string(),
                "Cat3".to_string(),
                "Note9".to_string(),
            ],
            vec![
                "10".to_string(),
                "Item J".to_string(),
                "Inactive".to_string(),
                "2023-01-10".to_string(),
                "1000".to_string(),
                "Cat1".to_string(),
                "Note10".to_string(),
            ],
        ];
        Self {
            sidebar_visible: false, // Hidden by default now
            task_panel_visible: false,
            current_state: AppState::Crawl,
            sidebar_tab: 0,
            tasks: vec!["Sample Task 1".to_string(), "Sample Task 2".to_string()],
            table_data,
            logs_data: vec![
                "INFO - 2023-10-01 12:00:00 - Crawl started".to_string(),
                "DEBUG - 2023-10-01 12:00:05 - Found 50 URLs".to_string(),
                "ERROR - 2023-10-01 12:00:10 - Connection timeout on example.com".to_string(),
            ],
            connectors_data: vec![
                ("Google Search Console".to_string(), true),
                ("Google Analytics".to_string(), false),
                ("Screaming Frog".to_string(), true),
            ],
            table_scroll_x: 0,
            tab_rect: None,
            sidebar_tab_rect: None,
            keyword_rects: vec![],
            show_help: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
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

    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    pub fn toggle_task_panel(&mut self) {
        self.task_panel_visible = !self.task_panel_visible;
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

    pub fn scroll_table_left(&mut self) {
        if self.table_scroll_x > 0 {
            self.table_scroll_x -= 1;
        }
    }

    pub fn scroll_table_right(&mut self) {
        const VISIBLE_COLS: usize = 4;
        if self.table_scroll_x < 7 - VISIBLE_COLS {
            self.table_scroll_x += 1;
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
}
