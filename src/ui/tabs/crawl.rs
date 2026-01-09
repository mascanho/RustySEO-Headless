use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Crawl ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let mut content = vec![
        Line::from(vec![
            Span::raw("Welcome to "),
            Span::styled(
                "Atalaia SEO",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[SETTINGS]",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "[FILTERS]",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "[STATS]",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "[ACTIONS]",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from("Click a keyword above to open relevant tools."),
        Line::from(""),
    ];

    if app.is_crawling {
        content.push(Line::from(vec![
            Span::styled("ACTIVE CRAWL: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled(&app.input_url, Style::default().fg(Color::White)),
        ]));
        content.push(Line::from(""));
        content.push(Line::from("Analyzing site structure and extracting SEO metadata..."));
        content.push(Line::from("Check the 'Logs' tab for real-time URI discovery."));
    } else if app.crawl_progress >= 1.0 {
        content.push(Line::from(vec![
            Span::styled("CRAWL COMPLETE: ", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
            Span::styled(&app.input_url, Style::default().fg(Color::White)),
        ]));
        content.push(Line::from(""));
        content.push(Line::from("All discovered pages have been audited."));
        content.push(Line::from("Head to the 'Dashboard' to view the full SEO report."));
        content.push(Line::from(""));
        content.push(Line::from(" (Press Ctrl+I to start a new crawl) "));
    } else {
        content.push(Line::from("Enter URL to crawl:"));
        content.push(Line::from("______________________________"));
        content.push(Line::from(" (Press Ctrl+I to focus top bar) "));
    }

    app.keyword_rects.clear();
    let base_x = area.x + 1;
    let base_y = area.y + 3;

    app.keyword_rects
        .push(("settings".to_string(), Rect::new(base_x, base_y, 10, 1)));
    app.keyword_rects
        .push(("filters".to_string(), Rect::new(base_x + 12, base_y, 9, 1)));
    app.keyword_rects
        .push(("stats".to_string(), Rect::new(base_x + 23, base_y, 7, 1)));
    app.keyword_rects
        .push(("actions".to_string(), Rect::new(base_x + 32, base_y, 9, 1)));

    let p = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
