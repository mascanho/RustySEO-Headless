use crate::models::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Row, Table},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block) {
    let header = Row::new(vec!["Issue", "Urls", "% of"]).style(Style::default().fg(Color::Yellow));

    let rows: Vec<Row> = app
        .issues_table_data
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = app.issues_table_state.selected().map_or(false, |s| s == i);
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 100, 100)) // Red/orange for issues theme
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Row::new(row.clone()).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        vec![
            Constraint::Percentage(40), // Issue column takes 1/3
            Constraint::Min(8),         // Urls column
            Constraint::Min(6),         // % of column
        ],
    )
    .header(header)
    .block(content_block.title(Span::styled("  ", Style::default().fg(Color::Yellow))))
    .style(Style::default().fg(Color::White))
    .highlight_symbol("👉 ")
    .row_highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(255, 100, 100)) // Red/orange for issues theme
            .add_modifier(ratatui::style::Modifier::BOLD),
    );

    let mut table_state = app.issues_table_state.clone();
    f.render_stateful_widget(table, area, &mut table_state);
    app.issues_table_state = table_state;
}
