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
        .map(|s| s.connectors.ga4.clone())
        .unwrap_or_default();

    let status_line = if cfg.tokens.is_connected() {
        format!(
            "Connected  |  property: {}",
            if cfg.property_id.is_empty() {
                "(set connectors.ga4.property_id)"
            } else {
                &cfg.property_id
            }
        )
    } else {
        "Not connected - press 'c' to sign in with Google (needs client_id/client_secret in cli-settings.toml)".to_string()
    };

    let lines = super::status_lines(
        " Google Analytics 4 - daily traffic (last 28 days) ",
        status_line,
        app.ga4_state.loading,
        app.ga4_state.error.as_deref(),
    );
    let status = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)));
    f.render_widget(status, chunks[0]);

    if let Some(report) = &app.ga4_state.data {
        let mut header_cells = vec!["Date".to_string()];
        header_cells.extend(report.metric_headers.iter().cloned());
        let header = Row::new(
            header_cells
                .into_iter()
                .map(|h| Cell::from(h).style(Style::default().fg(accent_color).add_modifier(Modifier::BOLD))),
        )
        .height(1);

        let rows = report.rows.iter().map(|(dims, metrics)| {
            let mut cells: Vec<Cell> = dims.iter().map(|d| Cell::from(d.clone())).collect();
            cells.extend(metrics.iter().map(|m| Cell::from(m.clone())));
            Row::new(cells)
        });

        let mut widths = vec![Constraint::Length(12)];
        widths.extend(report.metric_headers.iter().map(|_| Constraint::Length(14)));

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Report ({} rows) ", report.rows.len()))
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
