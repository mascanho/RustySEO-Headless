use ratatui::{
    layout::{Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Crawl ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    
    let content = vec![
        Line::from(vec![
            Span::raw("Welcome to "),
            Span::styled("Atalaia SEO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[SETTINGS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("[FILTERS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("[STATS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("[ACTIONS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Click a keyword above to open relevant tools."),
        Line::from(""),
        Line::from("Enter URL to crawl:"),
        Line::from("______________________________"),
    ];

    app.keyword_rects.clear();
    let base_x = area.x + 1;
    let base_y = area.y + 3;
    
    app.keyword_rects.push(("settings".to_string(), Rect::new(base_x, base_y, 10, 1)));
    app.keyword_rects.push(("filters".to_string(), Rect::new(base_x + 12, base_y, 9, 1)));
    app.keyword_rects.push(("stats".to_string(), Rect::new(base_x + 23, base_y, 7, 1)));
    app.keyword_rects.push(("actions".to_string(), Rect::new(base_x + 32, base_y, 9, 1)));

    let p = Paragraph::new(content).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
