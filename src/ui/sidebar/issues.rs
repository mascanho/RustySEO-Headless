use crate::models::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Row, Table},
    Frame,
};

static ACCENT_COLOUR: Color = Color::Rgb(80, 140, 255);

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block) {
    let header =
        Row::new(vec![" Issues", " Urls", " % of"]).style(Style::default().fg(Color::Yellow));

    // Build rows from app.issues_table_data
    let rows: Vec<Row> = app
        .issues_table_data
        .iter()
        .map(|row_data| Row::new(row_data.clone()))
        .collect();

    let table = Table::new(
        rows,
        vec![
            Constraint::Percentage(70), // Issue column takes 1/3
            Constraint::Min(8),         // Urls column
            Constraint::Min(6),         // % of column
        ],
    )
    .header(header)
    .block(content_block.title(Span::styled("  ", Style::default().fg(Color::Yellow))))
    .style(Style::default().fg(Color::White))
    // .highlight_symbol("👉 ")
    .row_highlight_style(
        Style::default()
            .fg(Color::White)
            .bg(ACCENT_COLOUR)
            .add_modifier(ratatui::style::Modifier::BOLD),
    );

    let mut table_state = app.issues_table_state.clone();
    f.render_stateful_widget(table, area, &mut table_state);
    app.issues_table_state = table_state;
}
