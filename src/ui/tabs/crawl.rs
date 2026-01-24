use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let block = Block::default()
        .title(Span::styled(
            " 🕸️  Crawl Control Center ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut content = vec![
        Line::from(vec![
            Span::raw(" Welcome to "),
            Span::styled(
                "RustySEO [CLI]",
                Style::default()
                    .fg(accent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — The ultimate terminal SEO auditor."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [S] Settings  ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(100, 100, 150)),
            ),
            Span::raw("  "),
            Span::styled(
                "  [F] Filters   ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(100, 150, 100)),
            ),
            Span::raw("  "),
            Span::styled(
                "  [I] Stats     ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(150, 100, 100)),
            ),
            Span::raw("  "),
            Span::styled(
                "  [A] Actions   ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(150, 150, 100)),
            ),
        ]),
        Line::from(""),
    ];

    if app.is_crawling {
        content.push(Line::from(vec![
            Span::styled(
                " 🔄 ACTIVE CRAWL: ",
                Style::default()
                    .fg(accent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.input_url, Style::default().fg(Color::White)),
        ]));
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("  ➜  ", Style::default().fg(accent_color)),
            Span::raw("Analyzing site structure and extracting SEO metadata..."),
        ]));
        content.push(Line::from(vec![
            Span::styled("  ➜  ", Style::default().fg(accent_color)),
            Span::raw("Check the "),
            Span::styled("'Logs'", Style::default().fg(Color::Yellow)),
            Span::raw(" tab for real-time URI discovery."),
        ]));
    } else if app.crawl_progress >= 1.0 {
        content.push(Line::from(vec![
            Span::styled(
                " ✅ CRAWL COMPLETE: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.input_url, Style::default().fg(Color::White)),
        ]));
        content.push(Line::from(""));
        content.push(Line::from(
            "  All discovered pages have been audited successfully.",
        ));
        content.push(Line::from(vec![
            Span::raw("  Head to the "),
            Span::styled("'Dashboard'", Style::default().fg(Color::Yellow)),
            Span::raw(" to view the full SEO report."),
        ]));
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("  (Press ", Style::default().fg(Color::Gray)),
            Span::styled("Ctrl+I", Style::default().fg(Color::Cyan)),
            Span::styled(" to start a new crawl)", Style::default().fg(Color::Gray)),
        ]));
    } else {
        content.push(Line::from(vec![
            Span::styled(" 🔍 Ready to audit. ", Style::default().fg(Color::Cyan)),
            Span::raw("Enter a URL to begin the crawl:"),
        ]));
        content.push(Line::from(
            "  ________________________________________________",
        ));
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("  (Press ", Style::default().fg(Color::Gray)),
            Span::styled("Ctrl+I", Style::default().fg(Color::Cyan)),
            Span::styled(" to focus the input bar)", Style::default().fg(Color::Gray)),
        ]));
    }

    // Refresh hitboxes for mouse or key interactions
    app.keyword_rects.clear();
    let base_x = area.x + 3; // Adjusted for padding
    let base_y = area.y + 3;

    app.keyword_rects
        .push(("settings".to_string(), Rect::new(base_x, base_y, 14, 1)));
    app.keyword_rects
        .push(("filters".to_string(), Rect::new(base_x + 16, base_y, 14, 1)));
    app.keyword_rects
        .push(("stats".to_string(), Rect::new(base_x + 32, base_y, 14, 1)));
    app.keyword_rects
        .push(("actions".to_string(), Rect::new(base_x + 48, base_y, 14, 1)));

    let p = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_widget(p, area);
}
