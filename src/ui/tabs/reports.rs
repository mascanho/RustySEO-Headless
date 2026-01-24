// TODO: Create a nice report layout for technical and less technical people and show here.
// A dashboard-like thing could work. With a nice overview that could potentially be then generated
// into a PDF.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::models::App;

pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
    let border_color = Color::Rgb(40, 45, 60);

    let block = Block::default()
        .title(Span::styled(
            " 📈 Audit Reports ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let p = Paragraph::new("🚀 Advanced report generation modules are currently in development.\nStay tuned for PDF, CSV, and HTML export capabilities.")
        .block(block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}
