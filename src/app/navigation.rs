use crate::app::AppState;
use crate::models::App;

impl App {
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

    pub fn next_sidebar_tab(&mut self) {
        self.sidebar_tab = (self.sidebar_tab + 1) % 6;
    }

    pub fn previous_sidebar_tab(&mut self) {
        self.sidebar_tab = if self.sidebar_tab == 0 {
            5
        } else {
            self.sidebar_tab - 1
        };
    }

    pub fn reset(&mut self) {
        self.sidebar_visible = false;
        self.task_panel_visible = false;
        self.current_state = AppState::Dashboard;
        self.sidebar_tab = 0;
    }

    pub fn next_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::External,
            AppState::External => AppState::Internal,
            AppState::Internal => AppState::Redirects,
            AppState::Redirects => AppState::Images,
            AppState::Images => AppState::Css,
            AppState::Css => AppState::Javascript,
            AppState::Javascript => AppState::CoreWebVitals,
            AppState::CoreWebVitals => AppState::Content,
            AppState::Content => AppState::Files,
            AppState::Files => AppState::CustomExtractor,
            AppState::CustomExtractor => AppState::Dashboard,
        }
    }

    pub fn previous_state(&mut self) {
        self.current_state = match self.current_state {
            AppState::Dashboard => AppState::CustomExtractor,
            AppState::External => AppState::Dashboard,
            AppState::Internal => AppState::External,
            AppState::Redirects => AppState::Internal,
            AppState::Images => AppState::Redirects,
            AppState::Css => AppState::Images,
            AppState::Javascript => AppState::Css,
            AppState::CoreWebVitals => AppState::Javascript,
            AppState::Content => AppState::CoreWebVitals,
            AppState::Files => AppState::Content,
            AppState::CustomExtractor => AppState::Files,
        }
    }

    pub fn get_state_index(&self) -> usize {
        match self.current_state {
            AppState::Dashboard => 0,
            AppState::External => 1,
            AppState::Internal => 2,
            AppState::Redirects => 3,
            AppState::Images => 4,
            AppState::Css => 5,
            AppState::Javascript => 6,
            AppState::CoreWebVitals => 7,
            AppState::Content => 8,
            AppState::Files => 9,
            AppState::CustomExtractor => 10,
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

    pub fn next_files_row(&mut self) {
        let len = self.files_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.files_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    let total_pages =
                        (self.files_full_filtered_table_data.len() + self.files_page_size - 1)
                            / self.files_page_size;
                    if self.files_current_page + 1 < total_pages {
                        self.files_current_page += 1;
                        self.apply_files_pagination();
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
        self.files_table_state.select(Some(i));
    }

    pub fn previous_files_row(&mut self) {
        let len = self.files_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.files_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.files_current_page > 0 {
                        self.files_current_page -= 1;
                        self.apply_files_pagination();
                        self.files_filtered_table_data.len().saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.files_table_state.select(Some(i));
    }

    pub fn next_files_page(&mut self) {
        let total_pages = (self.files_full_filtered_table_data.len() + self.files_page_size - 1)
            / self.files_page_size;
        if self.files_current_page + 1 < total_pages {
            self.files_current_page += 1;
            self.apply_files_pagination();
        }
    }

    pub fn previous_files_page(&mut self) {
        if self.files_current_page > 0 {
            self.files_current_page -= 1;
            self.apply_files_pagination();
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

    pub fn next_external_row(&mut self) {
        let len = self.external_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.external_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    let total_pages = (self.external_full_filtered_table_data.len()
                        + self.external_page_size
                        - 1)
                        / self.external_page_size;
                    if self.external_current_page + 1 < total_pages {
                        self.external_current_page += 1;
                        self.apply_external_pagination();
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
        self.external_table_state.select(Some(i));
    }

    pub fn previous_external_row(&mut self) {
        let len = self.external_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.external_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.external_current_page > 0 {
                        self.external_current_page -= 1;
                        self.apply_external_pagination();
                        self.external_filtered_table_data.len().saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.external_table_state.select(Some(i));
    }

    pub fn next_external_page(&mut self) {
        let total_pages = (self.external_full_filtered_table_data.len() + self.external_page_size
            - 1)
            / self.external_page_size;
        if self.external_current_page + 1 < total_pages {
            self.external_current_page += 1;
            self.apply_external_pagination();
        }
    }

    pub fn previous_external_page(&mut self) {
        if self.external_current_page > 0 {
            self.external_current_page -= 1;
            self.apply_external_pagination();
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
                        self.extractor_filtered_table_data.len().saturating_sub(1)
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
                        self.images_filtered_table_data.len().saturating_sub(1)
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

    pub fn get_current_detail_len(&self) -> usize {
        if let Some(page_data) = &self.selected_page_details {
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
        let selected = self.detail_table_state.selected().unwrap_or(0);
        if selected == 0 { return; }

        if let Some(page_data) = &mut self.selected_page_details {
            match self.detail_tab {
                3 => {
                    page_data.anchor_links.swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
                4 => {
                    page_data.outlinks.swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
                5 => {
                    page_data.images.swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
                8 => {
                    page_data.headings.swap(selected, selected - 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
                _ => {}
            }
        }
    }

    pub fn move_detail_row_down(&mut self) {
        let selected = self.detail_table_state.selected().unwrap_or(0);
        let len = self.get_current_detail_len();
        if len == 0 || selected >= len - 1 { return; }

        if let Some(page_data) = &mut self.selected_page_details {
            match self.detail_tab {
                3 => {
                    page_data.anchor_links.swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected + 1));
                }
                4 => {
                    page_data.outlinks.swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected - 1));
                }
                5 => {
                    page_data.images.swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected + 1));
                }
                8 => {
                    page_data.headings.swap(selected, selected + 1);
                    self.detail_table_state.select(Some(selected + 1));
                }
                _ => {}
            }
        }
    }

    pub fn next_issues_row(&mut self) {
        let len = self.issues_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.issues_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.issues_table_state.select(Some(i));
    }

    pub fn previous_issues_row(&mut self) {
        let len = self.issues_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.issues_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.issues_table_state.select(Some(i));
    }

    pub fn next_redirects_row(&mut self) {
        let len = self.redirects_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.redirects_table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    let total_pages = (self.redirects_full_filtered_table_data.len()
                        + self.redirects_page_size
                        - 1)
                        / self.redirects_page_size;
                    if self.redirects_current_page + 1 < total_pages {
                        self.redirects_current_page += 1;
                        self.apply_redirects_pagination();
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
        self.redirects_table_state.select(Some(i));
    }

    pub fn previous_redirects_row(&mut self) {
        let len = self.redirects_filtered_table_data.len();
        if len == 0 {
            return;
        }
        let i = match self.redirects_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.redirects_current_page > 0 {
                        self.redirects_current_page -= 1;
                        self.apply_redirects_pagination();
                        self.redirects_filtered_table_data.len().saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.redirects_table_state.select(Some(i));
    }

    pub fn next_redirects_page(&mut self) {
        let total_pages =
            (self.redirects_full_filtered_table_data.len() + self.redirects_page_size - 1)
                / self.redirects_page_size;
        if self.redirects_current_page + 1 < total_pages {
            self.redirects_current_page += 1;
            self.apply_redirects_pagination();
        }
    }

    pub fn previous_redirects_page(&mut self) {
        if self.redirects_current_page > 0 {
            self.redirects_current_page -= 1;
            self.apply_redirects_pagination();
        }
    }
}
