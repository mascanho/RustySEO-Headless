use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(10),
        ])
        .split(area.inner(Margin::new(1, 1)));

    let analysis = vec![
        Line::from(vec![Span::styled(
            " Content Metrics ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Title Length:   ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[3], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  H1 Length:      ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[5], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  Desc Length:    ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[7], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Word Count:     ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[18], Style::default().fg(Color::White)),
            Span::raw(" words "),
        ]),
    ];

    let p = Paragraph::new(analysis)
        .block(Block::default())
        .wrap(Wrap { trim: true });
    f.render_widget(p, chunks[0]);

    f.render_widget(block, area);
}