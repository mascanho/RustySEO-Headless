use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, headings: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let mut content = vec![
        Line::from(vec![Span::styled(
            " 📝 Page Headings (H1-H6) ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
    ];

    if headings.is_empty() {
        content.push(Line::from(vec![Span::styled(
            "No headings found on this page.",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for (i, heading) in headings.iter().enumerate() {
            let heading_text = heading.trim();
            if !heading_text.is_empty() {
                content.push(Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Cyan)),
                    Span::raw(heading_text),
                ]));
            }
        }
    }

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Headings Overview ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
