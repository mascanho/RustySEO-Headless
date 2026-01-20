use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30), // Progress Bar
            Constraint::Min(0),     // App Info / Status
        ])
        .split(area);

    // Progress Bar
    let label = format!("{:.0}%", app.crawl_progress * 100.0);
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Crawl Progress ",
                    Style::default().fg(Color::Yellow),
                ))
                .border_style(Style::default().fg(border_color))
                .bg(Color::Rgb(15, 15, 25)),
        )
        .gauge_style(
            Style::default()
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 40))
                .add_modifier(Modifier::BOLD),
        )
        .use_unicode(true)
        .label(label)
        .percent((app.crawl_progress * 100.0) as u16);
    f.render_widget(gauge, chunks[0]);

    // Status / System Info
    let status_prefix = if app.is_crawling {
        format!(" 🔄 {} ", app.input_url)
    } else if app.crawl_progress >= 1.0 {
        format!(" ✅ {} ", app.input_url)
    } else {
        " 💤 STATUS: IDLE ".to_string()
    };

    let status_text = vec![Line::from(vec![
        Span::styled(
            status_prefix,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | ", Style::default().fg(border_color)),
        Span::styled(
            {
                let total_filtered = app.full_filtered_table_data.len();
                let _total_pages = (total_filtered + app.page_size - 1) / app.page_size;
                format!(
                    " 🔗 URLs: {}",
                    total_filtered,
                    // app.current_page + 1,
                    // total_pages
                )
            },
            Style::default().fg(Color::Green),
        ),
        Span::styled(" | ", Style::default().fg(border_color)),
        Span::styled(
            format!(" 📜 Logs(L): {} ", app.logs_data.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" | ", Style::default().fg(border_color)),
        Span::styled(
            " 🤖 AI(A) ",
            Style::default()
                .fg(Color::Rgb(200, 100, 255))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | ", Style::default().fg(border_color)),
        Span::styled(
            " ⚡ JS: ",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        if app
            .settings
            .as_ref()
            .map(|s| s.crawler.enable_javascript)
            .unwrap_or(false)
        {
            Span::styled(format!("{}", "ON"), Style::default().fg(Color::Green))
        } else {
            Span::styled(format!("{}", "OFF"), Style::default().fg(Color::Red))
        },
        Span::styled(" | ", Style::default().fg(border_color)),
        Span::styled(
            " ⚡ PSI: ",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        if app
            .settings
            .as_ref()
            .map(|s| s.crawler.enable_javascript)
            .unwrap_or(false)
        {
            Span::styled(format!("{}", "ON"), Style::default().fg(Color::Green))
        } else {
            Span::styled(format!("{}", "OFF"), Style::default().fg(Color::Red))
        },
        Span::styled(" | ", Style::default().fg(border_color)),
        // MAX PAGES
        Span::styled("Max URLs: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!(
                "{}",
                app.settings
                    .as_ref()
                    .map(|s| s.crawler.max_pages)
                    .unwrap_or(0)
            ),
            Style::default().fg(Color::Rgb(200, 100, 255)),
        ),
        // FINDING
        Span::styled(" | ", Style::default().fg(border_color)),
        if app.current_state == AppState::Dashboard {
            if app.show_search {
                Span::styled(
                    " 🔍 FINDING... ",
                    Style::default()
                        .fg(Color::Rgb(255, 170, 0))
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED),
                )
            } else if !app.search_query.is_empty() {
                Span::styled(
                    format!(" 🔍 FILTER: '{}' ", app.search_query),
                    Style::default()
                        .fg(Color::Rgb(255, 170, 0))
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(" 🔍 FIND(Ctrl+F) ", Style::default().fg(Color::DarkGray))
            }
        } else {
            Span::raw("")
        },
        if app.current_state == AppState::Dashboard {
            Span::styled(" | ", Style::default().fg(border_color))
        } else {
            Span::raw("")
        },
        Span::styled(" ⌨️  Help: '?' ", Style::default().fg(Color::Gray)),
    ])];

    let p = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "System Status ",
                    Style::default().fg(Color::Yellow),
                ))
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .alignment(ratatui::layout::Alignment::Right);

    f.render_widget(p, chunks[1]);
}
