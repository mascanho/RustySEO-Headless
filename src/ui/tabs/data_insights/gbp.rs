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
        .map(|s| s.connectors.gbp.clone())
        .unwrap_or_default();

    let status_line = if cfg.tokens.is_connected() {
        format!(
            "Connected  |  account: {}",
            if cfg.account_id.is_empty() {
                "(set connectors.gbp.account_id)"
            } else {
                &cfg.account_id
            }
        )
    } else {
        "Not connected - press 'c' to sign in with Google (needs client_id/client_secret in cli-settings.toml)".to_string()
    };

    let lines = super::status_lines(
        " Google Business Profile - locations ",
        status_line,
        app.gbp_state.loading,
        app.gbp_state.error.as_deref(),
    );
    let status = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)));
    f.render_widget(status, chunks[0]);

    if let Some(locations) = &app.gbp_state.data {
        let header = Row::new(
            ["Title", "Address", "Phone", "Website"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(accent_color).add_modifier(Modifier::BOLD))),
        )
        .height(1);

        let rows = locations.locations.iter().map(|l| {
            Row::new(vec![
                Cell::from(l.title.clone()),
                Cell::from(l.address.clone()),
                Cell::from(l.phone.clone()),
                Cell::from(l.website.clone()),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Min(24),
                Constraint::Length(16),
                Constraint::Length(24),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Locations ({}) ", locations.locations.len()))
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
