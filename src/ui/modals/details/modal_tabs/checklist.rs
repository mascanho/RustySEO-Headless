use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let checklist = vec![
        Line::from(vec![Span::styled(
            " ⚔️  SEO Health Check ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(if row_data[2].len() > 60 {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Title too long (over 60 chars)"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("Title length is optimal"),
            ]
        }),
        Line::from(if row_data[6].len() > 160 {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Meta description exceeds 160 chars"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("Meta description length is good"),
            ]
        }),
        Line::from(if row_data[4].is_empty() {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Missing H1 heading"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("H1 heading present and valid"),
            ]
        }),
        Line::from(if row_data[8].contains("200") {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("HTTP Status OK (200)"),
            ]
        } else {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Critical HTTP Status Issue"),
            ]
        }),
        Line::from(""),
        Line::from(vec![Span::styled(
            " 💡 Recommendations ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Ensure keyword density is balanced (1.5% - 2.0%)."),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Optimize internal linking for high-value pages."),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Add ALT tags to images for better accessibility."),
        ]),
    ];
    let p = Paragraph::new(checklist)
        .block(block.title(Span::styled(
            "Automated SEO Audit Checklist ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area.inner(Margin::new(1, 0)));
}
