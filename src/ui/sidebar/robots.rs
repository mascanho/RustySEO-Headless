use crate::models::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &App, area: Rect, block: Block) {
    if app.robots_txt_content.is_empty() {
        let msg = if app.robots_urls_loading {
            " Fetching robots.txt…"
        } else {
            " No robots.txt found or crawl not started."
        };

        let p = Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray)))
            .block(block.title(Span::styled(
                " robots.txt ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        f.render_widget(p, area);
        return;
    }

    let lines: Vec<Line> = app
        .robots_txt_content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if lower.starts_with("user-agent:") {
                Line::from(vec![
                    Span::styled("User-agent", Style::default().fg(Color::Rgb(80, 140, 255)).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        &trimmed[10..],
                        Style::default().fg(Color::White),
                    ),
                ])
            } else if lower.starts_with("disallow:") {
                Line::from(vec![
                    Span::styled("Disallow", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        &trimmed[8..],
                        Style::default().fg(Color::Rgb(255, 150, 150)),
                    ),
                ])
            } else if lower.starts_with("allow:") {
                Line::from(vec![
                    Span::styled("Allow", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        &trimmed[5..],
                        Style::default().fg(Color::Rgb(150, 255, 150)),
                    ),
                ])
            } else if lower.starts_with("sitemap:") {
                Line::from(vec![
                    Span::styled("Sitemap", Style::default().fg(Color::Rgb(200, 100, 255)).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        &trimmed[7..],
                        Style::default().fg(Color::Rgb(200, 180, 255)),
                    ),
                ])
            } else if trimmed.starts_with('#') {
                Line::from(Span::styled(trimmed, Style::default().fg(Color::DarkGray)))
            } else if trimmed.is_empty() {
                Line::from("")
            } else {
                Line::from(Span::styled(trimmed, Style::default().fg(Color::Gray)))
            }
        })
        .collect();

    let p = Paragraph::new(lines)
        .block(block.title(Span::styled(
            " robots.txt ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )))
        .wrap(Wrap { trim: false })
        .scroll((app.sidebar_scroll as u16, 0));

    f.render_widget(p, area);
}
