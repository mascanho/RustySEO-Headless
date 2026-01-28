use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::models::App;

/// Renders the Keywords tab with results table
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    app.table_rect = Some(area);
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Initial population if empty
    if app.keywords_filtered_table_data.is_empty()
        && !app.keywords_table_data.is_empty()
        && app.keywords_search_query.is_empty()
    {
        app.apply_keywords_filter();
    }

    let header_titles = ["ID", "Keyword", "URL", "Page Word Count", "Relevance"];

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
        .keywords_filtered_table_data
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = app.keywords_table_state.selected() == Some(i);

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
                Cell::from(entry.id.to_string()),
                Cell::from(entry.keyword.clone()),
                Cell::from(entry.url.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(entry.word_count.to_string()),
                Cell::from(format!("#{}", entry.relevance)),
            ];

            Row::new(cells).style(row_style).height(1)
        });

    let widths = vec![
        Constraint::Length(6),      // ID
        Constraint::Percentage(25), // Keyword
        Constraint::Percentage(45), // URL
        Constraint::Length(18),     // Word Count
        Constraint::Length(12),     // Relevance
    ];

    let total_pages = (app.keywords_full_filtered_table_data.len() + app.keywords_page_size - 1)
        / app.keywords_page_size.max(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    format!(
                        " Keyword Analysis ({}) ",
                        app.keywords_full_filtered_table_data.len()
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_bottom(
                    Line::from(Span::styled(
                        format!(
                            " Page {} of {} ",
                            app.keywords_current_page + 1,
                            total_pages.max(1)
                        ),
                        Style::default().fg(Color::DarkGray).italic(),
                    ))
                    .alignment(Alignment::Right),
                )
                .border_style(Style::default().fg(border_color)),
        )
        .column_spacing(1)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_stateful_widget(table, area, &mut app.keywords_table_state);

    // Floating Search Bar at bottom right
    if app.show_keywords_search {
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
                " Search Keywords ",
                Style::default().fg(Color::Cyan).bold(),
            ));

        let search_text = format!("> {}", app.keywords_search_query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, search_area);
        f.render_widget(search_paragraph, search_area);
    }
}
