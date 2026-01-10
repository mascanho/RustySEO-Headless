use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::models::App;

pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
    let border_color = Color::Rgb(40, 45, 60);

    let block = Block::default()
        .title(Span::styled(
            " 💬 AI Assistant ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let p = Paragraph::new("🤖 Atalaia AI is being trained on SEO best practices.\nSoon you will be able to ask questions about your audit directly here.")
        .block(block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}
