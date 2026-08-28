use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let border_color = Color::Rgb(40, 45, 60);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    let cfg = app
        .settings
        .as_ref()
        .map(|s| s.connectors.ahrefs.clone())
        .unwrap_or_default();

    let status_line = if cfg.api_token.is_empty() {
        "No API token - requires a paid Ahrefs plan with API access; set connectors.ahrefs.api_token/target in cli-settings.toml".to_string()
    } else {
        format!(
            "API token set  |  target: {}",
            if cfg.target.is_empty() {
                "(set connectors.ahrefs.target)"
            } else {
                &cfg.target
            }
        )
    };

    let lines = super::status_lines(
        " Ahrefs - domain rating & backlinks ",
        status_line,
        app.ahrefs_state.loading,
        app.ahrefs_state.error.as_deref(),
    );
    let status = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)));
    f.render_widget(status, chunks[0]);

    let body = if let Some(report) = &app.ahrefs_state.data {
        format!(
            "Domain Rating: {:.1}\nAhrefs Rank: {}\nBacklinks: {}\nReferring Domains: {}",
            report.domain_rating, report.ahrefs_rank, report.backlinks, report.referring_domains
        )
    } else {
        "No data yet - press Enter to fetch.".to_string()
    };

    let msg = Paragraph::new(body).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(msg, chunks[1]);
}
