use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let block = Block::default()
        .title(Span::styled(
            " 📄 System Logs ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let log_items: Vec<ListItem> = app
        .logs_data
        .iter()
        .map(|log| {
            let (icon, color) = if log.contains("ERROR") {
                (" ✘ ", Color::Red)
            } else if log.contains("DEBUG") {
                (" ⚙ ", Color::DarkGray)
            } else if log.contains("SYSTEM") {
                (" 🖥 ", Color::Cyan)
            } else if log.contains("FOUND") {
                (" 🔍 ", Color::Green)
            } else {
                (" ℹ ", Color::Rgb(100, 150, 255))
            };

            ListItem::new(Line::from(vec![
                Span::styled(icon, Style::default().fg(color)),
                Span::styled(log, Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(log_items)
        .block(block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(accent_color)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(" ➔ ");

    f.render_stateful_widget(list, area, &mut app.logs_state);
}
