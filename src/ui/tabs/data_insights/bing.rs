use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let border_color = Color::Rgb(40, 45, 60);
    let accent_color = Color::Rgb(80, 140, 255);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    let cfg = app
        .settings
        .as_ref()
        .map(|s| s.connectors.bing.clone())
        .unwrap_or_default();

    let status_line = if cfg.api_key.is_empty() {
        "No API key - get one from Bing Webmaster Tools > Settings > API Access, then set connectors.bing.api_key/site_url in cli-settings.toml".to_string()
    } else {
        format!(
            "API key set  |  site: {}",
            if cfg.site_url.is_empty() {
                "(set connectors.bing.site_url)"
            } else {
                &cfg.site_url
            }
        )
    };

    let lines = super::status_lines(
        " Bing Webmaster Tools - top queries ",
        status_line,
        app.bing_state.loading,
        app.bing_state.error.as_deref(),
    );
    let status = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)));
    f.render_widget(status, chunks[0]);

    if let Some(stats) = &app.bing_state.data {
        let header = Row::new(
            ["Query", "Clicks", "Impressions", "Avg Click Pos", "Avg Impr Pos"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(accent_color).add_modifier(Modifier::BOLD))),
        )
        .height(1);

        let rows = stats.rows.iter().map(|r| {
            Row::new(vec![
                Cell::from(r.query.clone()),
                Cell::from(r.clicks.to_string()),
                Cell::from(r.impressions.to_string()),
                Cell::from(format!("{:.1}", r.avg_click_position)),
                Cell::from(format!("{:.1}", r.avg_impression_position)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Min(30),
                Constraint::Length(10),
                Constraint::Length(14),
                Constraint::Length(15),
                Constraint::Length(15),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Queries ({}) ", stats.rows.len()))
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

        f.render_widget(table, chunks[1]);
    } else {
        let msg = Paragraph::new("No data yet - press Enter to fetch.").block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(msg, chunks[1]);
    }
}
