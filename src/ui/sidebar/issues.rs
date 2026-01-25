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
        .map(|row_data| {
            let cells = row_data
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    use ratatui::layout::Alignment;
                    use ratatui::text::Line;
                    use ratatui::widgets::Cell;

                    if i > 0 {
                        // Columns 1 and 2 (URLs and % of) - centered with red styling for issues count
                        let style = if i == 1 && c.parse::<usize>().unwrap_or(0) > 0 {
                            Style::default().fg(Color::Red)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        Cell::from(Line::from(c.clone()).alignment(Alignment::Center)).style(style)
                    } else {
                        // Column 0 (Issues) - red text if count > 0
                        if c.parse::<usize>().unwrap_or(0) > 0 {
                            Cell::from(c.clone()).style(Style::default().fg(Color::Red))
                        } else {
                            Cell::from(c.clone())
                        }
                    }
                })
                .collect::<Vec<_>>();
            Row::new(cells)
        })
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
