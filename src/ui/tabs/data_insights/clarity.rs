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

    let has_token = app
        .settings
        .as_ref()
        .map(|s| !s.connectors.clarity.api_token.is_empty())
        .unwrap_or(false);

    let status_line = if has_token {
        "API token set".to_string()
    } else {
        "No API token - generate one in Clarity > Settings > Data Export, then set connectors.clarity.api_token in cli-settings.toml".to_string()
    };

    let lines = super::status_lines(
        " Microsoft Clarity - live project insights (last 3 days) ",
        status_line,
        app.clarity_state.loading,
        app.clarity_state.error.as_deref(),
    );
    let status = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)));
    f.render_widget(status, chunks[0]);

    if let Some(insights) = &app.clarity_state.data {
        let header = Row::new(
            ["Metric", "Summary"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(accent_color).add_modifier(Modifier::BOLD))),
        )
        .height(1);

        let rows = insights.metrics.iter().map(|m| {
            let name = m
                .get("metricName")
                .and_then(|v| v.as_str())
                .unwrap_or("(unknown)")
                .to_string();
            let summary = m
                .get("information")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .map(|first| first.to_string())
                .unwrap_or_else(|| "(no data)".to_string());
            Row::new(vec![Cell::from(name), Cell::from(summary)])
        });

        let table = Table::new(rows, [Constraint::Length(24), Constraint::Min(30)])
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Metrics ({}) ", insights.metrics.len()))
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
