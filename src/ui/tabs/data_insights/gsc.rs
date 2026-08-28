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
        .map(|s| s.connectors.search_console.clone())
        .unwrap_or_default();

    let status_line = if cfg.tokens.is_connected() {
        format!(
            "Connected  |  site: {}",
            if cfg.site_url.is_empty() {
                "(set connectors.search_console.site_url)"
            } else {
                &cfg.site_url
            }
        )
    } else {
        "Not connected - press 'c' to sign in with Google (needs client_id/client_secret in cli-settings.toml)".to_string()
    };

    let lines = super::status_lines(
        " Google Search Console - Search Analytics (last 28 days, by query) ",
        status_line,
        app.gsc_state.loading,
        app.gsc_state.error.as_deref(),
    );
    let status = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)));
    f.render_widget(status, chunks[0]);

    if let Some(report) = &app.gsc_state.data {
        let header = Row::new(
            ["Query", "Clicks", "Impressions", "CTR", "Position"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(accent_color).add_modifier(Modifier::BOLD))),
        )
        .height(1);

        let rows = report.rows.iter().map(|r| {
            Row::new(vec![
                Cell::from(r.keys.join(" / ")),
                Cell::from(format!("{:.0}", r.clicks)),
                Cell::from(format!("{:.0}", r.impressions)),
                Cell::from(format!("{:.2}%", r.ctr * 100.0)),
                Cell::from(format!("{:.1}", r.position)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Min(30),
                Constraint::Length(10),
                Constraint::Length(14),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Queries ({}) ", report.rows.len()))
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
