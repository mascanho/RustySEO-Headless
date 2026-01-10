use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let info = vec![
        Line::from(vec![
            Span::styled(
                " 🔗 URL: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::styled(&row_data[1], Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 📝 Title: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[2]),
        ]),
        Line::from(vec![
            Span::styled(
                " 🏷️  H1:    ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[4]),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " 📄 Meta Description: ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(vec![Span::raw(format!("   {}", &row_data[6]))]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 📡 Status Code: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::styled(
                &row_data[8],
                if row_data[8].contains("200") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
    ];
    let p = Paragraph::new(info)
        .block(block.title(Span::styled(
            " General Information ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
