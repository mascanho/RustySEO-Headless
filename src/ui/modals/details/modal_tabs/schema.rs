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
            " 📋 Structured Data Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  📊 Schema Types Found: ",
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("3", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  ✅ Valid Schemas: ", Style::default().fg(Color::Green)),
            Span::raw("3/3"),
        ]),
        Line::from(vec![
            Span::styled("  📝 JSON-LD Format: ", Style::default().fg(Color::Cyan)),
            Span::styled("Yes", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🔍 Detected Schema Types: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Organization"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("WebSite"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Article"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🎯 Schema Validation: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    ✅ Organization: ", Style::default().fg(Color::Green)),
            Span::raw("Complete and valid"),
        ]),
        Line::from(vec![
            Span::styled("    ✅ WebSite: ", Style::default().fg(Color::Green)),
            Span::raw("Breadcrumb navigation enabled"),
        ]),
        Line::from(vec![
            Span::styled("    ⚠️  Article: ", Style::default().fg(Color::Yellow)),
            Span::raw("Missing publication date"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🚀 Rich Results Potential: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Knowledge Panel eligibility"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Enhanced search appearance"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Schema Markup Details ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
