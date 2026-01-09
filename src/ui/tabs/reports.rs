use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::models::App;

pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
    let block = Block::default().title(" Reports ").borders(Borders::ALL);
    let p = Paragraph::new("Reports generation coming soon...")
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
