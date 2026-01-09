use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10), // Progress Bar
            Constraint::Min(0),         // App Info / Status
        ])
        .split(area);

    // Progress Bar
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Crawl Progress "),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Blue)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .percent((app.crawl_progress * 100.0) as u16);
    f.render_widget(gauge, chunks[0]);

    // Status / System Info
    let status_prefix = if app.is_crawling {
        format!(" [CRAWLING: {}] ", app.input_url)
    } else if app.crawl_progress >= 1.0 {
        format!(" [FINISHED: {}] ", app.input_url)
    } else {
        " [STATUS: IDLE] ".to_string()
    };

    let status_text = format!(
        "{} | URLs: {} | Logs: {} | Help: '?' ",
        status_prefix,
        app.table_data.len(),
        app.logs_data.len()
    );
    let p = Paragraph::new(status_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" System Status "),
    );
    f.render_widget(p, chunks[1]);
}
