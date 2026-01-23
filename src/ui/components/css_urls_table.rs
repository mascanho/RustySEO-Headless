use crate::models::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

/// Standard colors for consistency
const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);
const BORDER_COLOR: Color = Color::Rgb(40, 45, 60);

/// Renders the CSS URLs table showing unique CSS files and their usage statistics.
pub fn render_css_urls_table(f: &mut Frame, app: &mut App, area: Rect) {
    app.table_rect = Some(area);

    // Initial population if empty
    if app.css_urls_filtered_table_data.is_empty()
        && !app.css_urls_table_data.is_empty()
        && app.css_urls_search_query.is_empty()
    {
        app.apply_css_urls_filter();
    }

    let header_titles = ["#", "CSS URL", "Pages Using"];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(ACCENT_COLOR)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let selected_idx = app.css_urls_table_state.selected();
    let rows = create_rows(&app.css_urls_filtered_table_data, selected_idx);

    let widths = [
        Constraint::Length(5),  // #
        Constraint::Min(60),    // CSS URL
        Constraint::Length(12), // Pages Using
    ];

    let total_pages = calculate_total_pages(app);
    let pagination_info = format!(
        " Page {} of {} ",
        app.css_urls_current_page + 1,
        total_pages
    );

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(Span::styled(
                    format!(
                        " 🎨 CSS URLs ({}) ",
                        app.css_urls_full_filtered_table_data.len()
                    ),
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
        .row_highlight_style(Style::default().bg(ACCENT_COLOR))
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_stateful_widget(table, area, &mut app.css_urls_table_state);

    // Render search bar if active
    if app.show_css_urls_search {
        render_search_bar(f, &app.css_urls_search_query, area);
    }
}

fn create_rows<'a>(
    data: &'a Vec<crate::models::CssUrl>,
    selected_idx: Option<usize>,
) -> Vec<Row<'a>> {
    data.iter()
        .enumerate()
        .map(|(i, css_url)| {
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
                Cell::from(format!(" {} ", css_url.id)).style(row_style),
                Cell::from(format!(" {} ", truncate_url(&css_url.url))).style(row_style),
                Cell::from(format!(" {} ", css_url.page_count))
                    .style(row_style.fg(Color::Green).bold()),
            ];

            Row::new(cells).height(1)
        })
        .collect()
}

fn truncate_url(url: &str) -> String {
    url.to_string()
}

fn calculate_total_pages(app: &App) -> usize {
    let total_items = app.css_urls_full_filtered_table_data.len();
    if total_items == 0 {
        1
    } else {
        (total_items + app.css_urls_page_size - 1) / app.css_urls_page_size
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
        .title(" 🔍 Fuzzy Search (CSS URLs) ");

    let paragraph = Paragraph::new(format!(" Query: {}_", query))
        .block(block)
        .style(Style::default().bg(Color::Rgb(30, 35, 50)));

    f.render_widget(paragraph, search_area);
}
