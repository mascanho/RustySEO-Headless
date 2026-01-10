use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " 🔗 Incoming Links Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📊 Total Inlinks: ", Style::default().fg(Color::Cyan)),
            Span::styled("127", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  🎯 Domain Authority: ", Style::default().fg(Color::Cyan)),
            Span::styled("72/100", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Top Referring Domains: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("example.com (23 links)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("blog.example.org (18 links)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("news.site.com (15 links)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📈 Link Quality Distribution: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    High Quality: ", Style::default().fg(Color::Green)),
            Span::raw("67%"),
        ]),
        Line::from(vec![
            Span::styled("    Medium Quality: ", Style::default().fg(Color::Yellow)),
            Span::raw("28%"),
        ]),
        Line::from(vec![
            Span::styled("    Low Quality: ", Style::default().fg(Color::Red)),
            Span::raw("5%"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Inbound Link Profile ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
