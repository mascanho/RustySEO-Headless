use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::models::App;

/// Renders the Custom Extractor tab with results table
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    app.table_rect = Some(area);
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Ensure we have filtered data if it was just initialized
    if app.extractor_filtered_table_data.is_empty()
        && !app.extractor_table_data.is_empty()
        && app.extractor_search_query.is_empty()
    {
        app.extractor_filtered_table_data = app.extractor_table_data.clone();
        app.extractor_full_filtered_table_data = app.extractor_table_data.clone();
    }

    let header_titles = [
        "ID",
        "URL",
        "Element",
        "Snippet",
    ];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let rows = app
        .extractor_filtered_table_data
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let is_selected = app.extractor_table_state.selected() == Some(i);

            let mut row_style = if i % 2 == 0 {
                Style::default().bg(Color::Rgb(20, 20, 30))
            } else {
                Style::default().bg(Color::Rgb(25, 25, 40))
            };

            if is_selected {
                row_style = row_style
                    .fg(Color::White)
                    .bg(accent_color)
                    .add_modifier(Modifier::BOLD);
            }

            let cells = vec![
                Cell::from((data.id).to_string()),
                Cell::from(data.url.as_str()),
                Cell::from(data.element.as_str()).style(Style::default().fg(Color::Yellow)),
                Cell::from(data.snippet.as_str()).style(Style::default().fg(Color::Gray)),
            ];

            Row::new(cells).style(row_style).height(1)
        });

    let widths = vec![
        Constraint::Length(6),            // ID
        Constraint::Percentage(30),       // URL
        Constraint::Length(10),           // Element
        Constraint::Percentage(54),       // Snippet
    ];

    let total_pages = (app.extractor_full_filtered_table_data.len() + app.extractor_page_size - 1)
        / app.extractor_page_size.max(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    format!(
                        " Custom Search Matches ({}) ",
                        app.extractor_full_filtered_table_data.len()
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_bottom(
                    Line::from(Span::styled(
                        format!(
                            " Page {} of {} ",
                            app.extractor_current_page + 1,
                            total_pages
                        ),
                        Style::default().fg(Color::DarkGray).italic(),
                    ))
                    .alignment(Alignment::Right),
                )
                .border_style(Style::default().fg(border_color)),
        )
        .column_spacing(1)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_stateful_widget(table, area, &mut app.extractor_table_state);

    // Floating Search Bar at bottom right
    if app.show_extractor_search {
        let search_area = Rect {
            x: area.x + area.width.saturating_sub(40),
            y: area.y + area.height.saturating_sub(3),
            width: 38,
            height: 3,
        };

        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .bg(Color::Rgb(25, 25, 40))
            .title(Span::styled(
                " Fuzzy Search ",
                Style::default().fg(Color::Cyan).bold(),
            ));

        let search_text = format!("> {}", app.extractor_search_query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, search_area);
        f.render_widget(search_paragraph, search_area);
    }
}
