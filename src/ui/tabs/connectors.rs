use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let _accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let block = Block::default()
        .title(Span::styled(
            " Internal Links ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let connector_items: Vec<ListItem> = app
        .connectors_data
        .iter()
        .map(|(name, status)| {
            let (icon, status_text, status_color) = if *status {
                ("🟢", " ACTIVE  ", Color::Green)
            } else {
                ("🔴", " OFF-LINE", Color::Red)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default()),
                Span::styled(format!(" {:<20} ", name), Style::default().fg(Color::White)),
                Span::styled(" │ ", Style::default().fg(border_color)),
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
        })
        .collect();

    let list = List::new(connector_items)
        .block(block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_widget(list, area);
}
