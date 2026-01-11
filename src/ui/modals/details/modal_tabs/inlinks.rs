use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " 🔗 Incoming Links Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        // Show the number of incoming links
        Line::from(vec![Span::styled(
            format!("Total Incoming Links: {}", row_data.len()),
            Style::default().fg(accent_color),
        )]),
        // Display all the links and their anchors
        Line::from(vec![Span::styled(
            "Links:",
            Style::default().fg(accent_color),
        )]),
        Line::from(vec![Span::styled(
            "Anchor:",
            Style::default().fg(accent_color),
        )]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Inbound Link Profile ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
