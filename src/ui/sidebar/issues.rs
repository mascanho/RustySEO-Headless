use crate::models::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Row, Table},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block) {
    let header = Row::new(vec!["Issue", "Urls", "% of"]).style(Style::default().fg(Color::Yellow));

    let rows: Vec<Row> = app
        .issues_table_data
        .iter()
        .map(|row| Row::new(row.clone()))
        .collect();

    let table = Table::new(
        rows,
        vec![
            Constraint::Percentage(33), // Issue column takes 1/3
            Constraint::Min(8),         // Urls column
            Constraint::Min(6),         // % of column
        ],
    )
    .header(header)
    .block(content_block.title(Span::styled(" Issues ", Style::default().fg(Color::Yellow))))
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::REVERSED));

    let mut table_state = app.issues_table_state.clone();
    f.render_stateful_widget(table, area, &mut table_state);
    app.issues_table_state = table_state;
}
