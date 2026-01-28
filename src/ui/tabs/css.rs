use ratatui::{layout::Rect, Frame};

use crate::models::App;
use crate::ui::components::css_urls_table::render_css_urls_table;

/// Renders the CSS tab showing unique CSS URLs and their usage statistics.
/// This tab displays a table of all unique CSS files found across crawled pages,
/// along with how many pages reference each CSS file.
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    render_css_urls_table(f, app, area);
}
