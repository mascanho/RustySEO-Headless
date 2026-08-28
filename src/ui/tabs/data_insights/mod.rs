use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::models::App;

mod ahrefs;
mod bing;
mod clarity;
mod ga4;
mod gbp;
mod gsc;
mod pagespeed;

const SUB_TABS: [&str; 7] = [
    "Search Console",
    "GA4",
    "Clarity",
    "Bing",
    "PageSpeed",
    "Business Profile",
    "Ahrefs",
];

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // A vertical list of connectors on the left, separate from the
    // horizontal main-tab bar above, so the two navigation levels don't
    // read as one confusing double row of tabs.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(area);

    let items: Vec<ListItem> = SUB_TABS
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.data_insights_tab {
                Style::default()
                    .fg(Color::White)
                    .bg(accent_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(Line::from(format!(" {} ", name))).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Connectors ")
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(list, chunks[0]);

    let content_block = Block::default()
        .borders(Borders::ALL)
        .title(" \u{2191}/\u{2193} or Tab/Shift+Tab switch \u{2022} c connect \u{2022} Enter fetch ")
        .border_style(Style::default().fg(border_color));
    let inner = content_block.inner(chunks[1]);
    f.render_widget(content_block, chunks[1]);

    match app.data_insights_tab {
        0 => gsc::render(f, app, inner),
        1 => ga4::render(f, app, inner),
        2 => clarity::render(f, app, inner),
        3 => bing::render(f, app, inner),
        4 => pagespeed::render(f, app, inner),
        5 => gbp::render(f, app, inner),
        _ => ahrefs::render(f, app, inner),
    }
}

/// Shared status header rendered at the top of every connector's body:
/// title, connection/config state, loading spinner, and last error. Kept as
/// plain `Line`s (not a widget) so each connector can drop it straight into
/// its own `Paragraph`.
pub(super) fn status_lines(
    title: &str,
    status_line: String,
    loading: bool,
    error: Option<&str>,
) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            title.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(status_line),
    ];
    if loading {
        lines.push(Line::from(Span::styled(
            "Fetching...",
            Style::default().fg(Color::Cyan),
        )));
    }
    if let Some(err) = error {
        lines.push(Line::from(Span::styled(
            format!("Error: {}", err),
            Style::default().fg(Color::Red),
        )));
    }
    lines
}
