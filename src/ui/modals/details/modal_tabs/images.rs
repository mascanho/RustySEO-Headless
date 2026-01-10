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
            " 🖼️  Image Optimization Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🖼️  Total Images: ", Style::default().fg(Color::Cyan)),
            Span::styled("12", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  📏 Average Size: ", Style::default().fg(Color::Cyan)),
            Span::styled("245 KB", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  ⚡ Loading Speed: ", Style::default().fg(Color::Cyan)),
            Span::styled("2.1s", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📋 ALT Text Status: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    ✅ With ALT: ", Style::default().fg(Color::Green)),
            Span::raw("10 images"),
        ]),
        Line::from(vec![
            Span::styled("    ❌ Missing ALT: ", Style::default().fg(Color::Red)),
            Span::raw("2 images"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📐 Image Dimensions: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Hero image: 1920x1080 (optimized)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Thumbnail: 300x200 (needs optimization)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Logo: 200x80 (SVG format)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  💡 Recommendations: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Compress images by 40% to improve load speed"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Add descriptive ALT text to remaining images"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Image Assets Overview ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
