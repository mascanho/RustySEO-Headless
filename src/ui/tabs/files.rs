use crate::models::App;
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

/// Standard colors for consistency
const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);
const BORDER_COLOR: Color = Color::Rgb(40, 45, 60);

/// Renders the Files table showing unique files discovered.
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    app.table_rect = Some(area);

    // Initial population if empty
    if app.files_filtered_table_data.is_empty()
        && !app.files_table_data.is_empty()
        && app.files_search_query.is_empty()
    {
        app.apply_files_filter();
    }

    let header_titles = ["#", " URL", "File Type"];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(ACCENT_COLOR)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let selected_idx = app.files_table_state.selected();
    let rows = create_rows(&app.files_filtered_table_data, selected_idx);

    let widths = [
        Constraint::Length(5),  // #
        Constraint::Min(60),    // URL
        Constraint::Length(15), // File Type
    ];

    let total_pages = calculate_total_pages(app);
    let pagination_info = format!(" Page {} of {} ", app.files_current_page + 1, total_pages);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(Span::styled(
                    format!(
                        " Discovered Files ({}) ",
                        app.files_full_filtered_table_data.len()
                    ),
                    Style::default().fg(Color::Yellow).bold(),
                ))
                .title_bottom(
                    Line::from(Span::styled(
                        pagination_info,
                        Style::default().fg(Color::DarkGray).italic(),
                    ))
                    .alignment(Alignment::Right),
                ),
        )
        .column_spacing(1)
        .row_highlight_style(Style::default().bg(ACCENT_COLOR))
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_stateful_widget(table, area, &mut app.files_table_state);

    // Render search bar if active
    if app.show_files_search {
        render_search_bar(f, &app.files_search_query, area);
    }
}

fn create_rows<'a>(
    data: &'a Vec<crate::models::FileEntry>,
    selected_idx: Option<usize>,
) -> Vec<Row<'a>> {
    if data.is_empty() {
        let no_files_row = Row::new(vec![
            Cell::from(""),
            Cell::from(" No files found during crawl ").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .italic()
                    .bg(Color::Rgb(20, 20, 30)),
            ),
            Cell::from(""),
        ])
        .height(1);
        return vec![no_files_row];
    }

    data.iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = selected_idx == Some(i);
            let base_style = if i % 2 == 0 {
                Style::default().bg(Color::Rgb(20, 20, 30))
            } else {
                Style::default().bg(Color::Rgb(25, 25, 40))
            };

            let mut row_style = base_style;
            if is_selected {
                row_style = row_style.fg(Color::White).add_modifier(Modifier::BOLD);
            }

            let cells = vec![
                Cell::from(format!(" {} ", file.id)).style(row_style),
                Cell::from(format!(" {} ", file.url)).style(row_style),
                Cell::from(format!(" {} ", file.filetype))
                    .style(row_style.fg(Color::Yellow).bold()),
            ];

            Row::new(cells).height(1)
        })
        .collect()
}

fn calculate_total_pages(app: &App) -> usize {
    let total_items = app.files_full_filtered_table_data.len();
    if total_items == 0 {
        1
    } else {
        (total_items + app.files_page_size - 1) / app.files_page_size
    }
}

fn render_search_bar(f: &mut Frame, query: &str, area: Rect) {
    let search_area = Rect {
        x: area.x + area.width / 4,
        y: area.y + area.height / 2 - 1,
        width: area.width / 2,
        height: 3,
    };

    f.render_widget(Clear, search_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT_COLOR))
        .title(" 🔍 Fuzzy Search (Files) ");

    let paragraph = Paragraph::new(format!(" Query: {}_", query))
        .block(block)
        .style(Style::default().bg(Color::Rgb(30, 35, 50)));

    f.render_widget(paragraph, search_area);
}
