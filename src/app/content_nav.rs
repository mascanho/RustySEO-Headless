//! Keyboard focus, scrolling, copy and open-in-browser support for the two
//! side panels on the Content tab (N-Grams and Duplicate Content), which
//! otherwise only ever mirrored whatever page was selected in the main
//! table. Mirrors the row derivation in `ui::tabs::content` so the key
//! handler and the renderer never disagree on what row N points to.

use crate::models::App;

impl App {
    pub fn content_selected_page_id(&self) -> Option<usize> {
        self.content_table_state
            .selected()
            .and_then(|i| self.content_filtered_table_data.get(i))
            .and_then(|row| row.first())
            .and_then(|id_str| id_str.parse::<usize>().ok())
    }

    /// (url, match label) pairs for the Duplicate Content panel, in the same
    /// order as `ui::tabs::content::render_duplicate_content_panel`.
    pub fn content_duplicate_rows(&self) -> Vec<(String, String)> {
        let Some(id) = self.content_selected_page_id() else {
            return Vec::new();
        };
        let mut rows = Vec::new();
        for pair in &self.duplicate_pairs {
            let other_id = if pair.id_a == id {
                Some(pair.id_b)
            } else if pair.id_b == id {
                Some(pair.id_a)
            } else {
                None
            };
            let Some(other_id) = other_id else { continue };
            let Some(other) = self.page_summaries.get(other_id.saturating_sub(1)) else {
                continue;
            };
            let label = if pair.distance == 0 {
                "Exact".to_string()
            } else {
                let similarity = 100.0 * (64 - pair.distance) as f64 / 64.0;
                format!("{:.0}%", similarity)
            };
            rows.push((other.url.clone(), label));
        }
        rows
    }

    /// (n, phrase, count) rows for the N-Grams panel, in the same order as
    /// `ui::tabs::content::render_ngrams_panel`.
    pub fn content_ngrams_rows(&self) -> Vec<(String, String, usize)> {
        let Some(id) = self.content_selected_page_id() else {
            return Vec::new();
        };
        let Some(summary) = self.page_summaries.get(id.saturating_sub(1)) else {
            return Vec::new();
        };
        let groups: [(&str, &Vec<(String, usize)>); 4] = [
            ("1", &summary.ngrams.unigrams),
            ("2", &summary.ngrams.bigrams),
            ("3", &summary.ngrams.trigrams),
            ("4", &summary.ngrams.quadgrams),
        ];
        let mut rows = Vec::new();
        for (n, phrases) in groups {
            for (phrase, count) in phrases.iter().take(6) {
                rows.push((n.to_string(), phrase.clone(), *count));
            }
        }
        rows
    }

    pub fn next_content_ngrams_row(&mut self) {
        let len = self.content_ngrams_rows().len();
        if len == 0 {
            self.content_ngrams_state.select(None);
            return;
        }
        let i = match self.content_ngrams_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.content_ngrams_state.select(Some(i));
    }

    pub fn previous_content_ngrams_row(&mut self) {
        let len = self.content_ngrams_rows().len();
        if len == 0 {
            self.content_ngrams_state.select(None);
            return;
        }
        let i = match self.content_ngrams_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.content_ngrams_state.select(Some(i));
    }

    pub fn next_content_duplicate_row(&mut self) {
        let len = self.content_duplicate_rows().len();
        if len == 0 {
            self.content_duplicate_state.select(None);
            return;
        }
        let i = match self.content_duplicate_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.content_duplicate_state.select(Some(i));
    }

    pub fn previous_content_duplicate_row(&mut self) {
        let len = self.content_duplicate_rows().len();
        if len == 0 {
            self.content_duplicate_state.select(None);
            return;
        }
        let i = match self.content_duplicate_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.content_duplicate_state.select(Some(i));
    }

