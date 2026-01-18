use crate::{models::App, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let modal_area = centered_rect(70, 60, area);

    let accent_color = Color::Rgb(255, 180, 80); // Match JS theme
    let border_color = accent_color;

    let block = Block::default()
        .title(Span::styled(
            " 📄 Pages using this JavaScript ",
            Style::default().fg(Color::Yellow).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(Color::Rgb(15, 15, 25));

    f.render_widget(Clear, modal_area);
    f.render_widget(block.clone(), modal_area);

    let inner_area = block.inner(modal_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // List
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    let items: Vec<ListItem> = app
        .js_pages_list
        .iter()
        .map(|url| {
            ListItem::new(Line::from(vec![
                Span::styled(" 🔗 ", Style::default().fg(Color::Cyan)),
                Span::styled(url, Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(accent_color)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.js_pages_state);

    let footer = Paragraph::new(Span::styled(
        " Esc/q: Close | ↑/k ↓/j: Navigate ",
        Style::default().fg(Color::DarkGray).italic(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
