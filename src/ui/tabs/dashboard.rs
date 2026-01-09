use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let header_titles = ["ID", "Name", "Status", "Date", "Value", "Category", "Notes"];
    
    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
    }))
    .style(Style::default().bg(Color::Rgb(30, 30, 60)))
    .height(1);

    let rows = app.table_data.iter().map(|data| {
        let cells = data.iter().map(|c| {
            let style = match c.as_str() {
                "Active" => Style::default().fg(Color::Green),
                "Inactive" => Style::default().fg(Color::Red),
                _ => Style::default(),
            };
            Cell::from(c.as_str()).style(style)
        });
        Row::new(cells).height(1).bottom_margin(0)
    });

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(20),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Percentage(15),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" SEO Data Overview "))
        .column_spacing(2)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_widget(table, area);
}
