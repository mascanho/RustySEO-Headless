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

    let api_key_set = app
        .settings
        .as_ref()
        .map(|s| !s.connectors.pagespeed.api_key.is_empty())
        .unwrap_or(false);

    let status_line = if api_key_set {
        format!("API key set  |  target: {}", app.input_url)
    } else {
        "No API key - set connectors.pagespeed.api_key in cli-settings.toml (Google Cloud > APIs & Services > PageSpeed Insights API)".to_string()
    };

    let lines = super::status_lines(
        " PageSpeed Insights - on-demand Core Web Vitals (mobile) ",
        status_line,
        app.pagespeed_insights_state.loading,
        app.pagespeed_insights_state.error.as_deref(),
    );
    let status = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)));
    f.render_widget(status, chunks[0]);

    let body = if let Some(cwv) = &app.pagespeed_insights_state.data {
        format!(
            "Performance Score: {}\nFCP: {}\nLCP: {}\nCLS: {}\nTBT: {}\nSpeed Index: {}",
            cwv.performance_score, cwv.fcp, cwv.lcp, cwv.cls, cwv.tbt, cwv.speed_index
        )
    } else {
        "No data yet - press Enter to fetch (tests the currently loaded site's URL).".to_string()
    };

    let msg = Paragraph::new(body).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(msg, chunks[1]);
}
