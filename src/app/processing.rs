use crate::models::App;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

impl App {
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

    pub fn apply_external_filter(&mut self) {
        if self.external_search_query.is_empty() {
            if self.external_full_filtered_table_data.len() != self.external_table_data.len() {
                self.external_full_filtered_table_data = self.external_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for ext in &self.external_table_data {
                let search_blob = format!("{} {} {}", ext.source, ext.destination, ext.anchor);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.external_search_query) {
                    scored_data.push((score, ext.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.external_full_filtered_table_data = scored_data
                .into_iter()
                .map(|(_, ext)| ext)
                .collect();
        }

        let total_pages = (self.external_full_filtered_table_data.len()
            + self.external_page_size
            - 1)
            / self.external_page_size.max(1);
        if self.external_current_page >= total_pages {
            self.external_current_page = total_pages.saturating_sub(1);
        }

        self.apply_external_pagination();
    }

    pub fn apply_external_pagination(&mut self) {
        let start = self.external_current_page * self.external_page_size;
        let end =
            (start + self.external_page_size).min(self.external_full_filtered_table_data.len());
        self.external_filtered_table_data =
            self.external_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.external_table_state.selected() {
            if selected >= self.external_filtered_table_data.len() {
                if self.external_filtered_table_data.is_empty() {
                    self.external_table_state.select(None);
                } else {
                    self.external_table_state
                        .select(Some(self.external_filtered_table_data.len() - 1));
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

    pub fn apply_files_filter(&mut self) {
        if self.files_search_query.is_empty() {
            if self.files_full_filtered_table_data.len() != self.files_table_data.len() {
                self.files_full_filtered_table_data = self.files_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for file in &self.files_table_data {
                let search_blob = format!("{} {}", file.url, file.filetype);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.files_search_query) {
                    scored_data.push((score, file.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.files_full_filtered_table_data =
                scored_data.into_iter().map(|(_, file)| file).collect();
        }

        let total_pages = (self.files_full_filtered_table_data.len() + self.files_page_size - 1)
            / self.files_page_size;
        if self.files_current_page >= total_pages {
            self.files_current_page = total_pages.saturating_sub(1);
        }

        self.apply_files_pagination();
    }

    pub fn apply_files_pagination(&mut self) {
        let start = self.files_current_page * self.files_page_size;
        let end = (start + self.files_page_size).min(self.files_full_filtered_table_data.len());
        self.files_filtered_table_data = self.files_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.files_table_state.selected() {
            if selected >= self.files_filtered_table_data.len() {
                if self.files_filtered_table_data.is_empty() {
                    self.files_table_state.select(None);
                } else {
                    self.files_table_state
                        .select(Some(self.files_filtered_table_data.len() - 1));
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

    pub fn apply_images_filter(&mut self) {
        if self.images_search_query.is_empty() {
            // Only update if lengths differ to avoid redundant massive clones
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
                scored_data.into_iter().map(|(_, entry)| entry).collect();
        }

        let total_pages = (self.images_full_filtered_table_data.len() + self.images_page_size - 1)
            / self.images_page_size;
        if self.images_current_page >= total_pages {
            self.images_current_page = total_pages.saturating_sub(1);
        }

        self.apply_images_pagination();
    }

    pub fn apply_images_pagination(&mut self) {
        let start = self.images_current_page * self.images_page_size;
        let end = (start + self.images_page_size).min(self.images_full_filtered_table_data.len());
        self.images_filtered_table_data =
            self.images_full_filtered_table_data[start..end].to_vec();

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

    pub fn apply_redirects_filter(&mut self) {
        if self.redirects_search_query.is_empty() {
            if self.redirects_full_filtered_table_data.len() != self.redirects_table_data.len() {
                self.redirects_full_filtered_table_data = self.redirects_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for entry in &self.redirects_table_data {
                let search_blob = format!("{} {}", entry.initial_url, entry.status_code);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.redirects_search_query) {
                    scored_data.push((score, entry.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.redirects_full_filtered_table_data =
                scored_data.into_iter().map(|(_, entry)| entry).collect();
        }

        let total_pages = (self.redirects_full_filtered_table_data.len()
            + self.redirects_page_size
            - 1)
            / self.redirects_page_size;
        if self.redirects_current_page >= total_pages {
            self.redirects_current_page = total_pages.saturating_sub(1);
        }

        self.apply_redirects_pagination();
    }

    pub fn apply_redirects_pagination(&mut self) {
        let start = self.redirects_current_page * self.redirects_page_size;
        let end = (start + self.redirects_page_size)
            .min(self.redirects_full_filtered_table_data.len());
        self.redirects_filtered_table_data =
            self.redirects_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.redirects_table_state.selected() {
            if selected >= self.redirects_filtered_table_data.len() {
                if self.redirects_filtered_table_data.is_empty() {
                    self.redirects_table_state.select(None);
                } else {
                    self.redirects_table_state
                        .select(Some(self.redirects_filtered_table_data.len() - 1));
                }
            }
        }
    }

    pub fn apply_keywords_filter(&mut self) {
        if self.keywords_search_query.is_empty() {
            if self.keywords_full_filtered_table_data.len() != self.keywords_table_data.len() {
                self.keywords_full_filtered_table_data = self.keywords_table_data.clone();
            }
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_data = Vec::new();
            for entry in &self.keywords_table_data {
                let search_blob = format!("{} {}", entry.keyword, entry.url);
                if let Some(score) = matcher.fuzzy_match(&search_blob, &self.keywords_search_query) {
                    scored_data.push((score, entry.clone()));
                }
            }
            scored_data.sort_by(|a, b| b.0.cmp(&a.0));
            self.keywords_full_filtered_table_data =
                scored_data.into_iter().map(|(_, entry)| entry).collect();
        }

        let total_pages = (self.keywords_full_filtered_table_data.len() + self.keywords_page_size
            - 1)
            / self.keywords_page_size.max(1);
        if self.keywords_current_page >= total_pages {
            self.keywords_current_page = total_pages.saturating_sub(1);
        }

        self.apply_keywords_pagination();
    }

    pub fn apply_keywords_pagination(&mut self) {
        let start = self.keywords_current_page * self.keywords_page_size;
        let end = (start + self.keywords_page_size).min(self.keywords_full_filtered_table_data.len());
        self.keywords_filtered_table_data =
            self.keywords_full_filtered_table_data[start..end].to_vec();

        if let Some(selected) = self.keywords_table_state.selected() {
            if selected >= self.keywords_filtered_table_data.len() {
                if self.keywords_filtered_table_data.is_empty() {
                    self.keywords_table_state.select(None);
                } else {
                    self.keywords_table_state
                        .select(Some(self.keywords_filtered_table_data.len() - 1));
                }
            }
        }
    }
}
