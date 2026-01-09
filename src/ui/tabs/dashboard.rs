use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let header_titles = [
        "ID", "URL", "Title", "Len", "H1", "H1 Len", "H2", "H2 Len", "Status",
    ];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )
    }))
    .style(Style::default().bg(Color::Rgb(30, 30, 60)))
    .height(1);

    let rows = app.table_data.iter().map(|data| {
        // We only show a subset of columns in the table to keep it readable
        // [ID, URL, Title, Title Len, H1, H1 Len, Status]
        let displayed_data = vec![
            &data[0], // ID
            &data[1], // URL
            &data[2], // Title
            &data[3], // Title Len
            &data[4], // H1
            &data[5], // H1 Len
            &data[6], // H2
            &data[7], // H2 Len
            &data[8], // Status
        ];

        let cells = displayed_data.iter().map(|c| {
            let style = match c.as_str() {
                s if s.contains("200 OK") => Style::default().fg(Color::Green),
                s if s.contains("404") => Style::default().fg(Color::Red),
                s if s.contains("301") => Style::default().fg(Color::Blue),
                _ => Style::default(),
            };
            Cell::from(c.as_str()).style(style)
        });
        Row::new(cells).height(1).bottom_margin(0)
    });

    let widths = [
        Constraint::Length(4),      // ID
        Constraint::Percentage(30), // URL
        Constraint::Percentage(25), // Title
        Constraint::Length(5),      // Len
        Constraint::Percentage(15), // H1
        Constraint::Length(7),      // H1 Len
        Constraint::Percentage(10), // H2
        Constraint::Length(7),      // H2 Len
        Constraint::Min(10),        // Status
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SEO Audit - Dashboard "),
        )
        .column_spacing(2)
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}
