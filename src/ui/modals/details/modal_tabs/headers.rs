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
            " 📨 HTTP Headers Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📡 Response Headers: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    Server: ", Style::default().fg(Color::Cyan)),
            Span::raw("nginx/1.18.0"),
        ]),
        Line::from(vec![
            Span::styled("    Content-Type: ", Style::default().fg(Color::Cyan)),
            Span::raw("text/html; charset=UTF-8"),
        ]),
        Line::from(vec![
            Span::styled("    Content-Length: ", Style::default().fg(Color::Cyan)),
            Span::raw("127,456 bytes"),
        ]),
        Line::from(vec![
            Span::styled("    Cache-Control: ", Style::default().fg(Color::Cyan)),
            Span::raw("max-age=3600"),
        ]),
        Line::from(vec![
            Span::styled("    X-Frame-Options: ", Style::default().fg(Color::Cyan)),
            Span::styled("DENY", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🛡️  Security Headers: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "    ✅ Content-Security-Policy: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("Implemented"),
        ]),
        Line::from(vec![
            Span::styled(
                "    ✅ X-Content-Type-Options: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("nosniff"),
        ]),
        Line::from(vec![
            Span::styled(
                "    ❌ Strict-Transport-Security: ",
                Style::default().fg(Color::Red),
            ),
            Span::raw("Missing"),
        ]),
        Line::from(vec![
            Span::styled(
                "    ✅ X-XSS-Protection: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("1; mode=block"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ⚡ Performance Headers: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "    ✅ Accept-Encoding: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("gzip, deflate"),
        ]),
        Line::from(vec![
            Span::styled("    ⚠️  Vary: ", Style::default().fg(Color::Yellow)),
            Span::raw("Accept-Encoding (consider User-Agent)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🎯 SEO Impact: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Good security header implementation"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Consider adding HSTS header"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " HTTP Response Headers ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
