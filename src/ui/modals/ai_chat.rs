use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ai::gemini;
use crate::models::App;
use crate::ui::centered_rect;

fn wrap_line(line: &str, width: usize) -> Vec<String> {
    let mut result = vec![];
    let mut current = String::new();
    for word in line.split_whitespace() {
        if current.len() + word.len() + 1 > width {
            if !current.is_empty() {
                result.push(current);
                current = word.to_string();
            } else {
                result.push(word.to_string());
            }
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

pub async fn send_message(app: &mut App) -> Result<String, Box<dyn std::error::Error>> {
    if app.ai_input.trim().is_empty() {
        return Err("Empty message".into());
    }

    let settings = app.settings.as_ref().ok_or("Settings not loaded")?;

    // Add user message to history
    app.ai_chat_history.push(crate::models::ChatLog {
        role: "user".to_string(),
        content: app.ai_input.clone(),
    });

    // Get AI response
    let response = gemini::ask(&app.ai_input, settings).await?;

    // Add AI response to history
    app.ai_chat_history.push(crate::models::ChatLog {
        role: "assistant".to_string(),
        content: response.clone(),
    });

    // Enable auto-scroll after new message
    app.ai_chat_auto_scroll = true;

    // Clear input
    app.ai_input.clear();

    Ok(response)
}

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

    let text_width = history_area.width.saturating_sub(6) as usize; // borders 2 + indent 4

    // 1. Render Chat History as scrollable Paragraph
    let mut chat_lines: Vec<Line> = Vec::new();
    for msg in &app.ai_chat_history {
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

        chat_lines.push(header);
        for line in msg.content.lines() {
            let wrapped = wrap_line(line, text_width);
            for w in wrapped {
                chat_lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(w, content_style),
                ]));
            }
        }
        chat_lines.push(Line::from(""));
    }

    // Calculate visual content height (now accurate since lines are wrapped)
    let content_height = chat_lines.len();
    let visible_height = (history_area.height.saturating_sub(2)) as usize; // Subtract borders

    // Auto-scroll to bottom if enabled
    if app.ai_chat_auto_scroll {
        if content_height > visible_height {
            app.ai_chat_scroll = content_height - visible_height;
        } else {
            app.ai_chat_scroll = 0;
        }
    } else {
        // Clamp manual scroll
        let max_scroll = content_height.saturating_sub(visible_height);
        if app.ai_chat_scroll > max_scroll {
            app.ai_chat_scroll = max_scroll;
        }
    }

    let chat_text = Text::from(chat_lines);

    let history_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 🤖 RustySEO  ",
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(accent_color))
        .bg(Color::Rgb(15, 15, 25));

    let history_paragraph = Paragraph::new(chat_text)
        .block(history_block)
        .scroll((app.ai_chat_scroll as u16, 0));

    f.render_widget(history_paragraph, history_area);

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
