use crate::{models::App, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let Some(data) = app.seo_score_data.clone() else {
        return;
    };

    let area = f.area();
    let modal_area = centered_rect(60, 55, area);

    let score_color = if data.score >= 80 {
        Color::Green
    } else if data.score >= 50 {
        Color::Yellow
    } else {
        Color::Red
    };

    let block = Block::default()
        .title(Span::styled(
            " SEO Score ",
            Style::default().fg(Color::Yellow).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(score_color))
        .bg(Color::Rgb(15, 15, 25));

    f.render_widget(Clear, modal_area);
    f.render_widget(block.clone(), modal_area);

    let inner_area = block.inner(modal_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // URL
            Constraint::Length(3), // Score
            Constraint::Min(0),    // Factors
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    let url_line = Paragraph::new(Span::styled(
        data.url.clone(),
        Style::default().fg(Color::DarkGray).italic(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(url_line, chunks[0]);

    let score_line = Paragraph::new(Line::from(vec![Span::styled(
        format!("{}/100", data.score),
        Style::default()
            .fg(score_color)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(score_line, chunks[1]);

    let items: Vec<ListItem> = data
        .factors
        .iter()
        .map(|factor| {
            let (icon, color) = if factor.passed {
                ("✓", Color::Green)
            } else {
                ("✗", Color::Red)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                Span::styled(
                    format!("{}: ", factor.label),
                    Style::default().fg(Color::White).bold(),
                ),
                Span::styled(factor.detail.clone(), Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[2]);

    let footer = Paragraph::new(Span::styled(
        " Esc/q: Close ",
        Style::default().fg(Color::DarkGray).italic(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}
