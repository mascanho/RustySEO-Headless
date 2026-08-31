//! State transitions for the "Menus" side panel (Ctrl+M). See
//! `crate::ui::menu_panel` for the panel itself and the menu data.

use crate::models::App;
use crate::ui::menu_panel::total_entries;

impl App {
    /// Toggle the Menus panel. Opening it closes the regular sidebar so the two
    /// right-hand panels never overlap.
    pub fn toggle_menu_panel(&mut self) {
        self.menu_panel_visible = !self.menu_panel_visible;
        if self.menu_panel_visible {
            self.sidebar_visible = false;
        } else {
            self.menu_panel_scroll = 0;
        }
    }

    pub fn menu_panel_next(&mut self) {
        let total = total_entries();
        if total == 0 {
            return;
        }
        self.menu_panel_selected = (self.menu_panel_selected + 1) % total;
    }

    pub fn menu_panel_prev(&mut self) {
        let total = total_entries();
        if total == 0 {
            return;
        }
        self.menu_panel_selected = if self.menu_panel_selected == 0 {
            total - 1
        } else {
            self.menu_panel_selected - 1
        };
    }

    pub fn menu_panel_first(&mut self) {
        self.menu_panel_selected = 0;
        self.menu_panel_scroll = 0;
    }

    pub fn menu_panel_last(&mut self) {
        self.menu_panel_selected = total_entries().saturating_sub(1);
    }
}
