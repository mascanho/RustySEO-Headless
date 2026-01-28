use ratatui::{layout::Rect, Frame};

use crate::models::App;
use crate::ui::components::external_links_table::render_external_links_table;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    render_external_links_table(f, app, area, "External Links");
}
