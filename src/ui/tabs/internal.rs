use ratatui::{
    Frame,
    layout::Rect,
};

use crate::models::App;
use crate::ui::components::internal_links_table::render_internal_links_table;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    // Replicate the dashboard table functionality for internal links
    render_internal_links_table(f, app, area, "🔗 Internal Links Master List");
}
