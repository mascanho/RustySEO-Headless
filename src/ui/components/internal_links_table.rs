use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::models::App;
use crate::crawler::PageData;

/// Standard colors for consistency
const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);
const BORDER_COLOR: Color = Color::Rgb(40, 45, 60);

/// Renders the internal links table, replicating the dashboard's look and feel.
pub fn render_internal_links_table(f: &mut Frame, app: &mut App, area: Rect, title: &str) {
    app.table_rect = Some(area);

    // Initial population if empty
    if app.internal_filtered_table_data.is_empty() && !app.internal_table_data.is_empty() && app.internal_search_query.is_empty() {
        app.apply_internal_filter();
    }

    let header_titles = ["#", "Source URL", "Destination URL", "Anchor Text", "Rel", "Status"];
    
    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(ACCENT_COLOR)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let selected_idx = app.internal_table_state.selected();
    let rows = create_rows(&app.internal_filtered_table_data, selected_idx, &app.page_data);

    let widths = [
        Constraint::Length(5),   // #
        Constraint::Min(40),     // Source
        Constraint::Min(40),     // Destination
        Constraint::Min(30),     // Anchor
        Constraint::Length(15),  // Rel
        Constraint::Length(10),  // Status
    ];

    let total_pages = calculate_total_pages(app);
    let pagination_info = format!(" Page {} of {} ", app.internal_current_page + 1, total_pages);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(Span::styled(
                    format!(" {} ({}) ", title, app.internal_full_filtered_table_data.len()),
                    Style::default().fg(ACCENT_COLOR).bold(),
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
        .row_highlight_style(Style::default().bg(Color::Rgb(50, 50, 80)))
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_stateful_widget(table, area, &mut app.internal_table_state);

    // Render search bar if active
    if app.show_internal_search {
        render_search_bar(f, &app.internal_search_query, area);
    }
}

fn create_rows<'a>(data: &'a Vec<Vec<String>>, selected_idx: Option<usize>, page_data: &'a Vec<PageData>) -> Vec<Row<'a>> {
    data.iter().enumerate().map(|(i, row)| {
        let is_selected = selected_idx == Some(i);
        let base_style = if i % 2 == 0 {
            Style::default().bg(Color::Rgb(20, 20, 30))
        } else {
            Style::default().bg(Color::Rgb(25, 25, 40))
        };

        let mut row_style = base_style;
        if is_selected {
            row_style = row_style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
        }

        let cells: Vec<Cell> = row.iter().enumerate().map(|(j, content)| {
            let mut display_content = content.clone();
            if j == 1 || j == 2 {
                // Truncate long URLs
                if display_content.len() > 60 {
                    display_content = format!("...{}", &display_content[display_content.len() - 57..]);
                }
            }

            if j == 5 {
                // Status Column
                let dest_url = &row[2];
                let status = lookup_status(dest_url, page_data);
                let color = if status.contains("200") {
                    Color::Green
                } else if status.contains("30") {
                    Color::Yellow
                } else if status.contains("40") || status.contains("50") {
                    Color::Red
                } else {
                    Color::DarkGray
                };
                Cell::from(format!(" {} ", status)).style(row_style.fg(color).bold())
            } else {
                Cell::from(format!(" {} ", display_content)).style(row_style)
            }
        }).collect();

        Row::new(cells).height(1)
    }).collect()
}

fn lookup_status(url: &str, page_data: &Vec<PageData>) -> String {
    for page in page_data {
        if page.url == url {
            return page.status.clone();
        }
    }
    "Pending".to_string()
}

fn calculate_total_pages(app: &App) -> usize {
    let total_items = app.internal_full_filtered_table_data.len();
    if total_items == 0 { 1 }
    else { (total_items + app.internal_page_size - 1) / app.internal_page_size }
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
        .title(" 🔍 Fuzzy Search (Internal Links) ");
    
    let paragraph = Paragraph::new(format!(" Query: {}_", query))
        .block(block)
        .style(Style::default().bg(Color::Rgb(30, 35, 50)));
    
    f.render_widget(paragraph, search_area);
}
