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

    let block = Block::default()
        .title(Span::styled(
            " 📄 SYSTEM LOGS ",
            Style::default()
                .fg(accent_color)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(accent_color));

    let log_items: Vec<ListItem> = app
        .logs_data
        .iter()
        .map(|log| {
            // Log format: [HH:MM:SS][LEVEL] message
            let (timestamp, rest) = if log.starts_with('[') && log.len() > 10 {
                (&log[1..9], &log[10..])
            } else {
                ("00:00:00", log.as_str())
            };

            let (icon, color, message) = if rest.contains("ERROR") {
                ("✘", Color::Red, rest.replace("[ERROR]", "").trim().to_string())
            } else if rest.contains("DEBUG") {
                ("⚙", Color::DarkGray, rest.replace("[DEBUG]", "").trim().to_string())
            } else if rest.contains("SYSTEM") {
                ("🖥", Color::Cyan, rest.replace("[SYSTEM]", "").trim().to_string())
            } else if rest.contains("WARN") {
                ("⚠", Color::Yellow, rest.replace("[WARN]", "").trim().to_string())
            } else if rest.contains("INFO") {
                ("ℹ", Color::Rgb(100, 150, 255), rest.replace("[INFO]", "").trim().to_string())
            } else {
                ("ℹ", Color::Rgb(100, 150, 255), rest.trim().to_string())
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(icon, Style::default().fg(color)),
                Span::styled(format!(" {}", message), Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(log_items)
        .block(block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(25, 25, 40))
                .add_modifier(ratatui::style::Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut app.logs_state);
}
