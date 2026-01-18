use ratatui::{Frame, layout::Rect};

use crate::models::App;
use crate::ui::components::js_urls_table::render_js_urls_table;

/// Renders the Javascript tab showing unique Javascript files and their usage statistics.
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    render_js_urls_table(f, app, area);
}
