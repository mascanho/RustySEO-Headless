use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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

    let logs_to_render = if app.log_search_query.is_empty() && !app.show_log_search {
        &app.logs_data
    } else {
        &app.filtered_logs_data
    };

    let log_items: Vec<ListItem> = logs_to_render
        .iter()
        .map(|log| {
            // Log format: [HH:MM:SS][LEVEL] message
            let (timestamp, rest) = if log.starts_with('[') && log.len() > 10 {
                (&log[1..9], &log[10..])
            } else {
                ("00:00:00", log.as_str())
            };

            let (icon, color, message) = if rest.contains("ERROR") {
                (
                    "✘",
                    Color::Red,
                    rest.replace("[ERROR]", "").trim().to_string(),
                )
            } else if rest.contains("DEBUG") {
                (
                    "⚙",
                    Color::DarkGray,
                    rest.replace("[DEBUG]", "").trim().to_string(),
                )
            } else if rest.contains("SYSTEM") {
                (
                    "🖥",
                    Color::Cyan,
                    rest.replace("[SYSTEM]", "").trim().to_string(),
                )
            } else if rest.contains("WARN") {
                (
                    "⚠",
                    Color::Yellow,
                    rest.replace("[WARN]", "").trim().to_string(),
                )
            } else if rest.contains("INFO") {
                (
                    "ℹ",
                    Color::Rgb(100, 150, 255),
                    rest.replace("[INFO]", "").trim().to_string(),
                )
            } else {
                ("ℹ", Color::Rgb(100, 150, 255), rest.trim().to_string())
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
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

    // Floating Log Search Bar
    if app.show_log_search {
        let search_area = Rect {
            x: area.x + area.width.saturating_sub(42),
            y: area.y + area.height.saturating_sub(3),
            width: 40.min(area.width),
            height: 3,
        };

        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .bg(Color::Rgb(20, 20, 30))
            .title(Span::styled(
                " 🔍 Search Logs ",
                Style::default().fg(Color::Cyan).bold(),
            ));

        let search_text = Paragraph::new(app.log_search_query.as_str())
            .block(search_block)
            .alignment(Alignment::Left);

        f.render_widget(Clear, search_area);
        f.render_widget(search_text, search_area);
    }
}
