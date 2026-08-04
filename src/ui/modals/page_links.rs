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
    let modal_area = centered_rect(75, 65, area);

    let accent_color = Color::Rgb(80, 140, 255);

    let block = Block::default()
        .title(Span::styled(
            format!(" Links Found ({}) ", app.page_links_list.len()),
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
        .constraints([
            Constraint::Min(0),    // List
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    if app.page_links_list.is_empty() {
        let empty = Paragraph::new("No links found on this page.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .page_links_list
            .iter()
            .map(|link| {
                let (tag, tag_color) = if link.is_internal {
                    ("INT", Color::Cyan)
                } else {
                    ("EXT", Color::Magenta)
                };
                let anchor = if link.anchor.trim().is_empty() {
                    "(no anchor text)".to_string()
                } else {
                    link.anchor.clone()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" [{}] ", tag), Style::default().fg(tag_color).bold()),
                    Span::styled(link.destination.clone(), Style::default().fg(Color::White)),
                    Span::styled(format!("  \"{}\"", anchor), Style::default().fg(Color::DarkGray)),
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

        f.render_stateful_widget(list, chunks[0], &mut app.page_links_state);
    }

    let footer = Paragraph::new(Span::styled(
        " Esc/q: Close | ↑/k ↓/j: Navigate | Enter: Open in Browser ",
        Style::default().fg(Color::DarkGray).italic(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);
}
