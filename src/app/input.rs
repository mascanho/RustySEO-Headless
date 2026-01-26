use crate::models::App;

impl App {
    pub fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
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

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    pub fn add_input(&mut self, new_input: String) {
        self.input.push_str(&new_input);
        self.cursor_position = self.input.len();
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    pub fn enter_bookmark_char(&mut self, new_char: char) {
        self.bookmark_input.insert(self.bookmark_cursor, new_char);
        self.move_bookmark_cursor_right();
    }

    pub fn delete_bookmark_char(&mut self) {
        if self.bookmark_cursor > 0 {
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

    pub fn clamp_bookmark_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.bookmark_input.len())
    }

    // Scroll Methods
    pub fn scroll_left(&mut self) {
        if self.horizontal_scroll > 0 {
            self.horizontal_scroll -= 1;
        }
    }

    pub fn scroll_right(&mut self, max_scroll: usize) {
        if self.horizontal_scroll < max_scroll {
            self.horizontal_scroll += 1;
        }
    }

    pub fn scroll_content_left(&mut self) {
        if self.content_horizontal_scroll > 0 {
            self.content_horizontal_scroll -= 1;
        }
    }

    pub fn scroll_content_right(&mut self, max_scroll: usize) {
        if self.content_horizontal_scroll < max_scroll {
            self.content_horizontal_scroll += 1;
        }
    }

    pub fn scroll_logs_left(&mut self) {
        if self.logs_horizontal_scroll > 0 {
            self.logs_horizontal_scroll -= 1;
        }
    }

    pub fn scroll_logs_right(&mut self, max_scroll: usize) {
        if self.logs_horizontal_scroll < max_scroll {
            self.logs_horizontal_scroll += 1;
        }
    }

    pub fn validate_table_state(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_table_data.len() {
                if self.filtered_table_data.is_empty() {
                    self.table_state.select(None);
                } else {
                    self.table_state
                        .select(Some(self.filtered_table_data.len().saturating_sub(1)));
                }
            }
        }
    }
}