    /// Row up/down, routed to whichever pane currently has focus.
    pub fn previous_content_focus_row(&mut self) {
        match self.content_focus {
            1 => self.previous_content_ngrams_row(),
            2 => self.previous_content_duplicate_row(),
            _ => {
                self.previous_content_row();
                self.reset_content_side_panel_selection();
            }
        }
    }

    pub fn next_content_focus_row(&mut self) {
        match self.content_focus {
            1 => self.next_content_ngrams_row(),
            2 => self.next_content_duplicate_row(),
            _ => {
                self.next_content_row();
                self.reset_content_side_panel_selection();
            }
        }
    }

    /// The N-Grams / Duplicate Content panels are keyed to whichever page row
    /// is selected in the main table - moving that selection invalidates
    /// whatever row index was highlighted in either side panel.
    pub fn reset_content_side_panel_selection(&mut self) {
        self.content_ngrams_state.select(None);
        self.content_duplicate_state.select(None);
    }

    /// Neovim split-style pane switching: the main table spans the full left
    /// column, N-Grams and Duplicate Content are stacked on the right.
    pub fn content_focus_right(&mut self) {
        if self.content_focus == 0 {
            self.content_focus = 1;
            if self.content_ngrams_state.selected().is_none() && !self.content_ngrams_rows().is_empty() {
                self.content_ngrams_state.select(Some(0));
            }
        }
    }

    pub fn content_focus_left(&mut self) {
        self.content_focus = 0;
    }

    pub fn content_focus_down(&mut self) {
        if self.content_focus == 1 {
            self.content_focus = 2;
            if self.content_duplicate_state.selected().is_none()
                && !self.content_duplicate_rows().is_empty()
            {
                self.content_duplicate_state.select(Some(0));
            }
        }
    }

    pub fn content_focus_up(&mut self) {
        if self.content_focus == 2 {
            self.content_focus = 1;
        }
    }

    /// Copy the focused pane's selected cell (phrase text, or a duplicate's
    /// URL) to the system clipboard.
    pub fn copy_content_focus_row(&mut self) {
        match self.content_focus {
            1 => {
                if let Some(i) = self.content_ngrams_state.selected() {
                    if let Some((_, phrase, _)) = self.content_ngrams_rows().get(i) {
                        let phrase = phrase.clone();
                        crate::ui::modals::dashboard_menu::copy_to_clipboard(phrase.clone());
                        self.log(format!("Copied phrase to clipboard: {}", phrase));
                    }
                }
            }
            2 => {
                if let Some(i) = self.content_duplicate_state.selected() {
                    if let Some((url, _)) = self.content_duplicate_rows().get(i) {
                        let url = url.clone();
                        crate::ui::modals::dashboard_menu::copy_to_clipboard(url.clone());
                        self.log(format!("Copied URL to clipboard: {}", url));
                    }
                }
            }
            _ => {
                if let Some(selected) = self.content_table_state.selected() {
                    if let Some(url) = self
                        .content_filtered_table_data
                        .get(selected)
                        .and_then(|row| row.get(1))
                    {
                        let url = url.clone();
                        crate::ui::modals::dashboard_menu::copy_to_clipboard(url.clone());
                        self.log(format!("Copied URL to clipboard: {}", url));
                    }
                }
            }
        }
    }

    /// Enter: open the focused pane's URL. The N-Grams panel has no URL, so
    /// this only acts on the main table and the Duplicate Content panel.
    pub fn activate_content_focus_row(&mut self) {
        match self.content_focus {
            2 => {
                if let Some(i) = self.content_duplicate_state.selected() {
                    if let Some((url, _)) = self.content_duplicate_rows().get(i) {
                        let url = url.clone();
                        crate::ui::modals::dashboard_menu::open_in_browser(&url);
                        self.log(format!("Opened URL in browser: {}", url));
                    }
                }
            }
            1 => {}
            _ => {
                if let Some(id) = self.content_selected_page_id() {
                    self.open_details(id);
                }
            }
        }
    }
}
