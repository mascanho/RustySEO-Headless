use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};
use std::collections::HashMap;
use crate::models::App;

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
    let rows = create_rows(&app.internal_filtered_table_data, selected_idx, &app.url_to_status);

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

fn create_rows<'a>(data: &'a Vec<crate::models::InternalLink>, selected_idx: Option<usize>, url_to_status: &'a HashMap<String, String>) -> Vec<Row<'a>> {
    data.iter().enumerate().map(|(i, link)| {
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

        let mut cells = vec![
            Cell::from(format!(" {} ", link.id)).style(row_style),
            Cell::from(format!(" {} ", truncate_url(&link.source))).style(row_style),
            Cell::from(format!(" {} ", truncate_url(&link.destination))).style(row_style),
            Cell::from(format!(" {} ", link.anchor)).style(row_style),
            Cell::from(format!(" {} ", link.rel)).style(row_style),
        ];

        // Status Column lookup based on Destination URL
        let status = lookup_status(&link.destination, url_to_status);
        let color = if status.contains("200") {
            Color::Green
        } else if status.contains("30") {
            Color::Yellow
        } else if status.contains("40") || status.contains("50") {
            Color::Red
        } else {
            Color::DarkGray
        };
        cells.push(Cell::from(format!(" {} ", status)).style(row_style.fg(color).bold()));
        
        Row::new(cells).height(1)
    }).collect()
}

fn truncate_url(url: &str) -> String {
    if url.len() > 64 {
        format!("...{}", &url[url.len() - 61..])
    } else {
        url.to_string()
    }
}

fn lookup_status(url: &str, url_to_status: &HashMap<String, String>) -> String {
    url_to_status.get(url).cloned().unwrap_or_else(|| "Pending".to_string())
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
