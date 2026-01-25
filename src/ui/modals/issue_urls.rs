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

    let accent_color = Color::Rgb(255, 100, 100); // Red/orange for issues theme
    let border_color = accent_color;

    let block = Block::default()
        .title(Span::styled(
            format!(" ⚠️  URLs with '{}' Issue ", app.current_issue_title),
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
            Constraint::Length(2), // Info/footer
        ])
        .split(inner_area);

    let items: Vec<ListItem> = app
        .issue_urls_list
        .iter()
        .enumerate()
        .map(|(i, url)| {
            let url_number = i + 1;
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>3}. ", url_number),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(""),
                Span::styled(url, Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(accent_color)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    // .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.issue_urls_state);

    // Split the bottom section into info and footer
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[2]);

    // Info section showing count
    let info_text = format!("Found {} URLs with this issue", app.issue_urls_list.len());
    let info = Paragraph::new(Line::from(vec![
        Span::styled(" ℹ️ ", Style::default().fg(Color::Cyan)),
        Span::styled(info_text, Style::default().fg(Color::Gray)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(info, bottom_chunks[0]);

    // Footer with controls
    let footer = Paragraph::new(Span::styled(
        " Esc/q: Close | ↑/k ↓/j: Navigate | Enter: Open URL | c: Copy URL ",
        Style::default().fg(Color::DarkGray).italic(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(
        footer,
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(chunks[2])[1],
    );
}
