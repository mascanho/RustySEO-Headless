use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::models::App;

/// Renders the Content tab with the same table as the Dashboard.
/// This allows for content-specific views and future customizations.
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    app.table_rect = Some(area);
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Ensure we have filtered data if it was just initialized
    if app.filtered_table_data.is_empty()
        && !app.table_data.is_empty()
        && app.search_query.is_empty()
    {
        app.filtered_table_data = app.table_data.clone();
    }

    let header_titles = [
        "ID", "URL", "Word Count", "KW 1", "KW 2", "KW 3", "KW 4", "KW 5", "KW 6", "KW 7", "KW 8",
        "KW 9", "KW 10",
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

    let rows = app.filtered_table_data.iter().enumerate().map(|(i, data)| {
        let is_selected = app.table_state.selected() == Some(i);

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

        let start = app.current_page * app.page_size;
        let full_idx = start + i;
        let mut displayed_data = vec![
            (full_idx + 1).to_string(), // Sequential ID
            data[1].clone(),            // URL
            data[18].clone(),           // Word Count
        ];

        // Add Top 10 Keywords (Indices 23 to 32)
        for j in 23..33 {
            if let Some(kw) = data.get(j) {
                displayed_data.push(kw.clone());
            } else {
                displayed_data.push(String::new());
            }
        }

        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            let content = if j == 1 {
                // URL
                let content = c.as_str();
                let char_count = content.chars().count();
                if char_count > 60 {
                    let start = app.horizontal_scroll.min(char_count.saturating_sub(50));
                    let end = (start + 60).min(char_count);
                    let sliced: String = content.chars().skip(start).take(end - start).collect();
                    if start > 0 {
                        format!("…{}", sliced)
                    } else {
                        sliced
                    }
                } else {
                    content.to_string()
                }
            } else {
                c.as_str().to_string()
            };

            let mut cell_style = Style::default();

            if j == 2 {
                // Word count column
                if let Ok(count) = content.trim().parse::<usize>() {
                    if count > 1000 {
                        cell_style = cell_style.fg(Color::Green).bold();
                    } else if count < 200 {
                        cell_style = cell_style.fg(Color::Red);
                    }
                }
            }

            if j >= 3 {
                // Keywords
                cell_style = cell_style.fg(Color::Cyan);
            }

            Cell::from(content).style(cell_style)
        });

        Row::new(cells).style(row_style).height(1)
    });

    let max_id_width = app.full_filtered_table_data.len().to_string().len().max(2) as u16 + 2;
    let mut widths = vec![
        Constraint::Length(max_id_width), // ID
        Constraint::Min(40),              // URL
        Constraint::Length(12),           // Word Count
    ];

    // Add 10 constraints for keywords
    for _ in 0..10 {
        widths.push(Constraint::Length(15));
    }

    let total_pages = (app.full_filtered_table_data.len() + app.page_size - 1) / app.page_size;
    let scroll_indicator = if app.horizontal_scroll > 0 {
        format!(" [Scroll: {}] ", app.horizontal_scroll)
    } else {
        String::new()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    format!(
                        " Content Audit ({}) ",
                        app.full_filtered_table_data.len()
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_bottom(
                    Line::from(Span::styled(
                        format!(
                            " Page {} of {} {} ",
                            app.current_page + 1,
                            total_pages,
                            scroll_indicator
                        ),
                        Style::default().fg(Color::DarkGray).italic(),
                    ))
                    .alignment(Alignment::Right),
                )
                .border_style(Style::default().fg(border_color)),
        )
        .column_spacing(1)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_stateful_widget(table, area, &mut app.table_state);

    // Floating Search Bar at bottom right
    if app.show_search {
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

        let search_text = format!("> {}", app.search_query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, search_area);
        f.render_widget(search_paragraph, search_area);
    }
}
