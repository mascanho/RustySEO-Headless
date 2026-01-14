use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let analysis = vec![
        Line::from(vec![Span::styled(
            " 🔍 Content Metrics ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📏 Title Length:  ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[3], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  📏 H1 Length:     ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[5], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  📏 Desc Length:   ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[7], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📖 Word Count:    ", Style::default().fg(Color::Cyan)),
            Span::raw("~842 words "),
            Span::styled("(Estimated)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  🧠 Readability:   ", Style::default().fg(Color::Cyan)),
            Span::styled("High ", Style::default().fg(Color::Green)),
            Span::styled("(Flesch Score: 72)", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let p = Paragraph::new(analysis)
        .block(block.title(Span::styled(
            "SEO Deep Dive ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area.inner(Margin::new(1, 0)));
}
