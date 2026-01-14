use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " ↗️  Outgoing Links Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🔗 Total Outlinks: ", Style::default().fg(Color::Cyan)),
            Span::styled("34", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  🔒 Internal Links: ", Style::default().fg(Color::Cyan)),
            Span::styled("28", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  🌐 External Links: ", Style::default().fg(Color::Cyan)),
            Span::styled("6", Style::default().fg(Color::Blue)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Top External Destinations: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("social-platform.com"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("affiliate-partner.org"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("industry-resource.net"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ⚠️  Link Quality Issues: ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(Color::Red)),
            Span::raw("2 links have nofollow attribute"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(Color::Yellow)),
            Span::raw("1 link points to 404 page"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            "Outbound Link Structure ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area.inner(Margin::new(1, 0)));
}
