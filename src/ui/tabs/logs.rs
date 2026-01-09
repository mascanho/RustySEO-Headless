use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" System Logs ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let log_items: Vec<ListItem> = app
        .logs_data
        .iter()
        .map(|log| {
            let color = if log.contains("ERROR") {
                Color::Red
            } else if log.contains("DEBUG") {
                Color::DarkGray
            } else if log.contains("SYSTEM") {
                Color::LightBlue
            } else {
                Color::Blue
            };
            ListItem::new(Line::from(ratatui::text::Span::styled(
                log,
                Style::default().fg(color),
            )))
        })
        .collect();

    let list = List::new(log_items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.logs_state);
}
