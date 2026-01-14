use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::models::App;
use crate::ui::centered_rect;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chat_area = centered_rect(70, 80, area);

    let accent_color = Color::Rgb(200, 100, 255); // Purple accent for AI
    let border_color = Color::Rgb(40, 45, 60);

    f.render_widget(Clear, chat_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Chat History
            Constraint::Length(3), // Input Area
        ])
        .split(chat_area);

    let history_area = chunks[0];
    let input_area = chunks[1];

    // 1. Render Chat History
    let history_items: Vec<ListItem> = app
        .ai_chat_history
        .iter()
        .map(|msg| {
            let (role_label, role_style, content_style) = if msg.role == "user" {
                (
                    " 👤 YOU ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::White),
                )
            } else {
                (
                    " 🤖 AI  ",
                    Style::default()
                        .fg(accent_color)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Rgb(200, 200, 220)),
                )
            };

            let header = Line::from(vec![
                Span::styled(role_label, role_style),
                Span::styled(
                    " ─────────────────────────────────",
                    Style::default().fg(border_color),
                ),
            ]);

            let mut lines = vec![header];
            for line in msg.content.lines() {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(line, content_style),
                ]));
            }
            lines.push(Line::from(""));

            ListItem::new(lines)
        })
        .collect();

    let history_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 🤖 RustyAI Copilot ",
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(accent_color))
        .bg(Color::Rgb(15, 15, 25));

    let history_list = List::new(history_items)
        .block(history_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(history_list, history_area);

    // 2. Render Input Area
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 💬 Ask anything... (Enter to Send, Esc to Close) ",
            Style::default().fg(Color::Gray),
        ))
        .border_style(Style::default().fg(Color::Blue))
        .bg(Color::Rgb(20, 20, 30));

    let input_p = Paragraph::new(app.ai_input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(input_p, input_area);

    // Render cursor in input area if active
    if app.show_ai_modal {
        f.set_cursor_position((
            input_area.x + app.ai_input.len() as u16 + 1,
            input_area.y + 1,
        ));
    }
}
