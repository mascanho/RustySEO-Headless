use crate::{models::App, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let modal_area = centered_rect(50, 30, area);

    let accent_color = if app.action_result_success {
        Color::Green
    } else {
        Color::Red
    };
    let icon = if app.action_result_success { "✓" } else { "✗" };

    let block = Block::default()
        .title(Span::styled(
            format!(" {} {} ", icon, app.action_result_title),
            Style::default().fg(Color::Yellow).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent_color))
        .bg(Color::Rgb(15, 15, 25));

    f.render_widget(Clear, modal_area);
    f.render_widget(block.clone(), modal_area);

    let inner_area = block.inner(modal_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner_area);

    let message = Paragraph::new(Line::from(app.action_result_message.clone()))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(message, chunks[0]);

    let footer = Paragraph::new(Span::styled(
        " Esc/q/Enter: Close ",
        Style::default().fg(Color::DarkGray).italic(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);
}
