use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, anchor_links: &[(String, String)], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " 🔗 Incoming Links Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        // Show the number of outgoing links
        Line::from(vec![Span::styled(
            format!("Total Outgoing Links: {}", anchor_links.len()),
            Style::default().fg(accent_color),
        )]),
    ]
    .into_iter()
    .chain(anchor_links.iter().map(|(href, text)| {
        Line::from(vec![
            Span::styled("Href: ", Style::default().fg(Color::Green)),
            Span::raw(href.clone()),
            Span::styled(" | Anchor: ", Style::default().fg(Color::Yellow)),
            Span::raw(text.clone()),
        ])
    }))
    .collect::<Vec<_>>();

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Inbound Link Profile ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
